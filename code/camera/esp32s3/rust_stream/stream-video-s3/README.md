# ESP32-S3 Camera Stream

Firmware for streaming OV2640 JPEG frames from an ESP32-S3 over Wi-Fi using Embassy and picoserve.

## Firmware Configuration

- Set Wi-Fi credentials via environment variables before building or flashing:
  - `WIFI_SSID`
  - `WIFI_PASS`
- Optional capture interval override (`CAPTURE_INTERVAL_MS`). Defaults to `1000`.
- If the variables are not set, the firmware falls back to `ESP32_WIFI` / `password`.
- On boot you will see a log line similar to:
  ```
  [wifi] starting with SSID 'MyNetwork' and password 'hunter2'
  ```

## HTTP Endpoints

- `GET /` – health probe, returns `OK`
- `GET /status` – plain text with frame counter, JPEG offsets, last length, checksum
- `GET /frame.jpg` – latest captured JPEG frame

The firmware runs a single HTTP worker task, so requests are served sequentially but the latest frame is always available.

Example download:
```
curl http://<device-ip>/frame.jpg --output frame.jpg
open frame.jpg
```

The `/status` endpoint is helpful for confirming new frames without fetching the full image.

## Debugging Tips

- Serial logs report capture progress, buffer truncation, and checksum.
- Capture loop logs `[capture] interval set to <ms>` so you can confirm the active cadence.
- If `curl` reports an unexpected EOF, cross-check the logged frame length against the downloaded size; timing out clients can leave the connection early.
- When changing configuration, reset the device to ensure env overrides take effect.
