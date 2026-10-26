//! The HUD's signs (converted 2026-10-21 from `hud::draw_signs` 0x4B4F2B and its four
//! twins `draw_signs_1a` 0x4B52F7, `_1b`, `_2a`, `_2b`): up to four signs of the HUD's
//! record, each a rectangle of the HUD's picture centred in a place 0x40 wide; and the
//! rows of the HUD, which move up when `dword_6D1502` is 0.

/// The signs a record holds.
pub const SIGNS: usize = 4;
/// The width of a sign's place.
pub const PLACE_WIDTH: u32 = 0x40;
/// The rectangle of sign 1 in the HUD's rectangles (`dword_75B940`); 0x10 bytes each.
pub const FIRST_RECTANGLE: u32 = 0x4C;

/// A row: `y`, or `lift` higher with the switch (`dword_6D1502`) 0.
pub fn row(y: u32, lift: u32, switch: u32) -> u32 {
    if switch == 0 { y.wrapping_sub(lift) } else { y }
}

/// A sign drawn.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Sign {
    /// Its place's left (`ebx` before the sprite is centred).
    pub place: u32,
    /// The sprite's x: the place plus half what the sign's width leaves (an arithmetic
    /// shift).
    pub x: u32,
    /// Its rectangle's byte offset in the rectangles: 0x4C + (v - 1) & 0xF, times 0x10.
    pub rectangle: u32,
}

/// The place of sign n: the first place plus 0x40 a sign.
pub fn place(first: u32, n: u32) -> u32 {
    first.wrapping_add(PLACE_WIDTH.wrapping_mul(n))
}

/// Sign n of value `v` (0 none) at `first`'s places; `width(k)` the widths' table
/// (`dword_75BF80`) at (v - 1) & 7, asked only for a sign.
pub fn sign(first: u32, n: u32, v: u32, width: impl FnOnce(u32) -> u32) -> Option<Sign> {
    if v == 0 {
        return None;
    }
    let w = width(v.wrapping_sub(1) & 7);
    let place = place(first, n);
    let x = ((PLACE_WIDTH.wrapping_sub(w) as i32) >> 1) as u32;
    Some(Sign {
        place,
        x: x.wrapping_add(place),
        rectangle: ((v.wrapping_sub(1) & 0xF) + FIRST_RECTANGLE) * 0x10,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn signs() {
        assert_eq!(row(0x36, 0x20, 0), 0x16);
        assert_eq!(row(0x36, 0x20, 1), 0x36);
        assert_eq!(sign(0x100, 1, 0, |_| unreachable!()), None);
        assert_eq!(
            sign(0x100, 2, 3, |k| {
                assert_eq!(k, 2);
                0x20
            }),
            Some(Sign {
                place: 0x180,
                x: 0x190,
                rectangle: 0x4E0
            })
        );
        // A sign wider than its place: centred to the left.
        assert_eq!(sign(0x100, 0, 0x11, |_| 0x44).unwrap().x, 0xFE);
        assert_eq!(sign(0, 0, 0x11, |_| 0).unwrap().rectangle, 0x4C0);
    }
}
