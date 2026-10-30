//! The debug overlay's figures (IDA 2026-10-30): `sub_4203E7` (a whole number), `sub_420419`
//! (a float as `+ddd.ddd`, or `+dddddd` when it rounds outside -999 .. 999), `sub_4205CF`
//! (eight hex digits), each written into the number text (`g_debugNumberText`, 0x180
//! bytes); the bars' lengths and the rectangle fill's clamping (`sub_41FF44`).

/// The number text's bytes.
pub const TEXT_LEN: usize = 0x180;

/// `sub_4203E7`: `value` in decimal, '-' first when negative, then a NUL.
pub fn whole(value: i32, text: &mut [u8; TEXT_LEN]) {
    let mut at = 0;
    if value < 0 {
        text[at] = b'-';
        at += 1;
    }
    let digits = value.unsigned_abs().to_string();
    text[at..at + digits.len()].copy_from_slice(digits.as_bytes());
    text[at + digits.len()] = 0;
}

/// `sub_4205CF`: `value`'s eight hex digits (lower case, `byte_5C5F28`), then a NUL.
pub fn hex(value: u32, text: &mut [u8; TEXT_LEN]) {
    let digits = format!("{value:08x}");
    text[..8].copy_from_slice(digits.as_bytes());
    text[8] = 0;
}

/// `sub_420419`, given the float rounded (`rounded`, its `fistp`) and rounded after a
/// multiplication by 1000 (`thousandths`): a sign, then with `rounded` in -999 .. 999 three
/// digits, '.', three digits and no NUL (the original's: what the text held after the eighth
/// byte is read on), else six digits of `rounded` and a NUL. A digit is the low byte of the
/// quotient plus '0' (past 9 not a digit: `rounded` of a million and more).
pub fn fixed(rounded: i32, thousandths: i32, text: &mut [u8; TEXT_LEN]) {
    let small = (-999..=999).contains(&rounded);
    let value = if small { thousandths } else { rounded };
    text[0] = if value < 0 { b'-' } else { b'+' };
    let mut rest = value.unsigned_abs();
    let mut at = 1;
    for (n, place) in [100_000u32, 10_000, 1_000, 100, 10].into_iter().enumerate() {
        if small && n == 3 {
            text[at] = b'.';
            at += 1;
        }
        text[at] = ((rest / place) as u8).wrapping_add(b'0');
        rest %= place;
        at += 1;
    }
    text[at] = (rest as u8).wrapping_add(b'0');
    if !small {
        text[at + 1] = 0;
    }
}

/// A bar's right edge: `left` plus its length, `length` the rounded `64 v / 100`; a length
/// below 0 is 0, and one past 64 is kept (the original's clamp skips its upper bound).
pub fn bar_right(left: u32, length: i32) -> u32 {
    left.wrapping_add(length.max(0) as u32)
}

/// `sub_41FF44`'s clamp of a rectangle's edges (left, top, right, bottom, inclusive) to the
/// screen, each by its sign compared with 0 and with the last column or row, then what is
/// filled: (left, top, columns, rows), or nothing when right is left of left or bottom above top.
pub fn clamped(
    [left, top, right, bottom]: [u32; 4],
    width: u32,
    height: u32,
) -> Option<(u32, u32, u32, u32)> {
    let clamp = |v: u32, last: u32| {
        let v = if (v as i32) < 0 { 0 } else { v };
        if (v.wrapping_sub(last) as i32) < 0 {
            v
        } else {
            last
        }
    };
    let (last_x, last_y) = (width.wrapping_sub(1), height.wrapping_sub(1));
    let (left, right) = (clamp(left, last_x), clamp(right, last_x));
    let (top, bottom) = (clamp(top, last_y), clamp(bottom, last_y));
    let columns = right.wrapping_sub(left);
    let rows = bottom.wrapping_sub(top);
    if (columns as i32) < 0 || (rows as i32) < 0 {
        return None;
    }
    Some((left, top, columns + 1, rows + 1))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn text(f: impl FnOnce(&mut [u8; TEXT_LEN])) -> Vec<u8> {
        let mut t = [b'x'; TEXT_LEN];
        f(&mut t);
        t[..12].to_vec()
    }

    #[test]
    fn figures_are_written_as_the_original_writes_them() {
        assert_eq!(text(|t| whole(-305, t)), b"-305\0xxxxxxx");
        assert_eq!(text(|t| whole(i32::MIN, t)), b"-2147483648\0");
        assert_eq!(text(|t| hex(0xBEEF, t)), b"0000beef\0xxx");
        // Small: no NUL, the ninth byte on is what was there.
        assert_eq!(text(|t| fixed(12, 12_345, t)), b"+012.345xxxx");
        assert_eq!(text(|t| fixed(-1, -500, t)), b"-000.500xxxx");
        assert_eq!(text(|t| fixed(-4_321, 0, t)), b"-004321\0xxxx");
        // A million: the first "digit" is ':'.
        assert_eq!(text(|t| fixed(1_000_000, 0, t)), b"+:00000\0xxxx");
    }

    #[test]
    fn bars_and_rectangles() {
        assert_eq!(bar_right(4, -3), 4);
        assert_eq!(bar_right(4, 200), 204);
        assert_eq!(clamped([4, 5, 68, 9], 800, 600), Some((4, 5, 65, 5)));
        assert_eq!(
            clamped([790, 598, 900, 700], 800, 600),
            Some((790, 598, 10, 2))
        );
        assert_eq!(clamped([10, 0, 5, 0], 800, 600), None);
        assert_eq!(clamped([u32::MAX, 0, 2, 0], 800, 600), Some((0, 0, 3, 1)));
    }
}
