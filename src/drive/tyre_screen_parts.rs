//! The tyre-change screen's fixed parts: the title's one or two quads (from the title's own
//! texture, `race::draw_tyre_change`) and nine more
//! (`drive::tyre_quad`, place 2) from texture rectangles set in `texture_rect` and screen
//! rectangles in `screen_rect`, placed from the layout (`TyreChange::parts`, `wheel`, the car's
//! half by `player`).

use crate::errors::Refused;
use crate::memory::session::PlainDword;
use crate::race::owner::Race;
use rc2k_core::drive::tyre_screen::{fixed_parts, prompt_parts, title_parts};

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
    let half = u32::from(shared.tyre_change().player.get() != 0);
    let prompt_width = shared.tyre_change().prompt_width[half as usize].get();
    let mut quads = fixed_parts(parts, wheel, half != 0).to_vec();
    // The prompt, when it is the one shown, is on the title texture in place of the three strips
    // of the tyre change texture.
    let prompt = if prompt_width > 0 {
        quads.drain(2..5);
        prompt_parts(parts, prompt_width, half)
    } else {
        Vec::new()
    };
    let mut title_quads = title.len() + prompt.len();
    for (n, (uv, rect)) in title.into_iter().chain(prompt).chain(quads).enumerate() {
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
