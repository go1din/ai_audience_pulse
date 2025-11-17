#![no_std]
#![no_main]

extern crate alloc;

use alloc::string::String;
use esp_backtrace as _; // provide panic handler & backtrace output
use esp_backtrace as _; // provides the panic handler + prints via UART

use embassy_executor::Spawner;
use embassy_time::{with_timeout, Timer};
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
    rng::Rng,
    timer::timg::TimerGroup,
};
use esp_println::println;
use heapless::String as HeaplessString;
use static_cell::StaticCell;
use stream_video_s3::{ov2640_tables, psram_log};

use ov2640_tables::{
    OV2640_800X600_JPEG,
    OV2640_JPEG,
    OV2640_JPEG_INIT,
    OV2640_YUV422,
};

const HTTP_TASK_POOL_SIZE: usize = 1;
const FRAME_SIZE: usize = 28 * 1024;

// Two frame buffers for simple ping-pong (double-buffered) capture.
static mut FRAME_BUFFER0: [u8; FRAME_SIZE] = [0u8; FRAME_SIZE];
static mut FRAME_BUFFER1: [u8; FRAME_SIZE] = [0u8; FRAME_SIZE];

const DESC_COUNT: usize = 32;

#[unsafe(link_section = ".dram2_uninit")]
static mut DMA_DESCRIPTORS0: [esp_hal::dma::DmaDescriptor; DESC_COUNT] =
    [esp_hal::dma::DmaDescriptor::EMPTY; DESC_COUNT];
#[unsafe(link_section = ".dram2_uninit")]
static mut DMA_DESCRIPTORS1: [esp_hal::dma::DmaDescriptor; DESC_COUNT] =
    [esp_hal::dma::DmaDescriptor::EMPTY; DESC_COUNT];

esp_bootloader_esp_idf::esp_app_desc!();


// Lower JPEG quality to reduce frame size and allocation pressure
const CAMERA_JPEG_QUALITY: u8 = 20;
// Load Wi-Fi credentials generated during build
const WIFI_SSID: &str = env!("WIFI_SSID");
const WIFI_PASS: &str = env!("WIFI_PASS");
const TIMEOUT: embassy_time::Duration = embassy_time::Duration::from_secs(30);

#[embassy_executor::task]
async fn net_task(
    mut runner: embassy_net::Runner<'static, esp_wifi::wifi::WifiDevice<'static>>,
) {
    runner.run().await;
}

#[embassy_executor::task]
async fn wifi_task(
    mut controller: esp_wifi::wifi::WifiController<'static>,
    stack: embassy_net::Stack<'static>,
) -> ! {
    use esp_wifi::wifi::{ClientConfiguration, Configuration as WifiConfiguration, Protocol, WifiEvent};
    use esp_println::println;

    println!(
        "[wifi] starting with SSID '{}' and password '{}'",
        WIFI_SSID, WIFI_PASS
    );
    Timer::after_millis(1000).await;

    let _ = controller.set_protocol(Protocol::P802D11B | Protocol::P802D11BG | Protocol::P802D11BGN);
    let _ = controller.set_configuration(&WifiConfiguration::Client(ClientConfiguration {
        ssid: WIFI_SSID.into(),
        password: WIFI_PASS.into(),
        ..Default::default()
    }));

    loop {
        match controller.start() {
            Ok(()) => {
                println!("[wifi] driver started");
                break;
            }
            Err(e) => {
                println!("[wifi] start error: {:?}", e);
                Timer::after_millis(1000).await;
            }
        }
    }

    loop {
        println!("[wifi] connecting...");
        match controller.connect() {
            Ok(()) => {
                println!("[wifi] connected");
                match with_timeout(TIMEOUT, stack.wait_config_up()).await {
                    Ok(()) => {
                        if let Some(v4) = stack.config_v4() {
                            println!("[wifi] got IPv4 address: {}", v4.address.address());
                        } else {
                            println!("[wifi] connected but DHCP did not provide an IPv4 address");
                        }
                    }
                    Err(_) => println!(
                        "[wifi] connected but timed out waiting for DHCP ({}s)",
                        TIMEOUT.as_secs()
                    ),
                }
                controller.wait_for_event(WifiEvent::StaDisconnected).await;
                println!("[wifi] disconnected; retrying");
                let _ = with_timeout(TIMEOUT, stack.wait_config_down()).await;
            }
            Err(e) => {
                println!("[wifi] connect error: {:?}", e);
            }
        }
        Timer::after_millis(2000).await;
    }
}

