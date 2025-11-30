# ML-KEM Cycle Benchmarks

This document explains how to run the hardware cycle-counter benchmark integrated into libcrux-ml-kem.

## 概述

我们已经集成了基于 `aarch64-bench` 的硬件周期计数器测量功能到 Rust 的 benchmark 系统中。这允许你获得比标准时间测量更精确的性能数据。

## Overview

We provide a benchmark framework based on aarch64-bench and integrated directly into Rust.
It allows you to measure actual CPU cycles, which is far more precise than normal time-based benchmarking.

Supported Platforms

* Linux AArch64 – uses PMU (Performance Monitoring Unit) cycle counters


## How to run

### Option 1: Stand-alone executable (recommended)

```bash
# Build and run the stand-alone cycle benchmark
cargo run --bin cycle-bench

# Or build first then run manually
cargo build --bin cycle-bench
./target/debug/cycle-bench
```

### Option 2: Criterion benchmark (runs many times)

```bash
# Run all benchmarks
cargo bench

# Run only the original time-based benchmark
cargo bench --bench ml-kem

# Run only the new cycle benchmark (will run multiple iterations)
cargo bench --bench ml-kem-cycle-bench
```

## Output Format

The cycle benchmark prints concise statistics in the same format as aarch64-bench:

```
ML-KEM 512 Cycle Benchmarks
============================
   keypair cycles = 12582
    encaps cycles = 14982
    decaps cycles = 19825

            percentile      1     10     20     30     40     50     60     70     80     90     99
   keypair percentiles:  12529  12560  12568  12574  12578  12582  12586  12590  12598  12607  13668
    encaps percentiles:  14935  14958  14966  14972  14977  14982  14986  14992  14999  15011  16073
    decaps percentiles:  19780  19801  19810  19815  19820  19825  19831  19837  19844  19857  20913
```

- **cycles**: median cycle count per operation
- **percentiles**: distribution of cycle counts across multiple runs

## 测量参数

- **Warm-up iterations**: 50
- **Measurement iterations**: 300
- **Rounds per test**: 500
- **Statistics**: medians + percentiles

## Permission Requirements

### Linux (PMU)

Some systems require elevated permissions to read hardware counters:

```bash
# Temporarily allow userspace access to PMU
echo 0 | sudo tee /proc/sys/kernel/perf_event_paranoid

# Or simply run with sudo
sudo cargo run --bin cycle-bench
```

## Technical Details

### Architecture

- **C layer**: `benches/cycle_counter/hal.c` - hardware abstraction
- **Rust layer**: `benches/ml-kem-cycle-bench.rs` - Criterion integration
- **Standalone binary**: `src/bin/cycle-bench.rs` - single-run benchmark
- **Build system**: `build.rs` - platform detection + C code compilation

### Differences vs Existing Benchmarks

- **Original**: time-based Criterion measurements
- **New**: cycle-accurate measurements using hardware counters

### Differences vs aarch64-bench

- Same measurement style and parameters
- Improved portability and integration within Rust
- Simplified, focusing only on ML-KEM-512 (keypair, encaps, decaps)