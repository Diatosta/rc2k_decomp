//! The other cars of a network race on the HUD's map disc: a small marker for each, in the colour
//! its marker on the stage bar has, where the car lies on the map round the first view's car (the
//! disc's centre). The disc's picture is turned with the car's heading, so a marker is carried
//! round by the disc's turn (`rc2k_core::hud::disc::map_marker`); one that falls outside the disc is
//! not drawn. Nothing outside a network race.

use crate::errors::Refused;
use crate::hud::owner::{Hud, Pen};
use crate::hud::store::RectAt;
use crate::race::owner_frame::Frame;

/// The sprite sheet's small round marker (14 by 14).
const RECT: u32 = 0x5D;

/// The markers, after the disc is drawn (its turn and size are the disc's own).
pub fn draw_map_cars(frame: &Frame, hud: &Hud, pen: Pen) -> Result<(), Refused> {
    let panel = &frame.panel_and_overlay.get().panel;
    let shown = panel.disc_cars.get();
    if shown == 0 {
        return Ok(());
    }
    let turn = crate::hud::draw_disc::texture_turn(hud);
    let size = hud.block.disc_size.get();
    let centre = [
        hud.block.disc_centre[0].get(),
        hud.block.disc_centre[1].get(),
    ];
    let [_, _, width, height] = RectAt::number(RECT).read(hud)?;
    let reach = size - width.max(height) as f32 / 2.0;
    let kept = hud.block.quad.colour.get();
    for n in (0..panel.disc_offsets.len()).filter(|&n| shown >> n & 1 != 0) {
        let offset = [
            panel.disc_offsets[n][0].get(),
            panel.disc_offsets[n][1].get(),
        ];
        let [dx, dy] =
            rc2k_core::hud::disc::map_marker(offset, turn, hud.block.disc_texture_scale.get());
        if dx * dx + dy * dy > reach * reach {
            continue;
        }
        let colour = 1 + n % (hud.block.marker_colours.len() - 1);
        hud.block
            .quad
            .colour
            .set(hud.block.marker_colours[colour].get());
        let left = (centre[0] + dx - width as f32 / 2.0).round() as u32;
        let top = (centre[1] + dy - height as f32 / 2.0).round() as u32;
        crate::hud::add_sprite::add_sprite(frame, hud, pen, left, top, RectAt::number(RECT))?;
    }
    hud.block.quad.colour.set(kept);
    Ok(())
}
