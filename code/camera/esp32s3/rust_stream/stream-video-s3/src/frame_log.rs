//! Frame diagnostics and logging utilities

extern crate alloc;

use alloc::string::String;
use core::fmt::Write;
use esp_println::println;

/// Log a sample of the captured frame buffer for diagnostics.
/// Displays hex dump and approximate "greenness" based on non-zero byte ratio.
pub fn log_frame_sample(buffer: &[u8], start: usize, len: usize, frame_sample_len: usize) {
    if len == 0 {
        println!("sample: empty frame");
        return;
    }
    if start >= buffer.len() {
        println!("sample: invalid range (start={} len={} available={})", start, len, buffer.len());
        return;
    }

    let available = buffer.len() - start;
    let sample_len = frame_sample_len.min(len).min(available);
    if sample_len == 0 {
        println!("sample: no bytes available for sampling");
        return;
    }

    let sample = &buffer[start..start + sample_len];
    let non_zero = sample.iter().filter(|&&b| b != 0).count();
    let approx_green = (non_zero as f32 / sample_len as f32) * 100.0;
    println!(
        "sample[0..{}] approx_green={:.1}% (non-zero {}/{})",
        sample_len.saturating_sub(1),
        approx_green,
        non_zero,
        sample_len
    );

    let mut line = String::new();
    for chunk in sample.chunks(16) {
        line.clear();
        for byte in chunk {
            let _ = write!(line, "{:02X} ", byte);
        }
        println!("{}", line);
    }
}
