#![no_std]
#![no_main]

use esp_hal::{
    delay::Delay,
    dma::DmaRxBuf,
    i2c::master::I2c,
    lcd_cam::{
        cam::{Camera, Config as CamConfig},
        LcdCam,
    },
    ledc::{
        channel::{self, ChannelIFace},
        timer::{self, TimerIFace},
        LSGlobalClkSource, Ledc, LowSpeed,
    },
};
use esp_println::{print, println};
use log::info;

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}

extern crate alloc;

// This creates a default app-descriptor required by the esp-idf bootloader.
esp_bootloader_esp_idf::esp_app_desc!();

/// JPEG frame buffer - 32KB
const FRAME_SIZE: usize = 32 * 1024;
static mut FRAME_BUFFER: [u8; FRAME_SIZE] = [0u8; FRAME_SIZE];

// DMA descriptors
const DESC_COUNT: usize = 16;
static mut DMA_DESCRIPTORS: [esp_hal::dma::DmaDescriptor; DESC_COUNT] = 
    [esp_hal::dma::DmaDescriptor::EMPTY; DESC_COUNT];

#[esp_hal::main]
fn main() -> ! {
    let peripherals = esp_hal::init(esp_hal::Config::default());

    esp_alloc::heap_allocator!(#[unsafe(link_section = ".dram2_uninit")] size: 73744);

    // Initialize logging
    esp_println::logger::init_logger(log::LevelFilter::Info);
    println!("=== Xiao ESP32S3 Camera Stream ===");
    info!("Initializing camera system...");

    let delay = Delay::new();

    // --- I2C for OV2640 sensor communication (SCCB) ---
    let sda = peripherals.GPIO40;
    let scl = peripherals.GPIO39;
    
    let mut i2c = I2c::new(peripherals.I2C0, esp_hal::i2c::master::Config::default())
        .unwrap()
        .with_sda(sda)
        .with_scl(scl);

    println!("I2C initialized");

    // --- XCLK = 10 MHz via LEDC PWM (camera master clock) ---
    let mut ledc = Ledc::new(peripherals.LEDC);
    ledc.set_global_slow_clock(LSGlobalClkSource::APBClk);
    
    let mut lstimer0 = ledc.timer::<LowSpeed>(timer::Number::Timer0);
    lstimer0
        .configure(timer::config::Config {
            duty: timer::config::Duty::Duty1Bit,
            clock_source: timer::LSClockSource::APBClk,
            frequency: esp_hal::time::Rate::from_mhz(10),
        })
        .unwrap();

    let mut channel0 = ledc.channel(channel::Number::Channel0, peripherals.GPIO10);
    channel0
        .configure(channel::config::Config {
            timer: &lstimer0,
            duty_pct: 50,
            drive_mode: esp_hal::gpio::DriveMode::PushPull,
        })
        .unwrap();

    println!("LEDC configured: 10MHz XCLK on GPIO10");

    // --- Initialize OV2640 sensor ---
    let addr = 0x30; // OV2640 I2C address
    
    println!("Resetting OV2640...");
    ov2640_reset(&mut i2c, addr);
    delay.delay_millis(100);
    
    println!("Configuring OV2640 for JPEG SVGA...");
    ov2640_jpeg_svga(&mut i2c, addr, 12);
    delay.delay_millis(50);

    // Verify sensor
    let mut sensor_id = [0u8; 2];
    i2c.write(addr, &[0xff, 0x01]).ok();
    i2c.write_read(addr, &[0x0a], &mut sensor_id[0..1]).ok();
    i2c.write_read(addr, &[0x0b], &mut sensor_id[1..2]).ok();
    println!("OV2640 ID: PID=0x{:02x} VER=0x{:02x}", sensor_id[0], sensor_id[1]);

    info!("OV2640 sensor initialized");

    // --- LCD_CAM peripheral for frame capture ---
    let lcd_cam = LcdCam::new(peripherals.LCD_CAM);
    
    let mut camera = Camera::new(
        lcd_cam.cam,
        peripherals.DMA_CH0,
        CamConfig::default(),
    )
    .unwrap()
    .with_pixel_clock(peripherals.GPIO13)
    .with_vsync(peripherals.GPIO38)
    .with_hsync(peripherals.GPIO47)
    .with_data0(peripherals.GPIO15)
    .with_data1(peripherals.GPIO17)
    .with_data2(peripherals.GPIO18)
    .with_data3(peripherals.GPIO16)
    .with_data4(peripherals.GPIO14)
    .with_data5(peripherals.GPIO12)
    .with_data6(peripherals.GPIO11)
    .with_data7(peripherals.GPIO48);

    println!("LCD_CAM configured");

    // Create DMA buffer
    let mut rx_buffer = unsafe {
        DmaRxBuf::new(
            &mut *core::ptr::addr_of_mut!(DMA_DESCRIPTORS),
            &mut *core::ptr::addr_of_mut!(FRAME_BUFFER)
        ).unwrap()
    };

    println!("Starting capture loop...\n");
    let mut frame_count = 0u32;

    loop {
        frame_count += 1;
        println!("[Frame {}] Starting capture...", frame_count);

        // Start DMA transfer
        let transfer = match camera.receive(rx_buffer) {
            Ok(t) => t,
            Err((e, cam, buf)) => {
                println!("Failed to start transfer: {:?}", e);
                camera = cam;
                rx_buffer = buf;
                delay.delay_millis(100);
                continue;
            }
        };

        // Wait for frame
        let (result, cam, buf) = transfer.wait();
        camera = cam;
        rx_buffer = buf;

        match result {
            Ok(()) => {
                let buffer = unsafe { &FRAME_BUFFER[..] };
                
                // Find JPEG markers
                let has_start = buffer.len() >= 2 && buffer[0] == 0xFF && buffer[1] == 0xD8;
                let jpeg_end = find_jpeg_end(buffer);
                
                if has_start {
                    if let Some(len) = jpeg_end {
                        println!("✓ Valid JPEG: {} bytes", len);
                        
                        // Show first few bytes for debugging
                        print!("  Data: ");
                        for i in 0..16.min(len) {
                            print!("{:02X} ", buffer[i]);
                        }
                        println!();
                    } else {
                        println!("⚠ JPEG header but no end marker");
                    }
                } else {
                    println!("✗ No JPEG header - first bytes: {:02X} {:02X}", 
                             buffer[0], buffer[1]);
                }
            }
            Err(e) => {
                println!("DMA error: {:?}", e);
            }
        }

        delay.delay_millis(1000);
    }
}

fn find_jpeg_end(buffer: &[u8]) -> Option<usize> {
    for i in 0..buffer.len().saturating_sub(1) {
        if buffer[i] == 0xFF && buffer[i + 1] == 0xD9 {
            return Some(i + 2);
        }
    }
    None
}

fn ov2640_reset<I: embedded_hal::i2c::I2c>(i2c: &mut I, addr: u8) {
    let _ = i2c.write(addr, &[0xff, 0x01]);
    let _ = i2c.write(addr, &[0x12, 0x80]);
}

fn wr<I: embedded_hal::i2c::I2c>(i2c: &mut I, addr: u8, reg: u8, val: u8) {
    let _ = i2c.write(addr, &[reg, val]);
}

fn ov2640_jpeg_svga<I: embedded_hal::i2c::I2c>(i2c: &mut I, addr: u8, quality: u8) {
    // Bank DSP
    wr(i2c, addr, 0xff, 0x00);
    wr(i2c, addr, 0x2c, 0xff);
    wr(i2c, addr, 0x2e, 0xdf);
    
    // Bank sensor
    wr(i2c, addr, 0xff, 0x01);
    wr(i2c, addr, 0x3c, 0x32);
    wr(i2c, addr, 0x11, 0x00);
    wr(i2c, addr, 0x09, 0x02);
    wr(i2c, addr, 0x04, 0x28);
    wr(i2c, addr, 0x13, 0xe5);
    wr(i2c, addr, 0x14, 0x48);
    wr(i2c, addr, 0x2c, 0x0c);
    wr(i2c, addr, 0x33, 0x78);
    wr(i2c, addr, 0x3a, 0x33);
    wr(i2c, addr, 0x3b, 0xfb);
    wr(i2c, addr, 0x3e, 0x00);
    wr(i2c, addr, 0x43, 0x11);
    wr(i2c, addr, 0x16, 0x10);
    wr(i2c, addr, 0x39, 0x02);
    wr(i2c, addr, 0x35, 0x88);
    wr(i2c, addr, 0x22, 0x0a);
    wr(i2c, addr, 0x37, 0x40);
    wr(i2c, addr, 0x23, 0x00);
    wr(i2c, addr, 0x34, 0xa0);
    wr(i2c, addr, 0x06, 0x02);
    wr(i2c, addr, 0x06, 0x88);
    wr(i2c, addr, 0x07, 0xc0);
    wr(i2c, addr, 0x0d, 0xb7);
    wr(i2c, addr, 0x0e, 0x01);
    wr(i2c, addr, 0x4c, 0x00);
    wr(i2c, addr, 0x4a, 0x81);
    wr(i2c, addr, 0x21, 0x99);
    wr(i2c, addr, 0x24, 0x40);
    wr(i2c, addr, 0x25, 0x38);
    wr(i2c, addr, 0x26, 0x82);
    wr(i2c, addr, 0x5c, 0x00);
    wr(i2c, addr, 0x63, 0x00);
    wr(i2c, addr, 0x46, 0x22);
    wr(i2c, addr, 0x0c, 0x3a);
    wr(i2c, addr, 0x5d, 0x55);
    wr(i2c, addr, 0x5e, 0x7d);
    wr(i2c, addr, 0x5f, 0x7d);
    wr(i2c, addr, 0x60, 0x55);
    wr(i2c, addr, 0x61, 0x70);
    wr(i2c, addr, 0x62, 0x80);
    wr(i2c, addr, 0x7c, 0x05);
    wr(i2c, addr, 0x20, 0x80);
    wr(i2c, addr, 0x28, 0x30);
    wr(i2c, addr, 0x6c, 0x00);
    wr(i2c, addr, 0x6d, 0x80);
    wr(i2c, addr, 0x6e, 0x00);
    wr(i2c, addr, 0x70, 0x02);
    wr(i2c, addr, 0x71, 0x94);
    wr(i2c, addr, 0x73, 0xc1);
    wr(i2c, addr, 0x3d, 0x34);
    wr(i2c, addr, 0x12, 0x04);
    wr(i2c, addr, 0x5a, 0x57);
    wr(i2c, addr, 0x4f, 0xbb);
    wr(i2c, addr, 0x50, 0x9c);
    
    // Bank DSP - JPEG
    wr(i2c, addr, 0xff, 0x00);
    wr(i2c, addr, 0xe0, 0x04);
    wr(i2c, addr, 0xc0, 0xc8);
    wr(i2c, addr, 0xc1, 0x96);
    wr(i2c, addr, 0x86, 0x3d);
    wr(i2c, addr, 0x50, 0x89);
    wr(i2c, addr, 0x51, 0x90);
    wr(i2c, addr, 0x52, 0x2c);
    wr(i2c, addr, 0x53, 0x00);
    wr(i2c, addr, 0x54, 0x00);
    wr(i2c, addr, 0x55, 0x88);
    wr(i2c, addr, 0x57, 0x00);
    wr(i2c, addr, 0x5a, 0xa0);
    wr(i2c, addr, 0x5b, 0x78);
    wr(i2c, addr, 0x5c, 0x00);
    wr(i2c, addr, 0xd3, 0x04);
    wr(i2c, addr, 0xe0, 0x00);
    
    // YUV422
    wr(i2c, addr, 0xff, 0x00);
    wr(i2c, addr, 0x05, 0x00);
    wr(i2c, addr, 0xda, 0x00);
    wr(i2c, addr, 0xd7, 0x03);
    wr(i2c, addr, 0xe0, 0x00);
    
    // SVGA 800x600
    wr(i2c, addr, 0xff, 0x00);
    wr(i2c, addr, 0xe0, 0x04);
    wr(i2c, addr, 0xc0, 0xc8);
    wr(i2c, addr, 0xc1, 0x96);
    wr(i2c, addr, 0x86, 0x3d);
    wr(i2c, addr, 0x50, 0x89);
    wr(i2c, addr, 0x51, 0x90);
    wr(i2c, addr, 0x52, 0x2c);
    wr(i2c, addr, 0x53, 0x00);
    wr(i2c, addr, 0x54, 0x00);
    wr(i2c, addr, 0x55, 0x88);
    wr(i2c, addr, 0x5a, 0xc8);
    wr(i2c, addr, 0x5b, 0x96);
    wr(i2c, addr, 0x5c, 0x00);
    wr(i2c, addr, 0xd3, 0x02);
    wr(i2c, addr, 0xe0, 0x00);
    
    // Quality
    wr(i2c, addr, 0xff, 0x00);
    wr(i2c, addr, 0xe0, 0x04);
    wr(i2c, addr, 0xdb, quality);
    wr(i2c, addr, 0xe0, 0x00);
    
    // Final
    wr(i2c, addr, 0xff, 0x01);
    wr(i2c, addr, 0x04, 0x28);
}