#[embassy_executor::task]
async fn mjpeg_task(
    stack: embassy_net::Stack<'static>,
    mut camera: Camera<'static>,
    mut rx0: DmaRxBuf,
    mut rx1: DmaRxBuf,
) -> ! {
    use embassy_net::tcp::TcpSocket;
    use embedded_io_async::Write as _;
    let mut rx_buf = [0u8; 1024];
    // let mut tx_buf = [0u8; 8024];
    let mut tx_buf = [0u8; 16384];
    println!("MJPEG streaming server listening on port 80 (path /)");
    loop {
        let mut socket = TcpSocket::new(stack, &mut rx_buf, &mut tx_buf);
        if let Err(e) = socket.accept(80).await {
            println!("mjpeg: accept error {:?}", e);
            continue;
        }
        println!("mjpeg: client connected");
        // Read minimal request (ignore contents). Attempt to consume until blank line or buffer fills.
        let mut req_buf = [0u8; 256];
        let mut total = 0usize;
        loop {
            match socket.read(&mut req_buf[total..]).await {
                Ok(0) | Err(_) => break,
                Ok(n) => {
                    total += n;
                    if total >= 4 && req_buf[..total].windows(4).any(|w| w == b"\r\n\r\n") { break; }
                    if total == req_buf.len() { break; }
                }
            }
        }
        // Write HTTP header for multipart stream
        let header = b"HTTP/1.1 200 OK\r\nContent-Type: multipart/x-mixed-replace; boundary=frame\r\nConnection: close\r\nCache-Control: no-cache\r\nPragma: no-cache\r\n\r\n";
        if let Err(e) = socket.write_all(header).await {
            println!("mjpeg: header write error {:?}", e);
            continue;
        }
        // First frame: capture into buffer 0 and wait so we have
        // something to send before starting the ping-pong loop.
        match camera.receive(rx0) {
            Ok(t) => {
                let (result, cam, buf) = t.wait();
                camera = cam;
                rx0 = buf;
                if let Err(err) = result {
                    println!("mjpeg: initial DMA error during frame capture: {:?}", err);
                    continue;
                }
            }
            Err((e, cam, buf)) => {
                println!("mjpeg: failed to start initial transfer: {:?}", e);
                camera = cam;
                rx0 = buf;
                continue;
            }
        }

        // Stream frames: double-buffered ping-pong between FRAME_BUFFER0 and FRAME_BUFFER1,
        // similar to Arduino's fb_count=2 / CAMERA_GRAB_LATEST behavior.
        let mut current_is_buf0 = true;
        loop {
            if current_is_buf0 {
                // Start capture into buffer 1 (rx1), stream from buffer 0.
                let transfer = match camera.receive(rx1) {
                    Ok(t) => t,
                    Err((e, cam, buf)) => {
                        println!("mjpeg: failed to start transfer (buf1): {:?}", e);
                        camera = cam;
                        rx1 = buf;
                        continue;
                    }
                };

                // Stream the completed frame from FRAME_BUFFER0.
                let buffer = unsafe { &FRAME_BUFFER0[..] };
                if stream_jpeg_frame(&mut socket, buffer).await.is_err() {
                    // On any socket error, drop the connection.
                    let (result, cam, buf) = transfer.wait();
                    camera = cam;
                    rx1 = buf;
                    if let Err(err) = result {
                        println!("mjpeg: DMA error during frame capture (buf1): {:?}", err);
                    }
                    break;
                }

                // Wait for capture into buffer 1 to complete.
                let (result, cam, buf) = transfer.wait();
                camera = cam;
                rx1 = buf;
                if let Err(err) = result {
                    println!("mjpeg: DMA error during frame capture (buf1): {:?}", err);
                    break;
                }

                current_is_buf0 = false;
            } else {
                // Start capture into buffer 0 (rx0), stream from buffer 1.
                let transfer = match camera.receive(rx0) {
                    Ok(t) => t,
                    Err((e, cam, buf)) => {
                        println!("mjpeg: failed to start transfer (buf0): {:?}", e);
                        camera = cam;
                        rx0 = buf;
                        continue;
                    }
                };

                // Stream the completed frame from FRAME_BUFFER1.
                let buffer = unsafe { &FRAME_BUFFER1[..] };
                if stream_jpeg_frame(&mut socket, buffer).await.is_err() {
                    let (result, cam, buf) = transfer.wait();
                    camera = cam;
                    rx0 = buf;
                    if let Err(err) = result {
                        println!("mjpeg: DMA error during frame capture (buf0): {:?}", err);
                    }
                    break;
                }

                // Wait for capture into buffer 0 to complete.
                let (result, cam, buf) = transfer.wait();
                camera = cam;
                rx0 = buf;
                if let Err(err) = result {
                    println!("mjpeg: DMA error during frame capture (buf0): {:?}", err);
                    break;
                }

                current_is_buf0 = true;
            }
        }
        println!("mjpeg: client disconnected");
    }
}

