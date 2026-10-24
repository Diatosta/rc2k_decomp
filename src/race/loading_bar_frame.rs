//! `race::loading_bar_frame` (0x4467C7): one frame of the loading bar while a stage loads
//! (the loading's callback `dword_623704`), with `dword_71C3E0` only: `dword_6365CC` 8;
//! nothing while quitting; with the window active and the surfaces not lost
//! (`dword_518380` not 1): `scene::begin_scene`, the view window chosen as unframed
//! (`dword_621514` 0 for `scene::choose_view_window`, put back), `race::loading_bar_quad`,
//! `scene::end_scene`, `scene::lock_frame`, an empty function, `scene::unlock_frame`,
//! `scene::present_frame`, carry clear. A failed step (or the window inactive, or the
//! surfaces lost) instead, unless quitting, restores the surfaces (`sub_405B10`, the
//! platform's) and leaves carry set when they were neither lost before nor after.
//!
//! In:  nothing
//! Out: nothing its callers read (conversion phase 3, 2026-10-24: the flags and registers
//!      it left went; its callers, `race::loadlevel_init`'s progress inside `pusha` and
//!      `scene::step_loading_bar`'s `show`, discard them)
//!
//! Caller: the stage's loading (through `dword_623704`). Its calls are the Rust versions
//! (the restore the platform's call).
//! Hook removed 2026-10-22 (direct calls): Rust callers run its `entry` (`calls::call`,
//! `routes.rs`); the registers above are the original's, for reference.

use crate::errors::Refused;
use crate::framework::{game_read, game_write, rva};
use crate::session::FLAGS;

const ON_RVA: usize = rva(0x71C3E0); // dword_71C3E0
const MARK_RVA: usize = rva(0x6365CC); // dword_6365CC
const FRAMED_RVA: usize = rva(0x621514); // dword_621514
const RESTORE: usize = 0x405B10; // sub_405B10: the surfaces restored (the platform's)

/// The Rust version (see the header).
pub unsafe fn loading_bar_frame() -> Result<(), Refused> {
    // Converted: `rc2k_core::race::loading::{FRAME_MARK, frame_tried}`.
    use crate::scene::*;
    use rc2k_core::race::loading::{FRAME_MARK, frame_tried};
    unsafe {
        if game_read::<u32>(ON_RVA) == 0 {
            return Ok(());
        }
        game_write(MARK_RVA, FRAME_MARK);
        if FLAGS.quitting.get() == 1 {
            return Ok(());
        }
        let drawn = 'frame: {
            let active: u32 = FLAGS.window_active.get();
            if active == 0 || !frame_tried(active, FLAGS.lost.get()) {
                break 'frame false;
            }
            // `scene::begin_scene` leaves the carry clear.
            begin_scene::begin_scene();
            let framed: u32 = game_read(FRAMED_RVA);
            game_write(FRAMED_RVA, 0u32);
            choose_view_window::choose_view_window();
            game_write(FRAMED_RVA, framed);
            crate::race::loading_bar_quad::loading_bar_quad()?;
            end_scene::end_scene();
            if lock_frame::lock_frame().0 < 1 {
                break 'frame false;
            }
            unlock_frame::unlock_frame();
            present_frame::present_frame()
        };
        if drawn || FLAGS.quitting.get() == 1 {
            return Ok(());
        }
        // The platform's restore, with ebx `dword_518380` (a local scratch: the flags it
        // leaves were read by no caller).
        crate::standalone::not_ported(RESTORE, "race::loading_bar_frame")
    }
}
