//! `hud::layout_0` (0x4B1D8C): the HUD of layout 0 (the record's +0xF4 neither 1 nor 2),
//! by the record's flags (+0xF8): 0x10 the dial (`hud::disc_0`) and a sprite; 0x40 the
//! stage bar (`hud::draw_stage_bar`), the marker (`hud::draw_marker`) and the two time
//! panels (sprite 0x29 + the record's +0x28 / +0x30 held to 0 .. 0x16, the times
//! `dword_75C074` / `dword_75C078` as digits); 8 the cut call (`sub_4B6A6D`: ecx 0 and an
//! empty function); always the icons (`hud::icons_0`); 0x20 two sprites (the second by the
//! record's +0x18) and the time +0x14 with its leading digits left out while they are 0;
//! 1 the sign by +0xCC (also into `dword_75BFAC`) and the needle (`hud::needle_0`); 0x80
//! the signs (`hud::draw_signs`); 2 the time +0x1C's last two digits, two sprites and the
//! time `dword_75C07C`. Textures (`dword_75C034`), the quad record's alpha and colour set
//! between, as the original.
//!
//! In:  ebp the HUD's record
//! Out: as the path leaves them (edx the last time's last digit, ebx 10 after a time)
//!
//! Caller: `sub_4B1CD0`. The draws through `part` (1 `add_sprite`, 2 `disc_0`, 3
//! `draw_stage_bar`, 4 `icons_0`, 5 `draw_marker`, 6 `draw_signs`, 7 `needle_0`), in the
//! game their Rust versions. Shadow mode runs the original alone (it draws). Self-test: a
//! copy whose draws go to a stand-in, against the Rust version, flags and times by lot,
//! at both test control words.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{Regs, game_addr, game_read, game_write, rva};
use crate::hud::add_quad::QUAD_RVA;
use crate::hud::batch::TEXTURE_RVA;
use crate::hud::draw_marker::RECTS_RVA;
use crate::hud::draw_signs::row;
use crate::service::harness::{Region, at};
use rc2k_core::hud::panels;

pub const TIMES_RVA: [usize; 3] = [rva(0x75C074), rva(0x75C078), rva(0x75C07C)];
pub const DIGITS_RVA: usize = rva(0x75C08C); // dword_75C08C .. dword_75C0A0: six
pub const SIGN_RVA: usize = rva(0x75C0A4); // dword_75C0A4
pub const PICK_RVA: usize = rva(0x75C040); // dword_75C040
pub const NEEDLE_KIND_RVA: usize = rva(0x75BFAC); // dword_75BFAC
pub const LEADING_RVA: usize = rva(0x75C03C); // dword_75C03C

/// A call by place (see the header).
pub type Part<'a> = &'a mut dyn FnMut(u32, &mut Regs) -> Result<(), Refused>;

/// A time made digits (`dword_75C08C` ..: `rc2k_core::hud::panels::time_digits`), each
/// plus `base`; eax, ebx and edx as the divisions leave them.
pub unsafe fn time_digits(g: &mut Regs, value: u32, base: u32) -> [u32; 6] {
    let d = panels::time_digits(value, base);
    for (n, v) in d.into_iter().enumerate() {
        unsafe { game_write(DIGITS_RVA + 4 * n, v) };
    }
    [g.eax, g.ebx, g.edx] = panels::division_registers(&d);
    d
}

/// A sprite: rectangle `rect` of `dword_75B940` at (x, y).
pub fn sprite(g: &mut Regs, rect: u32, x: u32, y: u32, part: &mut Part) -> Result<(), Refused> {
    g.eax = panels::rect_offset(rect);
    g.ebx = x;
    g.ecx = y;
    g.esi = (game_addr(RECTS_RVA) as u32).wrapping_add(g.eax);
    part(1, g)
}

/// The sign of a time into `dword_75C0A4`: `plus` or `minus`.
pub fn sign(value: u32, plus: u32, minus: u32) -> u32 {
    let s = panels::sign(value, plus, minus);
    unsafe { game_write(SIGN_RVA, s) };
    s
}

pub fn set(r: usize, v: u32) {
    unsafe { game_write(r, v) };
}

/// A sprite drawer for the shared panels: (registers, rectangle, x, y).
pub type Draw<'a> = &'a mut dyn FnMut(&mut Regs, u32, u32, u32) -> Result<(), Refused>;

