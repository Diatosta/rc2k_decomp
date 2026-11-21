//! Screen modes and the picture as a surface.
//!
//! A screen mode is a dword of bits: the low byte its size, the second its pixel kind (0x200 16
//! bits, 0x800 32), the third byte's low nibble the menus' own, the third's high nibble the card
//! (0x1000 .. primary, 0x2000 secondary) and Direct3D's marks above. The mode table holds the
//! modes found, 8 bytes each: the mode's bits and a dword of extra flags.

use crate::system::tables::{Entry, with_any_bit, with_bits};

/// The mode table's room, in entries.
pub const MODES_ROOM: u32 = 0x20;
/// A mode Direct3D can draw in.
pub(crate) const DRAWS_3D: u32 = 0x10_0000;
/// A mode with extra flags in its second dword.
pub(crate) const HAS_EXTRA: u32 = 0x20_0000;

/// The 3D modes marked: from the table's start, each entry with all of `kind`'s
/// bits, and from there the next with any of `depth`'s (the depth formats Direct3D has),
/// gets `DRAWS_3D` and, with `extra` not 0, `HAS_EXTRA` and `extra` in its second dword;
/// the search goes on one entry past it, until one finds nothing.
pub fn mark_3d(table: &mut [Entry], kind: u32, depth: u32, extra: u32) {
    let mut start = 0;
    loop {
        let Some(i) = with_bits(table[start..].iter().copied(), kind) else {
            return;
        };
        let Some(j) = with_any_bit(table[start + i..].iter().copied(), depth) else {
            return;
        };
        let e = &mut table[start + i + j];
        e[0] |= DRAWS_3D;
        if extra != 0 {
            e[0] |= HAS_EXTRA;
            e[1] |= extra;
        }
        start += i + j + 1;
    }
}

/// What looking a mode up in the mode table finds.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Lookup {
    /// No bits to look for: the mode is 0 but for the menus' nibble.
    NoBits,
    Missing,
    /// The entry's place.
    Found(usize),
}

/// The bits a mode is looked up by: all but the third byte's low nibble.
pub(crate) fn lookup_bits(mode: u32) -> u32 {
    mode & 0xF0_FFFF
}

/// A mode looked up: the first entry with all of the mode's bits.
pub fn look_up(table: impl IntoIterator<Item = Entry>, mode: u32) -> Lookup {
    let bits = lookup_bits(mode);
    if bits == 0 {
        return Lookup::NoBits;
    }
    with_bits(table, bits).map_or(Lookup::Missing, Lookup::Found)
}

/// The menus' mode tried for a race's mode: its pixel kind, at 800 x 600 (size bit 2)
/// with the menus' bits (0x320000).
pub fn menu_mode_for(race_mode: u32) -> u32 {
    race_mode & 0xFF00 | 0x32_0002
}

/// A picture's flags that name no size or no pixel kind; `code` gives the fatal error.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SurfaceError {
    Size,
    PixelKind,
}

impl SurfaceError {
    pub fn code(self) -> u32 {
        match self {
            SurfaceError::Size => 0x80A1_9091,
            SurfaceError::PixelKind => 0x80A1_9092,
        }
    }
}

/// The first entry whose key is `key`.
fn entry(t: impl IntoIterator<Item = Entry>, key: u32) -> Option<Entry> {
    t.into_iter().find(|e| e[0] == key)
}

/// The picture's surface block from its third dword on, for a picture with `flags`
/// (the low byte its size, bits 8 .. 11 its pixel kind) at `pixels`: width, height, bits and bytes
/// a pixel, bytes of a row, its dwords, `format` (the surface format's address), the pitch (the
/// row's bytes), the pixels and no gap after a row. `sizes` are screen sizes' entries (key, width
/// and height as words), `kinds` pixel kinds' (key, bits).
pub fn picture_surface(
    flags: u32,
    sizes: impl IntoIterator<Item = Entry>,
    kinds: impl IntoIterator<Item = Entry>,
    format: u32,
    pixels: u32,
) -> Result<[u32; 10], SurfaceError> {
    let size = entry(sizes, flags & 0xFF).ok_or(SurfaceError::Size)?;
    let (width, height) = (size[1] & 0xFFFF, size[1] >> 16);
    let kind = entry(kinds, flags & 0xF00).ok_or(SurfaceError::PixelKind)?;
    let bits = kind[1];
    let row_bytes = (bits >> 3).wrapping_mul(width);
    Ok([
        width,
        height,
        bits,
        bits >> 3,
        row_bytes,
        row_bytes >> 2,
        format,
        row_bytes,
        pixels,
        0,
    ])
}

/// The pixel kind's bits of a mode, and the two kinds: 16 bits a pixel and 32.
const PIXEL_KIND: u32 = 0xF00;
const KIND_16_BITS: u32 = 0x200;
const KIND_32_BITS: u32 = 0x800;

