#ifndef CYCLE_COUNTER_HAL_H
#define CYCLE_COUNTER_HAL_H

#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

void enable_cyclecounter(void);
void disable_cyclecounter(void);
uint64_t get_cyclecounter(void);

#ifdef __cplusplus
}
#endif

#endif /* CYCLE_COUNTER_HAL_H */