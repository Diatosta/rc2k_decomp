//! `hud::layout_1` (0x4B2743): the HUD of layout 1, two players side by side (the record's
//! +0xF8 flags the first's, +0xFC the second's): the dials (`hud::disc_1a` / `_1b`), a
//! black divider, the stage bars (`hud::draw_stage_bars_1`), the cut calls (`sub_4B6A78` /
//! `sub_4B6A83`: ecx 1 / 4 and an empty function), the icons (`hud::icons_1`), each
//! player's sprites scaled 0xC0 over `flt_75C00C` (`hud::add_scaled_sprite`), the markers
//! (`hud::draw_markers_1`), signs (`hud::draw_signs_1a` / `_1b`), needles
//! (`hud::needle_1a` / `_1b`), each player's time with its leading digits left out, the
//! four clocks (`dword_75C074`, `dword_75C078`, `dword_75C080`, `dword_75C084`, by the
//! first player's flag 0x40 and +0x28), and each player's lap panel (+0x1C / +0x4C,
//! `dword_75C07C` / `dword_75C088`). Textures, alpha and colour set between, as the
//! original.
//!
//! In:  ebp the HUD's record
//! Out: as the path leaves them
//!
//! Caller: `sub_4B1CD0`. The draws through `part` (1 `add_sprite`, 2 `disc_1a`, 3
//! `disc_1b`, 4 `add_solid_quad`, 5 `draw_stage_bars_1`, 6 `icons_1`, 7
//! `draw_markers_1`, 8 / 9 the signs, 10 / 11 the needles, 12 `add_scaled_sprite` with its
//! two scales), in the game their Rust versions. Shadow mode runs the original alone (it
//! draws). Self-test: a copy whose draws go to stand-ins (the scaled sprites' reading
//! their scales), against the Rust version, flags and times by lot, at both test control words.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{Regs, game_addr, game_read, rva};
use crate::hud::add_quad::QUAD_RVA;
use crate::hud::batch::TEXTURE_RVA;
use crate::hud::draw_markers_1::{int_over_80, taken_off};
use crate::hud::draw_signs::row;
use crate::hud::layout_0::*;

const SCRATCH_RVA: usize = rva(0x75C060); // dword_75C060
const UV_SCALE_RVA: usize = rva(0x75C00C); // flt_75C00C
pub const CLOCKS_RVA: [usize; 2] = [rva(0x75C080), rva(0x75C084)];
pub const LAP_2_RVA: usize = rva(0x75C088); // dword_75C088
pub const SCALED: u32 = 12;

/// A call by place (see the header), with its x87 values (st(0) first).
pub type Part<'a> = &'a mut dyn FnMut(u32, &mut Regs, &[[u8; 10]]) -> Result<(), Refused>;

/// A sprite scaled 0xC0 over `flt_75C00C` both ways; the scales then taken off into
/// `dword_75C060`.
pub fn scaled(g: &mut Regs, rect: u32, x: u32, y: u32, part: &mut Part) -> Result<(), Refused> {
    g.eax = rect.wrapping_mul(0x10);
    g.ebx = x;
    g.ecx = y;
    g.esi = (game_addr(crate::hud::draw_marker::RECTS_RVA) as u32).wrapping_add(g.eax);
    let k: u32 = unsafe { game_read(UV_SCALE_RVA) };
    set(SCRATCH_RVA, 0xC0);
    let v = int_over_80(0xC0, k);
    part(SCALED, g, &[v, v])?;
    set(SCRATCH_RVA, taken_off(v));
    Ok(())
}

/// The sign of the needle by the record's `field` (`dword_75C040`, `dword_75BFAC`, the
/// texture).
pub fn pick(field: u32) -> u32 {
    needle_sign(field)
}

