//! The tyre-change screen's fixed parts: ten quads
//! (`drive::tyre_quad`, place 2) from texture rectangles set in `texture_rect` and screen
//! rectangles in `screen_rect`, placed from the layout (`TyreChange::parts`, `wheel`, the car's
//! half by `player`).

use crate::errors::Refused;
use crate::memory::session::PlainDword;
use crate::race::owner::Race;
use rc2k_core::drive::tyre_screen::fixed_parts;

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
    for (uv, rect) in fixed_parts(parts, wheel, shared.tyre_change().player.get() != 0) {
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
                scene.scratch.game_scratch(),
            )
        }?;
    }
    Ok(())
}
