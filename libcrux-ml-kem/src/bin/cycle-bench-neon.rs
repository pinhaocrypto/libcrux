use libcrux_ml_kem::mlkem512;
use rand::{rngs::OsRng, TryRngCore};
use std::hint::black_box;

// FFI bindings to C cycle counter
extern "C" {
    fn enable_cyclecounter();
    fn disable_cyclecounter();
    fn get_cyclecounter() -> u64;
}

// Global cycle counter management
struct GlobalCycleCounter;

impl GlobalCycleCounter {
    fn enable() {
        unsafe { enable_cyclecounter() };
    }

    fn disable() {
        unsafe { disable_cyclecounter() };
    }

    fn get() -> u64 {
        unsafe { get_cyclecounter() }
    }
}

// Constants matching original aarch64-bench
const NWARMUP: usize = 50;
const NITERATIONS: usize = 300;
const NTESTS: usize = 500;

fn cmp_u64(a: &u64, b: &u64) -> std::cmp::Ordering {
    a.cmp(b)
}

fn print_median(txt: &str, cyc: &[u64]) {
    println!(
        "{:>10} cycles = {}",
        txt,
        cyc[NTESTS >> 1] / NITERATIONS as u64
    );
}

fn print_percentile_legend() {
    print!("{:>21}", "percentile");
    let percentiles = [1, 10, 20, 30, 40, 50, 60, 70, 80, 90, 99];
    for p in percentiles {
        print!("{:>8}", p);
    }
    println!();
}

fn print_percentiles(txt: &str, cyc: &[u64]) {
    let percentiles = [1, 10, 20, 30, 40, 50, 60, 70, 80, 90, 99];
    print!("{:>10} percentiles:", txt);
    for p in percentiles {
        let value = cyc[NTESTS * p / 100] / NITERATIONS as u64;
        print!("{:>8}", value);
    }
    println!();
}

// Check if cycle values look reasonable (not too large)
fn validate_cycles(cycles: &[u64]) -> bool {
    let median = cycles[NTESTS >> 1] / NITERATIONS as u64;

    // If median cycles > 100M, probably measuring nanoseconds instead of cycles
    if median > 100_000_000 {
        println!("Warning: Very large cycle counts detected. This may be measuring nanoseconds instead of CPU cycles.");
        println!("This is common on macOS where kperf access is restricted.");
        println!();
        false
    } else {
        true
    }
}

fn bench_operation<F>(_name: &str, mut operation: F) -> Vec<u64>
where
    F: FnMut(),
{
    let mut cycles = Vec::with_capacity(NTESTS);

    for _i in 0..NTESTS {
        // Warmup
        for _ in 0..NWARMUP {
            operation();
        }

        // Actual measurement
        let t0 = GlobalCycleCounter::get();
        for _ in 0..NITERATIONS {
            operation();
        }
        let t1 = GlobalCycleCounter::get();
        cycles.push(t1 - t0);
    }

    cycles.sort_by(cmp_u64);
    cycles
}

fn main() {
    // Enable cycle counter once at the beginning
    GlobalCycleCounter::enable();

    let mut rng = OsRng;

    // Setup seeds
    let mut seed1 = [0; 64];
    let mut seed2 = [0; 32];
    rng.try_fill_bytes(&mut seed1).unwrap();
    rng.try_fill_bytes(&mut seed2).unwrap();

    // Pre-generate keypair for encaps/decaps tests using Neon implementation
    #[cfg(feature = "simd128")]
    let keypair = mlkem512::neon::generate_key_pair(seed1);
    #[cfg(not(feature = "simd128"))]
    let keypair = mlkem512::portable::generate_key_pair(seed1);

    #[cfg(feature = "simd128")]
    let (ciphertext, _) = mlkem512::neon::encapsulate(keypair.public_key(), seed2);
    #[cfg(not(feature = "simd128"))]
    let (ciphertext, _) = mlkem512::portable::encapsulate(keypair.public_key(), seed2);

    #[cfg(feature = "simd128")]
    println!("ML-KEM 512 Cycle Benchmarks (Neon Optimized)");
    #[cfg(not(feature = "simd128"))]
    println!("ML-KEM 512 Cycle Benchmarks (Portable - Neon not available)");
    println!("============================");

    // Key generation benchmark
    let cycles_keygen = bench_operation("keypair", || {
        #[cfg(feature = "simd128")]
        black_box(mlkem512::neon::generate_key_pair(seed1));
        #[cfg(not(feature = "simd128"))]
        black_box(mlkem512::portable::generate_key_pair(seed1));
    });

    // Check if cycle counter is working properly
    if !validate_cycles(&cycles_keygen) {
        println!("Warning: Cycle counter may not be working properly on this system.");
        println!("Consider running with sudo or checking system permissions.");
        println!("Results may not be accurate.");
        println!();
    }

    // Encapsulation benchmark
    let cycles_encaps = bench_operation("encaps", || {
        #[cfg(feature = "simd128")]
        black_box(mlkem512::neon::encapsulate(keypair.public_key(), seed2));
        #[cfg(not(feature = "simd128"))]
        black_box(mlkem512::portable::encapsulate(keypair.public_key(), seed2));
    });

    // Decapsulation benchmark
    let cycles_decaps = bench_operation("decaps", || {
        #[cfg(feature = "simd128")]
        black_box(mlkem512::neon::decapsulate(
            keypair.private_key(),
            &ciphertext,
        ));
        #[cfg(not(feature = "simd128"))]
        black_box(mlkem512::portable::decapsulate(
            keypair.private_key(),
            &ciphertext,
        ));
    });

    // Print results in the desired format
    print_median("keypair", &cycles_keygen);
    print_median("encaps", &cycles_encaps);
    print_median("decaps", &cycles_decaps);

    println!();
    print_percentile_legend();
    print_percentiles("keypair", &cycles_keygen);
    print_percentiles("encaps", &cycles_encaps);
    print_percentiles("decaps", &cycles_decaps);

    // Disable cycle counter once at the end
    GlobalCycleCounter::disable();
}
