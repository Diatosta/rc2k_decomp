//! The other cars on the HUD's map disc: in a network race and in a split screen a small marker for
//! each, in the colour its marker on the stage bar has, where the car lies on the map round the
//! disc's own view's car (the disc's centre). The disc's picture is turned with the car's heading, so a marker is carried
//! round by the disc's turn (`rc2k_core::hud::disc::map_marker`); one that falls outside the disc is
//! not drawn. Nothing in a one-player race outside a network race.

use crate::errors::Refused;
use crate::hud::block_store::RectAt;
use crate::hud::owner::Hud;
use crate::hud::pen::Pen;
use crate::race::owner_frame::Frame;

/// The sprite sheet's small round marker (14 by 14).
pub(crate) const RECT: u32 = 0x5D;

/// The markers of the disc of view `slot` (0 to 3), after the disc is set up (its turn and size are
/// the disc's own). With `scaled` the markers shrink as the two players' layouts' sprites do.
pub(super) fn draw_map_cars(
    frame: &Frame,
    hud: &mut Hud,
    pen: Pen,
    slot: usize,
    scaled: bool,
) -> Result<(), Refused> {
    let panel = &frame.panel_and_overlay.get().panel;
    let (cars, offsets) = if slot == 0 {
        (&panel.disc_cars, &panel.disc_offsets)
    } else {
        (
            &panel.disc_cars_more[slot - 1],
            &panel.disc_offsets_more[slot - 1],
        )
    };
    let shown = cars.get();
    if shown == 0 {
        return Ok(());
    }
    let turn = crate::hud::draw_disc::texture_turn(hud);
    let size = hud.block.disc_size;
    let centre = [hud.block.disc_centre[0], hud.block.disc_centre[1]];
    let [_, _, width, height] = RectAt::number(RECT).read(hud)?;
    let k = hud.block.uv_scale_x;
    let scale = crate::hud::draw_markers::int_over_80(0xC0, k);
    let factor = if scaled {
        rc2k_core::math::extended::to_f64(scale)
    } else {
        1.0
    };
    let (width, height) = (
        rc2k_core::hud::quads::scaled_size(width, factor),
        rc2k_core::hud::quads::scaled_size(height, factor),
    );
    let reach = size - width.max(height) as f32 / 2.0;
    let network = panel.network_cars.get() > 0;
    let kept = hud.block.quad.colour;
    for n in (0..offsets.len()).filter(|&n| shown >> n & 1 != 0) {
        let offset = [offsets[n][0].get(), offsets[n][1].get()];
        let [dx, dy] = rc2k_core::hud::disc::map_marker(offset, turn, hud.block.disc_texture_scale);
        if dx * dx + dy * dy > reach * reach {
            continue;
        }
        // The colours of the markers on the stage bar: a network race's after the first, a split
        // screen's each player's own.
        let colour = if network {
            1 + n % (hud.block.marker_colours.len() - 1)
        } else {
            n % hud.block.marker_colours.len()
        };
        hud.block.quad.colour = hud.block.marker_colours[colour];
        let left = (centre[0] + dx - width as f32 / 2.0).round() as u32;
        let top = (centre[1] + dy - height as f32 / 2.0).round() as u32;
        if scaled {
            crate::hud::add_scaled_sprite::add_scaled_sprite(
                frame,
                hud,
                pen,
                left,
                top,
                RectAt::number(RECT),
                [scale, scale],
            )?;
        } else {
            crate::hud::add_sprite::add_sprite(frame, hud, pen, left, top, RectAt::number(RECT))?;
        }
    }
    hud.block.quad.colour = kept;
    Ok(())
}
