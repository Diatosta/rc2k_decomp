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
//! Compared in shadow mode (the options and both modes; the Rust version's log calls noted,
//! not written). Self-test: the original on options of the test's, with the real log
//!, 53 cases.
//!
//! Hook removed 2026-10-22 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{game_addr, game_read, rva};

pub const OPTIONS_RVA: usize = rva(0x506C88); // g_audioOptions
pub const QUALITY_MODES_RVA: usize = rva(0x4F8294); // g_qualityModes: 4 dwords
pub const MENU_MODE_RVA: usize = rva(0x4F82C4); // g_menuSoundMode
pub const RACE_MODE_RVA: usize = rva(0x4F82C8); // g_raceSoundMode
const HAS_3D_RVA: usize = rva(0x506B46); // g_has3d (unaligned)
const HAS_A3D2_RVA: usize = rva(0x506D5C); // g_hasA3d2
const HAS_EAX_RVA: usize = rva(0x506C70); // g_hasEax
const HW_3D_BUFFERS_RVA: usize = rva(0x506ACC); // g_soundCaps' free hardware 3D buffers
const VOICE_MANAGER_ON_RVA: usize = rva(0x506C54); // g_voiceManagerOn
const NEW_MODES_TEXT: usize = rva(0x4DA6BE); // "AUD: New modes"

pub use rc2k_core::audio::modes::{AudioOptions, Facts};

/// The machine's facts as the game holds them.
pub fn game_facts() -> Facts {
    unsafe {
        Facts {
            qualities: game_read(QUALITY_MODES_RVA),
            has_3d: (game_addr(HAS_3D_RVA) as *const u32).read_unaligned(),
            has_a3d2: game_read(HAS_A3D2_RVA),
            has_eax: game_read(HAS_EAX_RVA),
            hw_3d_buffers: game_read(HW_3D_BUFFERS_RVA),
            voice_manager_on: game_read(VOICE_MANAGER_ON_RVA),
        }
    }
}

/// The log calls: (0 text / 2 hex8 / 3 hex8 with a line's end, eax), as
/// `sound_device::Part`'s places.
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
            Logged::NewModes => log(0, game_addr(NEW_MODES_TEXT) as u32),
            Logged::Menu(m) => log(2, m),
            Logged::Race(m) => log(3, m),
        },
    )?)
}

/// On the game's variables, logging with `log`.
pub fn in_game(log: Log) -> Result<(), Refused> {
    unsafe {
        let o = &mut *(game_addr(OPTIONS_RVA) as *mut AudioOptions);
        let menu = &mut *(game_addr(MENU_MODE_RVA) as *mut u32);
        let race = &mut *(game_addr(RACE_MODE_RVA) as *mut u32);
        choose_modes(o, race, menu, &game_facts(), log)
    }
}

pub fn real_log(place: u32, eax: u32) {
    let _ = crate::audio::sound_device::game_part(place, eax);
}
