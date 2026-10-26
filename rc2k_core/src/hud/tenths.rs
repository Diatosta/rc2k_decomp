//! The HUD's times in tenths (converted 2026-10-21 from `hud::tenths` 0x4B1D2B): six
//! times of the HUD's record, each divided by 10 as the original's signed `idiv` does
//! (towards zero).

/// The six times' quotients, and the last one's remainder (left in edx).
pub fn tenths(times: [i32; 6]) -> ([i32; 6], i32) {
    // `idiv` by 10: no quotient can overflow.
    (times.map(|t| t / 10), times[5] % 10)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn times() {
        assert_eq!(
            tenths([0, 9, 10, -9, -11, i32::MIN]),
            ([0, 0, 1, 0, -1, -214_748_364], -8)
        );
        assert_eq!(tenths([0, 0, 0, 0, 0, -17]).1, -7);
    }
}
