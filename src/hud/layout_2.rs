//! The HUD of layout 2, two players one above the other: as layout 1, with a black divider at
//! 0x12B; the upper's rows move 0x10 down and the lower's 0x10 up with `Panel::flat` 0. Each
//! player's needle sign follows that player's own start value.

use crate::errors::Refused;
use crate::hud::block_store::texture;
use crate::hud::draw_signs::row_by;
use crate::hud::layout_common::{
    Keep, begin_dials, black_divider, clock_panel, gear_rect, keep, keep_layout, lap_panel,
    leading_time, needle_sign, scaled, sign, sprite, time_digits, white_opaque,
};
use crate::hud::owner::Hud;
use crate::hud::pen::Pen;
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
pub(super) fn layout_2(frame: &Frame, hud: &mut Hud, pen: Pen) -> Result<(), Refused> {
    use crate::hud::*;
    let panel = *frame.panel.borrow();
    let [f1, f2] = [panel.hud_flags[0], panel.hud_flags[1]];
    begin_dials(hud);
    if f1 & 0x10 != 0 {
        dials::disc(frame, hud, pen, &dials::DISC_2A)?;
    }
    hud.block.texture = texture::DIAL_SECOND;
    if f2 & 0x10 != 0 {
        dials::disc(frame, hud, pen, &dials::DISC_2B)?;
    }
    keep(hud, Keep::Whole);
    black_divider(frame, hud, pen, [0, 0x12B, 0x320, 0x12D])?;
    keep(hud, Keep::Bar);
    if f1 & 0x40 != 0 {
        draw_stage_bars::draw_stage_bars(frame, hud, pen, draw_stage_bars::STAGE_BARS_DROP_2)?;
    }
    keep_layout(hud);
    white_opaque(hud, texture::ICONS);
    icons::icons(frame, hud, pen, &icons::LAYOUT_2)?;
    white_opaque(hud, texture::NEEDLES_AND_DIGITS);
    if f1 & 0x20 != 0 {
        scaled(frame, hud, pen, 0x49, 7, upper(frame, 0xE8))?;
        let gear = gear_rect(hud, panel.view_a.gear);
        scaled(frame, hud, pen, gear, 0xD, upper(frame, 0xEF))?;
    }
    if f2 & 0x20 != 0 {
        scaled(frame, hud, pen, 0x49, 7, 0x223)?;
        let gear = gear_rect(hud, panel.view_b.gear);
        scaled(frame, hud, pen, gear, 0xD, 0x22A)?;
    }
    let p = needle_sign(hud, panel.start_values[0]);
    if f1 & 1 != 0 {
        scaled(frame, hud, pen, p, 0x19, upper(frame, 0xA3))?;
    }
    hud.block.texture = texture::SPRITES;
    if f1 & 0x10 != 0 {
        dials::map_cars(frame, hud, pen, &dials::DISC_2A, 0)?;
        scaled(frame, hud, pen, 0x5F, 0x2A7, upper(frame, 0xA3))?;
    }
    let p = needle_sign(hud, panel.start_values[1]);
    if f2 & 1 != 0 {
        scaled(frame, hud, pen, p, 0x19, 0x1DE)?;
    }
    hud.block.texture = texture::SPRITES;
    if f2 & 0x10 != 0 {
        dials::map_cars(frame, hud, pen, &dials::DISC_2B, 1)?;
        scaled(frame, hud, pen, 0x5F, 0x2A7, 0x1DE)?;
    }
    if f1 & 0x40 != 0 {
        keep(hud, Keep::BarMarker);
        draw_markers::draw_markers(frame, hud, pen, draw_markers::MARKER_ROWS_2)?;
        keep_layout(hud);
    }
    hud.block.quad.colour = 0xFFFF_FFFF;
    if f1 & 0x80 != 0 {
        draw_signs::draw_signs(frame, hud, pen, &draw_signs::LAYOUT_2A)?;
    }
    if f2 & 0x80 != 0 {
        draw_signs::draw_signs(frame, hud, pen, &draw_signs::LAYOUT_2B)?;
    }
    hud.block.texture = texture::NEEDLES_AND_DIGITS;
    if f1 & 1 != 0 {
        dials::needle(frame, hud, pen, &dials::NEEDLE_2A)?;
    }
    if f2 & 1 != 0 {
        dials::needle(frame, hud, pen, &dials::NEEDLE_2B)?;
    }
    let xs = [0x23, 0x37, 0x4B, 0x53, 0x67, 0x7B, 0x83, 0x97];
    let laps = [
        (
            f1,
            panel.view_a.place,
            [4, 4],
            0,
            [3, 1, 4, 0x26],
            hud.block.times[2],
            0x31,
        ),
        (
            f2,
            panel.view_b.place,
            [lower(frame, 0x142), lower(frame, 0x142)],
            1,
            [3, lower(frame, 0x13F), 4, lower(frame, 0x164)],
            hud.block.times[5],
            lower(frame, 0x16C),
        ),
    ];
    for (flags, field, last, mark, sprites, time, y) in laps {
        if flags & 2 == 0 {
            continue;
        }
        let v = field;
        if v != 0xFFFF_FFFF {
            let d = time_digits(hud, v, 2);
            sign(hud, v, 0, 1);
            sprite(frame, hud, pen, d[4], 0x55, last[0])?;
            sprite(frame, hud, pen, d[5], 0x81, last[1])?;
        }
        sprite(frame, hud, pen, mark, sprites[0], sprites[1])?;
        sprite(frame, hud, pen, 0x4A, sprites[2], sprites[3])?;
        let t = time;
        lap_panel(frame, hud, pen, t, xs, y)?;
    }
    for (flags, field, y) in [
        (f1, panel.view_a.speed, upper(frame, 0x107)),
        (f2, panel.view_b.speed, 0x245),
    ] {
        if flags & 0x20 != 0 {
            leading_time(frame, hud, pen, field, [0x43, 0x57, 0x6B], y)?;
        }
    }
    let left = [8, 0x16, 0x23, 0x2A, 0x38, 0x45, 0x4C, 0x5A];
    let right = [0x2B7, 0x2C5, 0x2D2, 0x2D9, 0x2E7, 0x2F4, 0x2FB, 0x309];
    keep(hud, Keep::Bar);
    for (times, y) in [
        ([hud.block.times[0], hud.block.times[1]], 0x120u32),
        ([hud.block.times[3], hud.block.times[4]], 0x12F),
    ] {
        if f1 & 0x40 != 0 {
            for (time, xs) in [(times[0], left), (times[1], right)] {
                let t = time;
                clock_panel(frame, hud, pen, t, xs, y)?;
            }
        }
    }
    keep_layout(hud);
    Ok(())
}
