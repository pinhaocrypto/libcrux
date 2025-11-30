use std::env;

fn main() {
    let target_arch = env::var("CARGO_CFG_TARGET_ARCH").unwrap();
    let disable_simd128 = read_env("LIBCRUX_DISABLE_SIMD128");
    let disable_simd256 = read_env("LIBCRUX_DISABLE_SIMD256");

    // Force a simd build. Make sure you know what you're doing.
    let enable_simd128 = read_env("LIBCRUX_ENABLE_SIMD128");
    let enable_simd256 = read_env("LIBCRUX_ENABLE_SIMD256");

    let simd128_possible = target_arch == "aarch64";
    if (simd128_possible || enable_simd128) && !disable_simd128 {
        // We enable simd128 on all aarch64 builds.
        println!("cargo:rustc-cfg=feature=\"simd128\"");
    }
    let simd126_possible = target_arch == "x86_64";
    if (simd126_possible || enable_simd256) && !disable_simd256 {
        // We enable simd256 on all x86_64 builds.
        // Note that this doesn't mean the required CPU features are available.
        // But the compiler will support them and the runtime checks ensure that
        // it's only used when available.
        //
        // We don't enable this on x86 because it seems to generate invalid code.
        println!("cargo:rustc-cfg=feature=\"simd256\"");
    }

    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap();

    // Only build cycle counter support for supported platforms
    if target_arch == "aarch64" || target_arch == "x86_64" {
        println!("cargo:rerun-if-changed=benches/cycle_counter/hal.c");

        let mut build = cc::Build::new();
        build.file("benches/cycle_counter/hal.c");

        // Check for manual CYCLES override first
        let cycles_override = env::var("CYCLES").ok();

        // Set cycle counter implementation based on CYCLES env var or platform
        match cycles_override.as_deref() {
            Some("PMU") => {
                println!("cargo:warning=Using PMU cycles (manual override)");
                build.define("PMU_CYCLES", None);
            }
            Some("PERF") => {
                println!("cargo:warning=Using PERF cycles (manual override)");
                build.define("PERF_CYCLES", None);
            }
            Some("MAC") => {
                println!("cargo:warning=Using MAC cycles (manual override)");
                build.define("MAC_CYCLES", None);
            }
            _ => {
                // Auto-detect based on platform
                match target_os.as_str() {
                    "linux" => {
                        if target_arch == "aarch64" {
                            println!(
                                "cargo:warning=Using PMU cycles (auto-detected for Linux AArch64)"
                            );
                            build.define("PMU_CYCLES", None);
                        } else {
                            println!(
                                "cargo:warning=Using PERF cycles (auto-detected for Linux x86_64)"
                            );
                            build.define("PERF_CYCLES", None);
                        }
                    }
                    "macos" => {
                        println!("cargo:warning=Using MAC cycles (auto-detected for macOS)");
                        build.define("MAC_CYCLES", None);
                    }
                    _ => {
                        println!("cargo:warning=Using PMU cycles (fallback)");
                        build.define("PMU_CYCLES", None);
                    }
                }
            }
        }

        build.compile("cycle_counter");

        println!("cargo:rustc-link-lib=static=cycle_counter");
    }
}

fn read_env(key: &str) -> bool {
    match env::var(key) {
        Ok(s) => s == "1" || s == "y" || s == "Y",
        Err(_) => false,
    }
}
