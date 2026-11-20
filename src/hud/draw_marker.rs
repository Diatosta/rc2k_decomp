//! A marker sprite (rectangle 0x5C) along the top at the share of the panel's `overlay_miles`
//! over its `distance[0]`, times `HudBlock.marker_k[0]` plus `marker_k[1]` (rounded through `fistp_scratch`),
//! coloured `HudBlock.marker_colours[0]`.

use crate::errors::Refused;
use crate::hud::block_store::RectAt;
use crate::hud::owner::Hud;
use crate::hud::pen::Pen;
use crate::race::owner_frame::Frame;
use rc2k_core::hud::quads::marker_x;

const RECT: u32 = 0x5C;

pub fn draw_marker(frame: &Frame, hud: &mut Hud, pen: Pen) -> Result<(), Refused> {
    // A part of a split screen: the part's player's own colour.
    let own = pen
        .scope
        .icon
        .map_or(0, |n| n.min(hud.block.marker_colours.len() - 1));
    hud.block.quad.colour = hud.block.marker_colours[own];
    {
        let f = f32::from_bits;
        let x = marker_x(
            f(frame.panel_and_overlay.get().panel.distance[0].get()),
            f(frame.panel_and_overlay.get().panel.overlay_miles.get()),
            hud.block.marker_k[0],
            hud.block.marker_k[1],
        );
        hud.block.fistp_scratch = x;
        crate::hud::add_sprite::add_sprite(frame, hud, pen, x, 0, RectAt::number(RECT))
    }
}

/// The other cars of a network race on the stage bar, each its own colour (the marker colours
/// after the first), the own car's marker is drawn after them; in a split screen of three or four
/// each part's bar shows the other players, each in their own colour. Nothing in other races.
pub(super) fn draw_other_markers(frame: &Frame, hud: &mut Hud, pen: Pen) -> Result<(), Refused> {
    let panel = &frame.panel_and_overlay.get().panel;
    let split = panel.split_players.get().min(4) as usize;
    let count = if split > 0 {
        split
    } else {
        panel.network_cars.get().min(8) as usize
    };
    if count == 0 {
        return Ok(());
    }
    let own = if split > 0 {
        pen.scope.icon.unwrap_or(0)
    } else {
        panel.network_own.get() as usize
    };
    let f = f32::from_bits;
    for n in (0..count).filter(|&n| n != own) {
        let colour = if split > 0 {
            n
        } else {
            1 + (n % (hud.block.marker_colours.len() - 1))
        };
        hud.block.quad.colour = hud.block.marker_colours[colour];
        let x = marker_x(
            f(panel.car_distances[n].get()),
            f(panel.overlay_miles.get()),
            hud.block.marker_k[0],
            hud.block.marker_k[1],
        );
        hud.block.fistp_scratch = x;
        crate::hud::add_sprite::add_sprite(frame, hud, pen, x, 0, RectAt::number(RECT))?;
    }
    Ok(())
}
