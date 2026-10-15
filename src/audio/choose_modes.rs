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
//! (unpaused), 53 cases.

use crate::errors::{BadInput, Refused};
use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_read, hook_asm, rva, shadow};
use crate::service::harness::{capture, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new("audio::choose_modes", rva(0x40A1CA), thunk, &ORIGINAL)
    .self_test(self_test)
    .self_test_unpaused();

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
const QUALITIES: u32 = 4;

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct AudioOptions {
    pub kept: u32,
    /// The screen's choices: quality, channels, DS3D, EAX, A3D, Dolby.
    pub choice: [u32; 6],
    /// The same, as last applied.
    pub applied: [u32; 6],
    pub has_3d: u32,
    pub has_a3d2: u32,
    pub has_eax: u32,
}

const _: () = assert!(std::mem::size_of::<AudioOptions>() == 0x40);

const QUALITY: usize = 0;
const CHANNELS: usize = 1;
const DS3D: usize = 2;
const EAX: usize = 3;
const A3D: usize = 4;
const DOLBY: usize = 5;

/// What it reads of the machine.
#[derive(Clone, Copy)]
pub struct Facts {
    pub qualities: [u32; 4],
    pub has_3d: u32,
    pub has_a3d2: u32,
    pub has_eax: u32,
    pub hw_3d_buffers: u32,
    pub voice_manager_on: u32,
}

impl Facts {
    pub fn game() -> Self {
        unsafe {
            Self {
                qualities: game_read(QUALITY_MODES_RVA),
                has_3d: (game_addr(HAS_3D_RVA) as *const u32).read_unaligned(),
                has_a3d2: game_read(HAS_A3D2_RVA),
                has_eax: game_read(HAS_EAX_RVA),
                hw_3d_buffers: game_read(HW_3D_BUFFERS_RVA),
                voice_manager_on: game_read(VOICE_MANAGER_ON_RVA),
            }
        }
    }
}

/// The log calls: (0 text / 2 hex8 / 3 hex8 with a line's end, eax), as
/// `sound_device::Part`'s places.
pub type Log<'a> = &'a mut dyn FnMut(u32, u32);

pub fn choose_modes(
    o: &mut AudioOptions,
    race: &mut u32,
    menu: &mut u32,
    f: &Facts,
    log: Log,
) -> Result<(), Refused> {
    if o.kept != 1 {
        o.kept = 1;
        let mode = *race;
        o.applied[QUALITY] = (0..3u32)
            .find(|&n| (mode ^ f.qualities[n as usize]) & 0xFF == 0)
            .unwrap_or(3);
        let mut set = |n: usize, on: bool| {
            if on {
                o.applied[n] = 1;
            }
        };
        set(CHANNELS, mode & 0x40_0000 != 0);
        set(DS3D, mode & 0x10_1000 == 0x10_1000);
        set(EAX, mode & 0x100_0000 != 0);
        set(A3D, mode & 0x2000 != 0);
        set(DOLBY, mode & 0x200_0000 != 0);
        o.has_3d = f.has_3d;
        o.has_a3d2 = f.has_a3d2;
        o.has_eax = f.has_eax;
        o.choice = o.applied;
        return Ok(());
    }
    if o.choice[QUALITY] >= QUALITIES {
        return Err(BadInput("a sound quality past the table").into());
    }
    let (c, a) = (o.choice, &mut o.applied);
    a[QUALITY] = c[QUALITY];
    a[CHANNELS] = c[CHANNELS];
    if c[DS3D] != a[DS3D] {
        a[DS3D] = c[DS3D];
        if c[DS3D] != 0 {
            a[A3D] = 0;
            a[DOLBY] = 0;
        } else {
            a[EAX] = 0;
        }
    } else if c[EAX] != a[EAX] && {
        a[EAX] = c[EAX];
        c[EAX] != 0
    } {
        a[DS3D] = 1;
        a[A3D] = 0;
        a[DOLBY] = 0;
    } else if c[A3D] != a[A3D] && {
        a[A3D] = c[A3D];
        c[A3D] != 0
    } {
        a[DS3D] = 0;
        a[EAX] = 0;
        a[DOLBY] = 0;
    } else if c[DOLBY] != a[DOLBY] {
        a[DOLBY] = c[DOLBY];
        if c[DOLBY] != 0 {
            a[DS3D] = 0;
            a[EAX] = 0;
            a[A3D] = 0;
        }
    }
    let mut mode = f.qualities[a[QUALITY] as usize];
    if a[CHANNELS] != 0 {
        mode |= 0x40_0000;
    }
    if a[DS3D] != 0 {
        mode |= 0x10_1000;
        if f.hw_3d_buffers != 0 && f.voice_manager_on == 1 {
            mode |= 0x80_0000;
        }
    }
    if a[A3D] != 0 {
        mode |= 0x10_2000;
    }
    if a[EAX] != 0 {
        mode |= 0x100_0000;
    }
    if a[DOLBY] != 0 {
        mode |= 0x204_0000;
    }
    if mode & 0xF_0000 == 0 {
        mode |= 0x1_0000;
    }
    if mode & 0xF000 == 0 {
        mode |= 0x1000;
    }
    *race = mode;
    *menu = 0x1_1221;
    log(0, game_addr(NEW_MODES_TEXT) as u32);
    log(2, *menu);
    log(3, *race);
    Ok(())
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
        choose_modes(o, race, menu, &Facts::game(), log)
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
    let f = Facts::game();
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
