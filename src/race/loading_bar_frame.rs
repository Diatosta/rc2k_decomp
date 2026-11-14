//! One frame of the loading bar while a stage loads (the `bar_function` callback). Outside
//! a stage's loading and the credits it does nothing; otherwise it sets `scene_state` to the frame
//! mark and, unless the game is quitting, draws `race::loading_bar_quad` in an unframed view window
//! and presents the frame. A frame that cannot be drawn (the window inactive, its surfaces lost, or
//! the lock refused) is skipped: nothing is shown and the load goes on; the next frame tries again.

use crate::errors::Refused;
/// A stage is loading (`race::load_stage`). The bar is drawn for every stage, over the loading
/// picture.
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

/// Draws a frame, at most one in 33 ms.
pub fn loading_bar_frame(o: crate::owners::Owners<'static>) -> Result<(), Refused> {
    use std::sync::atomic::Ordering::Relaxed;
    // At most some 30 frames a second, and the last: drawing every step's frame took 3/4 of a
    // load (882 ms of 1231).

    let begun = std::time::Instant::now();
    {
        let mut last = LAST_FRAME
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if o.race.shared.vars.get().bar_share.get() < 100
            && last.is_some_and(|last| begun.duration_since(last) < BAR_FRAME)
        {
            return Ok(());
        }
        *last = Some(begun);
    }
    let result = {
        let _timed = crate::race::load_times::scope(crate::race::load_times::Category::BarFrame);
        frame(o)
    };
    FRAMES[0].fetch_add(1, Relaxed);
    FRAMES[1].fetch_add(begun.elapsed().as_micros() as u64, Relaxed);
    result
}

fn frame(o: crate::owners::Owners<'static>) -> Result<(), Refused> {
    let (race, system, scene, gfx, d3d) = (o.race, o.system, o.scene, o.gfx, o.d3d);
    let session = &system.session;
    let shared = &race.shared;
    // Logic in `rc2k_core::race::loading::{FRAME_MARK, frame_tried}`.
    use crate::scene::*;
    use rc2k_core::race::loading::{FRAME_MARK, frame_tried};
    if !shared.work.get().in_credits.is_set()
        && !STAGE_LOADING.load(std::sync::atomic::Ordering::Relaxed)
    {
        return Ok(());
    }
    scene.lights.state.get().scene_state.set(FRAME_MARK);
    if session.flags.quitting.is_set() {
        return Ok(());
    }
    // A frame that cannot be drawn is skipped: the load goes on and the next frame tries again.
    'frame: {
        let active: u32 = session.flags.window_active.get();
        if active == 0 || !frame_tried(active, system.window_loop().lost.get()) {
            break 'frame;
        }
        begin_scene::begin_scene(race, d3d, crate::workers::game_scratch());
        let framed: u32 = shared.switches.get().editor_mode.get();
        shared.switches.get().editor_mode.set(0u32);
        choose_view_window::choose_view_window(race, scene, system);
        shared.switches.get().editor_mode.set(framed);
        crate::race::loading_bar_quad::loading_bar_quad(race, system, scene)?;
        end_scene::end_scene(d3d);
        if lock_frame::lock_frame(race, gfx, system).0 < 1 {
            break 'frame;
        }
        unlock_frame::unlock_frame(&race.frame, gfx);
        // A refused present is skipped as well.
        let _timed = crate::race::load_times::scope(crate::race::load_times::Category::Present);
        present_frame::present_frame(o);
    }
    Ok(())
}
