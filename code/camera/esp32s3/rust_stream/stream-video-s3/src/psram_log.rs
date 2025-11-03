//! Logging utilities for system diagnostics

extern crate alloc;

use esp_println::println;

/// Log PSRAM detection status and attempt to add external heap region if available.
/// 
/// # Safety
/// This function uses unsafe code to add PSRAM region to the heap allocator.
pub unsafe fn log_and_init_psram(psram: &esp_hal::peripherals::PSRAM) {
    let (psram_start, psram_size) = esp_hal::psram::psram_raw_parts(psram);
    println!("PSRAM raw parts: start={:p} size={}", psram_start, psram_size);
    
    let psram_status = if psram_size == 0 { "NO-PSRAM" } else { "PSRAM-DETECTED" };
    // Chip revision API not available in this esp-hal version; placeholder only.
    println!("Chip revision: <unavailable> | PSRAM status: {}", psram_status);
    
    if psram_size >= 64 * 1024 {
        unsafe {
            esp_alloc::HEAP.add_region(esp_alloc::HeapRegion::new(
                psram_start,
                psram_size,
                esp_alloc::MemoryCapability::External.into(),
            ));
        }
        println!("PSRAM heap added: {} bytes", psram_size);
        
        // Small allocation test to confirm external region usable
        let test_vec = alloc::vec![0u8; 32 * 1024];
        println!(
            "External alloc test vec len={} (first byte {})",
            test_vec.len(),
            test_vec.get(0).unwrap_or(&0)
        );
        core::mem::drop(test_vec);
    } else {
        println!(
            "PSRAM not detected or too small (size {}), external heap region skipped",
            psram_size
        );
    }
}