/// A clock panel: the time `t` (digits from 0x18), its sign 0x22 / 0x23, then its
/// rectangles (`panels::clock_rects`) at `xs`, row `y`.
pub unsafe fn clock_panel(
    g: &mut Regs,
    t: u32,
    xs: [u32; 8],
    y: u32,
    draw: Draw,
) -> Result<(), Refused> {
    unsafe { time_digits(g, t, panels::CLOCK_BASE) };
    sign(t, 0x22, 0x23);
    for (rect, x) in panels::clock_rects(t).into_iter().zip(xs) {
        draw(g, rect, x, y)?;
    }
    Ok(())
}

/// A split-time panel: the time `t` (digits from 0xC, sign 0 / 1), then its rectangles
/// (`panels::lap_rects`) at `xs`, row `y`.
pub unsafe fn lap_panel(
    g: &mut Regs,
    t: u32,
    xs: [u32; 8],
    y: u32,
    draw: Draw,
) -> Result<(), Refused> {
    unsafe { time_digits(g, t, panels::LAP_BASE) };
    sign(t, 0, 1);
    for (rect, x) in panels::lap_rects(t).into_iter().zip(xs) {
        draw(g, rect, x, y)?;
    }
    Ok(())
}

/// The time `v` (digits from 0xC, sign 0 / 1) as its last three digits at `xs`, row `y`,
/// the leading ones left out while they are 0 (`dword_75C03C`;
/// `panels::leading_digits`).
pub unsafe fn leading_time(
    g: &mut Regs,
    v: u32,
    xs: [u32; 3],
    y: u32,
    draw: Draw,
) -> Result<(), Refused> {
    unsafe { time_digits(g, v, panels::LAP_BASE) };
    sign(v, 0, 1);
    set(LEADING_RVA, 1);
    for (digit, slot, drawn) in panels::leading_digits(v) {
        g.eax = digit;
        if !drawn {
            continue;
        }
        draw(g, digit, xs[slot], y)?;
        if slot < 2 {
            set(LEADING_RVA, 0);
        }
    }
    Ok(())
}

/// The needle's sign by the record's `field` (`dword_75C040`, `dword_75BFAC`, the
/// texture, as the original writes them): its rectangle.
pub fn needle_sign(field: u32) -> u32 {
    set(PICK_RVA, 0x5E);
    set(TEXTURE_RVA, 0xC0);
    set(NEEDLE_KIND_RVA, field);
    let (pick, texture) = panels::needle_sign(field);
    if field != 0 {
        set(PICK_RVA, pick);
        set(TEXTURE_RVA, texture);
    }
    pick
}

