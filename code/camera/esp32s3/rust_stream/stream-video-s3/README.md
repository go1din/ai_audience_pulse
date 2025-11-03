# ESP32-S3 Camera Stream

Firmware for streaming OV2640 JPEG frames from an ESP32-S3 over Wi-Fi using Embassy.

## Firmware Configuration

- Set Wi-Fi credentials via environment variables before building or flashing:
  - `WIFI_SSID`
  - `WIFI_PASS`
- Optional capture interval override (`CAPTURE_INTERVAL_MS`). Defaults to `1000`.
- Optional debug mode (`DEBUG=1`) to enable verbose frame diagnostics. Defaults to `0` (disabled).
- If the variables are not set, the firmware falls back to `ESP32_WIFI` / `password`.
- On boot you will see a log line similar to:
  ```
  [wifi] starting with SSID 'MyNetwork' and password 'hunter2'
  ```

## MJPEG Streaming

The camera streams continuous MJPEG video on **port 81**:

- **Endpoint**: `http://<device-ip>:81/`
- **Format**: `multipart/x-mixed-replace; boundary=frame`
- **Usage**: Open in browser or use with video players that support MJPEG streams

Example:
```bash
# View in browser
open http://192.168.1.100:81/

# Stream with ffplay
ffplay http://192.168.1.100:81/

# Stream with VLC
vlc http://192.168.1.100:81/
```

**Note**: The HTTP server on port 80 with other endpoints (like `/frame.jpg`, `/status`) is currently disabled. Only MJPEG streaming on port 81 is active.

## Debugging Tips

- **DEBUG mode**: Set `DEBUG=1` before building to enable verbose frame diagnostics:
  ```bash
  DEBUG=1 cargo run
  ```
  This will display:
  - Hex dump of first 64 bytes of each captured frame
  - Green tint analysis (approximate percentage)
  - Detailed frame buffer information
  
  Default (`DEBUG=0` or unset) suppresses verbose output for production use.

- Serial logs report capture progress, buffer truncation, and checksum.
- Capture loop logs `[capture] interval set to <ms>` so you can confirm the active cadence.
- If `curl` reports an unexpected EOF, cross-check the logged frame length against the downloaded size; timing out clients can leave the connection early.
- When changing configuration, reset the device to ensure env overrides take effect.
