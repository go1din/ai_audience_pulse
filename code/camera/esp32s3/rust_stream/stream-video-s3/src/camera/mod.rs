#![allow(dead_code)] // driver helpers used by binaries; keep variants/functions visible

pub mod ov2640_tables;

use self::ov2640_tables::{
    OV2640_640X480_JPEG,
    OV2640_800X600_JPEG,
    OV2640_JPEG,
    OV2640_JPEG_INIT,
    OV2640_YUV422,
};

#[derive(Clone, Copy)]
pub enum Ov2640Resolution {
    Vga,
    Svga,
}

pub fn resolution_label(res: Ov2640Resolution) -> &'static str {
    match res {
        Ov2640Resolution::Vga => "VGA",
        Ov2640Resolution::Svga => "SVGA",
    }
}

pub fn ov2640_reset<I: embedded_hal::i2c::I2c>(i2c: &mut I, addr: u8) {
    let _ = i2c.write(addr, &[0xff, 0x01]);
    let _ = i2c.write(addr, &[0x12, 0x80]);
}

pub fn write_table<I: embedded_hal::i2c::I2c>(i2c: &mut I, addr: u8, table: &[(u8, u8)]) {
    for &(reg, val) in table {
        if reg == 0xFF && val == 0xFF {
            break;
        }
        let _ = i2c.write(addr, &[reg, val]);
    }
}

pub fn ov2640_load_jpeg_tables<I: embedded_hal::i2c::I2c>(i2c: &mut I, addr: u8) {
    write_table(i2c, addr, OV2640_JPEG_INIT);
    write_table(i2c, addr, OV2640_YUV422);
    write_table(i2c, addr, OV2640_JPEG);
}

pub fn ov2640_force_output_selector<I: embedded_hal::i2c::I2c>(i2c: &mut I, addr: u8) {
    // Force the correct output selector (some firmwares flip this)
    // DSP bank
    let _ = i2c.write(addr, &[0xFF, 0x00]);
    let _ = i2c.write(addr, &[0xDA, 0x10]);   // YUV422 path (required for JPEG pipeline)
    // Note: If still greenish, try UV-swap: change 0xDA, 0x10 to 0xDA, 0x11
    let _ = i2c.write(addr, &[0xD7, 0x03]);   // auto features enabled (as in esp32-camera)
}

pub fn ov2640_re_enable_auto_controls<I: embedded_hal::i2c::I2c>(i2c: &mut I, addr: u8) {
    // Re-enable auto white balance / exposure / gain after the tables
    // Sensor bank
    let _ = i2c.write(addr, &[0xFF, 0x01]);
    let _ = i2c.write(addr, &[0x13, 0xE7]);   // COM8: AWB|AGC|AEC ON
}

pub fn ov2640_set_jpeg<I: embedded_hal::i2c::I2c>(
    i2c: &mut I,
    addr: u8,
    quality: u8,
    resolution: Ov2640Resolution,
) {
    let quality = quality.min(63);
    let table = match resolution {
        Ov2640Resolution::Vga => OV2640_640X480_JPEG,
        Ov2640Resolution::Svga => OV2640_800X600_JPEG,
    };
    let _ = i2c.write(addr, &[0xFF, 0x01]);
    let _ = i2c.write(addr, &[0x15, 0x00]);
    write_table(i2c, addr, table);
    let _ = i2c.write(addr, &[0xFF, 0x00]);
    let _ = i2c.write(addr, &[0x44, quality]);
}

pub fn ov2640_set_vflip<I: embedded_hal::i2c::I2c>(i2c: &mut I, addr: u8, enable: bool) {
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
