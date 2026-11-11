//! The work copy's +0xC word (`noise_word`; Converted from
//! `physics::test_car_word` 0x44133F): its bits tested by rotation. What they are is not
//! known yet.

/// The test of the word rotated left by
/// `rotation` (taken mod 32) and masked to 16 bits; the original's flags of `cmp` with 1
/// (carry when 0, zero when 1).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WordTest {
    pub value: u32,
    pub carry: bool,
    pub zero: bool,
}

/// `physics::test_car_word`'s rule.
pub fn test_car_word(word: u32, rotation: u8) -> WordTest {
    let value = word.rotate_left(u32::from(rotation & 31)) & 0xFFFF;
    WordTest {
        value,
        carry: value == 0,
        zero: value == 1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn words() {
        assert_eq!(
            test_car_word(0x8000_0000, 1),
            WordTest {
                value: 1,
                carry: false,
                zero: true
            }
        );
        assert!(test_car_word(0x1_0000, 32).carry);
        assert_eq!(test_car_word(0x1234_5678, 0x24).value, 0x6781);
    }
}
