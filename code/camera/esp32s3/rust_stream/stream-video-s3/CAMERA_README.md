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
