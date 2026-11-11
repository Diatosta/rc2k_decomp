//! The race frame's per-view steps: the textures loaded again each frame, the mirror's
//! picture copied, the screen's tint, the marks' views and layout, the own car seen from
//! inside (converted from `race::forget_view_textures` 0x45B200,
//! `copy_mirror_picture` 0x455DAD, `draw_screen_tint` 0x455A11, `run_view_marks`
//! 0x456336 and `draw_own_car` 0x455AD3).

use crate::Invalid;

// ---- the textures loaded again ----

/// Where a texture to forget is asked for, by view (0, 1) or pair member.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Forget {
    /// The view's car's changing texture (its block's +0xADC).
    Changing(usize),
    /// The view's car's counted texture (+0x720, counted down once forgotten).
    Counted(usize),
    /// One of the pair `forget_flag_0` / `forget_flag_1`.
    Extra(usize),
}

/// The textures in the original's order, each with its bit in `view_textures_forgotten`.
pub const FORGET_ORDER: [(Forget, u32); 6] = [
    (Forget::Changing(0), 1),
    (Forget::Changing(1), 2),
    (Forget::Counted(0), 4),
    (Forget::Counted(1), 8),
    (Forget::Extra(0), 0x10),
    (Forget::Extra(1), 0x1000),
];

/// The mirror's bit, and what each forgetting adds to `texture_changes` (the mirror's after
/// its picture is made).
pub const MIRROR_BIT: u32 = 0x20;
pub const FORGET_COST: u32 = 0x4000;
pub const MIRROR_COST: u32 = 0x400;

/// A texture is forgotten when its switch (the field or variable) is not 0.
pub fn forgets(switch: u32) -> bool {
    switch != 0
}

/// The mirror is on (`mirror_wanted`) and not turned off (`mirror_switch`).
pub fn mirror_on(on: u32, off: u32) -> bool {
    on != 0 && off == 0
}

/// The texture slots there are.
pub const TEXTURE_SLOTS: u32 = 0x400;

/// The slots to forget are all inside the table.
pub fn check_slots(slots: &[u32]) -> Result<(), Invalid> {
    if slots.iter().any(|&s| s >= TEXTURE_SLOTS) {
        Err(Invalid("a texture slot past 0x3FF"))
    } else {
        Ok(())
    }
}

// ---- the mirror's picture ----

/// The two surfaces' sizes and steps for `copy_picture`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PictureCopy {
    pub width: u32,
    pub height: u32,
    /// Bytes from a row's start to the next's.
    pub from_pitch: u32,
    pub to_pitch: u32,
    /// Two bytes a pixel (else four).
    pub from_16_bit: bool,
    pub to_16_bit: bool,
}

/// What the copy reads and writes, the pixels by place, and the formats' conversions.
pub trait Surfaces {
    fn read16(&mut self, at: u32) -> u16;
    fn read32(&mut self, at: u32) -> u32;
    fn write16(&mut self, at: u32, v: u16);
    fn write32(&mut self, at: u32, v: u32);
    /// The screen's pixel taken apart (`gfx::pixel_to_components`).
    fn components(&mut self, pixel: u32) -> u32;
    /// The components put together in the destination's format
    /// (`gfx::components_to_pixel`).
    fn pixel(&mut self, components: u32) -> u32;
}

