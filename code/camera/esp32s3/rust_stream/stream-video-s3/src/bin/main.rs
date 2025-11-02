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

#[path = "../ov2640_tables.rs"]
mod ov2640_tables;

use ov2640_tables::{
    OV2640_800X600_JPEG,
    OV2640_JPEG,
    OV2640_JPEG_INIT,
    OV2640_YUV422,
};

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

    // --- Camera Power Control (Critical for XIAO ESP32S3 Sense) ---
    // The camera module needs proper power sequencing
    // Based on Arduino examples, we may need PWDN/RESET control
    println!("Configuring camera power/reset...");
    
    // For XIAO ESP32S3 Sense, the camera might be controlled via other means
    // or always powered. Let's try without explicit PWDN for now.

    println!("Camera power control configured");

    // --- XCLK = 20 MHz via LEDC PWM (camera master clock) ---
    // Arduino examples use 20MHz, let's try that
    let mut ledc = Ledc::new(peripherals.LEDC);
    ledc.set_global_slow_clock(LSGlobalClkSource::APBClk);
    
    let mut lstimer0 = ledc.timer::<LowSpeed>(timer::Number::Timer0);
    lstimer0
        .configure(timer::config::Config {
            duty: timer::config::Duty::Duty1Bit,
            clock_source: timer::LSClockSource::APBClk,
            frequency: esp_hal::time::Rate::from_mhz(20), // 20MHz like Arduino
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

    println!("LEDC configured: 20MHz XCLK on GPIO10");

    // --- Initialize OV2640 sensor ---
    let addr = 0x30; // OV2640 I2C address
    
    println!("Performing OV2640 initialization sequence (ESP-IDF style)...");
    
    // Step 1: Software reset
    println!("  Step 1: Software reset");
    ov2640_reset(&mut i2c, addr);
    delay.delay_millis(10);
    
    // Step 2: Initialize sensor (like ov2640_init in ESP-IDF driver)
    println!("  Step 2: Loading JPEG base tables");
    ov2640_load_jpeg_tables(&mut i2c, addr);
    delay.delay_millis(10);
    
    // Step 3: Configure for JPEG SVGA
    println!("  Step 3: Configuring JPEG SVGA mode");
    ov2640_set_svga_jpeg(&mut i2c, addr, 12);
    delay.delay_millis(10);

    // Verify sensor
    let mut sensor_id = [0u8; 2];
    i2c.write(addr, &[0xff, 0x01]).ok();
    i2c.write_read(addr, &[0x0a], &mut sensor_id[0..1]).ok();
    i2c.write_read(addr, &[0x0b], &mut sensor_id[1..2]).ok();
    println!("OV2640 ID: PID=0x{:02x} VER=0x{:02x}", sensor_id[0], sensor_id[1]);

    info!("OV2640 sensor initialized");

    // Additional register checks and configuration
    i2c.write(addr, &[0xff, 0x01]).ok(); // Sensor bank
    
    // Ensure COM10 has HREF/VSYNC properly configured
    // Bit 5: HREF changes to HSYNC, Bit 6: PCLK does not toggle during horizontal blank
    i2c.write(addr, &[0x15, 0x00]).ok(); // COM10: normal HREF/VSYNC
    
    // Check and set sensor active mode
    let mut reg_val = [0u8];
    i2c.write_read(addr, &[0x12], &mut reg_val).ok(); // COM7
    println!("COM7 (output format): 0x{:02X}", reg_val[0]);
    
    // Ensure sensor is not in standby
    i2c.write_read(addr, &[0x09], &mut reg_val).ok(); // COM2
    println!("COM2 (output drive): 0x{:02X}", reg_val[0]);
    
    i2c.write_read(addr, &[0x15], &mut reg_val).ok(); // COM10
    println!("COM10 (timing): 0x{:02X}", reg_val[0]);
    
    // **CRITICAL: Enable sensor output and start streaming**
    println!("Enabling sensor output...");
    i2c.write(addr, &[0xff, 0x01]).ok(); // Sensor bank
    
    // COM2 bit 0 controls output enable
    i2c.write(addr, &[0x09, 0x02]).ok(); // COM2: Output drive 2x, enable output
    
    // COM10: Enable VSYNC negative, HREF changes to HSYNC
    i2c.write(addr, &[0x15, 0x00]).ok(); // COM10: Normal operation
    
    // COM12: No scaling
    i2c.write(addr, &[0x3C, 0x00]).ok(); // COM12: No scaling
    
    // Explicitly start the sensor
    i2c.write(addr, &[0x12, 0x04]).ok(); // COM7: Output enable, JPEG mode
    
    // Try triggering a frame capture by toggling some registers
    i2c.write(addr, &[0xff, 0x00]).ok(); // DSP bank
    i2c.write(addr, &[0x05, 0x00]).ok(); // Ensure not in test pattern mode
    
    println!("Sensor registers configured for active capture");
    
    // Give camera time to stabilize (important for OV2640)
    delay.delay_millis(300);
    println!("Camera stabilization complete");

    println!("\n=== Setting up LCD_CAM ===");

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
    .with_h_enable(peripherals.GPIO47)
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
    let mut timeout_count = 0u32;

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

        // Try to wait for frame with timeout simulation
        // Note: transfer.wait() is blocking, so we can't add a real timeout here
        // In production, you'd use async or interrupts
        println!("Waiting for frame data...");
        
        // Check if we're stuck
        if frame_count > 1 && timeout_count > 3 {
            println!("\n⚠️  Multiple capture attempts failed!");
            println!("Recommendations:");
            println!("  1. Check camera is receiving light");
            println!("  2. Verify all wiring connections");
            println!("  3. Confirm camera power supply");
            println!("  4. Try power cycling the board");
            println!("\nContinuing to attempt capture...\n");
            timeout_count = 0;
        }
        
        let (result, cam, buf) = transfer.wait();
        camera = cam;
        rx_buffer = buf;

        match result {
            Ok(()) => {
                timeout_count = 0; // Reset timeout counter on success
                let buffer = unsafe { &FRAME_BUFFER[..] };

                if let Some((start, end)) = find_jpeg_range(buffer) {
                    let len = end - start;
                    println!("✓ Valid JPEG slice: {} bytes (offset {}..{})", len, start, end);

                    // Show first few bytes from the detected frame
                    print!("  Header: ");
                    for i in start..(start + 16).min(end) {
                        print!("{:02X} ", buffer[i]);
                    }
                    println!();

                    // Calculate checksum for detected slice
                    let checksum: u32 = buffer[start..end]
                        .iter()
                        .fold(0u32, |acc, &b| acc.wrapping_add(b as u32));
                    println!("  Checksum: 0x{:08X}", checksum);
                } else if let Some(start) = find_jpeg_start(buffer, 0) {
                    println!("⚠ Found JPEG start @ {} but no end marker within buffer", start);
                    print!("  Start bytes: ");
                    for i in start..(start + 16).min(buffer.len()) {
                        print!("{:02X} ", buffer[i]);
                    }
                    println!();
                } else {
                    println!("✗ No JPEG header - first bytes: {:02X} {:02X}", 
                             buffer[0], buffer[1]);
                }
            }
            Err(e) => {
                timeout_count += 1;
                println!("DMA error: {:?}", e);
            }
        }

        delay.delay_millis(1000);
    }
}

