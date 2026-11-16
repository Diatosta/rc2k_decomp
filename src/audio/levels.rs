//! The volume levels the audio thread shares with the menus: the general volume shown on the sound
//! page and the three levels a sample record's `volume_factor` can name. The menus change them and
//! save them in the options file; the audio thread reads them with every voice it starts.

use crate::memory::session::Dword;

/// The levels, each 0 .. 0x40 (`menu::option_steps`).
pub struct Levels {
    /// The "general volume" slider (`audio::primary`, `menu::run_front_end`).
    pub general_volume: Dword,
    /// The "race effects" slider.
    pub race_effects_volume: Dword,
    /// The "co-driver's voice" slider.
    pub codriver_voice_volume: Dword,
    /// The "car engine" slider.
    pub engine_volume: Dword,
}

impl Levels {
    pub const fn new() -> Self {
        Levels {
            general_volume: Dword::new(0x40),
            race_effects_volume: Dword::new(0x40),
            codriver_voice_volume: Dword::new(0x40),
            engine_volume: Dword::new(0x40),
        }
    }
}
