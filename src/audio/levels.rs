//! The volume levels the audio thread shares with the menus: the master volume shown on the
//! sound page (written when the primary buffer is set up), and the three levels a sample
//! record's `volume_factor` can name (the race sounds' and the other sounds' two). The menus
//! change them with the volume steps and save them in the options file; the audio thread reads
//! them with every voice it starts.

use crate::session::Dword;

/// The levels, each 0 .. 0x40 (`menu::option_steps`).
pub struct Levels {
    /// `g_masterVolumeShown`: the master volume shown (`audio::primary`, `menu::run_front_end`).
    pub master_volume_shown: Dword,
    /// `dword_755694`: the race sounds' volume.
    pub menu_sounds_a: Dword,
    /// `dword_75569C`: the sounds' first level.
    pub sounds_a: Dword,
    /// `dword_7556A0`: the sounds' second level.
    pub sounds_b: Dword,
}

impl Levels {
    pub const fn new() -> Self {
        Levels {
            master_volume_shown: Dword::new(0x40),
            menu_sounds_a: Dword::new(0x40),
            sounds_a: Dword::new(0x40),
            sounds_b: Dword::new(0x40),
        }
    }
}
