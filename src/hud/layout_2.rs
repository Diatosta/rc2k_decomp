//! `hud::layout_2` (0x4B3B64): the HUD of layout 2, two players one above the other (the
//! record's +0xF8 flags the upper's, +0xFC the lower's): the dials (`hud::disc_2a` /
//! `_2b`), a black divider at 0x12B, the stage bars (`hud::draw_stage_bars_2`), the cut
//! calls (`sub_4B6A8E` / `sub_4B6A99`: ecx 2 / 5), the icons (`hud::icons_2`), the scaled
//! sprites, the markers (`hud::draw_markers_2`), signs (`hud::draw_signs_2a` / `_2b`),
//! needles (`hud::needle_2a` / `_2b`), the lap panels (+0x1C / +0x4C, `dword_75C07C` /
//! `dword_75C088`), the times with their leading digits left out (+0x14 / +0x44) and the
//! four clocks (by the upper's flag 0x40 alone). The upper's rows move 0x10 down and the
//! lower's 0x10 up with `dword_6D1502` 0. Both needles' signs read the record's +0xD0
//! (layout 1 reads +0xCC for the first: section 16).
//!
//! In:  ebp the HUD's record
//! Out: as the path leaves them
//!
//! Caller: `sub_4B1CD0`. The draws through `part` as `hud::layout_1`'s (2 / 3 the dials,
//! 5 the bars, 6 the icons, 7 the markers, 8 / 9 the signs, 10 / 11 the needles of
//! layout 2). Shadow mode runs the original alone (it draws). Self-test:
//! `layout_1::test_layout`.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, game_addr, game_read, hook_asm, rva, shadow};
use crate::hud::add_quad::QUAD_RVA;
use crate::hud::batch::TEXTURE_RVA;
use crate::hud::draw_signs::row_by;
use crate::hud::layout_0::*;
use crate::hud::layout_1::{CLOCKS_RVA, LAP_2_RVA, Part, pick, scaled, test_layout};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("hud::layout_2", rva(0x4B3B64), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// The upper half's rows: 0x10 down with `dword_6D1502` 0.
fn upper(y: u32) -> u32 {
    row_by(y, 0xFFFF_FFF0)
}

/// The lower half's rows: 0x10 up with `dword_6D1502` 0.
fn lower(y: u32) -> u32 {
    row_by(y, 0x10)
}

/// The Rust version (see the header): the registers it leaves.
// Converted: `rc2k_core::hud::panels` (through `layout_0`'s panels and `needle_sign`),
// `rc2k_core::hud::quads::int_over` (the sprites' scales)
pub unsafe fn layout_2(entry: &Regs, part: Part) -> Result<Regs, Refused> {
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
    for (n, v) in [0, 0x12B, 0x320, 0x12D].into_iter().enumerate() {
        set(QUAD_RVA + 4 + 4 * n, v);
    }
    part(4, &mut g.clone(), &[])?;
    if f1 & 0x40 != 0 {
        part(5, &mut g, &[])?;
    }
    set(QUAD_RVA + 0x14, 0xFFFF_FFFF);
    set(QUAD_RVA, 0xFF);
    if f1 & 8 != 0 {
        g.ecx = 2;
    }
    if f2 & 8 != 0 {
        g.ecx = 5;
    }
    set(TEXTURE_RVA, 0xDC);
    part(6, &mut g, &[])?;
    set(QUAD_RVA + 0x14, 0xFFFF_FFFF);
    set(QUAD_RVA, 0xFF);
    set(TEXTURE_RVA, 0xC1);
    if f1 & 0x20 != 0 {
        scaled(&mut g, 0x49, 7, upper(0xE8), &mut part)?;
        scaled(
            &mut g,
            get(0x18).wrapping_add(0x41),
            0xD,
            upper(0xEF),
            &mut part,
        )?;
    }
    if f2 & 0x20 != 0 {
        scaled(&mut g, 0x49, 7, 0x223, &mut part)?;
        scaled(&mut g, get(0x48).wrapping_add(0x41), 0xD, 0x22A, &mut part)?;
    }
    let p = pick(get(0xD0));
    if f1 & 1 != 0 {
        scaled(&mut g, p, 0x19, upper(0xA3), &mut part)?;
    }
    set(TEXTURE_RVA, 0xC0);
    if f1 & 0x10 != 0 {
        scaled(&mut g, 0x5F, 0x2A7, upper(0xA3), &mut part)?;
    }
    let p = pick(get(0xD0));
    if f2 & 1 != 0 {
        scaled(&mut g, p, 0x19, 0x1DE, &mut part)?;
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
    let xs = [0x23, 0x37, 0x4B, 0x53, 0x67, 0x7B, 0x83, 0x97];
    let laps = [
        (f1, 0x1C, [4, 4], 0, [3, 1, 4, 0x26], TIMES_RVA[2], 0x31),
        (
            f2,
            0x4C,
            [lower(0x142), lower(0x142)],
            1,
            [3, lower(0x13F), 4, lower(0x164)],
            LAP_2_RVA,
            lower(0x16C),
        ),
    ];
    for (flags, field, last, mark, sprites, time, y) in laps {
        if flags & 2 == 0 {
            continue;
        }
        let v = get(field);
        if v != 0xFFFF_FFFF {
            let d = unsafe { time_digits(&mut g, v, 2) };
            sign(v, 0, 1);
            plain(&mut g, d[4], 0x55, last[0], &mut part)?;
            plain(&mut g, d[5], 0x81, last[1], &mut part)?;
        }
        plain(&mut g, mark, sprites[0], sprites[1], &mut part)?;
        plain(&mut g, 0x4A, sprites[2], sprites[3], &mut part)?;
        let t: u32 = unsafe { game_read(time) };
        unsafe {
            lap_panel(&mut g, t, xs, y, &mut |g, r, x, y| {
                plain(g, r, x, y, &mut part)
            })
        }?;
    }
    for (flags, field, y) in [(f1, 0x14, upper(0x107)), (f2, 0x44, 0x245)] {
        if flags & 0x20 != 0 {
            unsafe {
                leading_time(
                    &mut g,
                    get(field),
                    [0x43, 0x57, 0x6B],
                    y,
                    &mut |g, r, x, y| plain(g, r, x, y, &mut part),
                )
            }?;
        }
    }
    let left = [8, 0x16, 0x23, 0x2A, 0x38, 0x45, 0x4C, 0x5A];
    let right = [0x2B7, 0x2C5, 0x2D2, 0x2D9, 0x2E7, 0x2F4, 0x2FB, 0x309];
    for (times, y) in [
        ([TIMES_RVA[0], TIMES_RVA[1]], 0x120u32),
        (CLOCKS_RVA, 0x12F),
    ] {
        if f1 & 0x40 != 0 {
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
    Ok(g)
}

/// In the game: the draws' Rust versions.
pub fn game_part(place: u32, r: &mut Regs, v: &[[u8; 10]]) -> Result<(), Refused> {
    use crate::hud::*;
    unsafe {
        *r = match place {
            2 => dials::set_up(r, &disc_2a::SETUP, &mut add_triangle::game_part_disc)?,
            3 => dials::set_up(r, &disc_2b::SETUP, &mut add_triangle::game_part_disc)?,
            5 => draw_stage_bars_2::draw_stage_bars_2(r, &mut draw_stage_bar::game_part)?,
            6 => icons_2::icons_2(r, &mut icons_2::game_part)?,
            7 => draw_markers_2::draw_markers_2(r, &mut draw_markers_1::game_part)?,
            8 => draw_signs_2a::draw_signs_2a(r, &mut draw_signs::game_part)?,
            9 => draw_signs_2b::draw_signs_2b(r, &mut draw_signs::game_part)?,
            10 => dials::set_up(r, &needle_2a::SETUP, &mut draw_needle::game_part_needle)?,
            11 => dials::set_up(r, &needle_2b::SETUP, &mut draw_needle::game_part_needle)?,
            _ => return layout_1::game_part(place, r, v),
        };
    }
    Ok(())
}

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    match unsafe { layout_2(r, &mut game_part) } {
        Ok(left) => *r = Regs { esp: r.esp, ..left },
        Err(e) => unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) },
    }
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    let left = unsafe { layout_2(r, &mut game_part) }?;
    *r = Regs { esp: r.esp, ..left };
    Ok(())
}

fn self_test() -> (usize, usize) {
    const CALLS: [usize; 82] = [
        0x20, 0x3B, 0x94, 0xA7, 0xCC, 0xDD, 0xEC, 0x16F, 0x1D3, 0x238, 0x290, 0x33A, 0x3B1, 0x44F,
        0x4BA, 0x4D7, 0x4F2, 0x503, 0x51E, 0x52F, 0x5FB, 0x618, 0x635, 0x652, 0x709, 0x726, 0x743,
        0x760, 0x77D, 0x79A, 0x7B7, 0x7D4, 0x8AC, 0x8D5, 0x8FE, 0x927, 0x9EA, 0xA13, 0xA3C, 0xA65,
        0xA8E, 0xAB7, 0xAE0, 0xB09, 0xBEF, 0xC30, 0xC63, 0xD3D, 0xD72, 0xD99, 0xE60, 0xE7D, 0xE9A,
        0xEB7, 0xED4, 0xEF1, 0xF0E, 0xF2B, 0xFE2, 0xFFF, 0x101C, 0x1039, 0x1056, 0x1073, 0x1090,
        0x10AD, 0x1174, 0x1191, 0x11AE, 0x11CB, 0x11E8, 0x1205, 0x1222, 0x123F, 0x12F6, 0x1313,
        0x1330, 0x134D, 0x136A, 0x1387, 0x13A4, 0x13C1,
    ];
    test_layout(
        &HOOK,
        0x4B3B64,
        0x13C7,
        &CALLS,
        &[
            (0x4B70EE, 1),
            (0x4B6AA4, 2),
            (0x4B6B0A, 3),
            (0x4B7216, 4),
            (0x4B63C2, 5),
            (0x4B57FE, 6),
            (0x4B55EE, 7),
            (0x4B507F, 8),
            (0x4B51A3, 9),
            (0x4B6DEC, 10),
            (0x4B6E34, 11),
        ],
        &|r, p| unsafe { layout_2(r, p) },
    )
}
