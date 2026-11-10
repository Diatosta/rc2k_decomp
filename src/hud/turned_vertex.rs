//! `hud::turned_vertex` (0x4B7058): one vertex of a turned HUD shape at edi: the point (st(1) x,
//! st(0) y, rounded into `dword_75BFF0` / `dword_75BFF4`); its texture place (x plus `flt_75C298`,
//! y plus `flt_75C29C`, over `flt_75C00C`); the point turned by edx, times `dword_75C2B8` plus
//! `dword_75C2A0` / `dword_75C2A4`; colour white under the quad record's alpha, specular 0.
//!
//! Callers: `hud::draw_needle`'s four (code at 0x4B6FFE ..). The turn is its Rust version.

use crate::d3d::vertices_store::Vertex;
use crate::hud::owner::Hud;
use crate::math::turn_float_point_xz::turn_float_point_xz_plain;
use rc2k_core::hud::disc;

/// The point as `f64`s (what the x87 holds at the game's double precision): the vertex at `vertex`,
/// turned by `angle`. that goes to `g_scratchY` alone, which is written before it is read
/// everywhere: 0 here.
// Converted: `rc2k_core::hud::disc::{turned_texture, turned_place, white_under}`
pub fn turned_vertex(hud: &Hud, vertex: &Vertex, angle: u32, point: [f64; 2]) {
    let [y, x] = point.map(|v| v as f32);
    hud.block.point[1].set(y.to_bits());
    hud.block.point[0].set(x.to_bits());
    let scale = hud.block.uv_scale_x.get();
    vertex
        .u
        .set_bits(disc::turned_texture(x, hud.block.uv_shift[0].get(), scale).to_bits());
    vertex
        .v
        .set_bits(disc::turned_texture(y, hud.block.uv_shift[1].get(), scale).to_bits());
    let [tx, _, tz, _] = turn_float_point_xz_plain(x.to_bits(), 0, y.to_bits(), angle);
    hud.block.point[1].set(tz);
    hud.block.point[0].set(tx);
    let size = hud.block.needle_size.get();
    let [tx, tz] = [tx, tz].map(f32::from_bits);
    vertex
        .x
        .set_bits(disc::turned_place(tx, size, hud.block.needle_centre[0].get()).to_bits());
    vertex
        .y
        .set_bits(disc::turned_place(tz, size, hud.block.needle_centre[1].get()).to_bits());
    let colour = disc::white_under(hud.block.quad[0].get());
    vertex.colour.set(colour);
    vertex.specular.set(0);
}