/// The Rust version (see the header): the registers it leaves.
// Converted: `rc2k_core::hud::panels` (the digits, signs, panels, leading digits, the
// stage panel and the needle's sign)
pub unsafe fn layout_0(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut part = part;
    let mut g = *entry;
    let get = |o: u32| unsafe { ((entry.ebp + o) as *const u32).read_unaligned() };
    let flags = get(0xF8);
    set(QUAD_RVA, 0xE0);
    set(TEXTURE_RVA, 0xC6);
    if flags & 0x10 != 0 {
        part(2, &mut g)?;
    }
    set(TEXTURE_RVA, 0xC0);
    if flags & 0x40 != 0 {
        part(3, &mut g)?;
    }
    set(QUAD_RVA + 0x14, 0xFFFF_FFFF);
    set(QUAD_RVA, 0xFF);
    if flags & 8 != 0 {
        g.ecx = 0;
    }
    set(TEXTURE_RVA, 0xDC);
    part(4, &mut g)?;
    set(QUAD_RVA + 0x14, 0xFFFF_FFFF);
    set(QUAD_RVA, 0xFF);
    set(TEXTURE_RVA, 0xC1);
    if flags & 0x20 != 0 {
        sprite(&mut g, 0x49, 9, 0x215, &mut part)?;
        sprite(&mut g, get(0x18).wrapping_add(0x41), 0x11, 0x21D, &mut part)?;
    }
    let pick = needle_sign(get(0xCC));
    if flags & 1 != 0 {
        sprite(&mut g, pick, 0x21, 0x1B8, &mut part)?;
    }
    set(TEXTURE_RVA, 0xC0);
    if flags & 0x10 != 0 {
        sprite(&mut g, 0x5F, 0x282, 0x1B8, &mut part)?;
    }
    if flags & 0x40 != 0 {
        part(5, &mut g)?;
    }
    set(QUAD_RVA + 0x14, 0xFFFF_FFFF);
    if flags & 0x80 != 0 {
        part(6, &mut g)?;
    }
    set(TEXTURE_RVA, 0xC1);
    if flags & 1 != 0 {
        part(7, &mut g)?;
    }
    if flags & 0x40 != 0 {
        let panels = [
            (
                0x28u32,
                0x4Cu32,
                TIMES_RVA[0],
                [0x14u32, 0x22, 0x2F, 0x36, 0x44, 0x51, 0x58, 0x66],
            ),
            (
                0x30,
                0x2F8,
                TIMES_RVA[1],
                [0x2BD, 0x2CB, 0x2D8, 0x2DF, 0x2ED, 0x2FA, 0x301, 0x30F],
            ),
        ];
        for (field, x, time, xs) in panels {
            let Some(rect) = panels::stage_rect(get(field)) else {
                continue;
            };
            sprite(&mut g, rect, x, 1, &mut part)?;
            let t: u32 = unsafe { game_read(time) };
            unsafe {
                clock_panel(&mut g, t, xs, 0x13, &mut |g, r, x, y| {
                    sprite(g, r, x, y, &mut part)
                })
            }?;
        }
    }
    if flags & 2 != 0 {
        let v = get(0x1C);
        if v != 0xFFFF_FFFF {
            let d = unsafe { time_digits(&mut g, v, 2) };
            sign(v, 0, 1);
            sprite(&mut g, d[4], 0x60, row(0x36), &mut part)?;
            sprite(&mut g, d[5], 0x8C, row(0x36), &mut part)?;
        }
        sprite(&mut g, 0, 0xE, row(0x33), &mut part)?;
        sprite(&mut g, 0x4A, 0xF, row(0x58), &mut part)?;
        let t: u32 = unsafe { game_read(TIMES_RVA[2]) };
        let xs = [0x2E, 0x42, 0x56, 0x5E, 0x72, 0x86, 0x8E, 0xA2];
        unsafe {
            lap_panel(&mut g, t, xs, row(0x63), &mut |g, r, x, y| {
                sprite(g, r, x, y, &mut part)
            })
        }?;
    }
    if flags & 0x20 != 0 {
        let v = get(0x14);
        unsafe {
            leading_time(&mut g, v, [0x43, 0x57, 0x6B], 0x23F, &mut |g, r, x, y| {
                sprite(g, r, x, y, &mut part)
            })
        }?;
    }
    Ok(g)
}

/// In the game: the draws' Rust versions.
pub fn game_part(place: u32, r: &mut Regs) -> Result<(), Refused> {
    use crate::hud::*;
    unsafe {
        *r = match place {
            1 => return draw_marker::game_part(place, r),
            2 => dials::set_up(r, &disc_0::SETUP, &mut add_triangle::game_part_disc)?,
            3 => draw_stage_bar::draw_stage_bar(r, &mut draw_stage_bar::game_part)?,
            4 => icons_0::icons_0(r, &mut icons_0::game_part)?,
            5 => draw_marker::draw_marker(r, &mut draw_marker::game_part)?,
            6 => draw_signs::draw_signs(r, &mut draw_signs::game_part)?,
            _ => dials::set_up(r, &needle_0::SETUP, &mut draw_needle::game_part_needle)?,
        };
    }
    Ok(())
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    let left = unsafe { layout_0(r, &mut game_part) }?;
    *r = Regs { esp: r.esp, ..left };
    Ok(())
}

/// What the layouts write themselves (for their tests).
pub fn regions() -> Vec<Region> {
    let mut r: Vec<Region> = [
        TEXTURE_RVA,
        QUAD_RVA,
        QUAD_RVA + 0x14,
        PICK_RVA,
        NEEDLE_KIND_RVA,
        LEADING_RVA,
        SIGN_RVA,
    ]
    .map(at)
    .to_vec();
    r.push((game_addr(DIGITS_RVA) as u32, 0x18));
    r
}
