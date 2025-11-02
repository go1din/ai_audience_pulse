# OV2640 Camera Greenish Output Fix - TODO

## Problem
Camera output appears greenish. This is a common issue with OV2640 camera initialization where the color processing pipeline isn't configured correctly.

## Root Causes
1. **Order of table loading**: YUV422 must be applied before JPEG is enabled
2. **Output selector not forced**: Some firmware versions don't set the correct YUV422 path
3. **Auto controls disabled**: White balance, exposure, and gain may be disabled after table loading
4. **Color matrix**: Default color matrix can introduce green bias
5. **UV channel swap**: Some boards require UV swap (less common)

## Solution Steps

### ✅ 1. Apply YUV422 Before Enabling JPEG (Order Matters)

**Current Status**: Already correct in `ov2640_load_jpeg_tables()`:
- `OV2640_JPEG_INIT` (base initialization)
- `OV2640_YUV422` ← Must come first
- `OV2640_JPEG` ← Then enable JPEG

**Location**: `src/bin/main.rs:519-523`

### ✅ 2. Force the Correct Output Selector

After loading tables, explicitly set the YUV422 path:

```rust
// DSP bank (0xFF = 0x00)
i2c.write(addr, &[0xFF, 0x00]);
i2c.write(addr, &[0xDA, 0x10]);   // YUV422 path (required for JPEG pipeline)
i2c.write(addr, &[0xD7, 0x03]);   // auto features enabled (as in esp32-camera)
```

**Implementation**: Add after `ov2640_load_jpeg_tables()` call

### ✅ 3. Re-enable Auto White Balance / Exposure / Gain

After loading tables, re-enable auto controls:

```rust
// Sensor bank (0xFF = 0x01)
i2c.write(addr, &[0xFF, 0x01]);
i2c.write(addr, &[0x13, 0xE7]);   // COM8: AWB|AGC|AEC ON
```

**Implementation**: Add after color matrix configuration

### ✅ 4. Load Neutral Color Matrix + Disable Effects

Fixes green bias by setting a neutral color matrix:

```rust
// DSP bank
i2c.write(addr, &[0xFF, 0x00]);
i2c.write(addr, &[0x7C, 0x00]); i2c.write(addr, &[0x7D, 0x00]);        // SDE off
i2c.write(addr, &[0x7C, 0x03]); i2c.write(addr, &[0x7D, 0x40]); i2c.write(addr, &[0x7D, 0x40]); // mid saturation
// CMX1..6 + sign
i2c.write(addr, &[0x4F, 0xCA]); i2c.write(addr, &[0x50, 0xA8]); i2c.write(addr, &[0x51, 0x00]);
i2c.write(addr, &[0x52, 0x28]); i2c.write(addr, &[0x53, 0x70]); i2c.write(addr, &[0x54, 0x99]);
i2c.write(addr, &[0x58, 0x1A]);
```

**Implementation**: Add as a new function `ov2640_fix_color_matrix()`

### ✅ 5. UV-Swap Sanity Toggle (If Still Green)

If the issue persists after applying all fixes above, try the UV-swap toggle. Some boards need this:

```rust
// Try these two alternatives one at a time:

// Normal (YUV422) - try this first:
i2c.write(addr, &[0xFF, 0x00]);
i2c.write(addr, &[0xDA, 0x10]);   // normal (YUV422)

// UV swap (one bit differs) - try if normal doesn't work:
i2c.write(addr, &[0xFF, 0x00]);
i2c.write(addr, &[0xDA, 0x11]);   // UV swap
```

**Current Status**: ✅ Implemented as a configurable feature
**Location**: `src/bin/main.rs:85-109` (CAMERA_UV_SWAP constant) and `src/bin/main.rs:565-573` (function)

**To Apply UV-Swap**: Set the environment variable when building:
```bash
CAMERA_UV_SWAP=1 cargo build
# or
CAMERA_UV_SWAP=true cargo build
```

The function `ov2640_force_output_selector()` now accepts a `uv_swap` parameter and will use `0x11` instead of `0x10` when enabled.

## Implementation Order

The initialization sequence should be:

1. Software reset (`ov2640_reset`)
2. Load JPEG tables (`ov2640_load_jpeg_tables`) - already has correct order
3. **Force output selector** (new: DA=0x10, D7=0x03)
4. **Fix color matrix** (new: neutral matrix + disable effects)
5. Configure JPEG SVGA mode (`ov2640_set_svga_jpeg`)
6. **Re-enable auto controls** (new: COM8=0xE7)
7. Enable sensor output (existing code)

## Testing

After applying fixes:
1. Capture a test image
2. Check if greenish tint is reduced/eliminated
3. If still present, try UV-swap (DA=0x11 instead of 0x10)
4. Adjust color matrix values if needed for your specific board/environment

## References

- Based on esp32-camera driver patterns
- OV2640 datasheet register documentation
- Common fixes for green tint issues in embedded camera applications

