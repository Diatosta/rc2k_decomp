//! `effects::blend_corner` (0x4D0B4E): one point blended: its factor the record's float at
//! +0x90 + 4 x (the source point's three sign bits, x the highest; kept in
//! `flt_93C6C8`), each of x, y, z base + (source - base) by it, rounded (`fistp`).
//!
//! Caller: `effects::morph_object_points`.

#[cfg(test)]
use crate::effects::owner::effects;
use crate::errors::Refused;
use crate::math::x87;
use crate::race::effects_settings_store::CarEffects;
use crate::session::Float;

/// `block` the car record whose block holds the factors (the original's ebp), `to` where
/// the point goes (esi), `from` the point blended (edi), `base` the base point (edx's).
pub fn blend_corner(
    block: &CarEffects,
    to: u32,
    from: u32,
    base: [u32; 3],
    effects: &crate::effects::owner::Effects,
) -> Result<(), Refused> {
    {
        // `from` and `to` are places of `scene::object_points` (phase 13).
        let src = crate::scene::object_points::point(from)?;
        let signs = src.iter().fold(0, |b, &v| (b << 1) | (v >> 31));
        let f = x87::loaded(block.factors[signs as usize].bits());
        effects.clipping.blend_work.factor.set_bits(f);
        for (c, &s) in src.iter().enumerate() {
            let v = x87::int_lerp_to_int(s as i32, base[c] as i32, f);
            let c = 4 * c as u32;
            crate::scene::object_points::write(to + c, &v.to_le_bytes())?;
        }
    }
    Ok(())
}

/// The corner blend's factor (IDA 0x93C6C8 .. 0x93C6CC, the image's layout; phase 7, typed state).
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct BlendWork {
    /// 0x93C6C8: `flt_93C6C8`: the blend factor (`blend_corner`'s)
    pub factor: Float,
}

impl BlendWork {
    /// BlendWork's first values.
    pub const fn new() -> Self {
        Self {
            factor: Float::from_bits(0x0),
        }
    }
}

impl BlendWork {
    pub const START: usize = 0x93C6C8;
    pub const LEN: usize = 0x4;
}

#[cfg(test)]
mod blend_work_tests {
    use super::*;
    use core::mem::offset_of;

    fn bytes<T>(v: &T) -> Vec<u8> {
        let p = std::ptr::from_ref::<T>(v).cast::<u8>();
        (0..size_of::<T>())
            .map(|i| unsafe { p.add(i).read() })
            .collect()
    }

    #[test]
    fn blend_work_starts_as_the_games() {
        assert_eq!(size_of::<BlendWork>(), BlendWork::LEN);
        assert!(
            bytes(&effects().clipping.blend_work)
                == crate::moved::image_bytes(BlendWork::START, BlendWork::LEN),
            "BLEND_WORK starts differently from the image"
        );
        assert_eq!(offset_of!(BlendWork, factor), 0x93C6C8 - BlendWork::START);
    }
}
