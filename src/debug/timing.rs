//! The profiler's stages (the debug UI's "Profiler" tab, 2026-10-30; RAL.EXE's own timers
//! were taken out of the shipped game). `start` asks one atomic whether the UI is open and
//! reads no clock while it is closed: play pays one load and one branch a stage.

use std::sync::atomic::{AtomicU32, AtomicU64, Ordering::Relaxed};
use std::time::Instant;

/// What is timed.
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
}

/// Stages.
pub const STAGES: usize = 5;

static NANOS: [AtomicU64; STAGES] = [const { AtomicU64::new(0) }; STAGES];
static RUNS: [AtomicU64; STAGES] = [const { AtomicU64::new(0) }; STAGES];

/// The voices playing and stopping, noted by the audio thread each turn while the UI is open
/// (the lists are its own).
static VOICES: [AtomicU32; 2] = [const { AtomicU32::new(0) }; 2];

/// The audio thread's note (`on_lists`).
pub fn note_voices((playing, stopping): (u32, u32)) {
    VOICES[0].store(playing, Relaxed);
    VOICES[1].store(stopping, Relaxed);
}

/// The voices playing and stopping as last noted.
#[cfg(target_pointer_width = "64")]
pub fn voices() -> (u32, u32) {
    (VOICES[0].load(Relaxed), VOICES[1].load(Relaxed))
}

/// The race's ticks run (`drive::race_step`), whichever way each goes.
static TICKS: AtomicU32 = AtomicU32::new(0);

/// A tick is run.
#[inline]
pub fn note_tick() {
    TICKS.fetch_add(1, Relaxed);
}

/// The race's ticks so far.
#[cfg(target_pointer_width = "64")]
pub fn ticks() -> u32 {
    TICKS.load(Relaxed)
}

/// A stage begins: the clock, only while the UI is open.
#[inline]
pub fn start() -> Option<Instant> {
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
#[cfg(target_pointer_width = "64")]
pub fn counts() -> ([u64; STAGES], [u64; STAGES]) {
    (
        std::array::from_fn(|n| NANOS[n].load(Relaxed)),
        std::array::from_fn(|n| RUNS[n].load(Relaxed)),
    )
}
