//! `race::loading_bar_frame` (0x4467C7): one frame of the loading bar while a stage loads (the
//! loading's callback `bar_function`), with `in_credits` only: `scene_state` 8; nothing while
//! quitting; with the window active and the surfaces not lost (`WindowLoopState.lost` not 1):
//! `scene::begin_scene`, the view window chosen as unframed (`editor_mode` 0 for
//! `scene::choose_view_window`, put back), `race::loading_bar_quad`, `scene::end_scene`,
//! `scene::lock_frame`, an empty function, `scene::unlock_frame`, `scene::present_frame`, carry
//! clear. A failed step (or the window inactive, or the surfaces lost) instead, unless quitting,
//! restores the surfaces (0x405B10, the platform's) and leaves carry set when they were neither
//! lost before nor after.
//!
//! Caller: the stage's loading (through `bar_function`). Its calls are the Rust versions (the
//! restore the platform's call). a call by address goes through `routes.rs`.

use crate::errors::Refused;
use crate::race::owner::Race;
const RESTORE: usize = 0x405B10; // 0x405B10: the surfaces restored (the platform's)

/// A stage is loading (`race::load_stage`). Not the original's: it draws the bar for the
/// credits' race only; here for every stage, over the loading picture.
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

/// Timed.
pub fn loading_bar_frame(race: &Race) -> Result<(), Refused> {
    use std::sync::atomic::Ordering::Relaxed;
    // Not the original's: some 30 frames a second at most, and the last. Every step's frame
    // was 3/4 of a load (882 ms of 1231).

    let begun = std::time::Instant::now();
    {
        let mut last = LAST_FRAME
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if race.shared.vars.bar_share.get() < 100
            && last.is_some_and(|last| begun.duration_since(last) < BAR_FRAME)
        {
            return Ok(());
        }
        *last = Some(begun);
    }
    let result = frame(race);
    FRAMES[0].fetch_add(1, Relaxed);
    FRAMES[1].fetch_add(begun.elapsed().as_micros() as u64, Relaxed);
    result
}

fn frame(race: &Race) -> Result<(), Refused> {
    let system = crate::system::owner::system();
    let scene = crate::scene::owner::scene();
    let session = &system.session;
    let shared = &race.shared;
    // Converted: `rc2k_core::race::loading::{FRAME_MARK, frame_tried}`.
    use crate::scene::*;
    use rc2k_core::race::loading::{FRAME_MARK, frame_tried};
    if !shared.work.in_credits.is_set() && !STAGE_LOADING.load(std::sync::atomic::Ordering::Relaxed)
    {
        return Ok(());
    }
    scene.lights.state.scene_state.set(FRAME_MARK);
    if session.flags.quitting.is_set() {
        return Ok(());
    }
    let drawn = 'frame: {
        let active: u32 = session.flags.window_active.get();
        if active == 0 || !frame_tried(active, session.flags.lost.get()) {
            break 'frame false;
        }
        begin_scene::begin_scene();
        let framed: u32 = shared.switches.editor_mode.get();
        shared.switches.editor_mode.set(0u32);
        choose_view_window::choose_view_window(race, scene, system);
        shared.switches.editor_mode.set(framed);
        crate::race::loading_bar_quad::loading_bar_quad(race, system)?;
        end_scene::end_scene();
        if lock_frame::lock_frame(race).0 < 1 {
            break 'frame false;
        }
        unlock_frame::unlock_frame(&race.frame);
        present_frame::present_frame()
    };
    if drawn || session.flags.quitting.is_set() {
        return Ok(());
    }
    // The platform's restore, (with a local scratch value; the flags it
    // leaves were read by no caller).
    crate::standalone::not_ported(RESTORE, "race::loading_bar_frame")
}
