use rand::{rngs::OsRng, TryRngCore};
use std::collections::{BTreeMap, HashMap};
use std::hint::black_box;

use libcrux_ml_kem::{mlkem1024, mlkem512, mlkem768};

mod cycle_counter;
use cycle_counter::{cleanup_cycle_counter, init_cycle_counter, read_cycles};

// choose default arch based on target_arch and feature
#[cfg(all(target_arch = "x86_64", feature = "simd256"))]
const DEFAULT_ARCH: &str = "avx2";
#[cfg(all(target_arch = "aarch64", feature = "simd128"))]
const DEFAULT_ARCH: &str = "neon";
#[cfg(not(any(
    all(target_arch = "x86_64", feature = "simd256"),
    all(target_arch = "aarch64", feature = "simd128")
)))]
const DEFAULT_ARCH: &str = "portable";

fn measure_cycles<F: FnOnce()>(f: F) -> u64 {
    let start = read_cycles();
    black_box(f());
    let end = read_cycles();
    end.saturating_sub(start)
}

#[derive(Debug)]
struct BenchmarkStats {
    operation: String,
    security_level: u16,
    arch: String,
    api_type: String,
    measurements: Vec<u64>,
}

impl BenchmarkStats {
    fn new(operation: &str, security_level: u16, arch: &str, api_type: &str) -> Self {
        Self {
            operation: operation.to_string(),
            security_level,
            arch: arch.to_string(),
            api_type: api_type.to_string(),
            measurements: Vec::new(),
        }
    }

    fn add_measurement(&mut self, cycles: u64) {
        self.measurements.push(cycles);
    }

    fn median(&self) -> u64 {
        let mut sorted = self.measurements.clone();
        sorted.sort_unstable();
        sorted[sorted.len() / 2]
    }

    fn percentile(&self, p: usize) -> u64 {
        let mut sorted = self.measurements.clone();
        sorted.sort_unstable();
        let index = (sorted.len() * p / 100).min(sorted.len() - 1);
        sorted[index]
    }

    fn print_results(&self) {
        let median = self.median();
        println!("  {} cycles = {}", self.operation, median);
    }

    fn print_percentiles(&self) {
        let percentiles = [1, 10, 20, 30, 40, 50, 60, 70, 80, 90, 99];
        print!("{:>8} percentiles: ", self.operation);
        for &p in &percentiles {
            print!("{:>6} ", self.percentile(p));
        }
        println!();
    }
}

// ML-KEM 512 benchmark
fn benchmark_mlkem512(arch: &str, iterations: usize) -> Vec<BenchmarkStats> {
    let mut results = Vec::new();
    let mut rng = OsRng;

    macro_rules! run_benchmarks {
        ($impl_mod:path, $arch_name:expr) => {{
            use $impl_mod as implementation;

            // keygen benchmark
            let mut keygen_stats = BenchmarkStats::new("keypair", 512, $arch_name, "standard");
            let mut seed = [0u8; 64];

            for _ in 0..iterations {
                rng.try_fill_bytes(&mut seed).unwrap();
                let cycles = measure_cycles(|| {
                    let _keypair = implementation::generate_key_pair(seed);
                });
                keygen_stats.add_measurement(cycles);
            }
            results.push(keygen_stats);

            // encaps benchmark
            let mut encaps_stats = BenchmarkStats::new("encaps", 512, $arch_name, "standard");
            for _ in 0..iterations {
                rng.try_fill_bytes(&mut seed).unwrap();
                let keypair = implementation::generate_key_pair(seed);
                let mut encaps_seed = [0u8; 32];
                rng.try_fill_bytes(&mut encaps_seed).unwrap();

                let cycles = measure_cycles(|| {
                    let (_shared_secret, _ciphertext) =
                        implementation::encapsulate(keypair.public_key(), encaps_seed);
                });
                encaps_stats.add_measurement(cycles);
            }
            results.push(encaps_stats);

            // decaps benchmark
            let mut decaps_stats = BenchmarkStats::new("decaps", 512, $arch_name, "standard");
            for _ in 0..iterations {
                rng.try_fill_bytes(&mut seed).unwrap();
                let keypair = implementation::generate_key_pair(seed);
                let mut encaps_seed = [0u8; 32];
                rng.try_fill_bytes(&mut encaps_seed).unwrap();
                let (ciphertext, _shared_secret) =
                    implementation::encapsulate(keypair.public_key(), encaps_seed);

                let cycles = measure_cycles(|| {
                    let _shared_secret =
                        implementation::decapsulate(keypair.private_key(), &ciphertext);
                });
                decaps_stats.add_measurement(cycles);
            }
            results.push(decaps_stats);
        }};
    }

    match arch {
        "portable" => run_benchmarks!(mlkem512::portable, "portable"),
        #[cfg(feature = "simd128")]
        "neon" => run_benchmarks!(mlkem512::neon, "neon"),
        #[cfg(feature = "simd256")]
        "avx2" => run_benchmarks!(mlkem512::avx2, "avx2"),
        _ => panic!("Unsupported architecture: {}", arch),
    }

    results
}

