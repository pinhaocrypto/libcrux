# ML-KEM Benchmarks

This directory contains benchmarking tools for the libcrux ML-KEM implementation.

## Available Benchmarks

### `ml-kem.rs` - Criterion-based Benchmarks

Standard benchmarks using the Criterion framework for statistical analysis and reporting.

```bash
cargo bench --bench ml-kem
```

### `ml-kem-cycle.rs` - CPU Cycle Counter Benchmarks

High-precision benchmarks using hardware cycle counters for accurate performance measurement.

```bash
# macOS (requires sudo and explicit cycle counter selection)
sudo CYCLE_COUNTER=MAC_CYCLES cargo bench --bench ml-kem-cycle

# Linux
sudo CYCLE_COUNTER=PMU_CYCLES cargo bench --bench ml-kem-cycle
```

## ML-KEM Cycle Counter Benchmarks

The `ml-kem-cycle` benchmark provides precise cycle-level performance measurements for ML-KEM operations across different security levels and implementations.

### Supported Operations

- **Key Generation** (`keypair`) - Generate ML-KEM key pairs
- **Encapsulation** (`encaps`) - Encapsulate shared secrets
- **Decapsulation** (`decaps`) - Decapsulate shared secrets

### Supported Security Levels

- **ML-KEM-512** - AES-128 equivalent security
- **ML-KEM-768** - AES-192 equivalent security
- **ML-KEM-1024** - AES-256 equivalent security

### Supported Implementations

- **`portable`** - Pure Rust implementation (default)
- **`neon`** - ARM NEON SIMD optimized (ARM64 only)
- **`avx2`** - Intel AVX2 SIMD optimized (x86_64 only)

### Usage

#### macOS Usage (requires sudo)

```bash
# Run all benchmarks with default settings
sudo CYCLE_COUNTER=MAC_CYCLES cargo bench --bench ml-kem-cycle

# Benchmark specific architecture
sudo CYCLE_COUNTER=MAC_CYCLES cargo bench --bench ml-kem-cycle portable
sudo CYCLE_COUNTER=MAC_CYCLES cargo bench --bench ml-kem-cycle --features simd128 neon

# Benchmark specific security level
sudo CYCLE_COUNTER=MAC_CYCLES cargo bench --bench ml-kem-cycle portable 512
sudo CYCLE_COUNTER=MAC_CYCLES cargo bench --bench ml-kem-cycle portable 768
sudo CYCLE_COUNTER=MAC_CYCLES cargo bench --bench ml-kem-cycle portable 1024

# Custom number of iterations
sudo CYCLE_COUNTER=MAC_CYCLES cargo bench --bench ml-kem-cycle portable 512 5000
```

#### Linux Usage (requires sudo)

```bash
# Run all benchmarks with default settings
sudo CYCLE_COUNTER=PMU_CYCLES cargo bench --bench ml-kem-cycle

# Benchmark specific architecture
sudo CYCLE_COUNTER=PMU_CYCLES cargo bench --bench ml-kem-cycle portable
sudo CYCLE_COUNTER=PMU_CYCLES cargo bench --bench ml-kem-cycle --features simd256 avx2    # x86_64

# Benchmark specific security level
sudo CYCLE_COUNTER=PMU_CYCLES cargo bench --bench ml-kem-cycle portable 512

# Alternative: Use perf_event (may not require sudo)
CYCLE_COUNTER=PERF_CYCLES cargo bench --bench ml-kem-cycle portable
```

#### Cross-compilation

```bash
# Build for ARM64 target
sudo CYCLE_COUNTER=PMU_CYCLES cargo bench --bench ml-kem-cycle --target aarch64-unknown-linux-gnu --features simd128 neon
```

### Output Format

The benchmark outputs cycle counts and percentile distributions:

