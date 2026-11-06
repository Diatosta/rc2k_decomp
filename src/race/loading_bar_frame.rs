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
//! Caller: the stage's loading (through `dword_623704`). Its calls are the Rust versions (the
//! restore the platform's call). The registers above are the original's, for reference; a call
//! by address goes through `routes.rs`.

use crate::errors::Refused;
use crate::race::owner::race;
use crate::race::owner_course::Course;
use crate::race::switches_store::FRAME_SWITCHES;
use crate::race::vars_store::RACE_VARS;
use crate::race::work_store::RACE_WORK;
use crate::scene::light_store::LIGHT_STATE;
use crate::session::FLAGS;
const RESTORE: usize = 0x405B10; // sub_405B10: the surfaces restored (the platform's)

/// A stage is loading (`race::load_stage`). Not the original's: it draws the bar for the
/// credits' race only; here for every stage, over the loading picture (owner, 2026-10-30).
pub static STAGE_LOADING: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// The bar's frames and their microseconds since last asked (the load's log line).
static FRAMES: [std::sync::atomic::AtomicU64; 2] =
    [const { std::sync::atomic::AtomicU64::new(0) }; 2];

/// The time between the bar's frames, and when the last was.
const BAR_FRAME: std::time::Duration = std::time::Duration::from_millis(33);
static LAST_FRAME: std::sync::Mutex<Option<std::time::Instant>> = std::sync::Mutex::new(None);

/// The bar's frames and microseconds since last asked.
pub fn shown() -> (u64, u64) {
    use std::sync::atomic::Ordering::Relaxed;
    (FRAMES[0].swap(0, Relaxed), FRAMES[1].swap(0, Relaxed))
}

/// The Rust version (see the header), timed.
pub fn loading_bar_frame() -> Result<(), Refused> {
    let course = &race().course;
    use std::sync::atomic::Ordering::Relaxed;
    // Not the original's: some 30 frames a second at most, and the last. Every step's frame
    // was 3/4 of a load (882 ms of 1231, 2026-10-30).
    let begun = std::time::Instant::now();
    {
        let mut last = LAST_FRAME
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if RACE_VARS.bar_share.bits() < 100
            && last.is_some_and(|last| begun.duration_since(last) < BAR_FRAME)
        {
            return Ok(());
        }
        *last = Some(begun);
    }
    let result = frame(course);
    FRAMES[0].fetch_add(1, Relaxed);
    FRAMES[1].fetch_add(begun.elapsed().as_micros() as u64, Relaxed);
    result
}

fn frame(course: &Course) -> Result<(), Refused> {
    // Converted: `rc2k_core::race::loading::{FRAME_MARK, frame_tried}`.
    use crate::scene::*;
    use rc2k_core::race::loading::{FRAME_MARK, frame_tried};
    if RACE_WORK.in_credits.get() == 0 && !STAGE_LOADING.load(std::sync::atomic::Ordering::Relaxed)
    {
        return Ok(());
    }
    LIGHT_STATE.scene_state.set_bits(FRAME_MARK);
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
        let framed: u32 = FRAME_SWITCHES.editor_mode.bits();
        FRAME_SWITCHES.editor_mode.set_bits(0u32);
        choose_view_window::choose_view_window();
        FRAME_SWITCHES.editor_mode.set_bits(framed);
        crate::race::loading_bar_quad::loading_bar_quad(course)?;
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