fn find_jpeg_start(buffer: &[u8], from: usize) -> Option<usize> {
    for i in from..buffer.len().saturating_sub(1) {
        if buffer[i] == 0xFF && buffer[i + 1] == 0xD8 {
            return Some(i);
        }
    }
    None
}

fn find_jpeg_end(buffer: &[u8], from: usize) -> Option<usize> {
    for i in from..buffer.len().saturating_sub(1) {
        if buffer[i] == 0xFF && buffer[i + 1] == 0xD9 {
            return Some(i + 2);
        }
    }
    None
}

fn find_jpeg_range(buffer: &[u8]) -> Option<(usize, usize)> {
    let start = find_jpeg_start(buffer, 0)?;
    let end = find_jpeg_end(buffer, start + 2)?;
    Some((start, end))
}

fn ov2640_reset<I: embedded_hal::i2c::I2c>(i2c: &mut I, addr: u8) {
    let _ = i2c.write(addr, &[0xff, 0x01]);
    let _ = i2c.write(addr, &[0x12, 0x80]);
}

fn write_table<I: embedded_hal::i2c::I2c>(i2c: &mut I, addr: u8, table: &[(u8, u8)]) {
    for &(reg, val) in table {
        if reg == 0xFF && val == 0xFF {
            break;
        }
        let _ = i2c.write(addr, &[reg, val]);
    }
}

fn ov2640_load_jpeg_tables<I: embedded_hal::i2c::I2c>(i2c: &mut I, addr: u8) {
    write_table(i2c, addr, OV2640_JPEG_INIT);
    write_table(i2c, addr, OV2640_YUV422);
    write_table(i2c, addr, OV2640_JPEG);
}

fn ov2640_set_svga_jpeg<I: embedded_hal::i2c::I2c>(i2c: &mut I, addr: u8, quality: u8) {
    let quality = quality.min(63);

    let _ = i2c.write(addr, &[0xFF, 0x01]); // Sensor bank
    let _ = i2c.write(addr, &[0x15, 0x00]); // Enable free-running VSYNC

    write_table(i2c, addr, OV2640_800X600_JPEG);

    let _ = i2c.write(addr, &[0xFF, 0x00]); // DSP bank
    let _ = i2c.write(addr, &[0x44, quality]); // JPEG quality
}
