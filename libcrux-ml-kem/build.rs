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

    // Build cycle counter C library for benchmarks
    build_cycle_counter();
}

fn build_cycle_counter() {
    let mut build = cc::Build::new();
    
    build.file("benches/cycle_counter/hal.c");
    
    // set PMU_CYCLES or MAC_CYCLES based on target_os
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap();
    let target_arch = env::var("CARGO_CFG_TARGET_ARCH").unwrap();
    
    match target_os.as_str() {
        "linux" => {
            // use PMU_CYCLES by default
            build.define("PMU_CYCLES", None);
        }
        "macos" => {
            build.define("MAC_CYCLES", None);
        }
        _ => {
            // fallback to time measurement
            println!("cargo:warning=Using fallback time measurement for {}", target_os);
        }
    }
    
    // architecture specific configuration
    match target_arch.as_str() {
        "x86_64" | "aarch64" => {
            // PMU_CYCLES is set in OS check
        }
        _ => {
            println!("cargo:warning=Cycle counter may not be accurate on {}", target_arch);
        }
    }
    
    build.compile("cycle_counter");
    
    // tell cargo to rebuild when these files change
    println!("cargo:rerun-if-changed=benches/cycle_counter/hal.c");
    println!("cargo:rerun-if-changed=benches/cycle_counter/hal.h");
}

fn read_env(key: &str) -> bool {
    match env::var(key) {
        Ok(s) => s == "1" || s == "y" || s == "Y",
        Err(_) => false,
    }
}