//! The loading bar: the `Progress` sink the loaders report to, and the frame it draws. Outside a
//! stage's loading and the credits a frame does nothing; otherwise it sets `scene_state` to the
//! frame mark and, unless the game is quitting, draws `race::loading_bar_quad` in an unframed view
//! window and presents the frame. A frame that cannot be drawn (the window inactive, its surfaces
//! lost, or the lock refused) is skipped: nothing is shown and the load goes on; the next frame
//! tries again.

use crate::errors::Refused;
use rc2k_core::scene::loading::BarEnds;

/// Where a load reports how far it has got.
pub trait Progress {
    /// The load is `share` percent done (0 ..= 100): shows a frame.
    fn mark(&mut self, share: u32) -> Result<(), Refused>;

    /// The share the texture marks (`textures_done`) run between.
    fn set_span(&mut self, low: u32, high: u32);

    /// `done` of `textures` system textures are loaded: the share moves on within the span (and
    /// stays where it is with no textures), and a frame is shown.
    fn textures_done(&mut self, textures: u32, done: u32) -> Result<(), Refused>;
}

/// The loading bar on the screen: a `Progress` that draws a frame at each mark, unless one was
/// shown under 33 ms before.
pub struct LoadingBar {
    o: crate::owners::Owners<'static>,
    share: u32,
    span: BarEnds,
    /// The share the other players were last told (a game with others only).
    told: Option<u32>,
}

impl LoadingBar {
    /// A bar at 0 whose texture span is the whole bar.
    pub fn new(o: crate::owners::Owners<'static>) -> Self {
        Self {
            o,
            share: 0,
            span: BarEnds { low: 0, high: 100 },
            told: None,
        }
    }

    /// The other players told the share when it has moved.
    fn tell(&mut self) {
        if self.told != Some(self.share) {
            self.told = Some(self.share);
            crate::net::loading::report(self.o.net, self.share);
        }
    }
}

impl Progress for LoadingBar {
    fn mark(&mut self, share: u32) -> Result<(), Refused> {
        self.share = share;
        self.tell();
        loading_bar_frame(self.o, share)
    }

    fn set_span(&mut self, low: u32, high: u32) {
        self.span = BarEnds { low, high };
    }

    fn textures_done(&mut self, textures: u32, done: u32) -> Result<(), Refused> {
        if let Some(share) = self.span.progress(done, textures) {
            self.share = share;
        }
        self.tell();
        loading_bar_frame(self.o, self.share)
    }
}

/// A stage is loading (`race::load_stage`). The bar is drawn for every stage, over the loading
/// picture.
pub static STAGE_LOADING: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// The bar's frames and their microseconds since last asked (the load's log line).
static FRAMES: [std::sync::atomic::AtomicU64; 2] =
    [const { std::sync::atomic::AtomicU64::new(0) }; 2];

/// The least time between two frames of the loading screen (the picture's and the bar's), and
/// when the last was shown.
const BAR_FRAME: std::time::Duration = std::time::Duration::from_millis(33);
static LAST_FRAME: std::sync::Mutex<Option<std::time::Instant>> = std::sync::Mutex::new(None);

/// The bar's frames and microseconds since last asked.
pub fn shown() -> (u64, u64) {
    use std::sync::atomic::Ordering::Relaxed;
    (FRAMES[0].swap(0, Relaxed), FRAMES[1].swap(0, Relaxed))
}

/// The loading picture was just shown: the bar's first frame waits its 33 ms after it.
pub fn picture_shown() {
    *LAST_FRAME
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(std::time::Instant::now());
}

/// Whether a frame may be shown at `now`, the last one having been shown at `last`.
fn frame_due(last: Option<std::time::Instant>, now: std::time::Instant) -> bool {
    last.is_none_or(|last| now.duration_since(last) >= BAR_FRAME)
}

/// Draws a frame showing `share` percent, unless a frame of the loading screen was shown under
/// 33 ms ago: the load does not wait for the screen, and the bar's last steps may go unseen
/// (the race's first frame follows).
fn loading_bar_frame(o: crate::owners::Owners<'static>, share: u32) -> Result<(), Refused> {
    use std::sync::atomic::Ordering::Relaxed;
    // Drawing every step's frame took 3/4 of a load (882 ms of 1231).
    let begun = std::time::Instant::now();
    {
        let mut last = LAST_FRAME
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if !frame_due(*last, begun) {
            return Ok(());
        }
        *last = Some(begun);
    }
    let result = {
        let _timed = crate::race::load_times::scope(crate::race::load_times::Category::BarFrame);
        frame(o, share)
    };
    #[cfg(feature = "remote")]
    hold_for_shots();
    FRAMES[0].fetch_add(1, Relaxed);
    FRAMES[1].fetch_add(begun.elapsed().as_micros() as u64, Relaxed);
    result
}

/// Test builds: while the file `hold` in the remote folder (`RC2K_REMOTE`) exists, every bar frame
/// lasts as many milliseconds as it holds, so that the loading screen can be photographed.
#[cfg(feature = "remote")]
fn hold_for_shots() {
    let Some(folder) = std::env::var_os("RC2K_REMOTE") else {
        return;
    };
    let held = std::fs::read_to_string(std::path::Path::new(&folder).join("hold"));
    if let Some(ms) = held.ok().and_then(|v| v.trim().parse::<u64>().ok()) {
        std::thread::sleep(std::time::Duration::from_millis(ms));
    }
}

fn frame(o: crate::owners::Owners<'static>, share: u32) -> Result<(), Refused> {
    let (race, system, scene, gfx, d3d) = (o.race, o.system, o.scene, o.gfx, &o.gfx.d3d);
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
        begin_scene::begin_scene(race, d3d, o.scene.scratch.game_scratch());
        let framed: u32 = shared.switches.get().editor_mode.get();
        shared.switches.get().editor_mode.set(0u32);
        choose_view_window::choose_view_window(race, scene, system);
        shared.switches.get().editor_mode.set(framed);
        crate::race::loading_bar_quad::loading_bar_quad(race, system, scene, o.net, share)?;
        end_scene::end_scene(d3d);
        if lock_frame::lock_frame(race, gfx, system).0 < 1 {
            break 'frame;
        }
        // The other players' loads, as 2D text over the frame.
        crate::net::loading::draw(o)?;
        unlock_frame::unlock_frame(&race.frame, gfx);
        // A refused present is skipped as well.
        let _timed = crate::race::load_times::scope(crate::race::load_times::Category::Present);
        present_frame::present_frame(o);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_frame_is_due_33_ms_after_the_last() {
        let last = std::time::Instant::now();
        let after = |millis| last + std::time::Duration::from_millis(millis);
        assert!(frame_due(None, last));
        assert!(!frame_due(Some(last), last));
        assert!(!frame_due(Some(last), after(32)));
        assert!(frame_due(Some(last), after(33)));
        // A time before the last frame's (two threads' clocks read out of order) is no frame.
        assert!(!frame_due(Some(after(5)), last));
    }
}
