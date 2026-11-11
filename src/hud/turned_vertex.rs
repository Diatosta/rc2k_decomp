//! `hud::turned_vertex` (0x4B7058): one vertex of a turned HUD shape: the point (x and y from the x87
//! stack, rounded into the vertex scratch pair); its texture place (x and y plus the shape's
//! texture offsets, over the texture size); the point turned by the given turn, times the
//! shape's scale plus its centre; colour white under the quad record's alpha, specular 0.
//!
//! Callers: `hud::draw_needle`'s four (four calls). The turn is `math::rotate_float_point_about_y`.

use crate::d3d::vertices_store::Vertex;
use crate::hud::owner::Hud;
use crate::math::rotate_float_point_about_y::rotate_float_point_about_y_plain;
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
        .set(disc::turned_texture(x, hud.block.uv_shift[0].get(), scale));
    vertex
        .v
        .set(disc::turned_texture(y, hud.block.uv_shift[1].get(), scale));
    let [tx, _, tz, _] = rotate_float_point_about_y_plain(x.to_bits(), 0, y.to_bits(), angle);
    hud.block.point[1].set(tz);
    hud.block.point[0].set(tx);
    let size = hud.block.needle_size.get();
    let [tx, tz] = [tx, tz].map(f32::from_bits);
    vertex.x.set(disc::turned_place(
        tx,
        size,
        hud.block.needle_centre[0].get(),
    ));
    vertex.y.set(disc::turned_place(
        tz,
        size,
        hud.block.needle_centre[1].get(),
    ));
    let colour = disc::white_under(hud.block.quad.alpha.get());
    vertex.colour.set(colour);
    vertex.specular.set(0);
}
