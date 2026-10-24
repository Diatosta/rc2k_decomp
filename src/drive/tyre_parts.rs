//! The tyre-change screen's drawing (stream B, 2026-10-18): `drive::tyre_change_screen`
//! (0x445047), `drive::tyre_screen` (0x41C5FA), `drive::tyre_screen_parts` (0x41C790),
//! `drive::tyre_turned_quad` (0x41C969), `drive::tyre_quad` (0x41DD3B). Their calls by
//! place: 0 `d3d::set_texture` (0x41ACF5), 1 `d3d::set_render_state` (0x41AB5B), 2
//! `tyre_quad`, 3 `tyre_turned_quad`, 4 `tyre_screen_parts`, 5 the device's
//! DrawPrimitive (`sub_41ADE1`: platform, a call into the game), 6 `tyre_screen`. In the
//! game the Rust versions and the game's three; in the self-tests copies whose calls go
//! to stand-ins noting them (with the vertices for 5), against the Rust versions with the
//! same.
//!
//! This file: the places, the quads' shared steps and the tests' frame.

use crate::errors::Refused;
use crate::physics::flow::{a, p, x87_run};

/// `g_polygonVertices`: the four vertices (0x20 bytes each) a quad draws.
pub const VERTS: usize = 0x9605A0;

/// `d3d::set_texture` (0x41ACF5): eax the texture.
pub fn set_texture(texture: u32) {
    crate::d3d::set_texture::set_texture(texture);
}

/// `d3d::set_render_state` (0x41AB5B): eax the value, ebx the state.
pub fn set_render_state(value: u32, state: u32) {
    crate::d3d::set_render_state::set_render_state(value, state);
}

/// The quads' fixed words (0x41DD68 .. / 0x41CA49 ..): z 0, rhw 1.0, the colour, the
/// specular 0.
pub fn quad_words(colour: u32) {
    for k in 0..4 {
        let v = VERTS + 0x20 * k;
        p(v + 8, 0);
        p(v + 0xC, 0x3F80_0000);
        p(v + 0x10, colour);
        p(v + 0x14, 0);
    }
}

/// The quads' shared tail (0x41DDA1 ..): the texture places from the rectangle at `uv`
/// (x, y, w, h, whole numbers) x `flt_5C2C64`, and the places scaled to the screen
/// (`g_screenWidth` x `flt_5C2C68`, `g_screenHeight` x `flt_5C2C6C`, through
/// `dword_5183CC`).
pub fn quad_tail(uv: u32) {
    x87_run!(
        "fild dword ptr [{s}]",
        "fmul dword ptr [{rc} + 0x1C2C64]",
        "fst dword ptr [{v} + 0x18]",
        "fstp dword ptr [{v} + 0x58]",
        "fild dword ptr [{s}]",
        "fiadd dword ptr [{s} + 0x8]",
        "fmul dword ptr [{rc} + 0x1C2C64]",
        "fst dword ptr [{v} + 0x38]",
        "fstp dword ptr [{v} + 0x78]",
        "fild dword ptr [{s} + 0x4]",
        "fmul dword ptr [{rc} + 0x1C2C64]",
        "fst dword ptr [{v} + 0x1C]",
        "fstp dword ptr [{v} + 0x3C]",
        "fild dword ptr [{s} + 0x4]",
        "fiadd dword ptr [{s} + 0xC]",
        "fmul dword ptr [{rc} + 0x1C2C64]",
        "fst dword ptr [{v} + 0x5C]",
        "fstp dword ptr [{v} + 0x7C]",
        "fild dword ptr [{sc}]",
        "fmul dword ptr [{rc} + 0x1C2C68]",
        "fstp dword ptr [{t}]",
        "fld dword ptr [{v}]",
        "fmul dword ptr [{t}]",
        "fstp dword ptr [{v}]",
        "fld dword ptr [{v} + 0x20]",
        "fmul dword ptr [{t}]",
        "fstp dword ptr [{v} + 0x20]",
        "fld dword ptr [{v} + 0x40]",
        "fmul dword ptr [{t}]",
        "fstp dword ptr [{v} + 0x40]",
        "fld dword ptr [{v} + 0x60]",
        "fmul dword ptr [{t}]",
        "fstp dword ptr [{v} + 0x60]",
        "fild dword ptr [{sc} + 0x4]",
        "fmul dword ptr [{rc} + 0x1C2C6C]",
        "fstp dword ptr [{t}]",
        "fld dword ptr [{v} + 0x4]",
        "fmul dword ptr [{t}]",
        "fstp dword ptr [{v} + 0x4]",
        "fld dword ptr [{v} + 0x24]",
        "fmul dword ptr [{t}]",
        "fstp dword ptr [{v} + 0x24]",
        "fld dword ptr [{v} + 0x44]",
        "fmul dword ptr [{t}]",
        "fstp dword ptr [{v} + 0x44]",
        "fld dword ptr [{v} + 0x64]",
        "fmul dword ptr [{t}]",
        "fstp dword ptr [{v} + 0x64]";
        s = uv, v = a(VERTS), sc = crate::session::SCREEN.width.addr(), t = crate::session::MACHINE.scratch.addr(), rc = crate::race::store::asm_base()
    );
}

/// The draw (`d3d::draw_vertex_strip`, 0x41ADE1) of 4 vertices at `g_polygonVertices`;
/// its result (Direct3D's, always 0) is read by no one: the quads keep every register
/// round it (`pusha` / `popa`).
pub fn draw() -> Result<(), Refused> {
    unsafe { crate::d3d::draw_vertex_strip::draw_vertex_strip(a(VERTS), 4) };
    Ok(())
}
