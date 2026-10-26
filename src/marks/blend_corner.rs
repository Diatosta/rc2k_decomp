//! `marks::blend_corner` (0x4D0B4E): one point blended: its factor the record's float at
//! +0x90 + 4 x (the source point's three sign bits, x the highest; kept in
//! `flt_93C6C8`), each of x, y, z base + (source - base) by it, rounded (`fistp`).
//!
//! In:  the record (a car's +0x29C), where the result goes, the source point, the base
//!      point (the original's ebp, esi, edi, edx)
//! Out: nothing (no caller reads the registers it leaves)
//!
//! Caller: `marks::morph_object_points`.

use crate::errors::{BadInput, Refused};
use crate::math::x87;
use crate::pointers::GamePtr;
use crate::session::Float;

fn get(a: u32) -> Result<u32, BadInput> {
    GamePtr::new(a).read("a game field")
}

/// The Rust version (see the header): `block` the block (the original's ebp), `to` where
/// the point goes (esi), `from` the point blended (edi), `base` the base point (edx).
pub fn blend_corner(block: u32, to: u32, from: u32, base: u32) -> Result<(), Refused> {
    {
        let src = [get(from)?, get(from + 4)?, get(from + 8)?];
        let signs = src.iter().fold(0, |b, &v| (b << 1) | (v >> 31));
        let f = x87::loaded(get(block + 0x90 + 4 * signs)?);
        BLEND_WORK.factor.set_bits(f);
        for (c, &s) in src.iter().enumerate() {
            let c = 4 * c as u32;
            let v = x87::int_lerp_to_int(s as i32, get(base + c)? as i32, f);
            GamePtr::new(to + c).write::<i32>(v, "a mark's field")?;
        }
    }
    Ok(())
}

/// The corner blend's factor (IDA 0x93C6C8 .. 0x93C6CC, the image's layout; phase 7, typed state).
#[repr(C)]
pub struct BlendWork {
    /// 0x93C6C8: `flt_93C6C8`: the blend factor (`blend_corner`'s)
    pub factor: Float,
}

/// BlendWork's first values.
pub static BLEND_WORK: BlendWork = BlendWork {
    factor: Float::from_bits(0x0),
};

impl BlendWork {
    pub const START: usize = 0x93C6C8;
    pub const LEN: usize = 0x4;
    /// Where the byte IDA calls `START` is now (`moved::MOVED`'s home).
    pub fn home() -> usize {
        &BLEND_WORK as *const BlendWork as usize
    }
}

#[cfg(test)]
mod blend_work_tests {
    use super::*;
    use core::mem::offset_of;

    fn bytes<T>(v: &T) -> Vec<u8> {
        let p = v as *const T as *const u8;
        (0..size_of::<T>())
            .map(|i| unsafe { p.add(i).read() })
            .collect()
    }

    #[test]
    fn blend_work_starts_as_the_games() {
        assert_eq!(size_of::<BlendWork>(), BlendWork::LEN);
        assert!(
            bytes(&BLEND_WORK) == crate::moved::image_bytes(BlendWork::START, BlendWork::LEN),
            "BLEND_WORK starts differently from the image"
        );
        assert_eq!(offset_of!(BlendWork, factor), 0x93C6C8 - BlendWork::START);
    }
}
