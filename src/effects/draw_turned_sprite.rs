//! `effects::draw_turned_sprite` (0x4CBD12): a square sprite at a place already in the
//! view's frame (st(0) x, st(1) y, st(2) z, rounded into `flt_8EB8D5` .. `flt_8EB8DD`),
//! turned by the angle ebx. Unless the depth is at or below `dword_8EB961` (a signed
//! compare of the bits): the four vertices' colour ecx | 0xFFFFFF, texture places from
//! `flt_93A21A` .. `flt_93A236`, depth times `flt_8EB96D`; half the side (edx times
//! `flt_624088` / depth) and its turned corners (`g_sines` / `g_cosines`[ebx]) in
//! `dword_8EB8BD` .. `dword_8EB8C9`; the corners around the projected place;
//! `effects::clip_quad` on `g_polygonVertices`, and unless all is outside `scene::draw_fan`
//! and `g_polygonVertexCount` 4.
//!
//! In:  st(0) .. st(2) the place (popped), ebx the angle, ecx the colour, edx the size
//! Out: nothing (no caller reads a register it leaves)
//!
//! Callers: `sub_4D2DED` and its two siblings (0x4D2ECF, 0x4D2F00). Refuses an angle of
//! 0x1000 or more (the tables' size) and null marks settings (`clip_quad`'s refusal),
//! before anything.

use crate::effects::clip_quad::{clip_quad, draw_fan};
use crate::effects::frame_state::DRAW_WORK;
use crate::errors::{BadInput, Refused};
use crate::math::x87;
use crate::scene::draw_store::DRAW;
use crate::scene::matrix3::rounded;
use crate::session::Float;

use crate::d3d::vertices_store::VERTICES;
const ANGLES: u32 = 0x1000;

/// The Rust version (see the header): `angle` (the original's ebx), `colour` (ecx, its
/// low three bytes made 0xFF), `size` (edx), `place` (st(0) .. st(2)).
pub fn draw_turned_sprite(
    angle: u32,
    colour: u32,
    size: u32,
    place: [[u8; 10]; 3],
) -> Result<(), Refused> {
    if angle >= ANGLES {
        return Err(BadInput("an angle past the sine tables").into());
    }
    {
        let [a, b, c] = place.map(rounded);
        // clip_quad's refusal (no settings) before anything is written.
        if (c as i32) > DRAW_WORK.near.bits() as i32
            && crate::effects::setup::SETTINGS.pointer.get() == 0
        {
            return Err(BadInput("no marks settings").into());
        }
        for (n, v) in [a, b, c].into_iter().enumerate() {
            DRAW_WORK.turned[n].set_bits(v);
        }
        if (c as i32) <= DRAW_WORK.near.bits() as i32 {
            return Ok(());
        }
        let colour = colour | 0xFF_FFFF;
        let k = [
            DRAW.view_scale.bits(),
            DRAW.view_centre[0].bits(),
            DRAW.view_centre[1].bits(),
            DRAW_WORK.z_k.bits(),
        ];
        let sin = crate::scene::engine_tables_store::sine(angle as usize).bits();
        let cos = crate::scene::engine_tables_store::cosine(angle as usize).bits();
        let sq = x87::turned_sprite(a, b, c, k, size as i32, sin, cos);
        for n in 0..4 {
            let v = &VERTICES.vertices[n];
            v.colour.set(colour);
            v.u.set_bits(x87::loaded(SPRITE_UV.uv[2 * n].bits()));
            v.v.set_bits(x87::loaded(SPRITE_UV.uv[2 * n + 1].bits()));
            v.z.set_bits(sq[1]);
            v.x.set_bits(sq[10 + n]);
            v.y.set_bits(sq[6 + n]);
        }
        for (n, v) in sq[2..6].iter().enumerate() {
            DRAW_WORK.sprite_half[n].set_bits(*v);
        }
        if clip_quad(crate::d3d::vertices_store::piece_addr())? {
            return Ok(());
        }
        draw_fan();
        DRAW.polygon_vertex_count.set_bits(4u32);
    }
    Ok(())
}

/// The turned sprite's texture places (IDA 0x93A21A .. 0x93A23A, the image's layout; phase 7, typed state).
#[repr(C)]
pub struct SpriteUv {
    /// 0x93A21A: `flt_93A21A` .. `flt_93A236`: four (u, v) pairs, one per corner
    pub uv: [Float; 8],
}

/// SpriteUv's first values.
pub static SPRITE_UV: SpriteUv = SpriteUv {
    uv: [
        Float::from_bits(0x0),
        Float::from_bits(0x0),
        Float::from_bits(0x3F7FF972),
        Float::from_bits(0x0),
        Float::from_bits(0x3F7FF972),
        Float::from_bits(0x3F7FF972),
        Float::from_bits(0x0),
        Float::from_bits(0x3F7FF972),
    ],
};

impl SpriteUv {
    pub const START: usize = 0x93A21A;
    pub const LEN: usize = 0x20;
    /// Where the byte IDA calls `START` is now (`moved::MOVED`'s home).
    pub fn home() -> usize {
        crate::pointers::game_address(&raw const SPRITE_UV) as usize
    }
}

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
        assert_eq!(size_of::<SpriteUv>(), SpriteUv::LEN);
        assert!(
            bytes(&SPRITE_UV) == crate::moved::image_bytes(SpriteUv::START, SpriteUv::LEN),
            "SPRITE_UV starts differently from the image"
        );
        assert_eq!(offset_of!(SpriteUv, uv), 0x93A21A - SpriteUv::START);
    }
}
