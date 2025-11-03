# Xiao ESP32-S3 Camera Integration

## Overview

This project integrates camera support for the Xiao ESP32-S3 development board with OV2640 camera sensor. The implementation initializes the camera hardware and prepares for JPEG frame capture.

## Current Status

✅ **Implemented:**
- I2C (SCCB) interface for camera sensor communication
- LEDC peripheral configured for 20MHz XCLK camera clock on GPIO10
- OV2640 sensor initialization for JPEG SVGA mode (800x600)
- Logging infrastructure

⏳ **To Be Implemented:**
- LCD_CAM peripheral configuration for frame capture
- DMA buffer setup for continuous JPEG frame capture
- Frame synchronization using VSYNC signals
- SD card or network streaming of captured frames

## Pin Configuration (Xiao ESP32-S3 + Camera)

| Function | GPIO | Description |
|----------|------|-------------|
| SIOD (SDA) | 40 | I2C Data line for camera sensor |
| SIOC (SCL) | 39 | I2C Clock line for camera sensor |
| XCLK | 10 | 20MHz master clock for camera |
| PCLK | 13 | Pixel clock input |
| VSYNC | 38 | Vertical sync (frame delimiter) |
| HREF | 47 | Horizontal reference |
| D0-D7 | 15,17,18,16,14,12,11,48 | 8-bit parallel data bus |

## Hardware Setup

1. **Xiao ESP32-S3** development board
2. **OV2640** camera module (JPEG-capable)
3. Proper wiring according to the pin configuration above

## Camera Configuration

- **Mode:** JPEG compression
- **Resolution:** SVGA (800x600)
- **Quality:** 12 (adjustable, range 0-63, lower = better quality)
- **I2C Address:** 0x30 (7-bit)
- **Clock:** 20MHz XCLK via LEDC PWM

## Building

```bash
cargo build
```

## Flashing

Use `espflash` or your preferred ESP32 flashing tool:

```bash
cargo espflash flash --monitor
```

## Next Steps for Full Camera Implementation

The current implementation provides the foundation. To complete camera capture functionality:

### 1. LCD_CAM Peripheral Configuration
The `esp-hal` 1.0.0 Camera API needs to be properly configured with:
- Data pins (8-bit parallel interface)
- Control pins (PCLK, VSYNC, HREF)
- JPEG pixel format
- Appropriate timing and synchronization settings

### 2. DMA Buffer Management
- Allocate ring buffers for continuous capture (currently stubbed with 16KB chunks)
- Configure DMA descriptors
- Handle buffer swapping during frame capture

### 3. Frame Capture Loop
- Start DMA transfer
- Wait for VSYNC to signal frame end
- Process captured JPEG data
- Stream to SD card or network

### 4. Storage/Streaming
- SD card SPI interface for local storage
- WiFi streaming for remote viewing
- MJPEG stream format for compatibility

## Notes

- The project uses `esp-hal` 1.0.0 which has a different API than previous versions
- Embassy async runtime was considered but simplified to blocking implementation for compatibility
- The OV2640 sensor configuration is minimal; extend with full register tables for production use
- PSRAM can be enabled for larger frame buffers if needed

## References