/// The Rust version (see the header): the registers it leaves.
// Converted: `rc2k_core::hud::panels` (through `layout_0`'s panels and `needle_sign`),
// `rc2k_core::hud::quads::int_over` (the sprites' scales)
pub unsafe fn layout_1(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut part = part;
    let mut g = *entry;
    let get = |o: u32| unsafe { ((entry.ebp + o) as *const u32).read_unaligned() };
    let [f1, f2] = [get(0xF8), get(0xFC)];
    let plain = |g: &mut Regs, rect: u32, x: u32, y: u32, part: &mut Part| {
        g.eax = rect.wrapping_mul(0x10);
        g.ebx = x;
        g.ecx = y;
        g.esi = (game_addr(crate::hud::draw_marker::RECTS_RVA) as u32).wrapping_add(g.eax);
        part(1, g, &[])
    };
    set(QUAD_RVA, 0xE0);
    set(TEXTURE_RVA, 0xC6);
    if f1 & 0x10 != 0 {
        part(2, &mut g, &[])?;
    }
    set(TEXTURE_RVA, 0xC7);
    if f2 & 0x10 != 0 {
        part(3, &mut g, &[])?;
    }
    set(TEXTURE_RVA, 0xC0);
    set(QUAD_RVA, 0xFF);
    set(QUAD_RVA + 0x14, 0);
    for (n, v) in [0x18F, 0, 0x191, 0x258].into_iter().enumerate() {
        set(QUAD_RVA + 4 + 4 * n, v);
    }
    part(4, &mut g.clone(), &[])?;
    if f1 & 0x40 != 0 {
        part(5, &mut g, &[])?;
    }
    set(QUAD_RVA + 0x14, 0xFFFF_FFFF);
    set(QUAD_RVA, 0xFF);
    if f1 & 8 != 0 {
        g.ecx = 1;
    }
    if f2 & 8 != 0 {
        g.ecx = 4;
    }
    set(TEXTURE_RVA, 0xDC);
    part(6, &mut g, &[])?;
    set(QUAD_RVA + 0x14, 0xFFFF_FFFF);
    set(QUAD_RVA, 0xFF);
    set(TEXTURE_RVA, 0xC1);
    if f1 & 0x20 != 0 {
        scaled(&mut g, 0x49, 7, 0x223, &mut part)?;
        scaled(&mut g, get(0x18).wrapping_add(0x41), 0xD, 0x22A, &mut part)?;
    }
    if f2 & 0x20 != 0 {
        scaled(&mut g, 0x49, 0x198, 0x223, &mut part)?;
        scaled(
            &mut g,
            get(0x48).wrapping_add(0x41),
            0x19E,
            0x22A,
            &mut part,
        )?;
    }
    let p = pick(get(0xCC));
    if f1 & 1 != 0 {
        scaled(&mut g, p, 0x19, 0x1DE, &mut part)?;
    }
    set(TEXTURE_RVA, 0xC0);
    if f1 & 0x10 != 0 {
        scaled(&mut g, 0x5F, 0x116, 0x1DE, &mut part)?;
    }
    let p = pick(get(0xD0));
    if f2 & 1 != 0 {
        scaled(&mut g, p, 0x1AA, 0x1DE, &mut part)?;
    }
    set(TEXTURE_RVA, 0xC0);
    if f2 & 0x10 != 0 {
        scaled(&mut g, 0x5F, 0x2A7, 0x1DE, &mut part)?;
    }
    if f1 & 0x40 != 0 {
        part(7, &mut g, &[])?;
    }
    set(QUAD_RVA + 0x14, 0xFFFF_FFFF);
    if f1 & 0x80 != 0 {
        part(8, &mut g, &[])?;
    }
    if f2 & 0x80 != 0 {
        part(9, &mut g, &[])?;
    }
    set(TEXTURE_RVA, 0xC1);
    if f1 & 1 != 0 {
        part(10, &mut g, &[])?;
    }
    if f2 & 1 != 0 {
        part(11, &mut g, &[])?;
    }
    for (flags, field, xs) in [
        (f1, 0x14, [0x43, 0x57, 0x6B]),
        (f2, 0x44, [0x1D4, 0x1E8, 0x1FC]),
    ] {
        if flags & 0x20 != 0 {
            unsafe {
                leading_time(&mut g, get(field), xs, 0x245, &mut |g, r, x, y| {
                    plain(g, r, x, y, &mut part)
                })
            }?;
        }
    }
    let left = [8, 0x16, 0x23, 0x2A, 0x38, 0x45, 0x4C, 0x5A];
    let right = [0x2B7, 0x2C5, 0x2D2, 0x2D9, 0x2E7, 0x2F4, 0x2FB, 0x309];
    for (times, y) in [([TIMES_RVA[0], TIMES_RVA[1]], 2u32), (CLOCKS_RVA, 0x11)] {
        if f1 & 0x40 != 0 && get(0x28) != 0xFFFF_FFFF {
            for (time, xs) in [(times[0], left), (times[1], right)] {
                let t: u32 = unsafe { game_read(time) };
                unsafe {
                    clock_panel(&mut g, t, xs, y, &mut |g, r, x, y| {
                        plain(g, r, x, y, &mut part)
                    })
                }?;
            }
        }
    }
    let laps = [
        (
            f1,
            0x1C,
            [0x55, 0x81],
            0u32,
            [3, 0x21, 4, 0x46],
            TIMES_RVA[2],
            [0x23, 0x37, 0x4B, 0x53, 0x67, 0x7B, 0x83, 0x97],
        ),
        (
            f2,
            0x4C,
            [0x1E6, 0x212],
            1,
            [0x194, 0x21, 0x195, 0x46],
            LAP_2_RVA,
            [0x1B4, 0x1C8, 0x1DC, 0x1E4, 0x1F8, 0x20C, 0x214, 0x228],
        ),
    ];
    for (flags, field, last, mark, sprites, time, xs) in laps {
        if flags & 2 == 0 {
            continue;
        }
        let v = get(field);
        if v != 0xFFFF_FFFF {
            let d = unsafe { time_digits(&mut g, v, 2) };
            sign(v, 0, 1);
            plain(&mut g, d[4], last[0], row(0x24), &mut part)?;
            plain(&mut g, d[5], last[1], row(0x24), &mut part)?;
        }
        plain(&mut g, mark, sprites[0], row(sprites[1]), &mut part)?;
        plain(&mut g, 0x4A, sprites[2], row(sprites[3]), &mut part)?;
        let t: u32 = unsafe { game_read(time) };
        unsafe {
            lap_panel(&mut g, t, xs, row(0x51), &mut |g, r, x, y| {
                plain(g, r, x, y, &mut part)
            })
        }?;
    }
    Ok(g)
}

