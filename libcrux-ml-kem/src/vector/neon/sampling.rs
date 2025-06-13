use super::intrinsics::*;
use super::serialize::deserialize_12;

#[inline(always)]
pub(crate) fn rej_sample(a: &[u8], out: &mut [i16]) -> usize {
    // Use portable implementation for now as NEON-optimized version needs more work
    crate::vector::portable::rej_sample(a, out)
}
