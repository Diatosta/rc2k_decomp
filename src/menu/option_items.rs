//! The options pages' remaining item handlers (on the sound and visual options pages): thirty small left/right
//! and action handlers, each named by what it changes. What each does, by its kind:
//! - a switch: a dword flipped (`xor .., 1`), nothing else (0x467EF8 a game flag,
//!   0x468211 `champ_view.switch_f8`, 0x468219 `switch_fc`; what they switch is not read);
//! - a switch with a copy: flipped, eax its value, then a second dword flipped too
//!   (0x46FFB3, 0x46FFC7) or given eax (0x470035, 0x470107, 0x470119, 0x47018B: the
//!   settings the game reads);
//! - 0x470017: `option_work.modes_switch` flipped, eax its value, the display's twin switch flipped, then the
//!   screen-mode list made again (`gfx::screen_mode_choices` with eax 0);
//! - the advanced sound page's switches (0x4701E9 DS3D, 0x4701F6 EAX, 0x470203 A3D,
//!   0x470210 Dolby, 0x47021D the channels): the work value flipped and
//!   `menu::apply_audio_choices`; its quality (0x47019F one more, 0 after 3; 0x4701C4 one
//!   less, 3 before 0), then the same;
//! - a volume: its work value one less (below 0: 0) or one more (above 0x40: 0x40), signed;
//!   then the master's (`g_masterVolumeShown`) is sent (`sound::send_command_8`, eax the
//!   value), the race music's (`option_work.other_volume`) stored in the game's music volume (eax the value),
//!   the race sounds' (`menu_sounds_a`) and the menu sounds' (`menu_sounds_c`) heard
//!   (`sound::play_menu_sound` with eax 0x80 and 1);
//! - a counter (0x468221 / 0x46823B: `captions.shown_group` 0 .. 5, 0x468255 / 0x46826F:
//!   `captions.shown_stage` 1 .. 6), one less or one more, past an end the other end (signed);
//! - 0x46FFDB: the screen mode chosen one less, before 0 the last; eax the new one.
//!
//! Result as above (the volumes: the value, or the sound when one is heard; the switches
//! with a copy: the value).
//!
//! Which item each is: the page tables' readings (the owner named the sound page's four volumes:
//! general, race music, race effects, menu effects).

use crate::audio::owner::AUDIO_OWNER;
use crate::errors::Refused;
use crate::gfx::owner::GFX;
use crate::gfx::owner::gfx;
use crate::menu::owner::MENUS;
use crate::race::settings_copy::{StepOperand, var};
use crate::sound::owner::SOUND;
use rc2k_core::menu::tables::{OptionSwitch, StageCounter, VolumeKind};

/// After a volume step.
#[derive(Clone, Copy)]
pub enum Then {
    /// `sound::send_command_8` with eax the value.
    Send,
    /// eax the value, stored there.
    Store(StepOperand),
    /// `sound::play_menu_sound` with eax this.
    Sound(u32),
}

/// What a handler does (see the header).
#[derive(Clone, Copy)]
pub enum Kind {
    Switch,
    /// Flipped, eax the value, `other` flipped (true) or given eax (false).
    SwitchCopy {
        other: StepOperand,
        flip: bool,
    },
    /// 0x470017.
    SwitchModes {
        other: StepOperand,
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
        other: StepOperand,
    },
}

/// A handler: its address and length as IDA shows them, its first instructions' bytes
/// (for the test's copy), the dword it changes (IDA address) and its kind.
#[derive(Clone, Copy)]
pub struct Item {
    pub var: StepOperand,
    pub kind: Kind,
}

/// The items name their variables by a `StepOperand`: a typed cell, or an IDA address
/// (`moved::place` answers where each lives now) for what is not typed yet.
fn get(x: StepOperand) -> i32 {
    x.get() as i32
}

fn set(x: StepOperand, v: i32) {
    x.set(v as u32);
}