- [esp-hal documentation](https://docs.esp-rs.org/esp-hal/)
- [OV2640 datasheet](https://www.ov.com/en/product/ov2640/)
- [Xiao ESP32-S3 Sense documentation](https://wiki.seeedstudio.com/xiao_esp32s3_getting_started/)

## Upstream `esp32-camera` Example Mapping

The official C driver uses a `camera_config_t` struct to declare pinout and capture parameters. Below is a mapping from that struct to our Rust `main.rs` initialization so you can quickly align settings or spot divergences that might affect color or stability.

| Field (`camera_config_t`) | Upstream Example Value (S3 WROOM) | Rust Equivalent / Status |
|---------------------------|------------------------------------|---------------------------|
| `pin_pwdn`                | 38 / -1 (board dependent)          | Not currently controlled (power-down pin not wired) |
| `pin_reset`               | -1 (software reset)                | Manual SCCB reset via `ov2640_reset` (0x12 <- 0x80) |
| `pin_xclk`                | 15 (S3 WROOM)                      | GPIO10 with LEDC 20MHz (`channel0`) – different pin chosen for Xiao layout |
| `pin_sccb_sda`            | 4                                  | GPIO40 (Xiao) |
| `pin_sccb_scl`            | 5                                  | GPIO39 (Xiao) |
| `pin_d0..d7`              | 11,9,8,10,12,18,17,16              | D0..D7 mapped to 15,17,18,16,14,12,11,48 (custom Xiao routing) |
| `pin_vsync`               | 6                                  | GPIO38 |
| `pin_href`                | 7                                  | GPIO47 |
| `pin_pclk`                | 13                                 | GPIO13 (matches upstream PCLK frequency need) |
| `xclk_freq_hz`            | 20_000_000                         | 20 MHz (LEDC) |
| `pixel_format`            | `PIXFORMAT_JPEG` / `RGB565`        | JPEG path configured via register tables; raw bus captured as JPEG bytes |
| `frame_size`              | `FRAMESIZE_QVGA`..`SVGA/UXGA`      | SVGA (800x600) (`ov2640_set_svga_jpeg`) |
| `jpeg_quality`            | 12 (higher quality)                | 45 (lower quality, reduces frame size) – env tunable planned |
| `fb_count`                | 1 or 2 (continuous)                | 1 logical buffer (single DMA target) |
| `fb_location`             | `CAMERA_FB_IN_PSRAM`               | Internal DRAM array `FRAME_BUFFER`; PSRAM added to heap for future expansion |
| `grab_mode`               | `CAMERA_GRAB_WHEN_EMPTY`           | Manual trigger per interval (similar semantic to WHEN_EMPTY) |
| `ledc_timer/channel`      | `LEDC_TIMER_0` / `LEDC_CHANNEL_0`  | Low-speed LEDC Timer0 / Channel0 |
| Auto controls (AWB/AGC)   | Enabled after init                 | Re-enabled (`ov2640_re_enable_auto_controls`) + optional advanced AWB deferred |

### Notable Differences & Their Impact
1. JPEG Quality (12 vs 45): Lower numeric value in OV series means higher quality. Our current `CAMERA_JPEG_QUALITY=45` reduces bandwidth/memory but can amplify color artifacts; consider moving toward 20–30 once stability confirmed.
2. Pin Mapping: Xiao board constraints required alternate GPIOs. Ensure these pins are capable of required input modes (no conflicts with Wi-Fi or PSRAM). If color issues persist, double-check for signal integrity on longer traces (particularly D2/D3 replacements).
3. Frame Buffer Location: Upstream often places buffers in PSRAM for higher resolutions; we currently use internal DRAM. For higher quality or dual buffering, allocate in PSRAM (ensure DMA capability flags in HAL support external region).
4. Advanced AWB/Gamma: Upstream applies sensor tuning implicitly inside `esp_camera_init`; we replicate via staged and deferred register sequences behind env flags (`CAMERA_ADV_AWB_DEFER`, `CAMERA_UPSTREAM_AWB_DEFER`, `CAMERA_GAMMA_DEFER`) to avoid early hangs found in testing.
5. Group Writes (E0 register): We defer the upstream group commit to post-first-frame to maintain capture stability observed on S3 + OV2640 combination.

### Recommended Steps to Align Further
1. Introduce an env var for JPEG quality (`CAMERA_JPEG_QUALITY`) to tune size vs fidelity.
2. Add optional second framebuffer in PSRAM for higher frame rate (mirroring upstream `fb_count=2`).
3. Implement a small color metric (decode JPEG header + sample MCU blocks) to objectively compare matrix/AWB variants before/after applying deferred sequences.
4. Validate electrical pin choices: confirm each data pin supports input and isn’t strapping or reserved; adjust if any intermittent bit flips appear.
5. Expand register table coverage for resolution changes (QVGA, VGA) to allow performance scaling.

### Pin Capability Notes (Xiao ESP32-S3)
| GPIO | Assigned Function | Input Capability | Special / Strapping | Notes |
|------|-------------------|------------------|---------------------|-------|
| 10   | XCLK (LEDC out)   | Output only used | Strapping: None     | Stable 20MHz; avoid other PWM on LS timer0 channel0. |
| 13   | PCLK               | Yes              | Strapping: None     | Ensure no conflicts with SPI or JTAG routing. |
| 38   | VSYNC              | Yes              | Strapping: None     | High-frequency edge; route shortest trace. |
| 47   | HREF               | Yes              | Strapping: None     | Large number sometimes used for USB on some boards; confirm free. |
| 15   | D0                 | Yes              | May be strapping during boot | Keep pull-ups/pull-downs neutral to avoid boot mode issues. |
| 17   | D1                 | Yes              | None                | — |
| 18   | D2                 | Yes              | Used by SPI in some modules | Avoid simultaneous SPI usage. |
| 16   | D3                 | Yes              | Strapping: None     | — |
| 14   | D4                 | Yes              | JTAG TMS possible   | If JTAG enabled, remap or disable JTAG. |
| 12   | D5                 | Yes              | JTAG TDI possible   | Same consideration as GPIO14. |
| 11   | D6                 | Yes              | JTAG TCK possible   | Same consideration. |
| 48   | D7                 | Yes              | High-number user IO | Verify board trace quality; sometimes near antenna zone. |
| 40   | SDA                | Yes (open-drain) | Strapping: None     | Pull-up provided externally or via internal; ensure ~10k. |
| 39   | SCL                | Yes (open-drain) | Strapping: None     | Same pull-up requirement as SDA. |

If unexpected color artifacts or intermittent data bit flips appear, probe D-lines with logic analyzer while toggling brightness to confirm stable transitions; consider reassigning any line overlapping with active peripheral/JTAG usage.

### Quick Rust-to-C Mapping Snippet
```text
camera_config_t.pin_xclk      -> LEDC channel0 on GPIO10 (20MHz)
camera_config_t.pin_sccb_*    -> I2C0 SDA=GPIO40, SCL=GPIO39
camera_config_t.pin_d0..d7    -> Camera::with_dataN() chain
camera_config_t.pin_vsync     -> Camera::with_vsync(GPIO38)
camera_config_t.pin_href      -> Camera::with_h_enable(GPIO47)
camera_config_t.pin_pclk      -> Camera::with_pixel_clock(GPIO13)
camera_config_t.frame_size    -> `ov2640_set_svga_jpeg`
camera_config_t.pixel_format  -> JPEG tables + DSP path (DA/D7 registers)
camera_config_t.jpeg_quality  -> Argument to `ov2640_set_svga_jpeg`
camera_config_t.fb_count      -> Single DMA buffer (could extend)
camera_config_t.auto controls -> `ov2640_re_enable_auto_controls()`
```

Use this matrix when you compare against any future ESP-IDF example or move to different resolutions/sensors.

