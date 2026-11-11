//! A car's marks' textures (from `race::car_number_textures`
//! 0x46203D): which lamps' sides show, the letter of the model's name by the car's +0x718,
//! the race number's digits, the stage's plate, whether the decals show.

/// What the number's digits read when there is none.
pub const NO_DIGIT: u32 = 999;

/// A lamp's side mark (+0x1B8 / +0x1BC): 1 unless |x| is below the limit (`decal_limit`)
/// or not a number (`fabs` / `fcomp`).
pub fn lamp_side(x: f32, limit: f32) -> u32 {
    u32::from(x.abs() >= limit)
}

/// The letter's place in the model's name and the remainder: the car's +0x718 (rounded
/// by the caller) held to 0 .. 0x2EDF (signed), over 250.
pub fn letter_place(value: i32) -> (u32, u32) {
    let held = value.clamp(0, 0x2EDF) as u32;
    (held / 250, held % 250)
}

/// The letter's number: a digit's value, a letter's from 10 on ('a' 10; `sub al, 0x30`,
/// then 0x27 off at 10 or more, unsigned).
pub fn letter_number(character: u8) -> u8 {
    let al = character.wrapping_sub(0x30);
    if al >= 10 { al.wrapping_sub(0x27) } else { al }
}

/// The race number's digits: the tens and the units (`dword_5CD6A0`, `dword_5CD6A4`) and
/// the units alone (`dword_5CD6AC`) for a number below ten; `NO_DIGIT` where none.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Digits {
    pub tens: u32,
    pub units: u32,
    pub alone: u32,
}

pub fn number_digits(number: u32) -> Digits {
    let (tens, units) = (number / 10, number % 10);
    if tens != 0 {
        Digits {
            tens,
            units,
            alone: NO_DIGIT,
        }
    } else {
        Digits {
            tens: NO_DIGIT,
            units: NO_DIGIT,
            alone: units,
        }
    }
}

/// The stage's plate: the stage less 11 held to 0 .. 0x45 (signed), its tens (the
/// plate's entry, `number_tens_textures`[]`) and units.
pub fn stage_plate(stage: u32) -> (u32, u32) {
    let s = (stage.wrapping_sub(0xB) as i32).clamp(0, 0x45) as u32;
    (s / 10, s % 10)
}

/// Whether the decals' bytes (+0xAC0 ..) go into the marks: while the car's +0x310 is
/// below the limit (`hundred`, 100) or not a number.
pub fn decals_shown(wear: f32, limit: f32) -> bool {
    !matches!(
        wear.partial_cmp(&limit),
        Some(core::cmp::Ordering::Greater | core::cmp::Ordering::Equal)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn marks() {
        assert_eq!(
            [-2.0f32, 0.5, f32::NAN, 1.0].map(|x| lamp_side(x, 1.0)),
            [1, 0, 0, 1]
        );
        assert_eq!(letter_place(-5), (0, 0));
        assert_eq!(letter_place(0x2EDF), (47, 249));
        assert_eq!(letter_place(i32::MIN), (0, 0));
        assert_eq!(letter_place(0x7FFF_FFFF), (47, 249));
        assert_eq!(letter_place(751), (3, 1));
        assert_eq!(b"7aA ".map(letter_number), [7, 10, 0xEA, 0xC9]);
        assert_eq!(
            number_digits(42),
            Digits {
                tens: 4,
                units: 2,
                alone: NO_DIGIT
            }
        );
        assert_eq!(number_digits(7).alone, 7);
        assert_eq!(number_digits(7).tens, NO_DIGIT);
        assert_eq!(stage_plate(0), (0, 0));
        assert_eq!(stage_plate(0x2A), (3, 1));
        assert_eq!(stage_plate(0x100), (6, 9));
        assert!(decals_shown(99.0, 100.0) && decals_shown(f32::NAN, 100.0));
        assert!(!decals_shown(100.0, 100.0));
    }
}
