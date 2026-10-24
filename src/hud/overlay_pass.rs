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
//! Caller: `sub_4547AE` (0x454786). The window is its Rust version; the rest through
//! `part` (1 / 4 the depth buffer off / on, 2 the states, 3 the HUD, 5 the render state,
//! 6 the strips, 7 the filters), in the game their Rust versions.
//! Hook removed 2026-10-22 (direct calls): Rust callers run its `entry` (`calls::call`,
//! `routes.rs`); the registers above are the original's, for reference.

use crate::errors::Refused;
use crate::framework::rva;
use crate::hud::store::{game_read, game_write};

const PLAYERS_RVA: usize = rva(0x6238A4); // dword_6238A4
const PLAYERS_COPY_RVA: usize = rva(0x6D14F2); // dword_6D14F2
const FLAGS_RVA: [usize; 2] = [rva(0x6D14F6), rva(0x6D14FA)]; // the HUD flags
const MODE_RVA: usize = rva(0x63DB98); // dword_63DB98
const BLOCK_RVA: [usize; 2] = [rva(0x710394), rva(0x711374)]; // the car blocks' +0xFA8
const CLEARED_RVA: [usize; 2] = [rva(0x6D1486), rva(0x6D1496)];
const ON_RVA: [usize; 2] = [rva(0x63E168), rva(0x63E16C)];

/// The HUD's layout: `dword_6238A4`, or 0 while one view is drawn (`dword_63DB98` 1).
/// The original reads `dword_6238A4` alone, which a replay after a split-screen race
/// leaves at 1 / 2 while its one view fills the screen: the divider, the second
/// player's bars and strip stayed drawn (section 16, fixed 2026-10-23).
pub fn shown_layout() -> u32 {
    unsafe {
        if game_read::<u32>(MODE_RVA) == 1 {
            0
        } else {
            game_read(PLAYERS_RVA)
        }
    }
}

/// The Rust version (see the header). What the original leaves in the registers is read
/// by none of its callers (register liveness, 2026-10-23).
pub unsafe fn overlay_pass() -> Result<(), Refused> {
    use crate::scene::depth_buffer_on::depth_buffer;
    unsafe {
        game_write(PLAYERS_COPY_RVA, shown_layout());
        depth_buffer(false);
        crate::hud::overlay_states::overlay_states();
        let kept: [u32; 2] = FLAGS_RVA.map(|r| game_read(r));
        if game_read::<u32>(MODE_RVA) == 2 {
            for n in 0..2 {
                if game_read::<u32>(BLOCK_RVA[n]) != 0 {
                    game_write(FLAGS_RVA[n], game_read::<u32>(FLAGS_RVA[n]) & 0x40);
                    game_write(CLEARED_RVA[n], 0u32);
                }
            }
        }
        let done = crate::hud::draw_hud::draw_hud();
        game_write(FLAGS_RVA[1], kept[1]);
        game_write(FLAGS_RVA[0], kept[0]);
        done?;
        depth_buffer(true);
        crate::scene::full_view_window::full_view_window();
        crate::scene::set_render_state::set_render_state(3, 3);
        game_write(ON_RVA[0], 1u32);
        game_write(ON_RVA[1], 1u32);
        crate::hud::draw_view_strips::draw_view_strips()?;
        depth_buffer(false);
        crate::scene::set_filters::set_filters();
    }
    Ok(())
}
