#![no_std]
#![no_main]

extern crate alloc;

use alloc::{boxed::Box, string::String, vec::Vec};
use core::{fmt::Write, sync::atomic::{AtomicU32, Ordering}};

use embassy_executor::Spawner;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::mutex::Mutex;
use embassy_time::Timer;
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
use picoserve as ps;
use ps::response::StatusCode;
use embedded_io_async::Write as AsyncWrite;
use static_cell::StaticCell;

#[path = "../ov2640_tables.rs"]
mod ov2640_tables;

use ov2640_tables::{
    OV2640_800X600_JPEG,
    OV2640_JPEG,
    OV2640_JPEG_INIT,
    OV2640_YUV422,
};

const HTTP_TASK_POOL_SIZE: usize = 1;
const FRAME_SIZE: usize = 64 * 1024;
#[unsafe(link_section = ".dram2_uninit")]
static mut FRAME_BUFFER: [u8; FRAME_SIZE] = [0u8; FRAME_SIZE];
static mut HTTP_RESPONSE_BUFFERS_PTR: [*mut [u8; FRAME_SIZE]; HTTP_TASK_POOL_SIZE] =
    [core::ptr::null_mut(); HTTP_TASK_POOL_SIZE];
const DESC_COUNT: usize = 32;
#[unsafe(link_section = ".dram2_uninit")]
static mut DMA_DESCRIPTORS: [esp_hal::dma::DmaDescriptor; DESC_COUNT] =
    [esp_hal::dma::DmaDescriptor::EMPTY; DESC_COUNT];

esp_bootloader_esp_idf::esp_app_desc!();

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    esp_println::println!("panic: {:?}", info);
    loop {}
}

struct FrameStore {
    start: usize,
    len: usize,
    checksum: u32,
}

impl FrameStore {
    const fn new() -> Self {
        Self {
            start: 0,
            len: 0,
            checksum: 0,
        }
    }
}

static FRAME_STORE: Mutex<CriticalSectionRawMutex, FrameStore> = Mutex::new(FrameStore::new());
static FRAME_COUNTER: AtomicU32 = AtomicU32::new(0);

const DEFAULT_CAPTURE_INTERVAL_MS: u64 = 1_000;
const WIFI_SSID: &str = match option_env!("WIFI_SSID") {
    Some(val) => val,
    None => "ESP32_WIFI",
};
const WIFI_PASS: &str = match option_env!("WIFI_PASS") {
    Some(val) => val,
    None => "password",
};
const TIMEOUT: embassy_time::Duration = embassy_time::Duration::from_secs(5);
const FRAME_NOT_READY: &[u8] = b"frame not ready\n";

enum FrameBody<'a> {
    Message(&'static [u8]),
    Jpeg(&'a [u8]),
}

impl<'a> ps::response::Content for FrameBody<'a> {
    fn content_type(&self) -> &'static str {
        match self {
            FrameBody::Message(_) => "text/plain; charset=utf-8",
            FrameBody::Jpeg(_) => "image/jpeg",
        }
    }

    fn content_length(&self) -> usize {
        match self {
            FrameBody::Message(bytes) => bytes.len(),
            FrameBody::Jpeg(bytes) => bytes.len(),
        }
    }

    async fn write_content<W: AsyncWrite>(
        self,
        mut writer: W,
    ) -> Result<(), W::Error> {
        let bytes = match self {
            FrameBody::Message(bytes) => bytes,
            FrameBody::Jpeg(bytes) => bytes,
        };
        writer.write_all(bytes).await
    }
}

fn http_frame_buffer_slice(start: usize, end: usize) -> &'static [u8] {
    unsafe { &FRAME_BUFFER[start..end] }
}

fn init_http_response_buffers() {
    for idx in 0..HTTP_TASK_POOL_SIZE {
        unsafe {
            if HTTP_RESPONSE_BUFFERS_PTR[idx].is_null() {
                HTTP_RESPONSE_BUFFERS_PTR[idx] =
                    Box::leak(Box::new([0u8; FRAME_SIZE])) as *mut [u8; FRAME_SIZE];
            }
        }
    }
}

fn http_response_buffer(worker_id: usize) -> &'static mut [u8; FRAME_SIZE] {
    let ptr = unsafe { HTTP_RESPONSE_BUFFERS_PTR[worker_id] };
    assert!(!ptr.is_null(), "HTTP response buffers not initialised");
    unsafe { &mut *ptr }
}


