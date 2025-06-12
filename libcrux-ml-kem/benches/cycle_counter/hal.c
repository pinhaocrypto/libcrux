/*
 * Cycle counter implementation for libcrux-ml-kem benchmarks
 */

#if defined(__linux__)
#if !defined(_GNU_SOURCE)
#define _GNU_SOURCE
#endif
#endif

#include "hal.h"

#if defined(PMU_CYCLES)

#if defined(__x86_64__)

void enable_cyclecounter(void) {}

void disable_cyclecounter(void) {}

uint64_t get_cyclecounter(void) {
  uint64_t result;
  __asm__ volatile("rdtsc; shlq $32,%%rdx; orq %%rdx,%%rax"
                   : "=a"(result)
                   :
                   : "%rdx");
  return result;
}

#elif defined(__AARCH64EL__) || defined(_M_ARM64) || defined(__aarch64__)

void enable_cyclecounter(void) {
  uint64_t tmp;
  __asm __volatile("mrs    %[tmp], pmcr_el0\n"
                   "orr    %[tmp], %[tmp], #1\n"
                   "msr    pmcr_el0, %[tmp]\n"
                   "mrs    %[tmp], pmcntenset_el0\n"
                   "orr    %[tmp], %[tmp], #1<<31\n"
                   "msr    pmcntenset_el0, %[tmp]\n"
                   : [tmp] "=r"(tmp));
}

void disable_cyclecounter(void) {
  uint64_t tmp;
  __asm __volatile("mov   %[tmp], #0x3f\n"
                   "orr   %[tmp], %[tmp], #1<<31\n"
                   "msr    pmcntenclr_el0, %[tmp]\n"
                   : [tmp] "=r"(tmp));
}

uint64_t get_cyclecounter(void) {
  uint64_t retval;
  __asm __volatile("mrs    %[retval], pmccntr_el0\n" : [retval] "=r"(retval));
  return retval;
}

#else
#error PMU_CYCLES option only supported on x86_64 and AArch64
#endif

#elif defined(PERF_CYCLES)

#include <asm/unistd.h>
#include <linux/perf_event.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/ioctl.h>
#include <sys/syscall.h>
#include <unistd.h>

static int perf_fd = 0;

void enable_cyclecounter(void) {
  struct perf_event_attr pe;
  memset(&pe, 0, sizeof(struct perf_event_attr));
  pe.type = PERF_TYPE_HARDWARE;
  pe.size = sizeof(struct perf_event_attr);
  pe.config = PERF_COUNT_HW_CPU_CYCLES;
  pe.disabled = 1;
  pe.exclude_kernel = 1;
  pe.exclude_hv = 1;

  perf_fd = syscall(__NR_perf_event_open, &pe, 0, -1, -1, 0);
  ioctl(perf_fd, PERF_EVENT_IOC_RESET, 0);
  ioctl(perf_fd, PERF_EVENT_IOC_ENABLE, 0);
}

void disable_cyclecounter(void) {
  ioctl(perf_fd, PERF_EVENT_IOC_DISABLE, 0);
  close(perf_fd);
}

uint64_t get_cyclecounter(void) {
  long long cpu_cycles;
  ioctl(perf_fd, PERF_EVENT_IOC_DISABLE, 0);
  ssize_t read_count = read(perf_fd, &cpu_cycles, sizeof(cpu_cycles));
  if (read_count < 0) {
    perror("read");
    exit(EXIT_FAILURE);
  } else if (read_count == 0) {
    printf("perf counter empty\n");
    exit(EXIT_FAILURE);
  }
  ioctl(perf_fd, PERF_EVENT_IOC_ENABLE, 0);
  return cpu_cycles;
}

#elif defined(MAC_CYCLES)

#include <dlfcn.h>
#include <pthread.h>
#include <stdio.h>
#include <stdlib.h>

