//! `hud::overlay_pass` (0x45A893): the race frame's 2D pass: `players` the players
//! (`split`); `scene::depth_buffer_off`; `hud::overlay_states`; with `views` 2, a
//! player whose car block's +0xFA8 is set (`dword_710394` / `dword_711374`) keeps only bit 0x40 of
//! its HUD flags (`hud_flags` / `dword_6D14FA`, put back after) and its `unk_6D1486` /
//! `unk_6D1496` is 0; `hud::draw_hud`; `scene::depth_buffer_on`; `scene::full_view_window`; render
//! state 3 = 3; `plain_first` and `plain_second` 1; `hud::draw_view_strips`; an empty function
//! (`nullsub_9`); `scene::depth_buffer_off`; `scene::set_filters`.
//!
//! Out: as the last call leaves them
//!
//! Caller: `race::show_frame` (0x454786). a call by address goes through `routes.rs`.

use crate::errors::Refused;
use crate::hud::owner::{Hud, Pen};
use crate::race::owner::Race;
use crate::race::owner_shared::RaceShared;

/// The HUD's layout: `split`, or 0 while one view is drawn (`views` 1).
/// The original reads `split` alone, which a replay after a split-screen race
/// leaves at 1 / 2 while its one view fills the screen: the divider, the second
/// player's bars and strip stayed drawn (section 16, fixed).
pub fn shown_layout(shared: &RaceShared) -> u32 {
    {
        if shared.view.views.get() == 1 {
            0
        } else {
            shared.vars.split.get()
        }
    }
}

pub fn overlay_pass(hud: &Hud, race: &Race) -> Result<(), Refused> {
    use crate::scene::depth_buffer_on::depth_buffer;
    race.frame
        .panel_and_overlay
        .panel
        .players
        .set(shown_layout(&race.shared));
    depth_buffer(false);
    crate::hud::overlay_states::overlay_states();
    let kept: [u32; 2] = [
        race.frame.panel_and_overlay.panel.hud_flags[0].get(),
        race.frame.panel_and_overlay.panel.hud_flags[1].get(),
    ];
    if race.shared.view.views.get() == 2 {
        for n in 0..2 {
            if crate::race::frame_view::car(n)?.view_covered.get() != 0 {
                race.frame.panel_and_overlay.panel.hud_flags[n]
                    .set(race.frame.panel_and_overlay.panel.hud_flags[n].get() & 0x40);
                race.frame.panel_and_overlay.panel.lights[n].width.set(0);
            }
        }
    }
    let scratch = crate::race::view_scratch::front_end();
    let system = crate::system::owner::system();
    let polygons = &scratch.polygon;
    let pen = Pen { polygons, system };
    let done = crate::hud::draw_hud::draw_hud(&race.frame, hud, pen);
    race.frame.panel_and_overlay.panel.hud_flags[1].set(kept[1]);
    race.frame.panel_and_overlay.panel.hud_flags[0].set(kept[0]);
    done?;
    depth_buffer(true);
    crate::scene::full_view_window::full_view_window(race, crate::scene::owner::scene(), system);
    crate::scene::set_render_state::set_render_state(3, 3);
    // Read by the next frame's first view (`view_scratch::filter_input`) and the menus.
    polygons.plain_first.set(1);
    polygons.plain_second.set(1);
    crate::hud::draw_view_strips::draw_view_strips(race, scratch, system)?;
    depth_buffer(false);
    crate::scene::set_filters::set_filters();
    Ok(())
}
