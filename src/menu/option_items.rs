//! The options pages' remaining item handlers (sound and visual options pages): thirty small
//! left/right and action handlers, each named by what it changes, of these kinds: a switch (a
//! dword flipped), a switch with a copy (flipped, then a second dword flipped or given the
//! value), the modes switch (also making the screen-mode list again), the advanced sound page's
//! switches and quality (then `menu::apply_audio_choices`), a volume (one less / more within
//! 0 ..= 0x40, signed, then sent or heard), a counter (`champ_view.shown_stage.group` 0 .. 5,
//! `shown_stage` 1 .. 6, wrapping) and the screen mode chosen (one less, before 0 the last).
//!
//! Which item each is: the page tables' readings (the sound page's four volumes: general, race
//! music, race effects, menu effects).

use crate::errors::Refused;
use crate::owners::Owners;
use crate::race::settings_copy::{StepOperand, var};
use rc2k_core::menu::tables::{OptionSwitch, StageCounter, VolumeKind};

/// After a volume step.
#[derive(Clone, Copy)]
pub enum Then {
    /// `sound::send_command_8` with the value.
    Send,
    /// The value, stored there.
    Store(StepOperand),
    /// `sound::play_menu_sound` with this sound.
    Sound(u32),
}

/// What a handler does (see the header).
#[derive(Clone, Copy)]
pub enum Kind {
    Switch,
    /// Flipped, then `other` flipped (true) or given the value (false).
    SwitchCopy {
        other: StepOperand,
        flip: bool,
    },
    /// `options.modes_switch` flipped, the display's twin switch flipped, then the
    /// screen-mode list made again.
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
    /// `other` is how many modes there are.
    PreviousMode {
        other: StepOperand,
    },
}

/// A handler: the dword it changes and its kind.
#[derive(Clone, Copy)]
pub struct Item {
    pub var: StepOperand,
    pub kind: Kind,
}

/// The items name their variables by a `StepOperand`: a typed cell of an owner.
fn get(o: &Owners<'static>, x: StepOperand) -> i32 {
    x.get_with(o) as i32
}

fn set(o: &Owners<'static>, x: StepOperand, v: i32) {
    x.set_with(o, v as u32);
}

