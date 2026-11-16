//! The HUD of layout 0 (`Panel::players` neither 1 nor 2), by the first view's HUD flags
//! (`Panel::hud_flags`): the dial, the stage bar with the marker and two time panels, the icons,
//! the gear and speed digits, the sign and needle, the signs and the place; each flag's items in
//! the order the code below draws them, with the texture, the quad record's alpha and colour set
//! between.

use crate::errors::Refused;
use crate::hud::draw_signs::row;
use crate::hud::layout_common::{
    begin_dials, clock_panel, gear_rect, lap_panel, leading_time, needle_sign, sign, sprite,
    time_digits, white_opaque,
};
use crate::hud::owner::{Hud, Pen};
use crate::hud::store::texture;
use crate::race::owner_frame::Frame;
use rc2k_core::hud::panels;

/// The HUD of one player.
pub fn layout_0(frame: &Frame, hud: &Hud, pen: Pen) -> Result<(), Refused> {
    use crate::hud::*;
    let panel = &frame.panel_and_overlay.get().panel;
    let flags = panel.hud_flags[0].get();
    begin_dials(hud);
    if flags & 0x10 != 0 {
        dials::disc(frame, hud, pen, &dials::DISC_0)?;
    }
    hud.block.texture.set(texture::SPRITES);
    if flags & 0x40 != 0 {
        draw_stage_bar::draw_stage_bar(frame, hud, pen)?;
    }
    white_opaque(hud, texture::ICONS);
    icons::icons(frame, hud, pen, &icons::LAYOUT_0)?;
    white_opaque(hud, texture::NEEDLES_AND_DIGITS);
    if flags & 0x20 != 0 {
        sprite(frame, hud, pen, 0x49, 9, 0x215)?;
        sprite(
            frame,
            hud,
            pen,
            gear_rect(frame.panel_and_overlay.get().panel.view_a.gear.get()),
            0x11,
            0x21D,
        )?;
    }
    let pick = needle_sign(hud, panel.start_values[0].get());
    if flags & 1 != 0 {
        sprite(frame, hud, pen, pick, 0x21, 0x1B8)?;
    }
    hud.block.texture.set(texture::SPRITES);
    if flags & 0x10 != 0 {
        sprite(frame, hud, pen, 0x5F, 0x282, 0x1B8)?;
    }
    if flags & 0x40 != 0 {
        draw_marker::draw_marker(frame, hud, pen)?;
    }
    hud.block.quad.colour.set(0xFFFF_FFFF);
    if flags & 0x80 != 0 {
        draw_signs::draw_signs(frame, hud, pen, &draw_signs::LAYOUT_0)?;
    }
    hud.block.texture.set(texture::NEEDLES_AND_DIGITS);
    if flags & 1 != 0 {
        dials::needle(frame, hud, pen, &dials::NEEDLE_0)?;
    }
    if flags & 0x40 != 0 {
        let panels = [
            (
                &panel.view_a.back,
                0x4Cu32,
                &hud.block.times[0],
                [0x14u32, 0x22, 0x2F, 0x36, 0x44, 0x51, 0x58, 0x66],
            ),
            (
                &panel.view_a.front,
                0x2F8,
                &hud.block.times[1],
                [0x2BD, 0x2CB, 0x2D8, 0x2DF, 0x2ED, 0x2FA, 0x301, 0x30F],
            ),
        ];
        for (field, x, time, xs) in panels {
            let Some(rect) = panels::stage_rect(field.get()) else {
                continue;
            };
            sprite(frame, hud, pen, rect, x, 1)?;
            let t = time.get();
            clock_panel(hud, t, xs, 0x13, &mut |r, x, y| {
                sprite(frame, hud, pen, r, x, y)
            })?;
        }
    }
    if flags & 2 != 0 {
        let v = panel.view_a.place.get();
        if v != 0xFFFF_FFFF {
            let d = time_digits(hud, v, 2);
            sign(hud, v, 0, 1);
            sprite(frame, hud, pen, d[4], 0x60, row(frame, 0x36))?;
            sprite(frame, hud, pen, d[5], 0x8C, row(frame, 0x36))?;
        }
        sprite(frame, hud, pen, 0, 0xE, row(frame, 0x33))?;
        sprite(frame, hud, pen, 0x4A, 0xF, row(frame, 0x58))?;
        let t = hud.block.times[2].get();
        let xs = [0x2E, 0x42, 0x56, 0x5E, 0x72, 0x86, 0x8E, 0xA2];
        lap_panel(hud, t, xs, row(frame, 0x63), &mut |r, x, y| {
            sprite(frame, hud, pen, r, x, y)
        })?;
    }
    if flags & 0x20 != 0 {
        let v = panel.view_a.speed.get();
        leading_time(hud, v, [0x43, 0x57, 0x6B], 0x23F, &mut |r, x, y| {
            sprite(frame, hud, pen, r, x, y)
        })?;
    }
    Ok(())
}
