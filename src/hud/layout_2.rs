//! `hud::layout_2` (0x4B3B64): the HUD of layout 2, two players one above the other (by
//! `Panel::hud_flags`, the upper's and the lower's): the dials (`dials::DISC_2A` / `_2B`), a black
//! divider at 0x12B, the stage bars (`hud::draw_stage_bars`), the cut calls (0x4B6A8E / 0x4B6A99),
//! the icons (`hud::icons`), the scaled sprites, the markers (`hud::draw_markers`), signs
//! (`draw_signs::LAYOUT_2A` / `_2B`), needles (`dials::NEEDLE_2A` / `_2B`), the lap panels
//! (`ViewA::place` / `ViewB::place`, the HUD's `times` 2 / 5), the speeds with their leading digits
//! left out and the four clocks. The upper's rows move 0x10 down and the lower's 0x10 up with
//! `Panel::flat` 0. Both needles' signs read the second start value (`Panel::start_values`; layout
//! 1 reads the first for the first: section 16).

use crate::errors::Refused;
use crate::hud::draw_signs::row_by;
use crate::hud::layout_common::{
    begin_dials, black_divider, clock_panel, gear_rect, lap_panel, leading_time, needle_sign,
    scaled, sign, sprite, time_digits, white_opaque,
};
use crate::hud::owner::Hud;
use crate::hud::store::texture;
use crate::race::owner_frame::Frame;

/// The upper half's rows: 0x10 down with `Panel::flat` 0.
fn upper(frame: &Frame, y: u32) -> u32 {
    row_by(frame, y, 0xFFFF_FFF0)
}

/// The lower half's rows: 0x10 up with `Panel::flat` 0.
fn lower(frame: &Frame, y: u32) -> u32 {
    row_by(frame, y, 0x10)
}