pub fn option_item(o: Owners<'static>, item: Item) -> Result<(), Refused> {
    let audio = o.audio;
    // The logic is in `rc2k_core::menu::steps` (`flip`, `quality`, `volume`, `counter`,
    // `previous_mode`).
    use rc2k_core::menu::steps;
    let v = item.var;
    let apply_audio = || crate::menu::option_steps::apply_audio_choices(o.menus, audio);
    match item.kind {
        Kind::Switch => set(&o, v, steps::flip(get(&o, v) as u32) as i32),
        Kind::SwitchCopy { other, flip } => {
            set(&o, v, steps::flip(get(&o, v) as u32) as i32);
            if flip {
                set(&o, other, steps::flip(get(&o, other) as u32) as i32);
            } else {
                set(&o, other, get(&o, v));
            }
        }
        Kind::SwitchModes { other } => {
            set(&o, v, steps::flip(get(&o, v) as u32) as i32);
            set(&o, other, steps::flip(get(&o, other) as u32) as i32);
            // `gfx::screen_mode_choices` makes the list again; its flags are read by no caller.
            crate::gfx::screen_mode_choices::screen_mode_choices(
                o.gfx, &o.gfx.d3d, o.system, false,
            )?;
        }
        Kind::SwitchAudio => {
            set(&o, v, steps::flip(get(&o, v) as u32) as i32);
            apply_audio()?;
        }
        Kind::Quality { up } => {
            set(&o, v, steps::quality(get(&o, v) as u32, up) as i32);
            apply_audio()?;
        }
        Kind::Volume { up, then } => {
            let value = steps::volume(get(&o, v), up);
            set(&o, v, value);
            match then {
                Then::Send => crate::sound::send_command_8::send_command_8(o.audio, value as u32),
                Then::Store(other) => set(&o, other, value),
                Then::Sound(sound) => {
                    crate::sound::play_menu_sound::play_menu_sound_in_game(
                        o.sound,
                        o.audio,
                        o.menus,
                        &o.system.session.picture_clock,
                        sound,
                    );
                }
            }
        }
        Kind::Counter { up, low, high } => {
            set(&o, v, steps::counter(get(&o, v), up, low, high));
        }
        Kind::PreviousMode { other } => {
            set(&o, v, steps::previous_mode(get(&o, v), get(&o, other)));
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
            pub fn run(o: Owners<'static>) -> Result<(), crate::errors::Refused> {
                option_item(o, ITEM)
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
/// pages (the first four on the sound page: general, race music, race effects, menu effects;
/// the others on the volume page).
pub fn volume(o: Owners<'static>, which: VolumeKind, up: bool) -> Result<(), Refused> {
    use crate::menu::option_steps::{Then as StepThen, Volume, volume_step};
    use std::sync::atomic::Ordering::Relaxed;
    if which == VolumeKind::Film {
        // The films' volume lives with the platform's sound (the films are played there).
        let now = crate::platform::sound::FILM_VOLUME.load(Relaxed) as i32;
        let next = rc2k_core::menu::steps::volume(now, up);
        crate::platform::sound::FILM_VOLUME.store(next as u32, Relaxed);
        return Ok(());
    }
    let work = &o.menus.options;
    let item = match which {
        VolumeKind::General => volume_item(var!(@o audio.levels.general_volume), up, Then::Send),
        VolumeKind::RaceMusic => volume_item(
            var!(@o menus.options.race_music_level),
            up,
            Then::Store(var!(@o sound.menu.options.race_music_volume)),
        ),
        VolumeKind::RaceEffects => volume_item(
            var!(@o audio.levels.race_effects_volume),
            up,
            Then::Sound(0x80),
        ),
        VolumeKind::MenuEffects => volume_item(
            var!(@o menus.options.menu_effects_volume),
            up,
            Then::Sound(1),
        ),
        // The volumes the page's own steps handle (`menu::option_steps::volume_step`).
        VolumeKind::MenuMusic => {
            volume_step(
                o,
                Volume {
                    value: &work.menu_music_volume,
                    up,
                    then: StepThen::Music,
                },
            );
            return Ok(());
        }
        VolumeKind::MenuVoices => {
            volume_step(
                o,
                Volume {
                    value: &work.menu_voices_volume,
                    up,
                    then: StepThen::Speech,
                },
            );
            return Ok(());
        }
        VolumeKind::CodriverVoice => {
            // Heard with one sound going down, another going up.
            let sound = if up { 0x20 } else { 0x10 };
            volume_step(
                o,
                Volume {
                    value: &o.audio.levels.codriver_voice_volume,
                    up,
                    then: StepThen::Sound(sound),
                },
            );
            return Ok(());
        }
        VolumeKind::Film => return Ok(()),
        VolumeKind::Engine => {
            volume_step(
                o,
                Volume {
                    value: &o.audio.levels.engine_volume,
                    up,
                    then: StepThen::Sound(0x40),
                },
            );
            return Ok(());
        }
    };
    option_item(o, item)
}

/// `ItemFn::OptionSwitch(which)`: the switch flipped; the visual ones also flip or copy
/// the setting the game reads, the audio ones apply the audio choices.
pub fn switch(o: Owners<'static>, which: OptionSwitch) -> Result<(), Refused> {
    if which == OptionSwitch::SoundInBackground {
        use std::sync::atomic::Ordering::Relaxed;
        let on = &crate::platform::sound::SOUND_IN_BACKGROUND;
        on.store(!on.load(Relaxed), Relaxed);
        return Ok(());
    }
    let copy = |other: StepOperand| Kind::SwitchCopy { other, flip: false };
    let flip = |other: StepOperand| Kind::SwitchCopy { other, flip: true };
    let (var, kind) = match which {
        OptionSwitch::RaceMusicOn => (var!(@o sound.menu.options.race_music_on), Kind::Switch),
        OptionSwitch::StageRecordsSet => (
            var!(@o menus.play.champ_view.switches.records_set),
            Kind::Switch,
        ),
        OptionSwitch::StageRecordsAlt => (
            var!(@o menus.play.champ_view.switches.records_alt),
            Kind::Switch,
        ),
        OptionSwitch::Mipmapping => (
            var!(@o menus.options.mipmapping),
            flip(var!(@o gfx.display.choices.mipmapping)),
        ),
        OptionSwitch::Buffering => (
            var!(@o menus.options.buffering),
            flip(var!(@o gfx.display.choices.buffering)),
        ),
        OptionSwitch::ScreenModeCard => (
            var!(@o menus.options.modes_switch),
            Kind::SwitchModes {
                other: var!(@o gfx.display.choices.card),
            },
        ),
        OptionSwitch::LensFlare => (
            var!(@o menus.options.lens_flare),
            copy(var!(@o race.shared.state.settings.lens_flare
            )),
        ),
        OptionSwitch::Reflections => (
            var!(@o menus.options.reflections),
            copy(var!(@o race.shared.state.settings.reflections
            )),
        ),
        OptionSwitch::CarShadows => (
            var!(@o menus.options.car_shadows),
            copy(var!(@o race.shared.state.settings.car_shadows
            )),
        ),
        OptionSwitch::Headlights => (
            var!(@o menus.options.headlights),
            copy(var!(@o race.shared.state.settings.headlights
            )),
        ),
        OptionSwitch::Ds3d => (var!(@o menus.options.ds3d), Kind::SwitchAudio),
        OptionSwitch::Eax => (var!(@o menus.options.eax), Kind::SwitchAudio),
        OptionSwitch::Dolby => (var!(@o menus.options.dolby), Kind::SwitchAudio),
        OptionSwitch::Channels => (var!(@o menus.options.channels), Kind::SwitchAudio),
        OptionSwitch::SoundInBackground => return Ok(()),
    };
    option_item(o, Item { var, kind })
}

/// `ItemFn::StageCounter { which, up }`: the stage group (0 .. 5) or the stage in its group
/// (1 .. 6) shown by the stage pages, one less or one more, past an end the other end.
pub fn stage_counter(o: Owners<'static>, which: StageCounter, up: bool) -> Result<(), Refused> {
    let (var, low, high) = match which {
        StageCounter::Group => (var!(@o menus.play.champ_view.shown_stage.group), 0, 5),
        StageCounter::InGroup => (var!(@o menus.play.champ_view.shown_stage.number), 1, 6),
    };
    option_item(
        o,
        Item {
            var,
            kind: Kind::Counter { up, low, high },
        },
    )
}

option_item!(
    previous_screen_mode,
    "menu::previous_screen_mode",
    Item {
        var: var!(@o gfx.display.choices.resolution),
        kind: Kind::PreviousMode {
            other: var!(@o gfx.display.choices.resolutions)
        },
    }
);
option_item!(
    next_audio_quality,
    "menu::next_audio_quality",
    Item {
        var: var!(@o menus.options.audio_quality),
        kind: Kind::Quality { up: true },
    }
);
option_item!(
    previous_audio_quality,
    "menu::previous_audio_quality",
    Item {
        var: var!(@o menus.options.audio_quality),
        kind: Kind::Quality { up: false },
    }
);
