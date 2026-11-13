//! Chooses the two sound modes (menu and race) from the advanced audio screen's options and the
//! machine's facts. The first call fills the options from the race mode (the quality: the first of
//! three quality modes whose low byte the mode has, else 3); later calls apply the first choice
//! that changed (quality, channels, DS3D, EAX, A3D, Dolby), keeping DS3D, EAX, A3D and Dolby
//! exclusive as the screen does, then build the race mode. "AUD: New modes" and both modes are
//! logged. Refuses a quality past the quality modes.

use crate::audio::owner::Audio;
use crate::audio::store::AudioShared;
use crate::errors::Refused;

const NEW_MODES_TEXT: &std::ffi::CStr = c"AUD: New modes"; // "AUD: New modes"

pub use rc2k_core::audio::modes::{AudioOptions, Facts};

/// The machine's facts.
pub fn game_facts(state: &AudioShared) -> Facts {
    Facts {
        qualities: state
            .quality_modes
            .each_ref()
            .map(super::super::session::Dword::get),
        has_3d: state.has_3d.get(),
        has_a3d2: state.has_a3d2.get(),
        has_eax: state.has_eax.get(),
        // Free hardware 3D buffers.
        hw_3d_buffers: state.sound_caps.get_u32(0x24),
        voice_manager_on: state.voice_manager_on.get(),
    }
}

/// The log calls: the text, the menu mode (hex8), the race mode (hex8 with a line's end).
pub type Log<'a> = &'a mut dyn FnMut(rc2k_core::audio::modes::Logged);

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
pub fn in_game(audio: &Audio, log: Log) -> Result<(), Refused> {
    let state = &audio.state;
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