/// `copy_mirror_picture`'s copy: each pixel taken apart, made opaque and put together in
/// the destination's format. The slip kept: the last pixel's bits go on from pixel to pixel
/// (`carried` first), and a 16-bit source pixel replaces only their low half, so the
/// components are taken of the pixel under the last conversion's high half.
pub fn copy_picture(c: &PictureCopy, from: u32, to: u32, carried: u32, s: &mut impl Surfaces) {
    let (mut read_at, mut write_at) = (from, to);
    let mut pixel = carried;
    for _ in 0..c.height {
        let (row_from, row_to) = (read_at, write_at);
        for _ in 0..c.width {
            if c.from_16_bit {
                pixel = (pixel & 0xFFFF_0000) | s.read16(read_at) as u32;
                read_at = read_at.wrapping_add(2);
            } else {
                pixel = s.read32(read_at);
                read_at = read_at.wrapping_add(4);
            }
            let components = s.components(pixel);
            pixel = s.pixel(components | 0xFF00_0000);
            if c.to_16_bit {
                s.write16(write_at, pixel as u16);
                write_at = write_at.wrapping_add(2);
            } else {
                s.write32(write_at, pixel);
                write_at = write_at.wrapping_add(4);
            }
        }
        read_at = row_from.wrapping_add(c.from_pitch);
        write_at = row_to.wrapping_add(c.to_pitch);
    }
}

// ---- the screen's tint ----

/// At night (`in_credits`): blue-grey half through.
pub const NIGHT_ALPHA: u32 = 0x80;
pub const NIGHT_COLOUR: u32 = 0x28_3080;

/// The sun's glare (`fog_far`) as the tint's alpha, held to 0 ..= 0xFF (by the
/// signs).
pub fn glare_alpha(glare: u32) -> u32 {
    if (glare as i32) < 0 {
        0
    } else if (glare.wrapping_sub(0xFF) as i32) >= 0 {
        0xFF
    } else {
        glare
    }
}

/// The glare's colour: red (`lamp_table_c`, a whole dword) << 16, green and blue bytes.
pub fn glare_colour(red: u32, green: u8, blue: u8) -> u32 {
    (red << 16) | ((green as u32) << 8) | blue as u32
}

// ---- the marks' views ----

/// Which views' marks show: (the other view's `other_marks_shown`, this one's `own_marks_shown`):
/// none in a replay (`scene_switch`), not the other's with `still`.
pub fn marks_shown(view: u32, replay: u32, no_other: u32) -> (u32, u32) {
    let (mut other, mut own) = (1u32.wrapping_sub(view), 1);
    if replay != 0 {
        other = 0;
        own = 0;
    }
    if no_other != 0 {
        other = 0;
    }
    (other, own)
}

/// The marks' layout's index: the view plus 2 x the layout set (`split`).
pub fn layout_index(view: u32, set: u32) -> u32 {
    view.wrapping_add(set.wrapping_mul(2))
}

/// Every car record's first word: 1 in modes 0 and 2, else 0 (the view's then 1).
pub fn first_word(mode: u32) -> u32 {
    u32::from(matches!(mode, 0 | 2))
}

/// The car records there are (`car_records` ..).
pub const CAR_RECORDS: u32 = 8;

/// `run_view_marks`' refusals: a null view block or car record, no marks settings.
pub fn check_view_marks(
    block: u32,
    record: impl Fn(u32) -> u32,
    settings: impl FnOnce() -> u32,
) -> Result<(), Invalid> {
    if block == 0 || (0..CAR_RECORDS).any(|b| record(b) == 0) {
        return Err(Invalid("a null view block or car record"));
    }
    if settings() == 0 {
        return Err(Invalid("no marks settings"));
    }
    Ok(())
}

// ---- the own car ----

/// The drawn view's car's number in `view_cars`[]` (its top two bits are flags).
pub fn car_number(entry: u32) -> u32 {
    entry & 0x3FFF_FFFF
}

/// The car seen from inside (+0x6FC bit 0x10).
pub fn seen_inside(flags: u32) -> bool {
    flags & 0x10 != 0
}

