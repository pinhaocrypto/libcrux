use std::hint::black_box;

use criterion::{criterion_group, criterion_main, Criterion};
use rand::{rngs::OsRng, TryRngCore};

use libcrux_ml_kem::mlkem512;

// FFI bindings to C cycle counter
extern "C" {
    fn enable_cyclecounter();
    fn disable_cyclecounter();
    fn get_cyclecounter() -> u64;
}

// Cycle counter wrapper
struct CycleCounter;

impl CycleCounter {
    fn new() -> Self {
        unsafe { enable_cyclecounter() };
        Self
    }

    fn get(&self) -> u64 {
        unsafe { get_cyclecounter() }
    }
}

impl Drop for CycleCounter {
    fn drop(&mut self) {
        unsafe { disable_cyclecounter() };
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
        print!("{:>7}", p);
    }
    println!();
}

fn print_percentiles(txt: &str, cyc: &[u64]) {
    let percentiles = [1, 10, 20, 30, 40, 50, 60, 70, 80, 90, 99];
    print!("{:>10} percentiles:", txt);
    for p in percentiles {
        print!("{:>7}", cyc[NTESTS * p / 100] / NITERATIONS as u64);
    }
    println!();
}

fn bench_operation<F>(mut operation: F) -> Vec<u64>
where
    F: FnMut(),
{
    let counter = CycleCounter::new();
    let mut cycles = Vec::with_capacity(NTESTS);

    for _ in 0..NTESTS {
        // Warmup
        for _ in 0..NWARMUP {
            operation();
        }

        // Actual measurement
        let t0 = counter.get();
        for _ in 0..NITERATIONS {
            operation();
        }
        let t1 = counter.get();
        cycles.push(t1 - t0);
    }

    cycles.sort_by(cmp_u64);
    cycles
}

// Static flag to ensure we only run once
static mut HAS_RUN: bool = false;

pub fn ml_kem_cycle_bench(c: &mut Criterion) {
    c.bench_function("ml-kem-cycles", |b| {
        b.iter(|| {
            unsafe {
                if HAS_RUN {
                    return 0; // Skip subsequent runs
                }
                HAS_RUN = true;
            }

            let mut rng = OsRng;

            // Setup seeds
            let mut seed1 = [0; 64];
            let mut seed2 = [0; 32];
            rng.try_fill_bytes(&mut seed1).unwrap();
            rng.try_fill_bytes(&mut seed2).unwrap();

            // Pre-generate keypair for encaps/decaps tests
            let keypair = mlkem512::portable::generate_key_pair(seed1);
            let (ciphertext, _) = mlkem512::portable::encapsulate(keypair.public_key(), seed2);

            println!("\nML-KEM 512 Cycle Benchmarks");
            println!("============================");

            // Key generation benchmark
            let cycles_keygen = bench_operation(|| {
                black_box(mlkem512::portable::generate_key_pair(seed1));
            });

            // Encapsulation benchmark
            let cycles_encaps = bench_operation(|| {
                black_box(mlkem512::portable::encapsulate(keypair.public_key(), seed2));
            });

            // Decapsulation benchmark
            let cycles_decaps = bench_operation(|| {
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

            0
        });
    });
}

criterion_group!(cycle_benches, ml_kem_cycle_bench);
criterion_main!(cycle_benches);
