//! `hud::draw_view_strip` (0x45B86E): three flat quads side by side for view ebp (a reading:
//! a strip of the screen's overlay): top `dword_6D1880`[ebp] (or [ebp + 6] with
//! `dword_6D1502` 0) and height 0x19 scaled from 600 to `g_screenHeight`, left
//! `dword_6D18B0`[ebp] and width 0x78 scaled from 800 to `g_screenWidth` (into
//! `dword_6D1870` .. `dword_6D187C`), texture rows 0x320000 apart; then the shared tail
//! (`overlay_tail`, 0x45B9AA, which `hud::draw_view_strips` jumps to as well): with
//! `dword_6D14F6` bit 0x40 and `dword_6238A4` 0, two more quads at the top corners
//! (texture `dword_6240BC` 0xC4).
//!
//! In:  ebp the view (0 .. 5)
//! Out: eax / ebx / ebp / ecx / edx as the last quad's set-up leaves them (see the code)
//!
//! Callers: `hud::draw_view_strips` (twice). Each quad's inputs (`dword_623DE8` ..
//! `dword_623E24`, `g_polygonAlpha` 0xFF, colour -1, `dword_6240E0` 1, `dword_6240E4` 0) then
//! `scene::draw_flat_quad` (inside `pusha` .. `popa`). Refuses a view past 5 (the tables' end),
//! before anything.

use crate::errors::{BadInput, Refused};
use crate::gfx::owner::gfx;
use crate::race::owner::Race;
use crate::race::owner::race;
use crate::race::owner_shared::Shared;
use crate::scene::owner::scene;
use crate::system::owner::system;
use rc2k_core::hud::strips;

/// The quad's input at byte offset `o` from `dword_623DE8`: its colour and see-through,
/// texture coordinates, then corners (three consecutive fields).
fn quad_input(shared: &Shared, o: usize) -> &crate::session::Dword {
    match o {
        0..0x8 => &shared.vars.quad_colour[o / 4],
        0x8..0x28 => &shared.vars.quad_uv[(o - 8) / 4],
        _ => &shared.vars.quad_corners[(o - 0x28) / 4],
    }
}

/// The strip's sizes into `dword_6D1870` .. (`strips::sizes`).
fn sizes(v: [u32; 4]) {
    let system = system();
    let (sizes, _) = strips::sizes(
        v,
        system.session.screen.width.get(),
        system.session.screen.height.get(),
    );
    for (n, x) in sizes.into_iter().enumerate() {
        gfx().strips.sizes[n].set(x);
    }
}

/// One flat quad: its inputs from `left`, `width`, `rows` (the texture's rows) and the
/// sizes (`strips::quad_inputs`), the texture when given, then the device's settings.
fn quad(left: u32, width: u32, rows: u32, texture: Option<u32>) -> Result<(), Refused> {
    let shared = &crate::race::owner::race().shared;
    let drawing = &scene().drawing;
    let course = &race().course;
    let sizes: [u32; 4] = std::array::from_fn(|n| gfx().strips.sizes[n].get());
    for (o, v) in strips::quad_inputs(left, width, &sizes, rows) {
        quad_input(shared, o).set_bits(v);
    }
    if let Some(t) = texture {
        drawing.scene.polygon_textures[0].set_bits(t);
    }
    drawing.scene.polygon_alpha.set_bits(0xFF);
    quad_input(shared, 0).set_bits(0xFFFF_FFFF);
    drawing.scene.polygon_sides[0].set_bits(1);
    drawing.scene.polygon_sides[1].set_bits(0);
    crate::scene::draw_flat_quad::draw_flat_quad(course)
}

/// The shared tail (0x45B9AA, see the header).
// Converted: `rc2k_core::hud::strips::{tail_shown, TAIL}`
pub fn overlay_tail(race: &Race) -> Result<(), Refused> {
    let frame = &race.frame;
    let (shown, players) = (
        frame.panel_and_overlay.panel.hud_flags[0].get(),
        crate::hud::overlay_pass::shown_layout(&race.shared),
    );
    if !strips::tail_shown(shown, players) {
        return Ok(());
    }
    for (v, rows) in strips::TAIL {
        sizes(v);
        let get = |n: usize| gfx().strips.sizes[n].get();
        quad(get(2), get(3), rows, Some(strips::TAIL_TEXTURE))?;
    }
    Ok(())
}

/// The Rust version (see the header): view `view`'s strip, then the shared tail.
// Converted: `rc2k_core::hud::strips::{VIEWS, top_entry, sizes, quad_inputs}`
pub fn draw_view_strip(race: &Race, view: u32) -> Result<(), Refused> {
    let frame = &race.frame;
    let strips = &gfx().strips;
    if view >= strips::VIEWS {
        return Err(BadInput("a view past the overlay's tables").into());
    }
    {
        let top = strips::top_entry(view, frame.panel_and_overlay.panel.flat.get());
        let top: u32 = strips.tops[top as usize].get();
        let left: u32 = strips.lefts[view as usize].get();
        sizes([top, strips::HEIGHT, left, strips::WIDTH]);
        let get = |n: usize| strips.sizes[n].get();
        let (mut left, width, mut rows) = (get(2), get(3), 0);
        for _ in 0..3 {
            quad(left, width, rows, None)?;
            left = left.wrapping_add(width);
            rows = rows.wrapping_add(strips::ROWS_STEP);
        }
        overlay_tail(race)
    }
}
