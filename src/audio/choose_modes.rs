//! `audio::choose_modes` (0x40A1CA): the two sound modes from the advanced audio screen's
//! options.
//!
//! In:  nothing
//! Out: every register preserved
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
//! both logged. Callers: `audio::start`, the advanced audio screen (0x4707A3).
//! Refuses a quality past `g_qualityModes` (the original reads past it).

use crate::audio::store::AUDIO;
use crate::errors::Refused;

const NEW_MODES_TEXT: &std::ffi::CStr = c"AUD: New modes"; // "AUD: New modes"

pub use rc2k_core::audio::modes::{AudioOptions, Facts};

/// The machine's facts as the game holds them.
pub fn game_facts() -> Facts {
    Facts {
        qualities: AUDIO.quality_modes.each_ref().map(|d| d.get()),
        has_3d: AUDIO.has_3d.get(),
        has_a3d2: AUDIO.has_a3d2.get(),
        has_eax: AUDIO.has_eax.get(),
        // `g_soundCaps`' free hardware 3D buffers (+0x24).
        hw_3d_buffers: AUDIO.sound_caps.get_u32(0x24),
        voice_manager_on: AUDIO.voice_manager_on.get(),
    }
}

/// The log calls: (0 text / 2 hex8 / 3 hex8 with a line's end, eax), as
/// `choose_modes::real_log`'s places.
pub type Log<'a> = &'a mut dyn FnMut(u32, u32);

/// Converted: `rc2k_core::audio::modes::choose_modes`.
pub fn choose_modes(
    o: &mut AudioOptions,
    race: &mut u32,
    menu: &mut u32,
    f: &Facts,
    log: Log,
) -> Result<(), Refused> {
    use rc2k_core::audio::modes::Logged;
    Ok(rc2k_core::audio::modes::choose_modes(
        o,
        race,
        menu,
        f,
        |l| match l {
            Logged::NewModes => log(0, crate::pointers::game_address(NEW_MODES_TEXT.as_ptr())),
            Logged::Menu(m) => log(2, m),
            Logged::Race(m) => log(3, m),
        },
    )?)
}

/// On the game's variables, logging with `log`.
pub fn in_game(log: Log) -> Result<(), Refused> {
    // The options and the two modes are worked on in place, as the original does.
    let mut o = AUDIO.audio_options.get();
    let (mut race, mut menu) = (AUDIO.race_sound_mode.get(), AUDIO.menu_sound_mode.get());
    let done = choose_modes(&mut o, &mut race, &mut menu, &game_facts(), log);
    AUDIO.audio_options.set(o);
    AUDIO.race_sound_mode.set(race);
    AUDIO.menu_sound_mode.set(menu);
    done
}

pub fn real_log(place: u32, eax: u32) {
    use crate::audio::sound_device::{log_hex, log_text};
    match place {
        0 => log_text(eax, false),
        2 => log_hex(eax, false),
        _ => log_hex(eax, true),
    }
}
