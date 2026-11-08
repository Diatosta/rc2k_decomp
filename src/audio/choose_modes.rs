//! `audio::choose_modes` (0x40A1CA): the two sound modes from the advanced audio screen's
//! options.
//!
//! `g_audioOptions` (`AudioOptions`, 0x40 bytes): +0 kept, +4 .. +0x18 the screen's six
//! choices (quality, channels, DS3D, EAX, A3D, Dolby), +0x1C .. +0x30 the six as last
//! applied, +0x34 3D there (`g_has3d`), +0x38 A3D2 there, +0x3C EAX there. The first call
//! fills it from `g_raceSoundMode` (quality: the first of `g_qualityModes`' three whose
//! low byte the mode has, else 3; the others from the mode's bits, set only) and the three
//! facts, the choices as the applied. Later calls apply the first choice that changed of
//! quality, channels, DS3D, EAX, A3D, Dolby, in that order and only one of the four that
//! exclude each other (DS3D on clears A3D and Dolby, off clears EAX; EAX on sets DS3D and
//! clears A3D and Dolby; A3D on clears the others; Dolby on likewise), then build the race
//! mode: the quality's mode, 0x400000 channels, DS3D 0x101000 (+ 0x800000 with free
//! hardware 3D buffers and `g_voiceManagerOn`), A3D 0x102000, EAX 0x1000000, Dolby 0x2040000; no
//! way bit: 0x10000; no device bit: 0x1000. The menu mode is 0x11221. "AUD: New modes" and
//! both logged. Callers: `audio::start`, the advanced audio screen (`menu::apply_audio_choices`,
//! 0x4707A3). Refuses a quality past `g_qualityModes` (the original reads past it).

use crate::audio::owner::audio;
use crate::audio::store::AudioState;
use crate::errors::Refused;

const NEW_MODES_TEXT: &std::ffi::CStr = c"AUD: New modes"; // "AUD: New modes"

pub use rc2k_core::audio::modes::{AudioOptions, Facts};

/// The machine's facts as the game holds them.
pub fn game_facts(state: &AudioState) -> Facts {
    Facts {
        qualities: state
            .quality_modes
            .each_ref()
            .map(super::super::session::Dword::get),
        has_3d: state.has_3d.get(),
        has_a3d2: state.has_a3d2.get(),
        has_eax: state.has_eax.get(),
        // `g_soundCaps`' free hardware 3D buffers (+0x24).
        hw_3d_buffers: state.sound_caps.get_u32(0x24),
        voice_manager_on: state.voice_manager_on.get(),
    }
}

/// The log calls: the text, the menu mode (hex8), the race mode (hex8 with a line's end).
pub type Log<'a> = &'a mut dyn FnMut(rc2k_core::audio::modes::Logged);

/// Converted: `rc2k_core::audio::modes::choose_modes`.
pub fn choose_modes(
    o: &mut AudioOptions,
    race: &mut u32,
    menu: &mut u32,
    f: &Facts,
    log: Log,
) -> Result<(), Refused> {
    Ok(rc2k_core::audio::modes::choose_modes(
        o, race, menu, f, log,
    )?)
}

/// On the game's variables, logging with `log`.
pub fn in_game(log: Log) -> Result<(), Refused> {
    let state = &audio().state;
    // The options and the two modes are worked on in place, as the original does.
    let mut o = state.audio_options.get();
    let (mut race, mut menu) = (state.race_sound_mode.get(), state.menu_sound_mode.get());
    let done = choose_modes(&mut o, &mut race, &mut menu, &game_facts(state), log);
    state.audio_options.set(o);
    state.race_sound_mode.set(race);
    state.menu_sound_mode.set(menu);
    done
}

pub fn real_log(l: rc2k_core::audio::modes::Logged) {
    use crate::audio::sound_device::{log_hex, log_text};
    use rc2k_core::audio::modes::Logged;
    match l {
        Logged::NewModes => log_text(NEW_MODES_TEXT, false),
        Logged::Menu(m) => log_hex(m, false),
        Logged::Race(m) => log_hex(m, true),
    }
}