```
ML-KEM-512 Benchmark Results
============================
keypair cycles = 34938
encaps cycles = 38331
decaps cycles = 43913
          percentile     1    10    20    30    40    50    60    70    80    90    99
keypair percentiles: 34129 34462 34632 34754 34848 34938 35039 35154 35286 35497 37627
encaps  percentiles: 37450 37895 38046 38152 38243 38331 38443 38547 38680 38946 43106
decaps  percentiles: 43110 43420 43591 43698 43811 43913 44016 44142 44283 44510 46454
```

### Cycle Counter Implementation

The benchmark uses hardware performance counters for accurate measurements:

- **Linux**: Performance Monitoring Unit (PMU) counters or perf_event
- **macOS**: Apple kperf framework (**requires sudo**)
- **Fallback**: High-resolution timers

#### Platform-specific Notes

**macOS (Apple Silicon/Intel):**
```bash
# REQUIRED: Use sudo and specify MAC_CYCLES
sudo CYCLE_COUNTER=MAC_CYCLES cargo bench --bench ml-kem-cycle

# The kperf framework requires elevated privileges
# You may see "kpc_force_all_ctrs_set failed" - this is normal
```

**Linux:**
```bash
# Option 1: PMU counters (recommended, requires sudo)
sudo CYCLE_COUNTER=PMU_CYCLES cargo bench --bench ml-kem-cycle

# Option 2: perf_event system (may not require sudo)
CYCLE_COUNTER=PERF_CYCLES cargo bench --bench ml-kem-cycle

# Option 3: Configure perf_event_paranoid to allow unprivileged access
echo 1 | sudo tee /proc/sys/kernel/perf_event_paranoid
CYCLE_COUNTER=PERF_CYCLES cargo bench --bench ml-kem-cycle
```

### Required Environment Variables

The benchmark **requires** explicit cycle counter selection:

| Platform | Required Command |
|----------|------------------|
| macOS | `sudo CYCLE_COUNTER=MAC_CYCLES` |
| Linux | `sudo CYCLE_COUNTER=PMU_CYCLES` or `CYCLE_COUNTER=PERF_CYCLES` |

### Dependencies

The cycle counter benchmarks require:
- **Build-time**: `cc` crate for compiling C code
- **Runtime**: Platform-specific performance counter access (**sudo required**)

### File Structure
```
benches/
├── cycle_counter/ # C implementation for cycle counting
│ ├── hal.c # Hardware abstraction layer
│ └── hal.h # Header file
├── cycle_counter.rs # Rust FFI bindings
├── ml-kem-cycle.rs # Main benchmark implementation
├── ml-kem.rs # Criterion benchmarks
└── README.md
```


### Troubleshooting

**Build Errors:**
- Ensure `cc` crate can find a C compiler
- Check that cycle counter source files exist in `cycle_counter/`

**Runtime Errors:**
- **Always use `sudo`** for cycle counter access
- **Always specify `CYCLE_COUNTER`** environment variable
- On macOS, "kpc_force_all_ctrs_set failed" warning is normal

**Permission Denied:**
```bash
# Make sure you're using sudo
sudo CYCLE_COUNTER=MAC_CYCLES cargo bench --bench ml-kem-cycle  # macOS
sudo CYCLE_COUNTER=PMU_CYCLES cargo bench --bench ml-kem-cycle  # Linux
```

**Inaccurate Results:**
- Disable CPU frequency scaling and turbo boost
- Run multiple times and compare percentile distributions
- Consider system load and background processes
- Use `sudo` for accurate cycle counting

### Example Commands Summary

```bash
# macOS - Basic usage
sudo CYCLE_COUNTER=MAC_CYCLES cargo bench --bench ml-kem-cycle

# macOS - Specific security level
sudo CYCLE_COUNTER=MAC_CYCLES cargo bench --bench ml-kem-cycle portable 768

# Linux - PMU counters
sudo CYCLE_COUNTER=PMU_CYCLES cargo bench --bench ml-kem-cycle

# Linux - perf_event (alternative)
CYCLE_COUNTER=PERF_CYCLES cargo bench --bench ml-kem-cycle