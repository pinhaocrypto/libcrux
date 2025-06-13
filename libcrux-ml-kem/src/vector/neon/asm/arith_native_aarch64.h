/*
 * Copyright (c) The mlkem-native project authors
 * SPDX-License-Identifier: Apache-2.0 OR ISC OR MIT
 */


#include <stdint.h>

extern const int16_t aarch64_ntt_zetas_layer12345[];
extern const int16_t aarch64_ntt_zetas_layer67[];
extern const int16_t aarch64_invntt_zetas_layer12345[];
extern const int16_t aarch64_invntt_zetas_layer67[];


void ntt_neon_asm(int16_t *p, const int16_t *twiddles12345,
                 const int16_t *twiddles56);
void intt_neon_asm(int16_t *p, const int16_t *twiddles12345,
                  const int16_t *twiddles56);

