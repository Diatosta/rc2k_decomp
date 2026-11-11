//! Module audio's owner (step 11, owned state): the audio stores in one struct, reached
//! through `audio()` by the code that does not take it as a parameter. The audio thread
//! reads them at the same time as the game thread (atomic cells).

use crate::audio::levels::Levels;
use crate::audio::names_store::SampleNames;
use crate::audio::store::{AudioState, SampleRecords};
use rc2k_core::audio::commands::Queue;
use std::sync::Mutex;

/// The audio's state.
pub struct Audio {
    /// The mixer, the voices, the options and the streams.
    pub state: AudioState,
    /// The sample records.
    pub sample_records: SampleRecords,
    /// The commands the game sends to the audio thread.
    pub commands: Mutex<Queue>,
    /// The sample files' names.
    pub sample_names: SampleNames,
    /// The volume levels shared with the menus.
    pub levels: Levels,
}

/// The one instance (not `AUDIO`: that is `store::AUDIO`'s old name). Reached through
/// `audio()`; `moved` registers its stores' places.
pub static AUDIO_OWNER: Audio = Audio {
    state: AudioState::new(),
    sample_records: SampleRecords::new(),
    commands: Mutex::new(Queue::new()),
    sample_names: SampleNames::new(),
    levels: Levels::new(),
};

/// The audio's state, for a caller that does not take it as a parameter yet.
pub fn audio() -> &'static Audio {
    &AUDIO_OWNER
}
