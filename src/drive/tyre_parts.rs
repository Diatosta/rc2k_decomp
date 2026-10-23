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
use crate::framework::{Regs, rva};
use crate::physics::flow::{a, base, p, x87_run};
use crate::race::car_starts::Part;

pub const TEXTURE: u32 = 0;
pub const STATE: u32 = 1;
pub const QUAD: u32 = 2;
pub const TURNED: u32 = 3;
pub const PARTS: u32 = 4;
pub const DRAW: u32 = 5;
pub const SCREEN: u32 = 6;

/// The places' functions (IDA addresses), by place.
pub const PLACES: [usize; 7] = [
    0x41ACF5, 0x41AB5B, 0x41DD3B, 0x41C969, 0x41C790, 0x41ADE1, 0x41C5FA,
];

/// `g_polygonVertices`: the four vertices (0x20 bytes each) a quad draws.
pub const VERTS: usize = 0x9605A0;

/// The calls in the game.
pub fn game_part(place: u32, r: &mut Regs) -> Result<(), Refused> {
    unsafe {
        match place {
            QUAD => *r = crate::drive::tyre_quad::tyre_quad(r, &mut game_part)?,
            TURNED => *r = crate::drive::tyre_turned_quad::tyre_turned_quad(r, &mut game_part)?,
            PARTS => *r = crate::drive::tyre_screen_parts::tyre_screen_parts(r)?,
            SCREEN => *r = crate::drive::tyre_screen::tyre_screen(r)?,
            _ => crate::calls::call(rva(PLACES[place as usize]), r),
        }
    }
    Ok(())
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
        "fmul dword ptr [{b} + 0x1C2C64]",
        "fst dword ptr [{v} + 0x18]",
        "fstp dword ptr [{v} + 0x58]",
        "fild dword ptr [{s}]",
        "fiadd dword ptr [{s} + 0x8]",
        "fmul dword ptr [{b} + 0x1C2C64]",
        "fst dword ptr [{v} + 0x38]",
        "fstp dword ptr [{v} + 0x78]",
        "fild dword ptr [{s} + 0x4]",
        "fmul dword ptr [{b} + 0x1C2C64]",
        "fst dword ptr [{v} + 0x1C]",
        "fstp dword ptr [{v} + 0x3C]",
        "fild dword ptr [{s} + 0x4]",
        "fiadd dword ptr [{s} + 0xC]",
        "fmul dword ptr [{b} + 0x1C2C64]",
        "fst dword ptr [{v} + 0x5C]",
        "fstp dword ptr [{v} + 0x7C]",
        "fild dword ptr [{b} + 0xF494C]",
        "fmul dword ptr [{b} + 0x1C2C68]",
        "fstp dword ptr [{b} + 0x1183CC]",
        "fld dword ptr [{v}]",
        "fmul dword ptr [{b} + 0x1183CC]",
        "fstp dword ptr [{v}]",
        "fld dword ptr [{v} + 0x20]",
        "fmul dword ptr [{b} + 0x1183CC]",
        "fstp dword ptr [{v} + 0x20]",
        "fld dword ptr [{v} + 0x40]",
        "fmul dword ptr [{b} + 0x1183CC]",
        "fstp dword ptr [{v} + 0x40]",
        "fld dword ptr [{v} + 0x60]",
        "fmul dword ptr [{b} + 0x1183CC]",
        "fstp dword ptr [{v} + 0x60]",
        "fild dword ptr [{b} + 0xF4950]",
        "fmul dword ptr [{b} + 0x1C2C6C]",
        "fstp dword ptr [{b} + 0x1183CC]",
        "fld dword ptr [{v} + 0x4]",
        "fmul dword ptr [{b} + 0x1183CC]",
        "fstp dword ptr [{v} + 0x4]",
        "fld dword ptr [{v} + 0x24]",
        "fmul dword ptr [{b} + 0x1183CC]",
        "fstp dword ptr [{v} + 0x24]",
        "fld dword ptr [{v} + 0x44]",
        "fmul dword ptr [{b} + 0x1183CC]",
        "fstp dword ptr [{v} + 0x44]",
        "fld dword ptr [{v} + 0x64]",
        "fmul dword ptr [{b} + 0x1183CC]",
        "fstp dword ptr [{v} + 0x64]";
        b = base(), s = uv, v = a(VERTS)
    );
}

/// The draw (`sub_41ADE1`): eax 4 vertices, ebx `g_polygonVertices`.
pub fn draw(r: &Regs, part: Part) -> Result<(), Refused> {
    let mut d = Regs {
        eax: 4,
        ebx: a(VERTS),
        ..*r
    };
    part(DRAW, &mut d)
}

// ---- for the self-tests ----
