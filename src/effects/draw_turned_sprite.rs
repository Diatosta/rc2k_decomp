//! `effects::draw_turned_sprite` (0x4CBD12): a square sprite at a place already in the
//! view's frame (st(0) x, st(1) y, st(2) z, rounded into the draw work's place), turned by
//! the angle. Unless the depth is at or below the nearest depth (a signed compare of the
//! bits): the four vertices' colour | 0xFFFFFF, texture places from the sprite's four (u, v)
//! pairs, depth times the depth scale; half the side (the size times the view scale / depth)
//! and its turned corners (the sine and cosine tables at the angle); the corners around the
//! projected place; `effects::clip_quad` on the polygon vertices, and unless all is outside
//! `scene::draw_fan` and the polygon's vertex count 4.
//!
//! Callers: `effects::draw_glow` and its two siblings. Refuses an angle of
//! 0x1000 or more (the tables' size) and null marks settings (`clip_quad`'s refusal),
//! before anything.

use crate::effects::clip_quad::{clip_quad, draw_fan};
use crate::errors::{BadInput, Refused};
use crate::math::x87;
use crate::scene::matrix3::rounded_value;

const ANGLES: u32 = 0x1000;

/// `angle` turns the sprite, `colour` has its low three bytes made 0xFF, `size` is the
/// side, `place` the place (st(0) .. st(2)), `engine_tables` the sine tables it turns by.
#[allow(
    clippy::too_many_arguments,
    reason = "the original's registers, the sine tables and the owners"
)]
pub fn draw_turned_sprite(
    engine_tables: &crate::scene::engine_tables_store::EngineTables,
    buffer: &crate::d3d::vertices_store::PolygonVertices,
    angle: u32,
    colour: u32,
    size: u32,
    place: [[u8; 10]; 3],
    scene: &crate::scene::owner::Scene,
    effects: &crate::effects::owner::Effects,
) -> Result<(), Refused> {
    if angle >= ANGLES {
        return Err(BadInput("an angle past the sine tables").into());
    }
    {
        let place = place.map(rounded_value);
        let near = effects.frame.draw_work.near.get();
        // clip_quad's refusal (no settings) before anything is written.
        if !x87::bits_at_or_below(place[2], near) && effects.setup.settings.is_set.get() == 0 {
            return Err(BadInput("no marks settings").into());
        }
        for (n, v) in place.into_iter().enumerate() {
            effects.frame.draw_work.turned[n].set(v);
        }
        if x87::bits_at_or_below(place[2], near) {
            return Ok(());
        }
        let colour = colour | 0xFF_FFFF;
        let k = effects.frame.draw_work.view_k();
        let sin = crate::scene::engine_tables_store::sine(engine_tables, angle as usize).get();
        let cos = crate::scene::engine_tables_store::cosine(engine_tables, angle as usize).get();
        let sq = x87::turned_sprite_floats(place, k, size as i32, sin, cos);
        for n in 0..4 {
            let v = &buffer.vertices[n];
            v.colour.set(colour);
            v.u.set(x87::loaded(SPRITE_UV.uv[2 * n]));
            v.v.set(x87::loaded(SPRITE_UV.uv[2 * n + 1]));
            v.z.set(sq[1]);
            v.x.set(sq[10 + n]);
            v.y.set(sq[6 + n]);
        }
        // The half sides as floats in the cells that hold whole numbers for the other sprites.
        for (n, v) in sq[2..6].iter().enumerate() {
            effects.frame.draw_work.sprite_half[n].set(v.to_bits());
        }
        if clip_quad(buffer, effects)? {
            return Ok(());
        }
        draw_fan(buffer, scene);
        crate::scene::polygon_work::front_end()
            .polygon_vertex_count
            .set(4);
    }
    Ok(())
}

/// The turned sprite's texture places (IDA 0x93A21A .. 0x93A23A).
#[repr(C)]
pub struct SpriteUv {
    /// 0x93A21A: `flt_93A21A` .. `flt_93A236`: four (u, v) pairs, one per corner
    pub uv: [f32; 8],
}

/// SpriteUv's first values.
pub static SPRITE_UV: SpriteUv = SpriteUv {
    uv: [0.0, 0.0, 0.9999, 0.0, 0.9999, 0.9999, 0.0, 0.9999],
};

#[cfg(test)]
mod sprite_uv_tests {
    use super::*;
    use core::mem::offset_of;

    fn bytes<T>(v: &T) -> Vec<u8> {
        let p = std::ptr::from_ref::<T>(v).cast::<u8>();
        (0..size_of::<T>())
            .map(|i| unsafe { p.add(i).read() })
            .collect()
    }

    #[test]
    fn sprite_uv_starts_as_the_games() {
        assert!(
            bytes(&SPRITE_UV)
                == crate::test_support::first_values::bytes("effects.draw_turned_sprite.SPRITE_UV"),
            "SPRITE_UV starts differently from the image"
        );
        assert_eq!(offset_of!(SpriteUv, uv), 0x0);
    }
}
