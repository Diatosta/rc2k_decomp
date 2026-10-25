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
//! Caller: `sub_4B1CD0`.

use crate::errors::Refused;
use crate::hud::draw_signs::row_by;
use crate::hud::layout_0::*;
use crate::hud::layout_1::{pick, plain, scaled};
use crate::hud::store::HUD;

/// The upper half's rows: 0x10 down with `dword_6D1502` 0.
fn upper(y: u32) -> u32 {
    row_by(y, 0xFFFF_FFF0)
}

/// The lower half's rows: 0x10 up with `dword_6D1502` 0.
fn lower(y: u32) -> u32 {
    row_by(y, 0x10)
}

/// The Rust version (see the header): the HUD of two players one above the other,
/// `record` the HUD's record.
// Converted: `rc2k_core::hud::panels` (through `layout_0`'s panels and `needle_sign`),
// `rc2k_core::hud::quads::int_over` (the sprites' scales)
pub unsafe fn layout_2(record: u32) -> Result<(), Refused> {
    use crate::hud::*;
    let get = |o: u32| unsafe { ((record + o) as *const u32).read_unaligned() };
    let [f1, f2] = [get(0xF8), get(0xFC)];
    HUD.quad[0].set(0xE0);
    HUD.texture.set(0xC6);
    if f1 & 0x10 != 0 {
        unsafe { dials::disc(record, &disc_2a::SETUP) }?;
    }
    HUD.texture.set(0xC7);
    if f2 & 0x10 != 0 {
        unsafe { dials::disc(record, &disc_2b::SETUP) }?;
    }
    HUD.texture.set(0xC0);
    HUD.quad[0].set(0xFF);
    HUD.quad[5].set(0);
    for (n, v) in [0, 0x12B, 0x320, 0x12D].into_iter().enumerate() {
        HUD.quad[1 + n].set(v);
    }
    unsafe { add_solid_quad::add_solid_quad() }?;
    if f1 & 0x40 != 0 {
        unsafe { draw_stage_bars_2::draw_stage_bars_2(record) }?;
    }
    HUD.quad[5].set(0xFFFF_FFFF);
    HUD.quad[0].set(0xFF);
    HUD.texture.set(0xDC);
    unsafe { icons_2::icons_2(record) }?;
    HUD.quad[5].set(0xFFFF_FFFF);
    HUD.quad[0].set(0xFF);
    HUD.texture.set(0xC1);
    if f1 & 0x20 != 0 {
        scaled(0x49, 7, upper(0xE8))?;
        scaled(get(0x18).wrapping_add(0x41), 0xD, upper(0xEF))?;
    }
    if f2 & 0x20 != 0 {
        scaled(0x49, 7, 0x223)?;
        scaled(get(0x48).wrapping_add(0x41), 0xD, 0x22A)?;
    }
    let p = pick(get(0xD0));
    if f1 & 1 != 0 {
        scaled(p, 0x19, upper(0xA3))?;
    }
    HUD.texture.set(0xC0);
    if f1 & 0x10 != 0 {
        scaled(0x5F, 0x2A7, upper(0xA3))?;
    }
    let p = pick(get(0xD0));
    if f2 & 1 != 0 {
        scaled(p, 0x19, 0x1DE)?;
    }
    HUD.texture.set(0xC0);
    if f2 & 0x10 != 0 {
        scaled(0x5F, 0x2A7, 0x1DE)?;
    }
    if f1 & 0x40 != 0 {
        unsafe { draw_markers_2::draw_markers_2(record) }?;
    }
    HUD.quad[5].set(0xFFFF_FFFF);
    if f1 & 0x80 != 0 {
        unsafe { draw_signs_2a::draw_signs_2a(record) }?;
    }
    if f2 & 0x80 != 0 {
        unsafe { draw_signs_2b::draw_signs_2b(record) }?;
    }
    HUD.texture.set(0xC1);
    if f1 & 1 != 0 {
        unsafe { dials::needle(record, &needle_2a::SETUP) }?;
    }
    if f2 & 1 != 0 {
        unsafe { dials::needle(record, &needle_2b::SETUP) }?;
    }
    let xs = [0x23, 0x37, 0x4B, 0x53, 0x67, 0x7B, 0x83, 0x97];
    let laps = [
        (f1, 0x1C, [4, 4], 0, [3, 1, 4, 0x26], &HUD.times[2], 0x31),
        (
            f2,
            0x4C,
            [lower(0x142), lower(0x142)],
            1,
            [3, lower(0x13F), 4, lower(0x164)],
            &HUD.times[5],
            lower(0x16C),
        ),
    ];
    for (flags, field, last, mark, sprites, time, y) in laps {
        if flags & 2 == 0 {
            continue;
        }
        let v = get(field);
        if v != 0xFFFF_FFFF {
            let d = time_digits(v, 2);
            sign(v, 0, 1);
            plain(d[4], 0x55, last[0])?;
            plain(d[5], 0x81, last[1])?;
        }
        plain(mark, sprites[0], sprites[1])?;
        plain(0x4A, sprites[2], sprites[3])?;
        let t = time.get();
        unsafe { lap_panel(t, xs, y, &mut plain) }?;
    }
    for (flags, field, y) in [(f1, 0x14, upper(0x107)), (f2, 0x44, 0x245)] {
        if flags & 0x20 != 0 {
            unsafe { leading_time(get(field), [0x43, 0x57, 0x6B], y, &mut plain) }?;
        }
    }
    let left = [8, 0x16, 0x23, 0x2A, 0x38, 0x45, 0x4C, 0x5A];
    let right = [0x2B7, 0x2C5, 0x2D2, 0x2D9, 0x2E7, 0x2F4, 0x2FB, 0x309];
    for (times, y) in [
        ([&HUD.times[0], &HUD.times[1]], 0x120u32),
        ([&HUD.times[3], &HUD.times[4]], 0x12F),
    ] {
        if f1 & 0x40 != 0 {
            for (time, xs) in [(times[0], left), (times[1], right)] {
                let t = time.get();
                unsafe { clock_panel(t, xs, y, &mut plain) }?;
            }
        }
    }
    Ok(())
}
