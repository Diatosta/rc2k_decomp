//! The tyre-change screen's fixed parts: the title's one or two quads (from the title's own
//! texture, `race::draw_tyre_change`) and nine more
//! (`drive::tyre_quad`, place 2) from texture rectangles set in `texture_rect` and screen
//! rectangles in `screen_rect`, placed from the layout (`TyreChange::parts`, `wheel`, the car's
//! half by `player`).

use crate::errors::Refused;
use crate::memory::session::PlainDword;
use crate::race::owner::Race;
use rc2k_core::drive::tyre_screen::{fixed_parts, title_parts};

fn set(cells: &[PlainDword; 4], v: [u32; 4]) {
    for (cell, x) in cells.iter().zip(v) {
        cell.set(x);
    }
}

/// Its quads (`drive::tyre_quad`).
pub fn tyre_screen_parts(
    race: &Race,
    scene: &crate::scene::owner::Scene,
    system: &crate::system::owner::System,
) -> Result<(), Refused> {
    let shared = &race.shared;
    let parts = std::array::from_fn(|k| shared.tyre_change().parts[k].get());
    let wheel = std::array::from_fn(|k| shared.tyre_change().wheel[k].get());
    let part = scene.scratch.game_scratch();
    let title = title_parts(parts, shared.tyre_change().title_width.get());
    let main_texture = shared.tyre_change().device_texture.get();
    crate::drive::tyre_parts::set_texture(part, shared.tyre_change().title_device_texture.get());
    let quads = fixed_parts(parts, wheel, shared.tyre_change().player.get() != 0);
    let mut title_quads = title.len();
    for (n, (uv, rect)) in title.into_iter().chain(quads).enumerate() {
        if n == title_quads {
            // The rest are on the tyre change texture again.
            crate::drive::tyre_parts::set_texture(part, main_texture);
            title_quads = usize::MAX;
        }
        set(&race.viewer_panel().texture_rect, uv); // ViewerAndKinds.viewer_panel.texture_rect[0]
        set(&race.viewer_panel().screen_rect, rect); // ViewerAndKinds.viewer_panel.screen_rect[0]
        {
            crate::drive::tyre_quad::tyre_quad(
                race.viewer_panel()
                    .texture_rect
                    .each_ref()
                    .map(crate::memory::session::PlainDword::get),
                &race.viewer_panel().screen_rect,
                0xFF_FFFF,
                race.viewer_panel(),
                system,
                part,
            )
        }?;
    }
    Ok(())
}
