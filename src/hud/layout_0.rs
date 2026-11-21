//! The HUD of layout 0 (`Panel::players` neither 1 nor 2), by the first view's HUD flags
//! (`Panel::hud_flags`): the dial, the stage bar with the marker and two time panels, the icons,
//! the gear and speed digits, the sign and needle, the signs and the place; each flag's items in
//! the order the code below draws them, with the texture, the quad record's alpha and colour set
//! between.

use crate::errors::Refused;
use crate::hud::block_store::texture;
use crate::hud::draw_signs::row;
use crate::hud::layout_common::{
    Keep, begin_dials, clock_panel, gear_rect, keep, keep_layout, lap_panel, leading_time,
    needle_sign, sign, sprite, time_digits, white_opaque,
};
use crate::hud::owner::Hud;
use crate::hud::pen::Pen;
use crate::race::owner_frame::Frame;
use rc2k_core::hud::panels;

/// The car's icon at the corner: the first full-size one, or with three or four players the
/// part's player's half-size one drawn as large (its texture rectangle a quarter of the first's).
fn car_icon(frame: &Frame, hud: &mut Hud, pen: Pen, y: u32) -> Result<(), Refused> {
    let Some(player) = pen.scope.icon else {
        return sprite(frame, hud, pen, 0, 0xE, y);
    };
    let [_, _, width, height] = crate::hud::block_store::RectAt::number(0).read(hud)?;
    let [left, top] = rc2k_core::scene::car_icons::half_place(player);
    let [w, h] = rc2k_core::scene::car_icons::HALF_SIZE;
    hud.block
        .quad
        .set_places(rc2k_core::hud::quads::sprite_places([left, top, w, h]));
    crate::hud::add_sprite::put_corners(hud, 0xE, y, width, height);
    crate::hud::add_quad::add_quad(frame, hud, pen)
}

/// The HUD of one player.
pub(super) fn layout_0(frame: &Frame, hud: &mut Hud, pen: Pen) -> Result<(), Refused> {
    layout_0_on(frame, hud, pen, texture::DIAL_FIRST)
}

/// The HUD of one player whose map picture is the texture `map`.
pub(super) fn layout_0_on(frame: &Frame, hud: &mut Hud, pen: Pen, map: u32) -> Result<(), Refused> {
    use crate::hud::*;
    let panel = *frame.panel.borrow();
    let flags = panel.hud_flags[0];
    begin_dials(hud);
    hud.block.texture = map;
    if flags & 0x10 != 0 {
        dials::disc(frame, hud, pen, &dials::DISC_0)?;
    }
    hud.block.texture = texture::SPRITES;
    // The progress bar runs over the whole width of the screen, whatever the HUD's box: its end
    // caps, and the clocks on them, keep to the edges and its middle stretches.
    if flags & 0x40 != 0 {
        keep(hud, Keep::Bar);
        draw_stage_bar::draw_stage_bar(frame, hud, pen)?;
        keep_layout(hud);
    }
    white_opaque(hud, texture::ICONS);
    icons::icons(frame, hud, pen, &icons::LAYOUT_0)?;
    white_opaque(hud, texture::NEEDLES_AND_DIGITS);
    if flags & 0x20 != 0 {
        sprite(frame, hud, pen, 0x49, 9, 0x215)?;
        let gear = gear_rect(hud, panel.view_a.gear);
        sprite(frame, hud, pen, gear, 0x11, 0x21D)?;
    }
    let pick = needle_sign(hud, panel.start_values[0]);
    if flags & 1 != 0 {
        sprite(frame, hud, pen, pick, 0x21, 0x1B8)?;
    }
    hud.block.texture = texture::SPRITES;
    if flags & 0x10 != 0 {
        // A part of a split screen draws the markers of its own view's disc.
        let slot = pen.scope.icon.map_or(0, |n| n.min(3));
        draw_map_cars::draw_map_cars(frame, hud, pen, slot, false)?;
        sprite(frame, hud, pen, 0x5F, 0x282, 0x1B8)?;
    }
    if flags & 0x40 != 0 {
        keep(hud, Keep::BarMarker);
        draw_marker::draw_other_markers(frame, hud, pen)?;
        draw_marker::draw_marker(frame, hud, pen)?;
        keep_layout(hud);
    }
    hud.block.quad.colour = 0xFFFF_FFFF;
    if flags & 0x80 != 0 {
        draw_signs::draw_signs(frame, hud, pen, &draw_signs::LAYOUT_0)?;
    }
    hud.block.texture = texture::NEEDLES_AND_DIGITS;
    if flags & 1 != 0 {
        dials::needle(frame, hud, pen, &dials::NEEDLE_0)?;
    }
    if flags & 0x40 != 0 {
        keep(hud, Keep::Bar);
        let panels = [
            (
                panel.view_a.back,
                0x4Cu32,
                hud.block.times[0],
                [0x14u32, 0x22, 0x2F, 0x36, 0x44, 0x51, 0x58, 0x66],
            ),
            (
                panel.view_a.front,
                0x2F8,
                hud.block.times[1],
                [0x2BD, 0x2CB, 0x2D8, 0x2DF, 0x2ED, 0x2FA, 0x301, 0x30F],
            ),
        ];
        for (field, x, time, xs) in panels {
            let Some(rect) = panels::stage_rect(field) else {
                continue;
            };
            sprite(frame, hud, pen, rect, x, 1)?;
            let t = time;
            clock_panel(frame, hud, pen, t, xs, 0x13)?;
        }
        keep_layout(hud);
    }
    if flags & 2 != 0 {
        let v = panel.view_a.place;
        if v != 0xFFFF_FFFF {
            let d = time_digits(hud, v, 2);
            sign(hud, v, 0, 1);
            sprite(frame, hud, pen, d[4], 0x60, row(frame, 0x36))?;
            sprite(frame, hud, pen, d[5], 0x8C, row(frame, 0x36))?;
        }
        car_icon(frame, hud, pen, row(frame, 0x33))?;
        sprite(frame, hud, pen, 0x4A, 0xF, row(frame, 0x58))?;
        let t = hud.block.times[2];
        let xs = [0x2E, 0x42, 0x56, 0x5E, 0x72, 0x86, 0x8E, 0xA2];
        lap_panel(frame, hud, pen, t, xs, row(frame, 0x63))?;
    }
    if flags & 0x20 != 0 {
        let v = panel.view_a.speed;
        leading_time(frame, hud, pen, v, [0x43, 0x57, 0x6B], 0x23F)?;
    }
    Ok(())
}