#define KPERF_LIST                                                             \
  F(int, kpc_get_counting, void)                                               \
  F(int, kpc_force_all_ctrs_set, int)                                          \
  F(int, kpc_set_counting, uint32_t)                                           \
  F(int, kpc_set_thread_counting, uint32_t)                                    \
  F(int, kpc_set_config, uint32_t, void *)                                     \
  F(int, kpc_get_config, uint32_t, void *)                                     \
  F(int, kpc_set_period, uint32_t, void *)                                     \
  F(int, kpc_get_period, uint32_t, void *)                                     \
  F(uint32_t, kpc_get_counter_count, uint32_t)                                 \
  F(uint32_t, kpc_get_config_count, uint32_t)                                  \
  F(int, kperf_sample_get, int *)                                              \
  F(int, kpc_get_thread_counters, int, unsigned, void *)

#define F(ret, name, ...)                                                      \
  typedef ret name##proc(__VA_ARGS__);                                         \
  static name##proc *name;
KPERF_LIST
#undef F

#define CFGWORD_EL0A64EN_MASK (0x20000)
#define CPMU_CORE_CYCLE 0x02
#define KPC_CLASS_FIXED (0)
#define KPC_CLASS_CONFIGURABLE (1)
#define KPC_CLASS_FIXED_MASK (1u << KPC_CLASS_FIXED)
#define KPC_CLASS_CONFIGURABLE_MASK (1u << KPC_CLASS_CONFIGURABLE)
#define COUNTERS_COUNT 10
#define KPC_MASK (KPC_CLASS_CONFIGURABLE_MASK | KPC_CLASS_FIXED_MASK)

uint64_t g_counters[COUNTERS_COUNT];
uint64_t g_config[COUNTERS_COUNT];

static void configure_rdtsc(void) {
  if (kpc_force_all_ctrs_set(1)) {
    printf("kpc_force_all_ctrs_set failed\n");
    return;
  }
  if (kpc_set_counting(KPC_MASK)) {
    printf("kpc_set_counting failed\n");
    return;
  }
  if (kpc_set_thread_counting(KPC_MASK)) {
    printf("kpc_set_thread_counting failed\n");
    return;
  }
  if (kpc_set_config(KPC_MASK, g_config)) {
    printf("kpc_set_config failed\n");
    return;
  }
}

static void init_rdtsc(void) {
  void *kperf = dlopen(
      "/System/Library/PrivateFrameworks/kperf.framework/Versions/A/kperf",
      RTLD_LAZY);
  if (!kperf) {
    printf("kperf = %p\n", kperf);
    return;
  }

#pragma GCC diagnostic push
#pragma GCC diagnostic ignored "-Wpedantic"

#define F(ret, name, ...)                                                      \
  name = (name##proc *)(dlsym(kperf, #name));                                  \
  if (!name) {                                                                 \
    printf("%s = %p\n", #name, (void *)name);                                  \
    return;                                                                    \
  }
  KPERF_LIST
#undef F

#pragma GCC diagnostic pop

  g_config[0] = CPMU_CORE_CYCLE | CFGWORD_EL0A64EN_MASK;
}

void enable_cyclecounter(void) {
  pthread_set_qos_class_self_np(QOS_CLASS_USER_INTERACTIVE, 0);
  init_rdtsc();
  configure_rdtsc();
}

void disable_cyclecounter(void) { return; }

uint64_t get_cyclecounter(void) {
  if (kpc_get_thread_counters(0, COUNTERS_COUNT, g_counters)) {
    printf("kpc_get_thread_counters failed\n");
    return 1;
  }
  return g_counters[2];
}

#else

// Fallback implementation using std::time
#include <time.h>

void enable_cyclecounter(void) {}
void disable_cyclecounter(void) {}

uint64_t get_cyclecounter(void) {
  struct timespec ts;
  clock_gettime(CLOCK_MONOTONIC, &ts);
  return (uint64_t)ts.tv_sec * 1000000000ULL + (uint64_t)ts.tv_nsec;
}

#endif