//! The options pages' remaining item handlers (2026-10-19; the standalone program met
//! them as "not ported" on the sound and visual options pages): thirty small left/right
//! and action handlers, each named by what it changes. What each does, by its kind:
//! - a switch: a dword flipped (`xor .., 1`), nothing else (0x467EF8 `dword_5180F4`,
//!   0x468211 `dword_7556F8`, 0x468219 `dword_7556FC`; what they switch is not read);
//! - a switch with a copy: flipped, eax its value, then a second dword flipped too
//!   (0x46FFB3, 0x46FFC7) or given eax (0x470035, 0x470107, 0x470119, 0x47018B: the
//!   `dword_75AD..` the game's settings read);
//! - 0x470017: `dword_75564C` flipped, eax its value, `dword_4F50C4` flipped, then the
//!   screen-mode list made again (`gfx::screen_mode_choices` with eax 0);
//! - the advanced sound page's switches (0x4701E9 DS3D, 0x4701F6 EAX, 0x470203 A3D,
//!   0x470210 Dolby, 0x47021D the channels): the work value flipped and
//!   `menu::apply_audio_choices`; its quality (0x47019F one more, 0 after 3; 0x4701C4 one
//!   less, 3 before 0), then the same;
//! - a volume: its work value one less (below 0: 0) or one more (above 0x40: 0x40), signed;
//!   then the master's (`g_masterVolumeShown`) is sent (`sound::send_command_8`, eax the
//!   value), the race music's (`dword_75568C`) stored in `dword_518100` (eax the value),
//!   the race sounds' (`dword_755694`) and the menu sounds' (`dword_755690`) heard
//!   (`sound::play_menu_sound` with eax 0x80 and 1);
//! - a counter (0x468221 / 0x46823B: `dword_75880E` 0 .. 5, 0x468255 / 0x46826F:
//!   `dword_758812` 1 .. 6), one less or one more, past an end the other end (signed);
//! - 0x46FFDB: the screen mode chosen (`dword_4F50C8`) one less, before 0 the last
//!   (`dword_4F50E4` - 1); eax the new one.
//!
//! In:  nothing
//! Out: eax as above (the volumes: the value, or the sound when one is heard; the switches
//!      with a copy: the value); what a callee leaves after it; no flags read
//!
//! Which item each is: the page tables' readings (the owner named the sound page's four
//! volumes: general, race music, race effects, menu effects). Their callees through `part`: in
//! the game their Rust versions.
//! Hooks removed 2026-10-22 (typed tables): the menus run their `entry` (`menu::table_calls`).

use crate::errors::Refused;
use crate::framework::{game_read, game_write, rva};

/// After a volume step.
#[derive(Clone, Copy)]
pub enum Then {
    /// `sound::send_command_8` with eax the value.
    Send,
    /// eax the value, stored there.
    Store(usize),
    /// `sound::play_menu_sound` with eax this.
    Sound(u32),
}

/// What a handler does (see the header).
#[derive(Clone, Copy)]
pub enum Kind {
    Switch,
    /// Flipped, eax the value, `other` flipped (true) or given eax (false).
    SwitchCopy {
        other: usize,
        flip: bool,
    },
    /// 0x470017.
    SwitchModes {
        other: usize,
    },
    /// Flipped, then `menu::apply_audio_choices`.
    SwitchAudio,
    /// The quality: up (0 after 3) or down (3 before 0), then `apply_audio_choices`.
    Quality {
        up: bool,
    },
    Volume {
        up: bool,
        then: Then,
    },
    Counter {
        up: bool,
        low: i32,
        high: i32,
    },
    /// 0x46FFDB: `other` is how many modes there are.
    PreviousMode {
        other: usize,
    },
}

/// A handler: its address and length as IDA shows them, its first instructions' bytes
/// (for the test's copy), the dword it changes (IDA address) and its kind.
#[derive(Clone, Copy)]
pub struct Item {
    pub var: usize,
    pub kind: Kind,
}

fn get(x: usize) -> i32 {
    unsafe { game_read(rva(x)) }
}
fn set(x: usize, v: i32) {
    unsafe { game_write(rva(x), v) }
}

