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

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_read, hook_asm, rva, shadow};
use crate::service::harness::{capture, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("audio::choose_modes", rva(0x40A1CA), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

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

fn regions() -> [(u32, usize); 2] {
    [
        (game_addr(OPTIONS_RVA) as u32, 0x40),
        (game_addr(MENU_MODE_RVA) as u32, 8),
    ]
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

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if !shadow() {
        if let Err(e) = in_game(&mut real_log) {
            unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) };
        }
        return;
    }
    let regions = regions();
    let before = capture(&regions);
    let mine = in_game(&mut |_, _| {});
    let after_mine = capture(&regions);
    restore(&regions, &before);
    unsafe { call_original(r) };
    let theirs = capture(&regions);
    // A refusal is the original's alone, not a mismatch.
    if mine.is_ok() && after_mine != theirs {
        HOOK.mismatch(|| {
            format!(
                "options and modes {:x?}: original {:x?}; rust {mine:?} {:x?}",
                before, theirs, after_mine
            )
        });
    }
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let regions = regions();
    let saved = capture(&regions);
    let f = game_facts();
    let mut cases = Vec::new();
    for race in [0x11221u32, 0x2441220, 0x1511220, 0x112222, 0x10220] {
        cases.push((AudioOptions::default(), race));
    }
    let base = AudioOptions {
        kept: 1,
        applied: [1, 0, 1, 0, 0, 0],
        has_3d: 1,
        ..AudioOptions::default()
    };
    for quality in [0u32, 3] {
        for n in 0..6 {
            for on in [0u32, 1] {
                let mut o = base;
                o.choice = o.applied;
                o.choice[0] = quality;
                if n > 0 {
                    o.choice[n] = on;
                }
                cases.push((o, 0));
                let mut p = o;
                p.applied = [quality, 1, 0, 1, 1, 1];
                cases.push((p, 0));
            }
        }
    }
    for (options, race) in cases {
        let set = || unsafe {
            *(game_addr(OPTIONS_RVA) as *mut AudioOptions) = options;
            *(game_addr(MENU_MODE_RVA) as *mut u32) = 0x5555;
            *(game_addr(RACE_MODE_RVA) as *mut u32) = race;
        };
        set();
        let mut r = Regs::SENTINEL;
        unsafe { call_original(&mut r) };
        let theirs = (capture(&regions), r.preserved_except(&Regs::SENTINEL, 0));
        set();
        let mut o = options;
        let (mut race_m, mut menu_m) = (race, 0x5555u32);
        let mine = choose_modes(&mut o, &mut race_m, &mut menu_m, &f, &mut |_, _| {});
        let mut expected = Vec::new();
        expected.extend(unsafe { std::slice::from_raw_parts(&o as *const _ as *const u8, 0x40) });
        expected.extend(menu_m.to_le_bytes());
        expected.extend(race_m.to_le_bytes());
        t.check(mine.is_ok() && theirs.1 && theirs.0 == expected, || {
            format!(
                "options {options:x?}, race {race:#x}: original {:x?}; rust {mine:?} {expected:x?}",
                theirs.0
            )
        });
    }
    restore(&regions, &saved);
    t.finish()
}
