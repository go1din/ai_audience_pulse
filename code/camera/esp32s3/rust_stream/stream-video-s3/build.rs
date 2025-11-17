fn main() {
    // --- .env logic ---
    let env_path = std::path::Path::new(".env");
    let example_path = std::path::Path::new(".env.example");

    // Always rerun if .env changes
    println!("cargo:rerun-if-changed=.env");
    println!("cargo:rerun-if-changed=.env.example");

    // If .env doesn't exist, create from .env.example
    if !env_path.exists() {
        if example_path.exists() {
            std::fs::copy(example_path, env_path).expect("Failed to copy .env.example to .env");
        }
    }

    // Read .env
    let mut wifi_ssid = String::from("ESP32_WIFI");
    let mut wifi_pass = String::from("password");
    if let Ok(contents) = std::fs::read_to_string(env_path) {
        for line in contents.lines() {
            if let Some(val) = line.strip_prefix("WIFI_SSID=") {
                wifi_ssid = val.trim().to_string();
            }
            if let Some(val) = line.strip_prefix("WIFI_PASS=") {
                wifi_pass = val.trim().to_string();
            }
        }
    }

    // Export Wi-Fi credentials as compile-time environment variables.
    println!("cargo:rustc-env=WIFI_SSID={}", wifi_ssid);
    println!("cargo:rustc-env=WIFI_PASS={}", wifi_pass);

    // --- linker logic ---
    linker_be_nice();
    // make sure linkall.x is the last linker script (otherwise might cause problems with flip-link)
    println!("cargo:rustc-link-arg=-Tlinkall.x");
}

fn linker_be_nice() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 {
        let kind = &args[1];
        let what = &args[2];

        match kind.as_str() {
            "undefined-symbol" => match what.as_str() {
                "_defmt_timestamp" => {
                    eprintln!();
                    eprintln!(
                        "💡 `defmt` not found - make sure `defmt.x` is added as a linker script and you have included `use defmt_rtt as _;`"
                    );
                    eprintln!();
                }
                "_stack_start" => {
                    eprintln!();
                    eprintln!("💡 Is the linker script `linkall.x` missing?");
                    eprintln!();
                }
                "esp_rtos_initialized" | "esp_rtos_yield_task" | "esp_rtos_task_create" => {
                    eprintln!();
                    eprintln!(
                        "💡 `esp-radio` has no scheduler enabled. Make sure you have initialized `esp-rtos` or provided an external scheduler."
                    );
                    eprintln!();
                }
                "embedded_test_linker_file_not_added_to_rustflags" => {
                    eprintln!();
                    eprintln!(
                        "💡 `embedded-test` not found - make sure `embedded-test.x` is added as a linker script for tests"
                    );
                    eprintln!();
                }
                _ => (),
            },
            // we don't have anything helpful for "missing-lib" yet
            _ => {
                std::process::exit(1);
            }
        }

        std::process::exit(0);
    }

    println!(
        "cargo:rustc-link-arg=-Wl,--error-handling-script={}",
        std::env::current_exe().unwrap().display()
    );
}
