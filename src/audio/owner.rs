//! The audio stores in one struct, handed by reference to the code that uses them. The audio
//! thread reads them beside the game thread (atomic cells); what only the audio thread touches is
//! in its own record (`audio::own`).

use crate::audio::levels::Levels;
use crate::audio::names_store::SampleNames;
use crate::audio::sample_records::SampleRecords;
use crate::audio::store::AudioShared;
use rc2k_core::audio::commands::Queue;
use std::sync::atomic::{AtomicBool, AtomicU32};
use std::sync::{Condvar, Mutex};

/// What lets the game's thread wait for the audio thread to take the queue without polling
/// it (`sound::queue`). Every field is read and changed with the queue locked.
pub struct TurnSignals {
    /// Told when a thread begins to wait for the queue to be taken: the audio thread cuts its
    /// pause between turns short.
    pub wanted: Condvar,
    /// Told when the audio thread has ended a turn.
    pub over: Condvar,
    /// How many threads wait for the queue to be taken.
    pub waiters: AtomicU32,
    /// The audio thread is in a turn: what it took from the queue may not be done yet.
    pub in_turn: AtomicBool,
}

pub struct Audio {
    /// What the game's thread and the audio thread both touch: the modes, the volumes, the voices,
    /// the options and the streams.
    pub state: AudioShared,
    /// The sample records.
    pub sample_records: SampleRecords,
    /// The commands the game sends to the audio thread.
    pub commands: Mutex<Queue>,
    /// The signals between a thread that waits for the queue and the audio thread.
    pub turns: TurnSignals,
    /// The sample files' names.
    pub sample_names: SampleNames,
    /// The volume levels shared with the menus.
    pub levels: Levels,
    /// Whether a race is loaded, as the course's in-race flag has it (set with it at the load and
    /// the tidy): the sound hash (`RC2K_SOUND_HASH`) logs only commands of a race.
    pub racing: crate::memory::session::Flag,
}

impl Audio {
    /// The state at start-up: every store in its first state.
    pub const fn new() -> Self {
        Self {
            state: AudioShared::new(),
            sample_records: SampleRecords::new(),
            commands: Mutex::new(Queue::new()),
            turns: TurnSignals {
                wanted: Condvar::new(),
                over: Condvar::new(),
                waiters: AtomicU32::new(0),
                in_turn: AtomicBool::new(false),
            },
            sample_names: SampleNames::new(),
            levels: Levels::new(),
            racing: crate::memory::session::Flag::new(false),
        }
    }
}
