//! Texts the menus compose for their screens: saved-game page numbers, spaced-out numbers,
//! yes/no captions, player numbers and the gearing choice.

use crate::Invalid;
use crate::strings::format::{Pad, decimal};

/// A text and its NUL in a place of `room` bytes. Refused: one that does not fit.
pub fn fit(mut text: Vec<u8>, room: usize) -> Result<Vec<u8>, Invalid> {
    if text.len() >= room {
        return Err(Invalid("a text longer than its place"));
    }
    text.push(0);
    Ok(text)
}

/// A number with at least two digits.
pub fn two_digits(n: u32) -> Vec<u8> {
    let mut text = if n < 10 {
        decimal(0, Pad::None)
    } else {
        Vec::new()
    };
    text.extend(decimal(n, Pad::None));
    text
}

/// The saved games' list numbers for page `page`: the caption ("n-m") and ten numbers (each at
/// most three digits and a NUL, in four bytes). Refused: numbers that do not fit (the compares
/// are signed, so a negative first number is refused too).
pub fn savegame_numbers(page: u32) -> Result<(Vec<u8>, Vec<Vec<u8>>), Invalid> {
    let first = page.wrapping_mul(10);
    if (first as i32) < 0 || first.wrapping_add(9) >= 1000 {
        return Err(Invalid("a page whose numbers do not fit their places"));
    }
    let numbers: Vec<Vec<u8>> = (0..10).map(|n| two_digits(first + n)).collect();
    let mut caption = two_digits(first);
    caption.push(b'-');
    caption.extend(two_digits(first + 9));
    caption.push(0);
    Ok((caption, numbers))
}

/// Where `draw_spaced_number` draws each character of a text of `len`, last first, as
/// (index, x): the last at x + 0x32, the next two 9 apart, the rest 0xD.
/// Refused: fewer than 3 or more than `room` / 2.
pub fn spaced_places(len: u32, x: i32, room: usize) -> Result<Vec<(usize, i32)>, Invalid> {
    if len < 3 {
        return Err(Invalid(
            "a spaced number shorter than 3: the original runs on",
        ));
    }
    if len as usize * 2 > room {
        return Err(Invalid("a spaced number past its cells"));
    }
    let mut x = x.wrapping_add(0x32);
    Ok((0..len as usize)
        .rev()
        .enumerate()
        .map(|(k, i)| {
            let at = (i, x);
            x = x.wrapping_sub(if k < 2 { 9 } else { 0xD });
            at
        })
        .collect())
}

/// The gearing choice stepped up (`on`) or down, round 0 ..= 2.
pub fn step_gearing(kept: i32, on: bool) -> u32 {
    (if on {
        let v = kept.wrapping_add(1);
        if v > 2 { 0 } else { v }
    } else {
        let v = kept.wrapping_sub(1);
        if v < 0 { 2 } else { v }
    }) as u32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn captions() {
        let (caption, numbers) = savegame_numbers(1).unwrap();
        assert_eq!(caption, b"10-19\0");
        assert_eq!(numbers[0], b"10");
        assert_eq!(savegame_numbers(0).unwrap().1[3], b"03");
        assert!(savegame_numbers(100).is_err());
        assert_eq!(
            spaced_places(4, 0, 0x100).unwrap(),
            [(3, 0x32), (2, 0x29), (1, 0x20), (0, 0x13)]
        );
        assert!(spaced_places(2, 0, 0x100).is_err());
        assert_eq!(fit(b"ab".to_vec(), 3).unwrap(), b"ab\0");
        assert!(fit(b"abc".to_vec(), 3).is_err());
        assert_eq!((step_gearing(2, true), step_gearing(0, false)), (0, 2));
    }
}