/// The depth cleared first when the car's +0x900 or +0x904, or `view_cleared`, is set
/// (each asked only when the ones before are 0).
pub fn clears_depth(a: u32, b: impl FnOnce() -> u32, c: impl FnOnce() -> u32) -> bool {
    a != 0 || b() != 0 || c() != 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn textures_and_tint() {
        assert!(forgets(1) && !forgets(0));
        assert!(mirror_on(1, 0) && !mirror_on(1, 1) && !mirror_on(0, 0));
        assert!(check_slots(&[0, 0x3FF]).is_ok() && check_slots(&[0x400]).is_err());
        assert_eq!(FORGET_ORDER.iter().fold(0, |b, f| b | f.1), 0x101F);
        assert_eq!(glare_alpha(u32::MAX), 0);
        assert_eq!(glare_alpha(0x100), 0xFF);
        assert_eq!(glare_alpha(0xFE), 0xFE);
        assert_eq!(glare_colour(0x12, 0x34, 0x56), 0x12_3456);
        assert_eq!(glare_colour(0x1FF, 0, 0), 0x1FF_0000);
    }

    #[test]
    fn views_and_car() {
        assert_eq!(marks_shown(0, 0, 0), (1, 1));
        assert_eq!(marks_shown(1, 0, 0), (0, 1));
        assert_eq!(marks_shown(0, 1, 0), (0, 0));
        assert_eq!(marks_shown(0, 0, 1), (0, 1));
        assert_eq!(layout_index(1, 2), 5);
        assert_eq!((first_word(0), first_word(2), first_word(1)), (1, 1, 0));
        assert!(check_view_marks(0, |_| 1, || 1).is_err());
        assert!(check_view_marks(1, |b| u32::from(b != 7), || 1).is_err());
        assert!(check_view_marks(1, |_| 1, || 0).is_err());
        assert!(check_view_marks(1, |_| 1, || 1).is_ok());
        assert_eq!(car_number(0xC000_0003), 3);
        assert!(seen_inside(0x30) && !seen_inside(0x20));
        assert!(clears_depth(1, || unreachable!(), || unreachable!()));
        assert!(clears_depth(0, || 0, || 2) && !clears_depth(0, || 0, || 0));
    }

    struct Test {
        mem: Vec<u8>,
    }

    impl Surfaces for Test {
        fn read16(&mut self, at: u32) -> u16 {
            u16::from_le_bytes([self.mem[at as usize], self.mem[at as usize + 1]])
        }
        fn read32(&mut self, at: u32) -> u32 {
            u32::from_le_bytes(self.mem[at as usize..at as usize + 4].try_into().unwrap())
        }
        fn write16(&mut self, at: u32, v: u16) {
            self.mem[at as usize..at as usize + 2].copy_from_slice(&v.to_le_bytes());
        }
        fn write32(&mut self, at: u32, v: u32) {
            self.mem[at as usize..at as usize + 4].copy_from_slice(&v.to_le_bytes());
        }
        fn components(&mut self, pixel: u32) -> u32 {
            pixel.rotate_left(8)
        }
        fn pixel(&mut self, components: u32) -> u32 {
            components ^ 0x0F0F
        }
    }

    #[test]
    fn picture() {
        let mut t = Test {
            mem: (0..64u32).map(|n| n as u8).collect(),
        };
        let c = PictureCopy {
            width: 2,
            height: 2,
            from_pitch: 8,
            to_pitch: 8,
            from_16_bit: true,
            to_16_bit: false,
        };
        copy_picture(&c, 0, 32, 0xABCD_0000, &mut t);
        // The first pixel: 0x0100 under 0xABCD's high half.
        let first = (0xABCD_0100u32.rotate_left(8) | 0xFF00_0000) ^ 0x0F0F;
        assert_eq!(t.read32(32), first);
        // The second keeps the first's high half.
        let second = (((first & 0xFFFF_0000) | 0x0302).rotate_left(8) | 0xFF00_0000) ^ 0x0F0F;
        assert_eq!(t.read32(36), second);
        // The next row from its pitch: the source's 8th byte on.
        let third = (((second & 0xFFFF_0000) | 0x0908).rotate_left(8) | 0xFF00_0000) ^ 0x0F0F;
        assert_eq!(t.read32(40), third);
        let untouched = t.mem.clone();
        copy_picture(&PictureCopy { height: 0, ..c }, 4, 8, 0, &mut t);
        assert_eq!(t.mem, untouched);
    }
}
