//! The work copy's `noise_word`: its bits tested by rotation.

/// The word rotated left by `rotation` (mod 32) and masked to 16 bits: carry when 0, zero when 1.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WordBits {
    pub value: u32,
    pub carry: bool,
    pub zero: bool,
}

/// Tests the word's bits at `rotation`.
pub fn car_word_bits(word: u32, rotation: u8) -> WordBits {
    let value = word.rotate_left(u32::from(rotation & 31)) & 0xFFFF;
    WordBits {
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
            car_word_bits(0x8000_0000, 1),
            WordBits {
                value: 1,
                carry: false,
                zero: true
            }
        );
        assert!(car_word_bits(0x1_0000, 32).carry);
        assert_eq!(car_word_bits(0x1234_5678, 0x24).value, 0x6781);
    }
}