/// The HUD of two players one above the other.
// Converted: `rc2k_core::hud::panels` (through `layout_0`'s panels and `needle_sign`),
// `rc2k_core::hud::quads::int_over` (the sprites' scales)
pub fn layout_2(frame: &Frame, hud: &Hud) -> Result<(), Refused> {
    use crate::hud::*;
    let panel = &frame.panel_and_overlay.panel;
    let [f1, f2] = [panel.hud_flags[0].get(), panel.hud_flags[1].get()];
    begin_dials(hud);
    if f1 & 0x10 != 0 {
        dials::disc(frame, hud, &dials::DISC_2A)?;
    }
    hud.block.texture.set(texture::DIAL_SECOND);
    if f2 & 0x10 != 0 {
        dials::disc(frame, hud, &dials::DISC_2B)?;
    }
    black_divider(frame, hud, [0, 0x12B, 0x320, 0x12D])?;
    if f1 & 0x40 != 0 {
        draw_stage_bars::draw_stage_bars(frame, hud, draw_stage_bars::STAGE_BARS_DROP_2)?;
    }
    white_opaque(hud, texture::ICONS);
    icons::icons(frame, hud, &icons::LAYOUT_2)?;
    white_opaque(hud, texture::NEEDLES_AND_DIGITS);
    if f1 & 0x20 != 0 {
        scaled(frame, hud, 0x49, 7, upper(frame, 0xE8))?;
        scaled(
            frame,
            hud,
            gear_rect(frame.panel_and_overlay.panel.view_a.gear.get()),
            0xD,
            upper(frame, 0xEF),
        )?;
    }
    if f2 & 0x20 != 0 {
        scaled(frame, hud, 0x49, 7, 0x223)?;
        scaled(
            frame,
            hud,
            gear_rect(frame.panel_and_overlay.panel.view_b.gear.get()),
            0xD,
            0x22A,
        )?;
    }
    let p = needle_sign(hud, panel.start_values[1].get());
    if f1 & 1 != 0 {
        scaled(frame, hud, p, 0x19, upper(frame, 0xA3))?;
    }
    hud.block.texture.set(texture::SPRITES);
    if f1 & 0x10 != 0 {
        scaled(frame, hud, 0x5F, 0x2A7, upper(frame, 0xA3))?;
    }
    let p = needle_sign(hud, panel.start_values[1].get());
    if f2 & 1 != 0 {
        scaled(frame, hud, p, 0x19, 0x1DE)?;
    }
    hud.block.texture.set(texture::SPRITES);
    if f2 & 0x10 != 0 {
        scaled(frame, hud, 0x5F, 0x2A7, 0x1DE)?;
    }
    if f1 & 0x40 != 0 {
        draw_markers::draw_markers(frame, hud, draw_markers::MARKER_ROWS_2)?;
    }
    hud.block.quad.colour.set(0xFFFF_FFFF);
    if f1 & 0x80 != 0 {
        draw_signs::draw_signs(frame, hud, &draw_signs::LAYOUT_2A)?;
    }
    if f2 & 0x80 != 0 {
        draw_signs::draw_signs(frame, hud, &draw_signs::LAYOUT_2B)?;
    }
    hud.block.texture.set(texture::NEEDLES_AND_DIGITS);
    if f1 & 1 != 0 {
        dials::needle(frame, hud, &dials::NEEDLE_2A)?;
    }
    if f2 & 1 != 0 {
        dials::needle(frame, hud, &dials::NEEDLE_2B)?;
    }
    let xs = [0x23, 0x37, 0x4B, 0x53, 0x67, 0x7B, 0x83, 0x97];
    let laps = [
        (
            f1,
            &panel.view_a.place,
            [4, 4],
            0,
            [3, 1, 4, 0x26],
            &hud.block.times[2],
            0x31,
        ),
        (
            f2,
            &panel.view_b.place,
            [lower(frame, 0x142), lower(frame, 0x142)],
            1,
            [3, lower(frame, 0x13F), 4, lower(frame, 0x164)],
            &hud.block.times[5],
            lower(frame, 0x16C),
        ),
    ];
    for (flags, field, last, mark, sprites, time, y) in laps {
        if flags & 2 == 0 {
            continue;
        }
        let v = field.get();
        if v != 0xFFFF_FFFF {
            let d = time_digits(hud, v, 2);
            sign(hud, v, 0, 1);
            sprite(frame, hud, d[4], 0x55, last[0])?;
            sprite(frame, hud, d[5], 0x81, last[1])?;
        }
        sprite(frame, hud, mark, sprites[0], sprites[1])?;
        sprite(frame, hud, 0x4A, sprites[2], sprites[3])?;
        let t = time.get();
        lap_panel(hud, t, xs, y, &mut |r, x, y| sprite(frame, hud, r, x, y))?;
    }
    for (flags, field, y) in [
        (f1, &panel.view_a.speed, upper(frame, 0x107)),
        (f2, &panel.view_b.speed, 0x245),
    ] {
        if flags & 0x20 != 0 {
            leading_time(hud, field.get(), [0x43, 0x57, 0x6B], y, &mut |r, x, y| {
                sprite(frame, hud, r, x, y)
            })?;
        }
    }
    let left = [8, 0x16, 0x23, 0x2A, 0x38, 0x45, 0x4C, 0x5A];
    let right = [0x2B7, 0x2C5, 0x2D2, 0x2D9, 0x2E7, 0x2F4, 0x2FB, 0x309];
    for (times, y) in [
        ([&hud.block.times[0], &hud.block.times[1]], 0x120u32),
        ([&hud.block.times[3], &hud.block.times[4]], 0x12F),
    ] {
        if f1 & 0x40 != 0 {
            for (time, xs) in [(times[0], left), (times[1], right)] {
                let t = time.get();
                clock_panel(hud, t, xs, y, &mut |r, x, y| sprite(frame, hud, r, x, y))?;
            }
        }
    }
    Ok(())
}
