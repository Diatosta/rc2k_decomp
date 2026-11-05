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
//! Caller: `sub_4B1CD0`.

use crate::errors::Refused;
use crate::hud::draw_markers_1::{int_over_80, taken_off};
use crate::hud::draw_signs::row;
use crate::hud::layout_0::*;
use crate::hud::store::{HUD, rect_place};
use crate::pointers::GamePtr;
use crate::race::panel_store::PANEL;

/// A sprite scaled 0xC0 over `flt_75C00C` both ways; the scales then taken off into
/// `dword_75C060`.
pub fn scaled(rect: u32, x: u32, y: u32) -> Result<(), Refused> {
    let at = rect_place(rect);
    let k = HUD.uv_scale_x.bits();
    HUD.scratch.set(0xC0);
    let v = int_over_80(0xC0, k);
    { crate::hud::add_scaled_sprite::add_scaled_sprite(x, y, at, [v, v]) }?;
    HUD.scratch.set(taken_off(v));
    Ok(())
}

/// A sprite: rectangle `rect` (0x10 a rectangle) of `dword_75B940` at (x, y).
pub fn plain(rect: u32, x: u32, y: u32) -> Result<(), Refused> {
    crate::hud::add_sprite::add_sprite(x, y, rect_place(rect))
}

/// The sign of the needle by the record's `field` (`dword_75C040`, `dword_75BFAC`, the
/// texture).
pub fn pick(field: u32) -> u32 {
    needle_sign(field)
}

/// The Rust version (see the header): the HUD of two players side by side, `record` the
/// HUD's record.
// Converted: `rc2k_core::hud::panels` (through `layout_0`'s panels and `needle_sign`),
// `rc2k_core::hud::quads::int_over` (the sprites' scales)
pub fn layout_1(record: GamePtr) -> Result<(), Refused> {
    use crate::hud::*;
    // A field of the record by an offset from a table or the caller, as the original walks it.
    let get = |o: u32| record.offset(o).read::<u32>("a hud record's field");
    let [f1, f2] = [PANEL.hud_flags[0].get(), PANEL.hud_flags[1].get()];
    HUD.quad[0].set(0xE0);
    HUD.texture.set(0xC6);
    if f1 & 0x10 != 0 {
        { dials::disc(record, &disc_1a::SETUP) }?;
    }
    HUD.texture.set(0xC7);
    if f2 & 0x10 != 0 {
        { dials::disc(record, &disc_1b::SETUP) }?;
    }
    HUD.texture.set(0xC0);
    HUD.quad[0].set(0xFF);
    HUD.quad[5].set(0);
    for (n, v) in [0x18F, 0, 0x191, 0x258].into_iter().enumerate() {
        HUD.quad[1 + n].set(v);
    }
    { add_solid_quad::add_solid_quad() }?;
    if f1 & 0x40 != 0 {
        { draw_stage_bars_1::draw_stage_bars_1() }?;
    }
    HUD.quad[5].set(0xFFFF_FFFF);
    HUD.quad[0].set(0xFF);
    HUD.texture.set(0xDC);
    { icons_1::icons_1(record) }?;
    HUD.quad[5].set(0xFFFF_FFFF);
    HUD.quad[0].set(0xFF);
    HUD.texture.set(0xC1);
    if f1 & 0x20 != 0 {
        scaled(0x49, 7, 0x223)?;
        scaled(PANEL.view_a.gear.get().wrapping_add(0x41), 0xD, 0x22A)?;
    }
    if f2 & 0x20 != 0 {
        scaled(0x49, 0x198, 0x223)?;
        scaled(PANEL.view_b.gear.get().wrapping_add(0x41), 0x19E, 0x22A)?;
    }
    let p = pick(PANEL.start_values[0].get());
    if f1 & 1 != 0 {
        scaled(p, 0x19, 0x1DE)?;
    }
    HUD.texture.set(0xC0);
    if f1 & 0x10 != 0 {
        scaled(0x5F, 0x116, 0x1DE)?;
    }
    let p = pick(PANEL.start_values[1].get());
    if f2 & 1 != 0 {
        scaled(p, 0x1AA, 0x1DE)?;
    }
    HUD.texture.set(0xC0);
    if f2 & 0x10 != 0 {
        scaled(0x5F, 0x2A7, 0x1DE)?;
    }
    if f1 & 0x40 != 0 {
        { draw_markers_1::draw_markers_1(record) }?;
    }
    HUD.quad[5].set(0xFFFF_FFFF);
    if f1 & 0x80 != 0 {
        { draw_signs_1a::draw_signs_1a(record) }?;
    }
    if f2 & 0x80 != 0 {
        { draw_signs_1b::draw_signs_1b(record) }?;
    }
    HUD.texture.set(0xC1);
    if f1 & 1 != 0 {
        { dials::needle(record, &needle_1a::SETUP) }?;
    }
    if f2 & 1 != 0 {
        { dials::needle(record, &needle_1b::SETUP) }?;
    }
    for (flags, field, xs) in [
        (f1, 0x14, [0x43, 0x57, 0x6B]),
        (f2, 0x44, [0x1D4, 0x1E8, 0x1FC]),
    ] {
        if flags & 0x20 != 0 {
            { leading_time(get(field)?, xs, 0x245, &mut plain) }?;
        }
    }
    let left = [8, 0x16, 0x23, 0x2A, 0x38, 0x45, 0x4C, 0x5A];
    let right = [0x2B7, 0x2C5, 0x2D2, 0x2D9, 0x2E7, 0x2F4, 0x2FB, 0x309];
    for (times, y) in [
        ([&HUD.times[0], &HUD.times[1]], 2u32),
        ([&HUD.times[3], &HUD.times[4]], 0x11),
    ] {
        if f1 & 0x40 != 0 && PANEL.view_a.back.get() != 0xFFFF_FFFF {
            for (time, xs) in [(times[0], left), (times[1], right)] {
                let t = time.get();
                { clock_panel(t, xs, y, &mut plain) }?;
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
            &HUD.times[2],
            [0x23, 0x37, 0x4B, 0x53, 0x67, 0x7B, 0x83, 0x97],
        ),
        (
            f2,
            0x4C,
            [0x1E6, 0x212],
            1,
            [0x194, 0x21, 0x195, 0x46],
            &HUD.times[5],
            [0x1B4, 0x1C8, 0x1DC, 0x1E4, 0x1F8, 0x20C, 0x214, 0x228],
        ),
    ];
    for (flags, field, last, mark, sprites, time, xs) in laps {
        if flags & 2 == 0 {
            continue;
        }
        let v = get(field)?;
        if v != 0xFFFF_FFFF {
            let d = time_digits(v, 2);
            sign(v, 0, 1);
            plain(d[4], last[0], row(0x24))?;
            plain(d[5], last[1], row(0x24))?;
        }
        plain(mark, sprites[0], row(sprites[1]))?;
        plain(0x4A, sprites[2], row(sprites[3]))?;
        let t = time.get();
        { lap_panel(t, xs, row(0x51), &mut plain) }?;
    }
    Ok(())
}
