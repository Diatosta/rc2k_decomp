//! The HUD's time panels (converted 2026-10-21 from `hud::layout_0` 0x4B1D8C,
//! `hud::layout_1` 0x4B2743 and `hud::layout_2`): a time in hundredths made six digits,
//! each a rectangle of the HUD's sprite sheet (`dword_75B940`, 0x10 bytes a rectangle), and
//! the rules the three layouts share: the clock and lap panels' rectangles, the times with
//! their leading digits left out, the stage panel's number and the needle's sign.

/// The six digits are the time's remainder by this (an hour and a half in hundredths:
/// minutes go up to 89).
pub const WRAP: u32 = 0x57E40;
/// The divisions after it: tens of minutes, minutes, tens of seconds, seconds, tenths;
/// what is left is the hundredths.
pub const DIVISORS: [u32; 5] = [60000, 6000, 1000, 100, 10];

/// A time's digits (`dword_75C08C` ..): its size (the original's `neg` of a negative,
/// wrapping) by `WRAP`, then by `DIVISORS`; each plus `base` (the first digit's rectangle).
pub fn time_digits(value: u32, base: u32) -> [u32; 6] {
    let mut r = if (value as i32) < 0 {
        value.wrapping_neg()
    } else {
        value
    };
    r %= WRAP;
    let mut d = [0u32; 6];
    for (n, k) in DIVISORS.into_iter().enumerate() {
        d[n] = r / k + base;
        r %= k;
    }
    d[5] = r + base;
    d
}

/// What the divisions leave in eax, ebx, edx: the tenths' digit, 10, the hundredths'.
pub fn division_registers(d: &[u32; 6]) -> [u32; 3] {
    [d[4], 10, d[5]]
}

/// The sign of a time (`dword_75C0A4`): `minus` for a negative one (signed), else `plus`.
pub fn sign(value: u32, plus: u32, minus: u32) -> u32 {
    if (value as i32) < 0 { minus } else { plus }
}

/// The rectangles of the clock panel: the sign (0x22 plus, 0x23 minus), the digits 1 .. 5
/// (from 0x18) with the colons 0x24 and 0x25 between.
pub const CLOCK_BASE: u32 = 0x18;
pub fn clock_rects(value: u32) -> [u32; 8] {
    let d = time_digits(value, CLOCK_BASE);
    let s = sign(value, 0x22, 0x23);
    [s, d[1], 0x24, d[2], d[3], 0x25, d[4], d[5]]
}

/// The rectangles of the lap panel: the six digits (from 0xC) with 0x16 and 0x17 between;
/// its sign is 0 or 1 (kept, not drawn).
pub const LAP_BASE: u32 = 0xC;
pub fn lap_rects(value: u32) -> [u32; 8] {
    let d = time_digits(value, LAP_BASE);
    [d[0], d[1], 0x16, d[2], d[3], 0x17, d[4], d[5]]
}

/// A time's last three digits (from 0xC) with its leading ones left out while they are 0
/// (`dword_75C03C` set first, cleared at the first digit drawn): (the digit, its slot 0 ..
/// 2, whether it is drawn) in order. The last is always drawn.
pub fn leading_digits(value: u32) -> [(u32, usize, bool); 3] {
    let d = time_digits(value, LAP_BASE);
    let mut leading = true;
    let mut out = [(0, 0, true); 3];
    for (slot, digit) in [d[3], d[4]].into_iter().enumerate() {
        let drawn = !(leading && digit == LAP_BASE);
        if drawn {
            leading = false;
        }
        out[slot] = (digit, slot, drawn);
    }
    out[2] = (d[5], 2, true);
    out
}

/// Whether `dword_75C03C` is left set after `leading_digits`: no digit but the last drawn.
pub fn leading_left(value: u32) -> bool {
    !leading_digits(value)[..2]
        .iter()
        .any(|&(_, _, drawn)| drawn)
}

/// The stage panel's rectangle by the record's +0x28 / +0x30 (none for -1): the number held
/// to 0 .. 0x16 (signed) from 0x29.
pub fn stage_rect(field: u32) -> Option<u32> {
    if field == u32::MAX {
        return None;
    }
    Some((field as i32).clamp(0, 0x16) as u32 + 0x29)
}

/// The needle's sign by the record's +0xCC (+0xD0): its rectangle (`dword_75C040`) and
/// texture: 0x5E on 0xC0, or with the field set 0x60 on 0xC2.
pub fn needle_sign(field: u32) -> (u32, u32) {
    if field != 0 {
        (0x60, 0xC2)
    } else {
        (0x5E, 0xC0)
    }
}

/// A sprite's rectangle's offset in the sheet (eax going in).
pub fn rect_offset(rect: u32) -> u32 {
    rect.wrapping_mul(0x10)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn panels() {
        // 1:23.45 (8345 hundredths).
        assert_eq!(time_digits(8345, 0), [0, 1, 2, 3, 4, 5]);
        assert_eq!(time_digits(8345u32.wrapping_neg(), 2), [2, 3, 4, 5, 6, 7]);
        assert_eq!(time_digits(WRAP + 7, 0), [0, 0, 0, 0, 0, 7]);
        assert_eq!(
            time_digits(0x8000_0000, 0),
            time_digits(0x8000_0000 % WRAP, 0)
        );
        assert_eq!(division_registers(&[0, 1, 2, 3, 4, 5]), [4, 10, 5]);
        assert_eq!(sign(u32::MAX, 0, 1), 1);
        assert_eq!(sign(5, 0x22, 0x23), 0x22);
        assert_eq!(
            clock_rects(8345),
            [0x22, 0x19, 0x24, 0x1A, 0x1B, 0x25, 0x1C, 0x1D]
        );
        assert_eq!(
            lap_rects(8345),
            [0xC, 0xD, 0x16, 0xE, 0xF, 0x17, 0x10, 0x11]
        );
        assert_eq!(
            leading_digits(5),
            [(0xC, 0, false), (0xC, 1, false), (0x11, 2, true)]
        );
        assert!(leading_left(5));
        assert_eq!(
            leading_digits(105),
            [(0xD, 0, true), (0xC, 1, true), (0x11, 2, true)]
        );
        assert_eq!(
            leading_digits(45),
            [(0xC, 0, false), (0x10, 1, true), (0x11, 2, true)]
        );
        assert!(!leading_left(45));
        assert_eq!(stage_rect(u32::MAX), None);
        assert_eq!(stage_rect(0xFFFF_FFFE), Some(0x29));
        assert_eq!(stage_rect(0x40), Some(0x29 + 0x16));
        assert_eq!(stage_rect(3), Some(0x2C));
        assert_eq!(needle_sign(0), (0x5E, 0xC0));
        assert_eq!(needle_sign(2), (0x60, 0xC2));
        assert_eq!(rect_offset(0x5C), 0x5C0);
    }
}
