//! `marks::draw_turned_sprite` (0x4CBD12): a square sprite at a place already in the
//! view's frame (st(0) x, st(1) y, st(2) z, rounded into `flt_8EB8D5` .. `flt_8EB8DD`),
//! turned by the angle ebx. Unless the depth is at or below `dword_8EB961` (a signed
//! compare of the bits): the four vertices' colour ecx | 0xFFFFFF, texture places from
//! `flt_93A21A` .. `flt_93A236`, depth times `flt_8EB96D`; half the side (edx times
//! `flt_624088` / depth) and its turned corners (`g_sines` / `g_cosines`[ebx]) in
//! `dword_8EB8BD` .. `dword_8EB8C9`; the corners around the projected place;
//! `marks::clip_quad` on `g_polygonVertices`, and unless all is outside `scene::draw_fan`
//! and `g_polygonVertexCount` 4.
//!
//! In:  st(0) .. st(2) the place (popped), ebx the angle, ecx the colour, edx the size
//! Out: nothing (no caller reads a register it leaves)
//!
//! Callers: `sub_4D2DED` and its two siblings (0x4D2ECF, 0x4D2F00). Refuses an angle of
//! 0x1000 or more (the tables' size) and null marks settings (`clip_quad`'s refusal),
//! before anything.

use crate::errors::{BadInput, Refused};
use crate::framework::rva;
use crate::marks::clip_quad::{clip_quad, draw_fan};
use crate::marks::store::{Slot, game_addr, game_read, game_write};
use crate::math::x87;
use crate::scene::matrix3::rounded;

const PLACE_RVA: Slot = Slot::at(0x8EB8D5); // flt_8EB8D5 .. flt_8EB8DD
const NEAR_RVA: Slot = Slot::at(0x8EB961); // dword_8EB961
const UV_RVA: Slot = Slot::at(0x93A21A); // flt_93A21A .. flt_93A236
const HALF_RVA: Slot = Slot::at(0x8EB8BD); // dword_8EB8BD .. dword_8EB8C9
const FOCAL_RVA: usize = rva(0x624088); // flt_624088
const CENTRE_RVA: [usize; 2] = [rva(0x62409C), rva(0x6240A0)];
const Z_K_RVA: Slot = Slot::at(0x8EB96D); // flt_8EB96D
const VERTICES_RVA: usize = rva(0x9605A0); // g_polygonVertices: TLVertex[4]
const COUNT_RVA: usize = rva(0x6240F8); // g_polygonVertexCount
const SINES_RVA: usize = rva(0xA23AE4); // g_sines
const COSINES_RVA: usize = rva(0xA24AE4); // g_cosines: 0x1000 entries
const ANGLES: u32 = 0x1000;

/// The Rust version (see the header): `angle` (the original's ebx), `colour` (ecx, its
/// low three bytes made 0xFF), `size` (edx), `place` (st(0) .. st(2)).
pub unsafe fn draw_turned_sprite(
    angle: u32,
    colour: u32,
    size: u32,
    place: [[u8; 10]; 3],
) -> Result<(), Refused> {
    if angle >= ANGLES {
        return Err(BadInput("an angle past the sine tables").into());
    }
    unsafe {
        let [a, b, c] = place.map(rounded);
        // clip_quad's refusal (no settings) before anything is written.
        if (c as i32) > game_read::<i32>(NEAR_RVA)
            && crate::marks::setup::SETTINGS.pointer.get() == 0
        {
            return Err(BadInput("no marks settings").into());
        }
        for (n, v) in [a, b, c].into_iter().enumerate() {
            game_write(PLACE_RVA + 4 * n, v);
        }
        if (c as i32) <= game_read::<i32>(NEAR_RVA) {
            return Ok(());
        }
        let colour = colour | 0xFF_FFFF;
        let k = [
            game_read(FOCAL_RVA),
            game_read(CENTRE_RVA[0]),
            game_read(CENTRE_RVA[1]),
            game_read(Z_K_RVA),
        ];
        let sin = game_read(SINES_RVA + 4 * angle as usize);
        let cos = game_read(COSINES_RVA + 4 * angle as usize);
        let sq = x87::turned_sprite(a, b, c, k, size as i32, sin, cos);
        for n in 0..4 {
            let v = VERTICES_RVA + 0x20 * n;
            game_write(v + 0x10, colour);
            game_write(v + 0x18, x87::loaded(game_read(UV_RVA + 8 * n)));
            game_write(v + 0x1C, x87::loaded(game_read(UV_RVA + 8 * n + 4)));
            game_write(v + 8, sq[1]);
            game_write(v, sq[10 + n]);
            game_write(v + 4, sq[6 + n]);
        }
        for (n, v) in sq[2..6].iter().enumerate() {
            game_write(HALF_RVA + 4 * n, *v);
        }
        if clip_quad(game_addr(VERTICES_RVA) as u32)? {
            return Ok(());
        }
        draw_fan();
        game_write(COUNT_RVA, 4u32);
    }
    Ok(())
}