#[esp_hal_embassy::main]
async fn main(spawner: Spawner) -> ! {
    let p = esp_hal::init(esp_hal::Config::default());

    // Allocate main heap + attempt to extend with PSRAM
    esp_alloc::heap_allocator!(size: 72 * 1024);
    unsafe {
        psram_log::log_and_init_psram(&p.PSRAM);
    }

    use log::LevelFilter;
    esp_println::logger::init_logger(LevelFilter::Info);
    println!("=== ESP32-S3 Camera Wi-Fi Stream ===");

    let mut rng = Rng::new(p.RNG);
    let timg0 = TimerGroup::new(p.TIMG0);
    let timg1 = TimerGroup::new(p.TIMG1);
    esp_hal_embassy::init(timg1.timer0);

    // Wi-Fi setup
    static WIFI_CTRL: StaticCell<esp_wifi::EspWifiController<'static>> = StaticCell::new();
    let wifi_ctrl = WIFI_CTRL
        .init(esp_wifi::init(timg0.timer0, rng).expect("wifi init"));
    let (controller, interfaces) = esp_wifi::wifi::new(wifi_ctrl, p.WIFI).expect("wifi new");

    static NET_STACK_RES: StaticCell<embassy_net::StackResources<{ HTTP_TASK_POOL_SIZE + 3 }>> =
        StaticCell::new();
    let (stack, runner) = {
        let cfg = embassy_net::Config::dhcpv4(Default::default());
        let seed = ((rng.random() as u64) << 32) | rng.random() as u64;
        embassy_net::new(
            interfaces.sta,
            cfg,
            NET_STACK_RES.init(embassy_net::StackResources::new()),
            seed,
        )
    };

    spawner.spawn(net_task(runner)).ok();
    spawner.spawn(wifi_task(controller, stack)).ok();

    // Camera setup
    let delay = Delay::new();

    let sda = p.GPIO40;
    let scl = p.GPIO39;
    let mut i2c = I2c::new(p.I2C0, esp_hal::i2c::master::Config::default())
        .unwrap()
        .with_sda(sda)
        .with_scl(scl);

    println!("I2C initialized");
    println!("Configuring camera power/reset...");
    println!("Camera power control configured");

    let mut ledc = Ledc::new(p.LEDC);
    ledc.set_global_slow_clock(LSGlobalClkSource::APBClk);
    let mut lstimer0 = ledc.timer::<LowSpeed>(timer::Number::Timer0);
    lstimer0
        .configure(timer::config::Config {
            duty: timer::config::Duty::Duty1Bit,
            clock_source: timer::LSClockSource::APBClk,
            frequency: esp_hal::time::Rate::from_mhz(24),
        })
        .unwrap();
    let mut channel0 = ledc.channel(channel::Number::Channel0, p.GPIO10);
    channel0
        .configure(channel::config::Config {
            timer: &lstimer0,
            duty_pct: 50,
            pin_config: channel::config::PinConfig::PushPull,
        })
        .unwrap();

    println!("LEDC configured: 24MHz XCLK on GPIO10");

    let addr = 0x30;
    println!("Performing OV2640 initialization sequence (ESP-IDF style)...");
    println!("  Step 1: Software reset");
    ov2640_reset(&mut i2c, addr);
    delay.delay_millis(10);

    println!("  Step 2: Loading JPEG base tables");
    ov2640_load_jpeg_tables(&mut i2c, addr);
    delay.delay_millis(10);

    println!("  Step 2a: Forcing output selector (YUV422 path)");
    ov2640_force_output_selector(&mut i2c, addr);
    delay.delay_millis(10);

    println!("  Step 3: Configuring JPEG SVGA mode (quality={})", CAMERA_JPEG_QUALITY);
    ov2640_set_svga_jpeg(&mut i2c, addr, CAMERA_JPEG_QUALITY);
    delay.delay_millis(10);

    println!("  Step 4: Re-enabling auto controls (AWB/AGC/AEC)");
    ov2640_re_enable_auto_controls(&mut i2c, addr);
    delay.delay_millis(10);

    println!("  Step 5: Setting vertical flip (VFLIP)");
    ov2640_set_vflip(&mut i2c, addr, true);
    delay.delay_millis(10);

    let mut sensor_id = [0u8; 2];
    i2c.write(addr, &[0xff, 0x01]).ok();
    i2c.write_read(addr, &[0x0a], &mut sensor_id[0..1]).ok();
    i2c.write_read(addr, &[0x0b], &mut sensor_id[1..2]).ok();
    println!("OV2640 ID: PID=0x{:02x} VER=0x{:02x}", sensor_id[0], sensor_id[1]);

    i2c.write(addr, &[0xff, 0x01]).ok();
    let mut reg_val = [0u8];
    i2c.write_read(addr, &[0x12], &mut reg_val).ok();
    println!("COM7 (output format): 0x{:02X}", reg_val[0]);
    i2c.write_read(addr, &[0x09], &mut reg_val).ok();
    println!("COM2 (output drive): 0x{:02X}", reg_val[0]);
    i2c.write_read(addr, &[0x15], &mut reg_val).ok();
    println!("COM10 (timing): 0x{:02X}", reg_val[0]);

    println!("Enabling sensor output...");
    i2c.write(addr, &[0xff, 0x01]).ok();
    i2c.write(addr, &[0x09, 0x02]).ok();
    i2c.write(addr, &[0x15, 0x00]).ok();
    i2c.write(addr, &[0x3C, 0x00]).ok();
    i2c.write(addr, &[0x12, 0x04]).ok();
    i2c.write(addr, &[0xff, 0x00]).ok();
    i2c.write(addr, &[0x05, 0x00]).ok();

    println!("Sensor registers configured for active capture");
    delay.delay_millis(300);
    println!("Camera stabilization complete");

    let lcd_cam = LcdCam::new(p.LCD_CAM);
    let camera = Camera::new(lcd_cam.cam, p.DMA_CH0, CamConfig::default())
        .unwrap()
        .with_pixel_clock(p.GPIO13)
        .with_vsync(p.GPIO38)
        .with_h_enable(p.GPIO47)
        .with_data0(p.GPIO15)
        .with_data1(p.GPIO17)
        .with_data2(p.GPIO18)
        .with_data3(p.GPIO16)
        .with_data4(p.GPIO14)
        .with_data5(p.GPIO12)
        .with_data6(p.GPIO11)
        .with_data7(p.GPIO48);

    // Two DMA RX buffers, each backed by its own descriptor array and frame buffer,
    // to mirror Arduino's fb_count = 2 double-buffered behavior.
    let (rx0, rx1) = unsafe {
        let rx0 = DmaRxBuf::new(
            &mut *core::ptr::addr_of_mut!(DMA_DESCRIPTORS0),
            &mut *core::ptr::addr_of_mut!(FRAME_BUFFER0),
        )
        .unwrap();
        let rx1 = DmaRxBuf::new(
            &mut *core::ptr::addr_of_mut!(DMA_DESCRIPTORS1),
            &mut *core::ptr::addr_of_mut!(FRAME_BUFFER1),
        )
        .unwrap();
        (rx0, rx1)
    };

    // Single HTTP MJPEG task that both captures frames and streams them,
    // using rx0/rx1 as a ping-pong (double-buffered) pipeline.
    spawner.spawn(mjpeg_task(stack, camera, rx0, rx1)).ok();

    // Nothing else to do in main; all work happens in Embassy tasks.
    loop {
        Timer::after_millis(1000).await;
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

fn jpeg_slice_from<'a>(buffer: &'a [u8]) -> Option<&'a [u8]> {
    let (start, end) = find_jpeg_range(buffer)?;
    let full_len = end - start;
    let max_len = FRAME_SIZE.saturating_sub(start);
    let copy_len = full_len.min(max_len);
    Some(&buffer[start..start + copy_len])
}