pub fn option_item(item: Item) -> Result<(), Refused> {
    let audio = crate::audio::owner::audio();
    // The logic is in `rc2k_core::menu::steps` (`flip`, `quality`, `volume`, `counter`,
    // `previous_mode`).
    use rc2k_core::menu::steps;
    let v = item.var;
    let apply_audio = || crate::menu::option_steps::apply_audio_choices(audio);
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
            crate::gfx::screen_mode_choices::screen_mode_choices(&gfx().display, false)?;
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

/// One handler: `option_item!(module, "menu::name", ITEM)`.
macro_rules! option_item {
    ($module:ident, $name:literal, $item:expr) => {
        pub mod $module {
            use super::*;

            const ITEM: Item = $item;

            /// Run by the typed tables (`menu::table_calls`).
            pub fn run() -> Result<(), crate::errors::Refused> {
                option_item(ITEM)
            }
        }
    };
}

// ---- the items of the options pages (parameterised: `ItemFn::OptionSwitch`, `Volume`,
// `StageCounter`) ----

/// A volume step on `var`, then `then` (sent, kept or heard).
fn volume_item(var: StepOperand, up: bool, then: Then) -> Item {
    Item {
        var,
        kind: Kind::Volume { up, then },
    }
}

/// `ItemFn::Volume { which, up }`: the volume `which` one step up or down on the sound
/// pages (the first four on the sound page, the owner's names: general, race music, race
/// effects, menu effects; the others on the volume page).
pub fn volume(
    menus: &crate::menu::owner::Menus,
    which: VolumeKind,
    up: bool,
) -> Result<(), Refused> {
    use crate::menu::option_steps::{
        MUSIC_LEVEL_RVA, SOUNDS_A_RVA, SOUNDS_B_RVA, SPEECH_LEVEL_RVA, Then as StepThen, Volume,
        volume_step,
    };
    let item = match which {
        VolumeKind::Master => {
            volume_item(var!(AUDIO_OWNER.levels.master_volume_shown), up, Then::Send)
        }
        VolumeKind::RaceMusic => volume_item(
            var!(MENUS.screens.option_work.other_volume),
            up,
            Then::Store(var!(SOUND.menu.options.race_music_volume)),
        ),
        VolumeKind::RaceSounds => volume_item(
            var!(AUDIO_OWNER.levels.menu_sounds_a),
            up,
            Then::Sound(0x80),
        ),
        VolumeKind::MenuSounds => volume_item(
            var!(MENUS.screens.option_work.menu_sounds_c),
            up,
            Then::Sound(1),
        ),
        // The volumes the page's own steps handle (`menu::option_steps::volume_step`).
        VolumeKind::Music => {
            volume_step(
                &menus.screens,
                Volume {
                    value: MUSIC_LEVEL_RVA,
                    up,
                    then: StepThen::Music,
                },
            );
            return Ok(());
        }
        VolumeKind::Speech => {
            volume_step(
                &menus.screens,
                Volume {
                    value: SPEECH_LEVEL_RVA,
                    up,
                    then: StepThen::Speech,
                },
            );
            return Ok(());
        }
        VolumeKind::MenuSoundsA => {
            // Heard with one sound going down, another going up.
            let sound = if up { 0x20 } else { 0x10 };
            volume_step(
                &menus.screens,
                Volume {
                    value: SOUNDS_A_RVA,
                    up,
                    then: StepThen::Sound(sound),
                },
            );
            return Ok(());
        }
        VolumeKind::MenuSoundsB => {
            volume_step(
                &menus.screens,
                Volume {
                    value: SOUNDS_B_RVA,
                    up,
                    then: StepThen::Sound(0x40),
                },
            );
            return Ok(());
        }
    };
    option_item(item)
}

/// `ItemFn::OptionSwitch(which)`: the switch flipped; the visual ones also flip or copy
/// the setting the game reads, the audio ones apply the audio choices.
pub fn switch(which: OptionSwitch) -> Result<(), Refused> {
    let copy = |other: StepOperand| Kind::SwitchCopy { other, flip: false };
    let flip = |other: StepOperand| Kind::SwitchCopy { other, flip: true };
    let (var, kind) = match which {
        OptionSwitch::RaceMusicOn => (var!(SOUND.menu.options.race_music_on), Kind::Switch),
        OptionSwitch::StageRecordsSet => (var!(MENUS.play.champ_view.switch_f8), Kind::Switch),
        OptionSwitch::StageRecordsAlt => (var!(MENUS.play.champ_view.switch_fc), Kind::Switch),
        OptionSwitch::Mipmapping => (
            var!(MENUS.screens.option_work.switch_0),
            flip(var!(GFX.display.choices.mipmapping)),
        ),
        OptionSwitch::Buffering => (
            var!(MENUS.screens.option_work.switch_1),
            flip(var!(GFX.display.choices.buffering)),
        ),
        OptionSwitch::ScreenModeCard => (
            var!(MENUS.screens.option_work.modes_switch),
            Kind::SwitchModes {
                other: var!(GFX.display.choices.card),
            },
        ),
        OptionSwitch::LensFlare => (
            var!(MENUS.screens.option_work.lens_flare),
            copy(var!(
                crate::race::owner::RACE.shared.state.settings.lens_flare
            )),
        ),
        OptionSwitch::Reflections => (
            var!(MENUS.screens.option_work.reflections),
            copy(var!(
                crate::race::owner::RACE.shared.state.settings.reflections
            )),
        ),
        OptionSwitch::CarShadows => (
            var!(MENUS.screens.option_work.car_shadows),
            copy(var!(
                crate::race::owner::RACE.shared.state.settings.car_shadows
            )),
        ),
        OptionSwitch::Headlights => (
            var!(MENUS.screens.option_work.headlights),
            copy(var!(
                crate::race::owner::RACE.shared.state.settings.headlights
            )),
        ),
        OptionSwitch::Ds3d => (var!(MENUS.screens.option_work.ds3d), Kind::SwitchAudio),
        OptionSwitch::Eax => (var!(MENUS.screens.option_work.eax), Kind::SwitchAudio),
        OptionSwitch::A3d => (var!(MENUS.screens.option_work.a3d), Kind::SwitchAudio),
        OptionSwitch::Dolby => (var!(MENUS.screens.option_work.dolby), Kind::SwitchAudio),
        OptionSwitch::Channels => (var!(MENUS.screens.option_work.channels), Kind::SwitchAudio),
    };
    option_item(Item { var, kind })
}

/// `ItemFn::StageCounter { which, up }`: the stage group (0 .. 5) or the stage in its group
/// (1 .. 6) shown by the stage pages, one less or one more, past an end the other end.
pub fn stage_counter(which: StageCounter, up: bool) -> Result<(), Refused> {
    let (var, low, high) = match which {
        StageCounter::Group => (var!(MENUS.tables.captions.shown_group), 0, 5),
        StageCounter::InGroup => (var!(MENUS.tables.captions.shown_stage), 1, 6),
    };
    option_item(Item {
        var,
        kind: Kind::Counter { up, low, high },
    })
}

// ---- the visual and advanced sound pages' remaining items ----

option_item!(
    previous_screen_mode,
    "menu::previous_screen_mode",
    Item {
        var: var!(GFX.display.choices.resolution),
        kind: Kind::PreviousMode {
            other: var!(GFX.display.choices.resolutions)
        },
    }
);
option_item!(
    next_audio_quality,
    "menu::next_audio_quality",
    Item {
        var: var!(MENUS.screens.option_work.audio_quality),
        kind: Kind::Quality { up: true },
    }
);
option_item!(
    previous_audio_quality,
    "menu::previous_audio_quality",
    Item {
        var: var!(MENUS.screens.option_work.audio_quality),
        kind: Kind::Quality { up: false },
    }
);