/// In the game: the draws' Rust versions.
pub fn game_part(place: u32, r: &mut Regs, v: &[[u8; 10]]) -> Result<(), Refused> {
    use crate::hud::*;
    unsafe {
        *r = match place {
            1 => return draw_marker::game_part(place, r),
            2 => dials::set_up(r, &disc_1a::SETUP, &mut add_triangle::game_part_disc)?,
            3 => dials::set_up(r, &disc_1b::SETUP, &mut add_triangle::game_part_disc)?,
            4 => return draw_stage_bar::game_part(place, r),
            5 => draw_stage_bars_1::draw_stage_bars_1(r, &mut draw_stage_bar::game_part)?,
            6 => icons_1::icons_1(r, &mut icons_1::game_part)?,
            7 => draw_markers_1::draw_markers_1(r)?,
            8 => draw_signs_1a::draw_signs_1a(r)?,
            9 => draw_signs_1b::draw_signs_1b(r)?,
            10 => dials::set_up(r, &needle_1a::SETUP, &mut draw_needle::game_part_needle)?,
            11 => dials::set_up(r, &needle_1b::SETUP, &mut draw_needle::game_part_needle)?,
            _ => return draw_markers_1::game_part(place, r, v),
        };
    }
    Ok(())
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    let left = unsafe { layout_1(r, &mut game_part) }?;
    *r = Regs { esp: r.esp, ..left };
    Ok(())
}