fn benchmark_mlkem768(arch: &str, iterations: usize) -> Vec<BenchmarkStats> {
    let mut results = Vec::new();
    let mut rng = OsRng;

    macro_rules! run_benchmarks {
        ($impl_mod:path, $arch_name:expr) => {{
            use $impl_mod as implementation;

            let mut keygen_stats = BenchmarkStats::new("keypair", 768, $arch_name, "standard");
            let mut seed = [0u8; 64];

            for _ in 0..iterations {
                rng.try_fill_bytes(&mut seed).unwrap();
                let cycles = measure_cycles(|| {
                    let _keypair = implementation::generate_key_pair(seed);
                });
                keygen_stats.add_measurement(cycles);
            }
            results.push(keygen_stats);

            let mut encaps_stats = BenchmarkStats::new("encaps", 768, $arch_name, "standard");
            for _ in 0..iterations {
                rng.try_fill_bytes(&mut seed).unwrap();
                let keypair = implementation::generate_key_pair(seed);
                let mut encaps_seed = [0u8; 32];
                rng.try_fill_bytes(&mut encaps_seed).unwrap();

                let cycles = measure_cycles(|| {
                    let (_shared_secret, _ciphertext) =
                        implementation::encapsulate(keypair.public_key(), encaps_seed);
                });
                encaps_stats.add_measurement(cycles);
            }
            results.push(encaps_stats);

            let mut decaps_stats = BenchmarkStats::new("decaps", 768, $arch_name, "standard");
            for _ in 0..iterations {
                rng.try_fill_bytes(&mut seed).unwrap();
                let keypair = implementation::generate_key_pair(seed);
                let mut encaps_seed = [0u8; 32];
                rng.try_fill_bytes(&mut encaps_seed).unwrap();
                let (ciphertext, _shared_secret) =
                    implementation::encapsulate(keypair.public_key(), encaps_seed);

                let cycles = measure_cycles(|| {
                    let _shared_secret =
                        implementation::decapsulate(keypair.private_key(), &ciphertext);
                });
                decaps_stats.add_measurement(cycles);
            }
            results.push(decaps_stats);
        }};
    }

    match arch {
        "portable" => run_benchmarks!(mlkem768::portable, "portable"),
        #[cfg(feature = "simd128")]
        "neon" => run_benchmarks!(mlkem768::neon, "neon"),
        #[cfg(feature = "simd256")]
        "avx2" => run_benchmarks!(mlkem768::avx2, "avx2"),
        _ => panic!("Unsupported architecture: {}", arch),
    }

    results
}

