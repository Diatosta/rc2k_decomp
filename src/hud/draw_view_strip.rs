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
//! `dword_623E24`, `g_polygonAlpha` 0xFF, colour -1, `dword_6240E0` 1, `dword_6240E4` 0)
//! then `scene::draw_flat_quad` through `part` (1), in the game its Rust version (inside
//! `pusha` .. `popa`). Refuses a view past 5 (the tables' end), before anything. Shadow
//! mode runs the original alone (it draws). Self-test: a copy whose quads go to a
//! stand-in noting their inputs, against the Rust version, views and settings by lot.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::{BadInput, Refused};
use crate::framework::{Regs, game_addr, game_read, game_write, rva};
use crate::service::harness::{Region, at};
use rc2k_core::hud::strips;
use std::sync::Mutex;

const TOPS_RVA: usize = rva(0x6D1880); // dword_6D1880: 12
const LEFTS_RVA: usize = rva(0x6D18B0); // dword_6D18B0: 6
const SIZES_RVA: usize = rva(0x6D1870); // dword_6D1870 .. dword_6D187C: top, height, left, width
const WIDTH_RVA: usize = rva(0x4F494C); // g_screenWidth
const HEIGHT_RVA: usize = rva(0x4F4950); // g_screenHeight
const FLAT_RVA: usize = rva(0x6D1502); // dword_6D1502
pub const QUAD_IN_RVA: usize = rva(0x623DE8); // dword_623DE8 .. dword_623E24
const ALPHA_RVA: usize = rva(0x6240D0); // g_polygonAlpha
pub const TEXTURE_RVA: usize = rva(0x6240BC); // dword_6240BC
const SIDES_RVA: [usize; 2] = [rva(0x6240E0), rva(0x6240E4)];
const SHOWN_RVA: usize = rva(0x6D14F6); // dword_6D14F6
const PLAYERS_RVA: usize = rva(0x6238A4); // dword_6238A4

/// A call by place: 1 `draw_flat_quad`.
pub type Part<'a> = &'a mut dyn FnMut(u32, &mut Regs) -> Result<(), Refused>;

fn set(r: usize, v: u32) {
    unsafe { game_write(r, v) };
}

/// The strip's sizes into `dword_6D1870` .. (`strips::sizes`); eax, edx, ebx as the last
/// division leaves them.
fn sizes(g: &mut Regs, v: [u32; 4]) {
    let (sizes, [eax, edx, ebx]) =
        unsafe { strips::sizes(v, game_read(WIDTH_RVA), game_read(HEIGHT_RVA)) };
    [g.eax, g.edx, g.ebx] = [eax, edx, ebx];
    for (n, x) in sizes.into_iter().enumerate() {
        set(SIZES_RVA + 4 * n, x);
    }
}

/// One flat quad: its inputs from ebx (left), ebp (width), ecx (texture rows) and the
/// sizes (`strips::quad_inputs`), the texture when given, then the device's settings.
fn quad(g: &Regs, texture: Option<u32>, part: &mut Part) -> Result<(), Refused> {
    let mut r = *g;
    let sizes: [u32; 4] = unsafe { game_read(SIZES_RVA) };
    for (o, v) in strips::quad_inputs(r.ebx, r.ebp, &sizes, r.ecx) {
        set(QUAD_IN_RVA + o, v);
    }
    r.eax = r.ecx.wrapping_add(strips::ROWS_STEP);
    if let Some(t) = texture {
        set(TEXTURE_RVA, t);
    }
    set(ALPHA_RVA, 0xFF);
    set(QUAD_IN_RVA, 0xFFFF_FFFF);
    set(SIDES_RVA[0], 1);
    set(SIDES_RVA[1], 0);
    part(1, &mut r)
}

