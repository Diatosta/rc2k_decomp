//! The HUD of layout 1, two players side by side (by `Panel::hud_flags`, the first's and the
//! second's): the dials, a black divider, the stage bars, the icons, each player's scaled
//! sprites, markers, signs, needles, speed with its leading digits left out, the four clocks and
//! each player's lap panel. Textures, alpha and colour set between.

use crate::errors::Refused;
use crate::hud::draw_signs::row;
use crate::hud::layout_common::{
    Keep, begin_dials, black_divider, clock_panel, gear_rect, keep, keep_layout, lap_panel,
    leading_time, needle_sign, scaled, sign, sprite, time_digits, white_opaque,
};
use crate::hud::owner::{Hud, Pen};
use crate::hud::store::texture;
use crate::race::owner_frame::Frame;

/// The HUD of two players side by side.
pub fn layout_1(frame: &Frame, hud: &Hud, pen: Pen) -> Result<(), Refused> {
    use crate::hud::*;
    let panel = &frame.panel_and_overlay.get().panel;
    let [f1, f2] = [panel.hud_flags[0].get(), panel.hud_flags[1].get()];
    begin_dials(hud);
    if f1 & 0x10 != 0 {
        dials::disc(frame, hud, pen, &dials::DISC_1A)?;
    }
    hud.block.texture.set(texture::DIAL_SECOND);
    if f2 & 0x10 != 0 {
        dials::disc(frame, hud, pen, &dials::DISC_1B)?;
    }
    keep(hud, Keep::Middle);
    black_divider(frame, hud, pen, [0x18F, 0, 0x191, 0x258])?;
    if f1 & 0x40 != 0 {
        draw_stage_bars::draw_stage_bars(frame, hud, pen, 0)?;
    }
    keep_layout(hud);
    white_opaque(hud, texture::ICONS);
    icons::icons(frame, hud, pen, &icons::LAYOUT_1)?;
    white_opaque(hud, texture::NEEDLES_AND_DIGITS);
    if f1 & 0x20 != 0 {
        scaled(frame, hud, pen, 0x49, 7, 0x223)?;
        scaled(
            frame,
            hud,
            pen,
            gear_rect(frame.panel_and_overlay.get().panel.view_a.gear.get()),
            0xD,
            0x22A,
        )?;
    }
    if f2 & 0x20 != 0 {
        scaled(frame, hud, pen, 0x49, 0x198, 0x223)?;
        scaled(
            frame,
            hud,
            pen,
            gear_rect(frame.panel_and_overlay.get().panel.view_b.gear.get()),
            0x19E,
            0x22A,
        )?;
    }
    let p = needle_sign(hud, panel.start_values[0].get());
    if f1 & 1 != 0 {
        scaled(frame, hud, pen, p, 0x19, 0x1DE)?;
    }
    hud.block.texture.set(texture::SPRITES);
    if f1 & 0x10 != 0 {
        scaled(frame, hud, pen, 0x5F, 0x116, 0x1DE)?;
    }
    let p = needle_sign(hud, panel.start_values[1].get());
    if f2 & 1 != 0 {
        scaled(frame, hud, pen, p, 0x1AA, 0x1DE)?;
    }
    hud.block.texture.set(texture::SPRITES);
    if f2 & 0x10 != 0 {
        scaled(frame, hud, pen, 0x5F, 0x2A7, 0x1DE)?;
    }
    if f1 & 0x40 != 0 {
        keep(hud, Keep::Middle);
        draw_markers::draw_markers(frame, hud, pen, draw_markers::MARKER_ROWS_1)?;
        keep_layout(hud);
    }
    hud.block.quad.colour.set(0xFFFF_FFFF);
    if f1 & 0x80 != 0 {
        draw_signs::draw_signs(frame, hud, pen, &draw_signs::LAYOUT_1A)?;
    }
    if f2 & 0x80 != 0 {
        draw_signs::draw_signs(frame, hud, pen, &draw_signs::LAYOUT_1B)?;
    }
    hud.block.texture.set(texture::NEEDLES_AND_DIGITS);
    if f1 & 1 != 0 {
        dials::needle(frame, hud, pen, &dials::NEEDLE_1A)?;
    }
    if f2 & 1 != 0 {
        dials::needle(frame, hud, pen, &dials::NEEDLE_1B)?;
    }
    for (flags, field, xs) in [
        (f1, &panel.view_a.speed, [0x43, 0x57, 0x6B]),
        (f2, &panel.view_b.speed, [0x1D4, 0x1E8, 0x1FC]),
    ] {
        if flags & 0x20 != 0 {
            leading_time(hud, field.get(), xs, 0x245, &mut |r, x, y| {
                sprite(frame, hud, pen, r, x, y)
            })?;
        }
    }
    let left = [8, 0x16, 0x23, 0x2A, 0x38, 0x45, 0x4C, 0x5A];
    let right = [0x2B7, 0x2C5, 0x2D2, 0x2D9, 0x2E7, 0x2F4, 0x2FB, 0x309];
    keep(hud, Keep::Middle);
    for (times, y) in [
        ([&hud.block.times[0], &hud.block.times[1]], 2u32),
        ([&hud.block.times[3], &hud.block.times[4]], 0x11),
    ] {
        if f1 & 0x40 != 0 && panel.view_a.back.get() != 0xFFFF_FFFF {
            for (time, xs) in [(times[0], left), (times[1], right)] {
                let t = time.get();
                clock_panel(hud, t, xs, y, &mut |r, x, y| {
                    sprite(frame, hud, pen, r, x, y)
                })?;
            }
        }
    }
    keep_layout(hud);
    let laps = [
        (
            f1,
            &panel.view_a.place,
            [0x55, 0x81],
            0u32,
            [3, 0x21, 4, 0x46],
            &hud.block.times[2],
            [0x23, 0x37, 0x4B, 0x53, 0x67, 0x7B, 0x83, 0x97],
        ),
        (
            f2,
            &panel.view_b.place,
            [0x1E6, 0x212],
            1,
            [0x194, 0x21, 0x195, 0x46],
            &hud.block.times[5],
            [0x1B4, 0x1C8, 0x1DC, 0x1E4, 0x1F8, 0x20C, 0x214, 0x228],
        ),
    ];
    for (flags, field, last, mark, sprites, time, xs) in laps {
        if flags & 2 == 0 {
            continue;
        }
        let v = field.get();
        if v != 0xFFFF_FFFF {
            let d = time_digits(hud, v, 2);
            sign(hud, v, 0, 1);
            sprite(frame, hud, pen, d[4], last[0], row(frame, 0x24))?;
            sprite(frame, hud, pen, d[5], last[1], row(frame, 0x24))?;
        }
        sprite(frame, hud, pen, mark, sprites[0], row(frame, sprites[1]))?;
        sprite(frame, hud, pen, 0x4A, sprites[2], row(frame, sprites[3]))?;
        let t = time.get();
        lap_panel(hud, t, xs, row(frame, 0x51), &mut |r, x, y| {
            sprite(frame, hud, pen, r, x, y)
        })?;
    }
    Ok(())
}
