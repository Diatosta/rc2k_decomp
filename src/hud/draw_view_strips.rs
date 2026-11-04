//! `hud::draw_view_strips` (0x45B7B1): unless `dword_5BEC94`, for each player's view block
//! (`g_viewBlocks`, `+4` for the second when `dword_6238A4` is not 0) whose +0xFA8 is 0 and
//! +0x728 is set: while its flash is on (+0x730 at most 0, or +0x72C modulo +0x730 below
//! half of +0x730), `hud::draw_view_strip` for view `dword_6238A4` (the second's plus 3)
//! with texture `dword_6240BC` 0xC4 (0xC5); then the strips' shared tail (the original
//! jumps to 0x45B9AA).
//!
//! In:  nothing
//! Out: as the path leaves them (esi the last block looked at)
//!
//! Caller: `sub_45A893` (0x45A92E).

use crate::errors::{BadInput, Refused};
use crate::hud::overlay_pass::shown_layout;
use crate::race::replay_file_store::REPLAY_FILE;
use crate::race::view_store::RACE_VIEW;
use crate::scene::draw_store::DRAW;

/// Whether the strip of the view block `handle` names is shown (see the header).
fn flashing(handle: u32) -> Result<bool, BadInput> {
    let car = crate::race::blocks_store::car_of(handle).ok_or(BadInput("a view's block"))?;
    Ok(rc2k_core::hud::strips::strip(
        car.view_covered.get(),
        car.notice_shown.get(),
        car.notice_time.get(),
        car.notice_time_b.get(),
    )
    .shown())
}

/// The Rust version (see the header). What the original leaves in the registers is read
/// by none of its callers (register liveness, 2026-10-23).
// Converted: `rc2k_core::hud::strips`
pub fn draw_view_strips() -> Result<(), Refused> {
    use crate::hud::draw_view_strip::{draw_view_strip, overlay_tail};
    use rc2k_core::hud::strips::{SECOND_VIEW, TEXTURES};
    {
        if REPLAY_FILE.hold_car_revs.bits() == 0 {
            if flashing(RACE_VIEW.view_blocks[0].get())? {
                DRAW.polygon_textures[0].set_bits(TEXTURES[0]);
                draw_view_strip(shown_layout())?;
            }
            if shown_layout() != 0 && flashing(RACE_VIEW.view_blocks[1].get())? {
                DRAW.polygon_textures[0].set_bits(TEXTURES[1]);
                draw_view_strip(shown_layout().wrapping_add(SECOND_VIEW))?;
            }
        }
        overlay_tail()
    }
}