fn benchmark_mlkem1024(arch: &str, iterations: usize) -> Vec<BenchmarkStats> {
    let mut results = Vec::new();
    let mut rng = OsRng;

    macro_rules! run_benchmarks {
        ($impl_mod:path, $arch_name:expr) => {{
            use $impl_mod as implementation;

            let mut keygen_stats = BenchmarkStats::new("keypair", 1024, $arch_name, "standard");
            let mut seed = [0u8; 64];

            for _ in 0..iterations {
                rng.try_fill_bytes(&mut seed).unwrap();
                let cycles = measure_cycles(|| {
                    let _keypair = implementation::generate_key_pair(seed);
                });
                keygen_stats.add_measurement(cycles);
            }
            results.push(keygen_stats);

            let mut encaps_stats = BenchmarkStats::new("encaps", 1024, $arch_name, "standard");
            for _ in 0..iterations {
                rng.try_fill_bytes(&mut seed).unwrap();
                let keypair = implementation::generate_key_pair(seed);
                let mut encaps_seed = [0u8; 32];
                rng.try_fill_bytes(&mut encaps_seed).unwrap();

                let cycles = measure_cycles(|| {
                    let (_shared_secret, _ciphertext) =
                        implementation::encapsulate(keypair.public_key(), encaps_seed);
                });
                encaps_stats.add_measurement(cycles);
            }
            results.push(encaps_stats);

            let mut decaps_stats = BenchmarkStats::new("decaps", 1024, $arch_name, "standard");
            for _ in 0..iterations {
                rng.try_fill_bytes(&mut seed).unwrap();
                let keypair = implementation::generate_key_pair(seed);
                let mut encaps_seed = [0u8; 32];
                rng.try_fill_bytes(&mut encaps_seed).unwrap();
                let (ciphertext, _shared_secret) =
                    implementation::encapsulate(keypair.public_key(), encaps_seed);

                let cycles = measure_cycles(|| {
                    let _shared_secret =
                        implementation::decapsulate(keypair.private_key(), &ciphertext);
                });
                decaps_stats.add_measurement(cycles);
            }
            results.push(decaps_stats);
        }};
    }

    match arch {
        "portable" => run_benchmarks!(mlkem1024::portable, "portable"),
        #[cfg(feature = "simd128")]
        "neon" => run_benchmarks!(mlkem1024::neon, "neon"),
        #[cfg(feature = "simd256")]
        "avx2" => run_benchmarks!(mlkem1024::avx2, "avx2"),
        _ => panic!("Unsupported architecture: {}", arch),
    }

    results
}

fn print_results(all_results: &[BenchmarkStats], security_level: u16) {
    println!("ML-KEM-{} Benchmark Results", security_level);
    println!("============================");

    // group by operation
    let mut by_operation: HashMap<&str, Vec<&BenchmarkStats>> = HashMap::new();
    for stats in all_results
        .iter()
        .filter(|s| s.security_level == security_level)
    {
        by_operation
            .entry(&stats.operation)
            .or_default()
            .push(stats);
    }

    let operation_order = ["keypair", "encaps", "decaps"];

    for &op in &operation_order {
        if let Some(stats_list) = by_operation.get(op) {
            if let Some(stats) = stats_list.first() {
                stats.print_results();
            }
        }
    }

    println!();
    println!("           percentile      1     10     20     30     40     50     60     70     80     90     99");

    for &op in &operation_order {
        if let Some(stats_list) = by_operation.get(op) {
            if let Some(stats) = stats_list.first() {
                stats.print_percentiles();
            }
        }
    }

    println!();
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    // skip program name and possible cargo bench arguments
    let mut arg_iter = args.iter().skip(1);

    // filter out cargo bench internal arguments
    let filtered_args: Vec<&String> = arg_iter
        .filter(|arg| !arg.starts_with("--") && !arg.contains("ml-kem-cycle"))
        .collect();

    // parse command line arguments
    let arch = filtered_args
        .get(0)
        .map(|s| s.as_str())
        .unwrap_or(DEFAULT_ARCH);
    let security_level: Option<u16> = filtered_args.get(1).and_then(|s| s.parse().ok());
    let iterations: usize = filtered_args
        .get(2)
        .and_then(|s| s.parse().ok())
        .unwrap_or(1000);

    // Initialize cycle counter
    init_cycle_counter();

    println!("Running ML-KEM benchmarks with {} implementation", arch);
    println!("Iterations per test: {}", iterations);
    println!();

    let mut all_results = Vec::new();

    match security_level {
        Some(512) => {
            let results = benchmark_mlkem512(arch, iterations);
            all_results.extend(results);
            print_results(&all_results, 512);
        }
        Some(768) => {
            let results = benchmark_mlkem768(arch, iterations);
            all_results.extend(results);
            print_results(&all_results, 768);
        }
        Some(1024) => {
            let results = benchmark_mlkem1024(arch, iterations);
            all_results.extend(results);
            print_results(&all_results, 1024);
        }
        Some(invalid) => {
            eprintln!(
                "Error: Invalid security level {}. Valid options are 512, 768, or 1024.",
                invalid
            );
            std::process::exit(1);
        }
        None => {
            // run all security levels
            for &level in &[512, 768, 1024] {
                let results = match level {
                    512 => benchmark_mlkem512(arch, iterations),
                    768 => benchmark_mlkem768(arch, iterations),
                    1024 => benchmark_mlkem1024(arch, iterations),
                    _ => unreachable!(),
                };
                all_results.extend(results);
                print_results(&all_results, level);
            }
        }
    }

    // Cleanup
    cleanup_cycle_counter();
}