fn capture_interval_millis() -> u64 {
    match option_env!("CAPTURE_INTERVAL_MS") {
        Some(val) => val
            .parse::<u64>()
            .ok()
            .filter(|ms| *ms > 0)
            .unwrap_or(DEFAULT_CAPTURE_INTERVAL_MS),
        None => DEFAULT_CAPTURE_INTERVAL_MS,
    }
}

#[embassy_executor::task]
async fn net_task(
    mut runner: embassy_net::Runner<'static, esp_wifi::wifi::WifiDevice<'static>>,
) {
    runner.run().await;
}

#[embassy_executor::task]
async fn wifi_task(mut controller: esp_wifi::wifi::WifiController<'static>) -> ! {
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
                controller.wait_for_event(WifiEvent::StaDisconnected).await;
                println!("[wifi] disconnected; retrying");
            }
            Err(e) => {
                println!("[wifi] connect error: {:?}", e);
            }
        }
        Timer::after_millis(2000).await;
    }
}

#[embassy_executor::task]
async fn ip_reporter_task(stack: embassy_net::Stack<'static>) {
    stack.wait_config_up().await;
    if let Some(v4) = stack.config_v4() {
        esp_println::println!("Wi-Fi connected with IPv4: {:?}", v4);
        esp_println::println!("🚀🐄🚀🐄🚀🐄🚀🐄🚀🐄🚀🐄🚀🐄🚀🐄🚀🐄🚀🐄");
    }
    loop {
        Timer::after_millis(60_000).await;
    }
}

#[embassy_executor::task]
async fn capture_task(
    mut camera: Camera<'static>,
    mut rx_buffer: DmaRxBuf,
) -> ! {
    let interval_ms = capture_interval_millis();
    println!("[capture] interval set to {} ms", interval_ms);

    loop {
        FRAME_COUNTER.fetch_add(1, Ordering::Relaxed);
        println!("[Frame {}] Starting capture...", FRAME_COUNTER.load(Ordering::Relaxed));

        let transfer = match camera.receive(rx_buffer) {
            Ok(t) => t,
            Err((e, cam, buf)) => {
                println!("Failed to start transfer: {:?}", e);
                camera = cam;
                rx_buffer = buf;
                Timer::after_millis(interval_ms).await;
                continue;
            }
        };

        println!("Waiting for frame data...");
        let (result, cam, buf) = transfer.wait();
        camera = cam;
        rx_buffer = buf;

        match result {
            Ok(()) => {
                let buffer = unsafe { &FRAME_BUFFER[..] };
                if let Some((start, end)) = find_jpeg_range(buffer) {
                    let full_len = end - start;
                    let max_len = FRAME_SIZE.saturating_sub(start);
                    let copy_len = full_len.min(max_len);
                    let checksum: u32 = buffer[start..start + copy_len]
                        .iter()
                        .fold(0u32, |acc, &b| acc.wrapping_add(b as u32));
                    println!(
                        "✓ Captured JPEG: {} bytes (serving {} bytes, checksum 0x{:08X})",
                        full_len, copy_len, checksum
                    );
                    if copy_len < full_len {
                        println!(
                            "⚠ Truncated frame: buffer can store {} of {} bytes",
                            copy_len, full_len
                        );
                    }
                    let mut store = FRAME_STORE.lock().await;
                    store.start = start;
                    store.len = copy_len;
                    store.checksum = checksum;
                } else {
                    println!("⚠ No complete JPEG frame detected");
                    let mut store = FRAME_STORE.lock().await;
                    store.len = 0;
                    store.start = 0;
                    store.checksum = 0;
                }
            }
            Err(e) => {
                println!("DMA error: {:?}", e);
            }
        }

        Timer::after_millis(interval_ms).await;
    }
}

async fn http_frame_route(worker_id: usize) -> impl ps::response::IntoResponse {
    let store = FRAME_STORE.lock().await;
    let len = store.len;
    let start = store.start;
    let checksum = store.checksum;
    drop(store);

    if len == 0 {
        return ps::response::Response::new(
            StatusCode::SERVICE_UNAVAILABLE,
            FrameBody::Message(FRAME_NOT_READY),
        );
    }

    let end = start.saturating_add(len).min(FRAME_SIZE);
    let copy_len = end.saturating_sub(start);
    let src = http_frame_buffer_slice(start, end);
    let dest = http_response_buffer(worker_id);
    dest[..copy_len].copy_from_slice(src);
    let body_slice = &dest[..copy_len];
    println!("HTTP /frame.jpg len={} checksum=0x{:08X}", copy_len, checksum);
    ps::response::Response::new(StatusCode::OK, FrameBody::Jpeg(body_slice))
}