async fn stream_jpeg_frame<S>(socket: &mut S, buffer: &[u8]) -> Result<(), ()>
where
    S: embedded_io_async::Write,
{
    let frame = match jpeg_slice_from(buffer) {
        Some(f) => f,
        None => {
            println!("mjpeg: no complete JPEG frame detected");
            return Ok(());
        }
    };

    let copy_len = frame.len();
    use core::fmt::Write as _;
    let mut hdr: HeaplessString<96> = HeaplessString::new();
    if write!(
        &mut hdr,
        "--frame\r\nContent-Type: image/jpeg\r\nContent-Length: {}\r\n\r\n",
        copy_len
    )
    .is_err()
    {
        return Err(());
    }

    socket.write_all(hdr.as_bytes()).await.map_err(|_| ())?;
    socket.write_all(frame).await.map_err(|_| ())?;
    socket.write_all(b"\r\n").await.map_err(|_| ())?;

    Ok(())
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

fn ov2640_force_output_selector<I: embedded_hal::i2c::I2c>(i2c: &mut I, addr: u8) {
    // Force the correct output selector (some firmwares flip this)
    // DSP bank
    let _ = i2c.write(addr, &[0xFF, 0x00]);
    let _ = i2c.write(addr, &[0xDA, 0x10]);   // YUV422 path (required for JPEG pipeline)
    // Note: If still greenish, try UV-swap: change 0xDA, 0x10 to 0xDA, 0x11
    let _ = i2c.write(addr, &[0xD7, 0x03]);   // auto features enabled (as in esp32-camera)
}

fn ov2640_re_enable_auto_controls<I: embedded_hal::i2c::I2c>(i2c: &mut I, addr: u8) {
    // Re-enable auto white balance / exposure / gain after the tables
    // Sensor bank
    let _ = i2c.write(addr, &[0xFF, 0x01]);
    let _ = i2c.write(addr, &[0x13, 0xE7]);   // COM8: AWB|AGC|AEC ON
}

fn ov2640_set_svga_jpeg<I: embedded_hal::i2c::I2c>(i2c: &mut I, addr: u8, quality: u8) {
    let quality = quality.min(63);
    let _ = i2c.write(addr, &[0xFF, 0x01]);
    let _ = i2c.write(addr, &[0x15, 0x00]);
    write_table(i2c, addr, OV2640_800X600_JPEG);
    let _ = i2c.write(addr, &[0xFF, 0x00]);
    let _ = i2c.write(addr, &[0x44, quality]);
}

fn ov2640_set_vflip<I: embedded_hal::i2c::I2c>(i2c: &mut I, addr: u8, enable: bool) {
    // Switch to sensor bank and mirror Arduino/esp32-camera behaviour:
    // enable VFLIP and VREF bits without touching UV order.
    let _ = i2c.write(addr, &[0xFF, 0x01]);
    let mut reg04 = [0u8];
    let _ = i2c.write_read(addr, &[0x04], &mut reg04);
    let mut new_val = reg04[0];
    if enable {
        // Set VREF_EN (0x10) and VFLIP_IMG (0x40)
        new_val |= 0x50;
    } else {
        new_val &= !0x50;
    }
    let _ = i2c.write(addr, &[0x04, new_val]);
}
