//! `effects::blend_corner`: one point blended: its factor the record's float at +0x90 + 4 x
//! (the source point's three sign bits, x the highest; kept in `BlendWork.factor`), each of
//! x, y, z base + (source - base) by it, rounded.

use crate::errors::Refused;
use crate::math::x87;
use crate::memory::session::PlainFloat;
use crate::race::effects_settings_store::CarEffects;

/// `block` the car record whose block holds the factors, `to` where the point goes, `from`
/// the point blended, `base` the base point.
pub(super) fn blend_corner(
    block: &CarEffects,
    to: u32,
    from: u32,
    base: [u32; 3],
    effects: &crate::effects::owner::Effects,
) -> Result<(), Refused> {
    {
        // `from` and `to` are places of `scene::object_points`.
        let src = effects.stage()?.object_points.point(from)?;
        let signs = src.iter().fold(0, |b, &v| (b << 1) | (v >> 31));
        let f = x87::loaded(&block.factors[signs as usize]);
        effects.clipping.blend_work.factor.set(f);
        for (c, &s) in src.iter().enumerate() {
            let v = x87::int_lerp_to_int(s as i32, base[c] as i32, f);
            let c = 4 * c as u32;
            effects
                .stage()?
                .object_points
                .write(to + c, &v.to_le_bytes())?;
        }
    }
    Ok(())
}

/// The corner blend's factor.
#[derive(rc2k_derive::Cells)]
pub struct BlendWork {
    /// The blend factor (`blend_corner`'s)
    pub factor: PlainFloat,
}

impl BlendWork {
    /// BlendWork's first values.
    pub const fn new() -> Self {
        Self {
            factor: PlainFloat::new(0.0),
        }
    }
}
