//! The profiler's stages (the debug UI's "Profiler" tab; RAL.EXE's own timers
//! were taken out of the shipped game). `start` asks one atomic whether the UI is open and
//! reads no clock while it is closed: play pays one load and one branch a stage.

use std::sync::atomic::{AtomicU32, AtomicU64, Ordering::Relaxed};
use std::time::Instant;

#[derive(Clone, Copy)]
pub enum Stage {
    /// `input::update`.
    Input,
    /// `drive::race_step`, the frame's physics ticks.
    Physics,
    /// `race::draw_race_scene`, with the 3D noted (`begin_scene` .. `end_scene`).
    Scene,
    /// `sound::race_sound_tick`, the game's commands to the sound.
    SoundTick,
    /// `audio::thread`'s turn (its own thread, every 20 ms), without its sleep.
    AudioTurn,
    /// The course's sorted faces drawn (`draw_view_scene`'s first `draw_entries`), every view's.
    CourseEntries,
    /// The depth-sorted entries drawn (the cars, objects and see-through faces), every view's.
    DepthEntries,
    /// `race::draw_mirror_view`, the whole of it.
    Mirror,
    /// A view's effects (`race::run_view_marks`: `effects::frame_marks`, `effects::run_frame`).
    Effects,
}

pub const STAGES: usize = 9;

static NANOS: [AtomicU64; STAGES] = [const { AtomicU64::new(0) }; STAGES];
static RUNS: [AtomicU64; STAGES] = [const { AtomicU64::new(0) }; STAGES];

/// The voices playing and stopping, noted by the audio thread each turn while the UI is open
/// (the lists are its own).
static VOICES: [AtomicU32; 2] = [const { AtomicU32::new(0) }; 2];

/// The audio thread's note of its voices playing and stopping (`on_lists`).
pub fn note_voices((playing, stopping): (u32, u32)) {
    VOICES[0].store(playing, Relaxed);
    VOICES[1].store(stopping, Relaxed);
}

/// The voices playing and stopping as last noted.
#[cfg(debug_ui)]
pub fn voices() -> (u32, u32) {
    (VOICES[0].load(Relaxed), VOICES[1].load(Relaxed))
}

/// The race's ticks run (`drive::race_step`), whichever way each goes.
static TICKS: AtomicU32 = AtomicU32::new(0);

#[inline]
pub fn note_tick() {
    TICKS.fetch_add(1, Relaxed);
}

#[cfg(any(debug_ui, all(target_os = "linux", target_arch = "aarch64")))]
pub fn ticks() -> u32 {
    TICKS.load(Relaxed)
}

/// The ground searches run while the UI is open (`physics::ground_search_at`, on every thread
/// that searches) and those that found a face under the point: the display page's "Ground searches" and "Found".
static GROUND_SEARCHES: [AtomicU32; 2] = [const { AtomicU32::new(0) }; 2];

/// A ground search is run (`found`: a face was under the point): counted only while the UI
/// is open, so play pays one load of a read-mostly flag and writes nothing.
#[inline]
pub fn note_ground_search(found: bool) {
    if !crate::debug::imgui_sdl::is_open() {
        return;
    }
    GROUND_SEARCHES[0].fetch_add(1, Relaxed);
    if found {
        GROUND_SEARCHES[1].fetch_add(1, Relaxed);
    }
}

/// The ground searches run and those that found a face, so far.
#[cfg(debug_ui)]
pub fn ground_searches() -> (u32, u32) {
    (
        GROUND_SEARCHES[0].load(Relaxed),
        GROUND_SEARCHES[1].load(Relaxed),
    )
}

/// A stage begins: the clock, only while the UI is open.
#[inline]
pub fn start() -> Option<Instant> {
    #[cfg(all(target_os = "linux", target_arch = "aarch64"))]
    return Some(Instant::now());
    #[cfg(not(all(target_os = "linux", target_arch = "aarch64")))]
    crate::debug::imgui_sdl::is_open().then(Instant::now)
}

/// A stage ends (`begun` is `start`'s).
#[inline]
pub fn stop(stage: Stage, begun: Option<Instant>) {
    if let Some(begun) = begun {
        NANOS[stage as usize].fetch_add(begun.elapsed().as_nanos() as u64, Relaxed);
        RUNS[stage as usize].fetch_add(1, Relaxed);
    }
}

/// Each stage's nanoseconds and runs so far.
#[cfg(any(debug_ui, all(target_os = "linux", target_arch = "aarch64")))]
pub fn counts() -> ([u64; STAGES], [u64; STAGES]) {
    (
        std::array::from_fn(|n| NANOS[n].load(Relaxed)),
        std::array::from_fn(|n| RUNS[n].load(Relaxed)),
    )
}