/// The shared tail (0x45B9AA, see the header).
// Converted: `rc2k_core::hud::strips::{tail_shown, TAIL}`
pub unsafe fn overlay_tail(g: &mut Regs, part: &mut Part) -> Result<(), Refused> {
    let (shown, players) = unsafe { (game_read(SHOWN_RVA), game_read(PLAYERS_RVA)) };
    if !strips::tail_shown(shown, players) {
        return Ok(());
    }
    for (v, rows) in strips::TAIL {
        sizes(g, v);
        let get = |n: usize| unsafe { game_read::<u32>(SIZES_RVA + 4 * n) };
        [g.ebx, g.ebp, g.edx, g.ecx] = [get(2), get(3), 0, rows];
        quad(g, Some(strips::TAIL_TEXTURE), part)?;
    }
    Ok(())
}

/// The Rust version (see the header): the registers it leaves.
// Converted: `rc2k_core::hud::strips::{VIEWS, top_entry, sizes, quad_inputs}`
pub unsafe fn draw_view_strip(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut part = part;
    let mut g = *entry;
    if g.ebp >= strips::VIEWS {
        return Err(BadInput("a view past the overlay's tables").into());
    }
    unsafe {
        let top = strips::top_entry(g.ebp, game_read(FLAT_RVA));
        let top: u32 = game_read(TOPS_RVA + 4 * top);
        let left: u32 = game_read(LEFTS_RVA + 4 * g.ebp as usize);
        sizes(&mut g, [top, strips::HEIGHT, left, strips::WIDTH]);
        let get = |n: usize| game_read::<u32>(SIZES_RVA + 4 * n);
        [g.ebx, g.ebp, g.ecx, g.edx] = [get(2), get(3), 0, 0];
        for _ in 0..3 {
            quad(&g, None, &mut part)?;
            g.ebx = g.ebx.wrapping_add(get(3));
            g.ecx = g.ecx.wrapping_add(strips::ROWS_STEP);
            g.edx += 1;
        }
        overlay_tail(&mut g, &mut part)?;
    }
    Ok(g)
}

pub fn game_part(_place: u32, r: &mut Regs) -> Result<(), Refused> {
    use crate::scene::draw_flat_quad as fq;
    *r = unsafe { fq::draw_flat_quad(r, &mut fq::game_part) }?;
    Ok(())
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    let left = unsafe { draw_view_strip(r, &mut game_part) }?;
    *r = Regs { esp: r.esp, ..left };
    Ok(())
}

/// A quad as the tests note it: the place, eax .. ebp, its inputs and texture.
type Noted = (u32, [u32; 6], [u32; 0x10], u32);

static NOTED: Mutex<Vec<Noted>> = Mutex::new(Vec::new());

/// For `install::stand_ins::set_handler` and the Rust side of the tests.
pub fn note_quad(place: u32, r: &mut Regs) {
    let inputs: [u32; 0x10] = unsafe { game_read(QUAD_IN_RVA) };
    let texture: u32 = unsafe { game_read(TEXTURE_RVA) };
    NOTED.lock().unwrap().push((
        place,
        [r.eax, r.ebx, r.ecx, r.edx, r.esi, r.ebp],
        inputs,
        texture,
    ));
}

pub fn take() -> Vec<Noted> {
    std::mem::take(&mut *NOTED.lock().unwrap())
}

/// What the strips write (for the tests).
pub fn regions() -> Vec<Region> {
    vec![
        (game_addr(SIZES_RVA) as u32, 0x10),
        (game_addr(QUAD_IN_RVA) as u32, 0x40),
        at(ALPHA_RVA),
        at(TEXTURE_RVA),
        at(SIDES_RVA[0]),
        at(SIDES_RVA[1]),
    ]
}

/// The settings by lot (for the tests): what they held is the caller's to put back
/// (`settings()`).
pub unsafe fn test_settings(case: u32) {
    set(FLAT_RVA, case & 1);
    set(SHOWN_RVA, if case.is_multiple_of(3) { 0 } else { 0x40 });
    set(PLAYERS_RVA, u32::from(case == 4));
    set(TEXTURE_RVA, 0xC5);
}

pub fn settings() -> Vec<Region> {
    [FLAT_RVA, SHOWN_RVA, PLAYERS_RVA].map(at).to_vec()
}
