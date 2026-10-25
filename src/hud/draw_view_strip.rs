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
use crate::framework::rva;
use crate::gfx::strips::STRIPS;
use crate::session::SCREEN;
use crate::store::{game_read, game_write};
use rc2k_core::hud::strips;

const FLAT_RVA: usize = rva(0x6D1502); // dword_6D1502
pub const QUAD_IN_RVA: usize = rva(0x623DE8); // dword_623DE8 .. dword_623E24
const ALPHA_RVA: usize = rva(0x6240D0); // g_polygonAlpha
pub const TEXTURE_RVA: usize = rva(0x6240BC); // dword_6240BC
const SIDES_RVA: [usize; 2] = [rva(0x6240E0), rva(0x6240E4)];
const SHOWN_RVA: usize = rva(0x6D14F6); // dword_6D14F6

fn set(r: impl crate::store::Where, v: u32) {
    unsafe { game_write(r, v) };
}

/// The strip's sizes into `dword_6D1870` .. (`strips::sizes`).
fn sizes(v: [u32; 4]) {
    let (sizes, _) = strips::sizes(v, SCREEN.width.get(), SCREEN.height.get());
    for (n, x) in sizes.into_iter().enumerate() {
        STRIPS.sizes[n].set(x);
    }
}

/// One flat quad: its inputs from `left`, `width`, `rows` (the texture's rows) and the
/// sizes (`strips::quad_inputs`), the texture when given, then the device's settings.
fn quad(left: u32, width: u32, rows: u32, texture: Option<u32>) -> Result<(), Refused> {
    let sizes: [u32; 4] = std::array::from_fn(|n| STRIPS.sizes[n].get());
    for (o, v) in strips::quad_inputs(left, width, &sizes, rows) {
        set(QUAD_IN_RVA + o, v);
    }
    if let Some(t) = texture {
        set(TEXTURE_RVA, t);
    }
    set(ALPHA_RVA, 0xFF);
    set(QUAD_IN_RVA, 0xFFFF_FFFF);
    set(SIDES_RVA[0], 1);
    set(SIDES_RVA[1], 0);
    unsafe { crate::scene::draw_flat_quad::draw_flat_quad() }
}

/// The shared tail (0x45B9AA, see the header).
// Converted: `rc2k_core::hud::strips::{tail_shown, TAIL}`
pub unsafe fn overlay_tail() -> Result<(), Refused> {
    let (shown, players) = unsafe {
        (
            game_read(SHOWN_RVA),
            crate::hud::overlay_pass::shown_layout(),
        )
    };
    if !strips::tail_shown(shown, players) {
        return Ok(());
    }
    for (v, rows) in strips::TAIL {
        sizes(v);
        let get = |n: usize| STRIPS.sizes[n].get();
        quad(get(2), get(3), rows, Some(strips::TAIL_TEXTURE))?;
    }
    Ok(())
}

/// The Rust version (see the header): view `view`'s strip, then the shared tail.
// Converted: `rc2k_core::hud::strips::{VIEWS, top_entry, sizes, quad_inputs}`
pub unsafe fn draw_view_strip(view: u32) -> Result<(), Refused> {
    if view >= strips::VIEWS {
        return Err(BadInput("a view past the overlay's tables").into());
    }
    unsafe {
        let top = strips::top_entry(view, game_read(FLAT_RVA));
        let top: u32 = STRIPS.tops[top as usize].get();
        let left: u32 = STRIPS.lefts[view as usize].get();
        sizes([top, strips::HEIGHT, left, strips::WIDTH]);
        let get = |n: usize| STRIPS.sizes[n].get();
        let (mut left, width, mut rows) = (get(2), get(3), 0);
        for _ in 0..3 {
            quad(left, width, rows, None)?;
            left = left.wrapping_add(width);
            rows = rows.wrapping_add(strips::ROWS_STEP);
        }
        overlay_tail()
    }
}