async fn http_status_route() -> impl ps::response::IntoResponse {
    let store = FRAME_STORE.lock().await;
    let count = FRAME_COUNTER.load(Ordering::Relaxed);
    let mut resp = String::new();
    let _ = write!(
        &mut resp,
        "frames={} start={} last_len={} checksum=0x{:08X}\n",
        count,
        store.start,
        store.len,
        store.checksum
    );
    let mut body = Vec::with_capacity(resp.len());
    body.extend_from_slice(resp.as_bytes());
    (StatusCode::OK, ("Content-Type", "text/plain"), body,)
}

#[embassy_executor::task(pool_size = HTTP_TASK_POOL_SIZE)]
async fn http_task(
    worker_id: usize,
    stack: embassy_net::Stack<'static>,
) -> ! {
    use ps::routing::get;

    let frame_route = {
        let worker_id = worker_id;
        move || http_frame_route(worker_id)
    };

    let app = ps::Router::new()
        .route("/frame.jpg", get(frame_route))
        .route("/status", get(http_status_route))
        .route("/", get(|| async { "OK\n" }));

    let cfg = ps::Config::new(ps::Timeouts {
        start_read_request: Some(TIMEOUT),
        persistent_start_read_request: Some(TIMEOUT),
        read_request: Some(TIMEOUT),
        write: Some(TIMEOUT),
    });

    let mut tcp_rx = [0u8; 2048];
    let mut tcp_tx = [0u8; 2048];
    let mut http_buf = [0u8; 2048];

    println!("HTTP server listening on port 80 (worker #{})", worker_id);
    ps::listen_and_serve(
        "camera-http",
        &app,
        &cfg,
        stack,
        80,
        &mut tcp_rx,
        &mut tcp_tx,
        &mut http_buf,
    )
    .await
}

#[esp_hal_embassy::main]
async fn main(spawner: Spawner) -> ! {
    let p = esp_hal::init(esp_hal::Config::default());

    esp_alloc::heap_allocator!(size: 160 * 1024);
    {
        let (psram_start, psram_size) = esp_hal::psram::psram_raw_parts(&p.PSRAM);
        if psram_size >= core::mem::size_of::<usize>() * 2 {
            unsafe {
                esp_alloc::HEAP.add_region(esp_alloc::HeapRegion::new(
                    psram_start,
                    psram_size,
                    esp_alloc::MemoryCapability::External.into(),
                ));
            }
            println!("PSRAM heap added: {} bytes", psram_size);
        } else {
            println!(
                "PSRAM not detected (size {}), skipping external heap region",
                psram_size
            );
        }
    }

    esp_println::logger::init_logger(log::LevelFilter::Info);
    println!("=== ESP32-S3 Camera Wi-Fi Stream ===");

    init_http_response_buffers();

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
    spawner.spawn(wifi_task(controller)).ok();
    spawner.spawn(ip_reporter_task(stack)).ok();
    for worker_id in 0..HTTP_TASK_POOL_SIZE {
        spawner.spawn(http_task(worker_id, stack)).ok();
    }

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
            frequency: esp_hal::time::Rate::from_mhz(20),
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

    println!("LEDC configured: 20MHz XCLK on GPIO10");

    let addr = 0x30;
    println!("Performing OV2640 initialization sequence (ESP-IDF style)...");
    println!("  Step 1: Software reset");
    ov2640_reset(&mut i2c, addr);
    delay.delay_millis(10);

    println!("  Step 2: Loading JPEG base tables");
    ov2640_load_jpeg_tables(&mut i2c, addr);
    delay.delay_millis(10);

    println!("  Step 3: Configuring JPEG SVGA mode");
    ov2640_set_svga_jpeg(&mut i2c, addr, 12);
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

    let rx_buffer = unsafe {
        DmaRxBuf::new(
            &mut *core::ptr::addr_of_mut!(DMA_DESCRIPTORS),
            &mut *core::ptr::addr_of_mut!(FRAME_BUFFER),
        )
        .unwrap()
    };

    spawner.spawn(capture_task(camera, rx_buffer)).ok();

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
    let _ = i2c.write(addr, &[0xFF, 0x01]);
    let _ = i2c.write(addr, &[0x15, 0x00]);
    write_table(i2c, addr, OV2640_800X600_JPEG);
    let _ = i2c.write(addr, &[0xFF, 0x00]);
    let _ = i2c.write(addr, &[0x44, quality]);
}
