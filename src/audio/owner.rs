//! The audio stores in one struct, handed by reference to the code that uses them; only the roots
//! (`Owners::the`, `SlotOwners::fetch`) call `audio()`. The audio thread reads them beside the game thread (atomic cells); what only the
//! audio thread touches is in its own record (`audio::own`).

use crate::audio::levels::Levels;
use crate::audio::names_store::SampleNames;
use crate::audio::store::{AudioShared, SampleRecords};
use rc2k_core::audio::commands::Queue;
use std::sync::Mutex;

pub struct Audio {
    /// What the game's thread and the audio thread both touch: the modes, the volumes, the voices,
    /// the options and the streams.
    pub state: AudioShared,
    /// The sample records.
    pub sample_records: SampleRecords,
    /// The commands the game sends to the audio thread.
    pub commands: Mutex<Queue>,
    /// The sample files' names.
    pub sample_names: SampleNames,
    /// The volume levels shared with the menus.
    pub levels: Levels,
    /// Whether a race is loaded, as the course's in-race flag has it (set with it at the load and
    /// the tidy): the sound hash (`RC2K_SOUND_HASH`) logs only commands of a race.
    pub racing: crate::session::Flag,
    /// The EAX environment the race asks for (a float's bits, 1.0 at the race's start): set by the
    /// game's thread from the first player's view (never by a drawing part), read by the audio
    /// thread's voice update.
    pub environment: crate::session::Dword,
}

impl Audio {
    /// The state at start-up: every store in its first state.
    pub const fn new() -> Self {
        Self {
            state: AudioShared::new(),
            sample_records: SampleRecords::new(),
            commands: Mutex::new(Queue::new()),
            sample_names: SampleNames::new(),
            levels: Levels::new(),
            racing: crate::session::Flag::new(false),
            environment: crate::session::Dword::new(0),
        }
    }
}

/// The one instance, reached through `audio()` by the roots and by tests.
pub static AUDIO_OWNER: Audio = Audio::new();

/// The audio's state, for the roots that hand it down and for tests.
pub fn audio() -> &'static Audio {
    &AUDIO_OWNER
}