/// The Rust version (see the header). What it leaves in registers is read by none of its
/// callers (the item call sites have nothing live after the call; register liveness,
/// 2026-10-23).
pub fn option_item(item: Item) -> Result<(), Refused> {
    // Converted: `rc2k_core::menu::steps` (`flip`, `quality`, `volume`, `counter`,
    // `previous_mode`).
    use rc2k_core::menu::steps;
    let v = item.var;
    let apply_audio = || unsafe { crate::menu::option_steps::apply_audio_choices() };
    match item.kind {
        Kind::Switch => set(v, steps::flip(get(v) as u32) as i32),
        Kind::SwitchCopy { other, flip } => {
            set(v, steps::flip(get(v) as u32) as i32);
            if flip {
                set(other, steps::flip(get(other) as u32) as i32);
            } else {
                set(other, get(v));
            }
        }
        Kind::SwitchModes { other } => {
            set(v, steps::flip(get(v) as u32) as i32);
            set(other, steps::flip(get(other) as u32) as i32);
            // `gfx::screen_mode_choices` with eax 0; its flags are read by no caller.
            crate::gfx::screen_mode_choices::screen_mode_choices(false)?;
        }
        Kind::SwitchAudio => {
            set(v, steps::flip(get(v) as u32) as i32);
            apply_audio()?;
        }
        Kind::Quality { up } => {
            set(v, steps::quality(get(v) as u32, up) as i32);
            apply_audio()?;
        }
        Kind::Volume { up, then } => {
            let value = steps::volume(get(v), up);
            set(v, value);
            match then {
                Then::Send => crate::sound::send_command_8::send_command_8(value as u32),
                Then::Store(other) => set(other, value),
                Then::Sound(sound) => crate::sound::play_menu_sound::play_menu_sound_in_game(sound),
            }
        }
        Kind::Counter { up, low, high } => {
            set(v, steps::counter(get(v), up, low, high));
        }
        Kind::PreviousMode { other } => {
            set(v, steps::previous_mode(get(v), get(other)));
        }
    }
    Ok(())
}

/// A hook for one handler: `option_item!(module, "menu::name", ITEM)`.
macro_rules! option_item {
    ($module:ident, $name:literal, $item:expr) => {
        pub mod $module {
            use super::*;

            const ITEM: Item = $item;

            /// The Rust version the typed tables run (`menu::table_calls`).
            pub fn run() -> Result<(), crate::errors::Refused> {
                option_item(ITEM)
            }
        }
    };
}

macro_rules! switch {
    ($module:ident, $name:literal, $address:expr, $len:expr, $var:expr, $kind:expr) => {
        option_item!(
            $module,
            $name,
            Item {
                var: $var as usize,
                kind: $kind,
            }
        );
    };
}

macro_rules! step {
    ($module:ident, $name:literal, $address:expr, $len:expr, $var:expr, $up:expr, $kind:expr) => {
        option_item!(
            $module,
            $name,
            Item {
                var: $var as usize,
                kind: $kind,
            }
        );
    };
}

// ---- the sound page ----

switch!(
    toggle_5180f4,
    "menu::toggle_5180f4",
    0x467EF8,
    8,
    0x5180F4,
    Kind::Switch
);
step!(
    master_volume_down,
    "menu::master_volume_down",
    0x467F00,
    0x24,
    0x755684,
    false,
    Kind::Volume {
        up: false,
        then: Then::Send
    }
);
step!(
    master_volume_up,
    "menu::master_volume_up",
    0x467F24,
    0x24,
    0x755684,
    true,
    Kind::Volume {
        up: true,
        then: Then::Send
    }
);
step!(
    race_music_volume_down,
    "menu::race_music_volume_down",
    0x467F9A,
    0x24,
    0x75568C,
    false,
    Kind::Volume {
        up: false,
        then: Then::Store(0x518100)
    }
);
step!(
    race_music_volume_up,
    "menu::race_music_volume_up",
    0x467FBE,
    0x24,
    0x75568C,
    true,
    Kind::Volume {
        up: true,
        then: Then::Store(0x518100)
    }
);
step!(
    race_sounds_volume_down,
    "menu::race_sounds_volume_down",
    0x467FE2,
    0x24,
    0x755694,
    false,
    Kind::Volume {
        up: false,
        then: Then::Sound(0x80)
    }
);
step!(
    race_sounds_volume_up,
    "menu::race_sounds_volume_up",
    0x468006,
    0x24,
    0x755694,
    true,
    Kind::Volume {
        up: true,
        then: Then::Sound(0x80)
    }
);
step!(
    menu_sounds_volume_down,
    "menu::menu_sounds_volume_down",
    0x468138,
    0x24,
    0x755690,
    false,
    Kind::Volume {
        up: false,
        then: Then::Sound(1)
    }
);
step!(
    menu_sounds_volume_up,
    "menu::menu_sounds_volume_up",
    0x46815C,
    0x24,
    0x755690,
    true,
    Kind::Volume {
        up: true,
        then: Then::Sound(1)
    }
);

