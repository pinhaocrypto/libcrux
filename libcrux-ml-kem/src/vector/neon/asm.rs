//! Assembly bindings for AArch64 NEON optimized ML-KEM operations

// External symbols from assembly files
extern "C" {
    fn ntt_neon_asm(poly: *mut i16, zetas_layer12345: *const i16, zetas_layer67: *const i16);
    fn intt_neon_asm(poly: *mut i16, zetas_layer12345: *const i16, zetas_layer67: *const i16);

    // Zeta tables from C files
    static aarch64_ntt_zetas_layer12345: [i16; 200];
    static aarch64_ntt_zetas_layer67: [i16; 32];
    static mlk_aarch64_invntt_zetas_layer12345: [i16; 200];
    static mlk_aarch64_invntt_zetas_layer67: [i16; 32];
}

/// Safe wrapper for forward NTT
///
/// # Safety
/// - `poly` must point to a valid array of exactly 256 i16 elements
/// - The array must be properly aligned for NEON operations
#[inline(always)]
pub unsafe fn ntt_asm(poly: *mut i16) {
    ntt_neon_asm(
        poly,
        aarch64_ntt_zetas_layer12345.as_ptr(),
        aarch64_ntt_zetas_layer67.as_ptr(),
    );
}

/// Safe wrapper for inverse NTT
///
/// # Safety  
/// - `poly` must point to a valid array of exactly 256 i16 elements
/// - The array must be properly aligned for NEON operations
#[inline(always)]
pub unsafe fn intt_asm(poly: *mut i16) {
    intt_neon_asm(
        poly,
        mlk_aarch64_invntt_zetas_layer12345.as_ptr(),
        mlk_aarch64_invntt_zetas_layer67.as_ptr(),
    );
}

/// Get forward NTT zetas for layers 1-5
#[inline(always)]
pub fn get_ntt_zetas_layer12345() -> &'static [i16; 200] {
    unsafe { &aarch64_ntt_zetas_layer12345 }
}

/// Get forward NTT zetas for layers 6-7  
#[inline(always)]
pub fn get_ntt_zetas_layer67() -> &'static [i16; 32] {
    unsafe { &aarch64_ntt_zetas_layer67 }
}

/// Get inverse NTT zetas for layers 1-5
#[inline(always)]
pub fn get_invntt_zetas_layer12345() -> &'static [i16; 200] {
    unsafe { &mlk_aarch64_invntt_zetas_layer12345 }
}

/// Get inverse NTT zetas for layers 6-7
#[inline(always)]
pub fn get_invntt_zetas_layer67() -> &'static [i16; 32] {
    unsafe { &mlk_aarch64_invntt_zetas_layer67 }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zetas_accessibility() {
        // Test that we can access the zeta constants without panicking
        let layer12345 = get_ntt_zetas_layer12345();
        let layer67 = get_ntt_zetas_layer67();
        let inv_layer12345 = get_invntt_zetas_layer12345();
        let inv_layer67 = get_invntt_zetas_layer67();

        assert_eq!(layer12345.len(), 200);
        assert_eq!(layer67.len(), 32);
        assert_eq!(inv_layer12345.len(), 200);
        assert_eq!(inv_layer67.len(), 32);

        // Basic sanity check - first element should not be zero
        // (since these are actual NTT twiddle factors)
        assert_ne!(layer12345[0], 0);
        assert_ne!(layer67[0], 0);
    }
}
