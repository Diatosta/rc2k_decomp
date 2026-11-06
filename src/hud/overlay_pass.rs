//! `hud::overlay_pass` (0x45A893): the race frame's 2D pass: `dword_6D14F2` the players
//! (`dword_6238A4`); `scene::depth_buffer_off`; `hud::overlay_states`; with `dword_63DB98`
//! 2, a player whose car block's +0xFA8 is set (`dword_710394` / `dword_711374`) keeps only
//! bit 0x40 of its HUD flags (`dword_6D14F6` / `dword_6D14FA`, put back after) and its
//! `unk_6D1486` / `unk_6D1496` is 0; `hud::draw_hud`; `scene::depth_buffer_on`;
//! `scene::full_view_window`; render state 3 = 3; `dword_63E168` and `dword_63E16C` 1;
//! `hud::draw_view_strips`; an empty function (`nullsub_9`); `scene::depth_buffer_off`;
//! `scene::set_filters`.
//!
//! In:  nothing
//! Out: as the last call leaves them
//!
//! Caller: `sub_4547AE` (0x454786). The registers above are the original's, for reference; a
//! call by address goes through `routes.rs`.

use crate::errors::Refused;
use crate::hud::owner::Hud;
use crate::race::blocks_store::CAR_BLOCKS;
use crate::race::level_grid_store::LEVEL_GRID;
use crate::race::panel_store::PANEL;
use crate::race::vars_store::RACE_VARS;
use crate::race::view_store::RACE_VIEW;

/// The HUD's layout: `dword_6238A4`, or 0 while one view is drawn (`dword_63DB98` 1).
/// The original reads `dword_6238A4` alone, which a replay after a split-screen race
/// leaves at 1 / 2 while its one view fills the screen: the divider, the second
/// player's bars and strip stayed drawn (section 16, fixed 2026-10-23).
pub fn shown_layout() -> u32 {
    {
        if RACE_VIEW.views.bits() == 1 {
            0
        } else {
            RACE_VARS.split.bits()
        }
    }
}

/// The Rust version (see the header). What the original leaves in the registers is read
/// by none of its callers (register liveness, 2026-10-23).
pub fn overlay_pass(hud: &Hud) -> Result<(), Refused> {
    use crate::scene::depth_buffer_on::depth_buffer;
    PANEL.players.set(shown_layout());
    depth_buffer(false);
    crate::hud::overlay_states::overlay_states();
    let kept: [u32; 2] = [PANEL.hud_flags[0].get(), PANEL.hud_flags[1].get()];
    if RACE_VIEW.views.bits() == 2 {
        for n in 0..2 {
            if CAR_BLOCKS.cars[n].view_covered.get() != 0 {
                PANEL.hud_flags[n].set(PANEL.hud_flags[n].get() & 0x40);
                PANEL.lights[n].fade.set(0);
            }
        }
    }
    let done = crate::hud::draw_hud::draw_hud(hud);
    PANEL.hud_flags[1].set(kept[1]);
    PANEL.hud_flags[0].set(kept[0]);
    done?;
    depth_buffer(true);
    crate::scene::full_view_window::full_view_window();
    crate::scene::set_render_state::set_render_state(3, 3);
    LEVEL_GRID.plain_first.set(1);
    LEVEL_GRID.plain_second.set(1);
    crate::hud::draw_view_strips::draw_view_strips()?;
    depth_buffer(false);
    crate::scene::set_filters::set_filters();
    Ok(())
}