// ---- the stage pages' switches and counters ----

switch!(
    toggle_7556f8,
    "menu::toggle_7556f8",
    0x468211,
    8,
    0x7556F8,
    Kind::Switch
);
switch!(
    toggle_7556fc,
    "menu::toggle_7556fc",
    0x468219,
    8,
    0x7556FC,
    Kind::Switch
);
step!(
    previous_75880e,
    "menu::previous_75880e",
    0x468221,
    0x1A,
    0x75880E,
    false,
    Kind::Counter {
        up: false,
        low: 0,
        high: 5
    }
);
step!(
    next_75880e,
    "menu::next_75880e",
    0x46823B,
    0x1A,
    0x75880E,
    true,
    Kind::Counter {
        up: true,
        low: 0,
        high: 5
    }
);
step!(
    previous_758812,
    "menu::previous_758812",
    0x468255,
    0x1A,
    0x758812,
    false,
    Kind::Counter {
        up: false,
        low: 1,
        high: 6
    }
);
step!(
    next_758812,
    "menu::next_758812",
    0x46826F,
    0x1A,
    0x758812,
    true,
    Kind::Counter {
        up: true,
        low: 1,
        high: 6
    }
);

// ---- the visual page ----

switch!(
    toggle_755640,
    "menu::toggle_755640",
    0x46FFB3,
    0x14,
    0x755640,
    Kind::SwitchCopy {
        other: 0x4F50D0,
        flip: true
    }
);
switch!(
    toggle_755644,
    "menu::toggle_755644",
    0x46FFC7,
    0x14,
    0x755644,
    Kind::SwitchCopy {
        other: 0x4F50CC,
        flip: true
    }
);
option_item!(
    previous_screen_mode,
    "menu::previous_screen_mode",
    Item {
        var: 0x4F50C8,
        kind: Kind::PreviousMode { other: 0x4F50E4 },
    }
);
switch!(
    toggle_75564c,
    "menu::toggle_75564c",
    0x470017,
    0x1E,
    0x75564C,
    Kind::SwitchModes { other: 0x4F50C4 }
);
switch!(
    toggle_755650,
    "menu::toggle_755650",
    0x470035,
    0x12,
    0x755650,
    Kind::SwitchCopy {
        other: 0x75AD00,
        flip: false
    }
);
switch!(
    toggle_75565c,
    "menu::toggle_75565c",
    0x470107,
    0x12,
    0x75565C,
    Kind::SwitchCopy {
        other: 0x75AD0C,
        flip: false
    }
);
switch!(
    toggle_755660,
    "menu::toggle_755660",
    0x470119,
    0x12,
    0x755660,
    Kind::SwitchCopy {
        other: 0x75AD10,
        flip: false
    }
);
switch!(
    toggle_755668,
    "menu::toggle_755668",
    0x47018B,
    0x12,
    0x755668,
    Kind::SwitchCopy {
        other: 0x75AD18,
        flip: false
    }
);

// ---- the advanced sound page ----

option_item!(
    next_audio_quality,
    "menu::next_audio_quality",
    Item {
        var: 0x75566C,
        kind: Kind::Quality { up: true },
    }
);
option_item!(
    previous_audio_quality,
    "menu::previous_audio_quality",
    Item {
        var: 0x75566C,
        kind: Kind::Quality { up: false },
    }
);
switch!(
    toggle_ds3d,
    "menu::toggle_ds3d",
    0x4701E9,
    0xD,
    0x755670,
    Kind::SwitchAudio
);
switch!(
    toggle_eax,
    "menu::toggle_eax",
    0x4701F6,
    0xD,
    0x755674,
    Kind::SwitchAudio
);
switch!(
    toggle_a3d,
    "menu::toggle_a3d",
    0x470203,
    0xD,
    0x755678,
    Kind::SwitchAudio
);
switch!(
    toggle_dolby,
    "menu::toggle_dolby",
    0x470210,
    0xD,
    0x75567C,
    Kind::SwitchAudio
);
switch!(
    toggle_channels,
    "menu::toggle_channels",
    0x47021D,
    0xD,
    0x755680,
    Kind::SwitchAudio
);