/// A mode at 32 bits a pixel: a kept 16-bit mode (a 1999 screen's) becomes the same size at
/// 32 bits; any other mode stays as it is.
pub fn at_32_bits(mode: u32) -> u32 {
    if mode & PIXEL_KIND == KIND_16_BITS {
        mode & !PIXEL_KIND | KIND_32_BITS
    } else {
        mode
    }
}

/// The race modes tried in this order when the kept ones do not both work: 800 x 600, then
/// 1024 x 768, then 640 x 480, each at 32 bits.
pub(crate) const RACE_CHOICES: [u32; 3] = [0x32_1802, 0x32_1808, 0x32_1801];

/// The menus' mode tried after the race's own kind at size 2.
pub(crate) const MENU_CHOICES: [u32; 1] = [0x32_1802];

/// The (menu, race) modes: the kept ones when both work (`works`, the menu's tried first); else,
/// when a 3D mode of size 2 is listed and one with both 3D bits (`listed`: a table entry with all
/// the bits), the first race choice that works and the first menu mode that does: the race's card
/// and kind at size 2, then `MENU_CHOICES`. None where no pair works.
pub fn choose(
    menu: u32,
    race: u32,
    mut works: impl FnMut(u32) -> bool,
    listed: impl Fn(u32) -> bool,
) -> Option<(u32, u32)> {
    if works(menu) && works(race) {
        return Some((menu, race));
    }
    if !listed(0x10_0002) || !listed(0x30_0000) {
        return None;
    }
    let race = RACE_CHOICES.into_iter().find(|&m| works(m))?;
    let own = (race & 0xFF00) | 0x32_0002;
    let menu = std::iter::once(own)
        .chain(MENU_CHOICES)
        .find(|&m| works(m))?;
    Some((menu, race))
}

#[cfg(test)]
mod tests {
    #[test]
    fn choose_keeps_working_modes_and_falls_back_in_order() {
        use super::choose;
        let primary = |m: u32| m & 0x3000 == 0x1000;
        let any = |_: u32| true;
        // Both kept modes work: kept.
        assert_eq!(
            choose(0x32_1802, 0x34_1802, primary, any),
            Some((0x32_1802, 0x34_1802))
        );
        // None kept (0): 800 x 600 at 32 bits for both.
        assert_eq!(choose(0, 0, primary, any), Some((0x32_1802, 0x32_1802)));
        // A kept secondary card's mode: chosen again.
        assert_eq!(
            choose(0x32_2802, 0x32_2802, primary, any),
            Some((0x32_1802, 0x32_1802))
        );
        // A kept 16-bit mode is the same size at 32 bits; a 32-bit one and none stay.
        assert_eq!(super::at_32_bits(0x34_1280), 0x34_1880);
        assert_eq!(super::at_32_bits(0x32_1801), 0x32_1801);
        assert_eq!(super::at_32_bits(0), 0);
        // No 3D mode of size 2 listed: it fails.
        assert_eq!(choose(0, 0, primary, |_| false), None);
        // Nothing works.
        assert_eq!(choose(0, 0, |_| false, any), None);
    }

    use super::*;

    #[test]
    fn marking() {
        let mut t: Vec<Entry> = vec![
            [0x1202, 0],
            [0x1802, 0],
            [0x2202, 0],
            [0x1201, 0],
            [0x1801, 0],
        ];
        // From a kind's entry the next with a depth bit is marked, of whatever kind.
        mark_3d(&mut t, 0x1000, 0x200, 0);
        assert_eq!(t[0][0], 0x10_1202);
        assert_eq!(t[2][0], 0x10_2202);
        assert_eq!(t[3][0], 0x10_1201);
        assert_eq!((t[1][0], t[4][0]), (0x1802, 0x1801));
        mark_3d(&mut t, 0x1000, 0x800, 0xA00);
        assert_eq!(t[1], [0x30_1802, 0xA00]);
        assert_eq!(t[4], [0x30_1801, 0xA00]);
    }

    #[test]
    fn lookups() {
        let t: [Entry; 2] = [[0x1201, 0], [0x1202, 0]];
        assert_eq!(look_up(t, 0x0F_0000), Lookup::NoBits);
        assert_eq!(look_up(t, 0x3F_1202), Lookup::Missing);
        assert_eq!(look_up(t, 0x0F_1202), Lookup::Found(1));
        assert_eq!(menu_mode_for(0x1234_5678), 0x32_5602);
    }

    #[test]
    fn surface() {
        let sizes: [Entry; 2] = [[1, 480 << 16 | 640], [2, 600 << 16 | 800]];
        let kinds: [Entry; 2] = [[0x200, 16], [0x800, 32]];
        assert_eq!(
            picture_surface(0x802, sizes, kinds, 0x4000, 0x9000),
            Ok([800, 600, 32, 4, 3200, 800, 0x4000, 3200, 0x9000, 0])
        );
        assert_eq!(
            picture_surface(0x803, sizes, kinds, 0, 0),
            Err(SurfaceError::Size)
        );
        assert_eq!(
            picture_surface(0x401, sizes, kinds, 0, 0),
            Err(SurfaceError::PixelKind)
        );
    }
}
