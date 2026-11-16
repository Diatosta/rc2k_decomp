//! The HUD's times in tenths: six times of the HUD's record, each divided by 10 (towards zero).

/// The six times' quotients, and the last one's remainder.
pub fn tenths(times: [i32; 6]) -> ([i32; 6], i32) {
    // No quotient can overflow.
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
