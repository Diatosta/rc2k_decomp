//! `scene::set_up_lit_engine` (0x415515): `scene::set_up_engine`, `race::
//! find_present_courses`, the light's settings (`g_lightLowest` 0x50, `g_lightHighest`
//! 0xFF, `g_lightLevel` and `g_lightStrength` 0xB4, `dword_635094` 0x6E, `dword_635098`
//! 0x1E) and `scene::fill_light_tables`, all inside `pusha` / `popa`.
//!
//! In:  nothing
//! Out: every register kept; no flags read
//!
//! Callers: `menu::play_films` and `scene::rebuild_front_end`, which call the Rust version
//! directly. Its calls by place: 0 `set_up_engine`, 1 `find_present_courses`, 2
//! `fill_light_tables`. It calls in its first five bytes: a private copy (its three calls
//! relocated) with a hand-made patch.
//! Refuses what its calls refuse (`find_present_courses` then runs in the game, through
//! its hook). Shadow mode runs the original alone (`set_up_engine` sets up the cars).
//! Self-test: a copy whose calls go to stand-ins that note them, against the Rust version
//! with the same, on the game's light settings (put back).

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_write, hook_asm, rva, shadow};
use crate::race::noted_calls::{Part, both, copy};
use crate::service::harness::{Region, capture, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("scene::set_up_lit_engine", rva(0x415515), thunk, &ORIGINAL)
        .private_copy(0x4E)
        .relocate_calls(&[0x1, 0x6, 0x47])
        .manual_patch()
        .self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

pub const ENGINE: u32 = 0;
pub const COURSES: u32 = 1;
pub const LIGHTS: u32 = 2;
const PLACES: [(usize, u32); 3] = [(0x442AF6, ENGINE), (0x4394F8, COURSES), (0x44F0AB, LIGHTS)];

/// `g_lightLowest`, `g_lightHighest`, `g_lightLevel`, `g_lightStrength`, `dword_635094`,
/// `dword_635098` and their values.
const SETTINGS: [(usize, u32); 6] = [
    (0x635084, 0x50),
    (0x635088, 0xFF),
    (0x63508C, 0xB4),
    (0x635090, 0xB4),
    (0x635094, 0x6E),
    (0x635098, 0x1E),
];

/// The steps (the registers kept).
pub fn set_up_lit_engine_in(r: &mut Regs, part: Part) -> Result<(), Refused> {
    let mut g = *r;
    part(ENGINE, &mut g)?;
    part(COURSES, &mut g)?;
    for (at, v) in SETTINGS {
        unsafe { game_write(rva(at), v) };
    }
    part(LIGHTS, &mut g)
}

/// The calls in the game: the Rust versions.
pub fn lit_part(place: u32, r: &mut Regs) -> Result<(), Refused> {
    unsafe {
        match place {
            ENGINE => crate::scene::set_up_engine::set_up_engine_in_game(r),
            COURSES => match crate::race::find_present_courses::find_present_courses(r) {
                Ok(left) => left.apply(r),
                // Its hook refuses it again and runs the original.
                Err(_) => crate::calls::call(rva(0x4394F8), r),
            },
            _ => crate::scene::fill_light_tables::fill_light_tables().apply(r),
        }
    }
    Ok(())
}

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    let mut g = *r;
    if let Err(e) = set_up_lit_engine_in(&mut g, &mut lit_part) {
        unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) };
    }
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    let mut g = *r;
    set_up_lit_engine_in(&mut g, &mut lit_part)?;
    Ok(())
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let copy = match copy(
        0x415515,
        0x4E,
        &[0x60, 0xE8, 0xDB, 0xD5, 0x02, 0x00],
        &[0x1, 0x6, 0x47],
        &PLACES,
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    let all: Vec<Region> = SETTINGS
        .iter()
        .map(|&(a, _)| (game_addr(rva(a)) as u32, 4))
        .collect();
    let saved = capture(&all);
    both(
        &mut t,
        "the settings",
        &copy,
        Regs::SENTINEL,
        &|| {
            restore(&all, &saved);
            for (a, _) in SETTINGS {
                unsafe { game_write(rva(a), 0xAAAA_AAAAu32) };
            }
        },
        &all,
        &set_up_lit_engine_in,
    );
    restore(&all, &saved);
    drop(copy);
    t.finish()
}
