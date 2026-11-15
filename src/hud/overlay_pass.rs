//! The race frame's 2D pass: the players (`split`); `scene::depth_buffer_off`;
//! `hud::overlay_states`; with `views` 2, a player whose car block's +0xFA8 is set (`view_covered`)
//! keeps only bit 0x40 of its HUD flags (put back after) and its start lights' icon width 0;
//! `hud::draw_hud`; `scene::depth_buffer_on`; `scene::full_view_window`; render state 3 = 3;
//! `plain_first` and `plain_second` 1; `hud::draw_view_strips`; `scene::depth_buffer_off`;
//! `scene::set_filters`.

use crate::errors::Refused;
use crate::hud::owner::{Hud, Pen};
use crate::race::owner::Race;
use crate::race::owner_shared::RaceShared;

/// The HUD's layout: `split`, or 0 while one view is drawn (`views` 1).
/// `split` alone is 1 / 2 in a replay after a split-screen race while its one view fills the
/// screen (fixed: `wiki/Fixed-bugs.md`, "Split-screen divider left in a replay").
pub fn shown_layout(shared: &RaceShared) -> u32 {
    {
        if shared.view.get().views.get() == 1 {
            0
        } else {
            shared.vars.get().split.get()
        }
    }
}

pub fn overlay_pass(
    hud: &Hud,
    race: &Race,
    system: &crate::system::owner::System,
    scene: &crate::scene::owner::Scene,
    gfx: &crate::gfx::owner::Gfx,
    net: &crate::net::owner::Net,
) -> Result<(), Refused> {
    use crate::scene::depth_buffer_on::depth_buffer;
    race.frame
        .panel_and_overlay
        .get()
        .panel
        .players
        .set(shown_layout(&race.shared));
    let part = crate::workers::game_scratch();
    depth_buffer(part, false);
    crate::hud::overlay_states::overlay_states(part);
    let kept: [u32; 2] = [
        race.frame.panel_and_overlay.get().panel.hud_flags[0].get(),
        race.frame.panel_and_overlay.get().panel.hud_flags[1].get(),
    ];
    if race.shared.view.get().views.get() == 2 {
        for n in 0..2 {
            if crate::race::frame_view::car(n)?.view_covered.get() != 0 {
                race.frame.panel_and_overlay.get().panel.hud_flags[n]
                    .set(race.frame.panel_and_overlay.get().panel.hud_flags[n].get() & 0x40);
                race.frame.panel_and_overlay.get().panel.lights[n]
                    .width
                    .set(0);
            }
        }
    }
    let scratch = crate::race::view_scratch::front_end();
    let polygons = &scratch.polygon;
    let pen = Pen {
        polygons,
        vertices: &part.vertices,
        part,
        system,
        scene,
        engine_tables: race.engine_tables(),
    };
    let done = crate::hud::draw_hud::draw_hud(&race.frame, hud, pen);
    race.frame.panel_and_overlay.get().panel.hud_flags[1].set(kept[1]);
    race.frame.panel_and_overlay.get().panel.hud_flags[0].set(kept[0]);
    done?;
    depth_buffer(part, true);
    crate::scene::full_view_window::full_view_window(race, scene, system);
    crate::scene::set_render_state::set_render_state(part, 3, 3);
    // Read by the next frame's first view (`view_scratch::filter_input`) and the menus.
    polygons.plain_first.set(1);
    polygons.plain_second.set(1);
    crate::hud::draw_view_strips::draw_view_strips(&crate::hud::draw_view_strip::Overlay {
        race,
        scratch,
        part,
        system,
        scene,
        strips: &gfx.strips,
        net,
    })?;
    depth_buffer(part, false);
    crate::scene::set_filters::set_filters(race, part);
    Ok(())
}
