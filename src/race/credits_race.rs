//! `race::credits_race` (0x463666): the credits' race (the demonstration the films play
//! between them): `dword_5C5CE0` 0, `g_holdCarRevs` 0, `dword_60519C` 0x12345678,
//! `dword_60101C` 2, `dword_621514` 0; `race::set_up_credits_race`,
//! `race::set_up_arcade_cars`, `race::set_start_states`; `dword_71C3E0` 1; "INIT CREDITS -
//! track", the stage, "car", the car kinds logged (level 1, each inside `pusha` /
//! `popa`); `race::load_stage`; `race::roll_credits` with eax 0; `dword_63DB84` 0;
//! `drive::play_race`; `dword_71C3E0` 0; `race::tidy_everything`.
//!
//! In:  nothing
//! Out: the registers as the calls leave them; no flags read (the callers' `pop` / `call`)
//!
//! Callers: `menu::play_films` (with ebp kept round it), which calls the Rust version
//! directly, and `sub_443104`. Its calls by place: 0 `set_up_credits_race`, 1
//! `set_up_arcade_cars`, 2 `set_start_states`, 3 `log_write_l1`, 4 `log_int_l1`, 5
//! `log_int_l1_nl`, 6 `load_stage`, 7 `roll_credits`, 8 `play_race`, 9 `tidy_everything`.
//! In the game the set-up and the logs are Rust; the others are called at their hooks
//! (each runs its Rust version and deals with its own refusals, as when the game calls
//! them). Ungated thunk: it plays a race. Shadow mode runs the original alone.
//! Self-test: a copy whose calls go to stand-ins that note them, against the Rust version
//! with the same, on the game's variables (put back).

use crate::errors::Refused;
use crate::framework::{
    RawHook, Regs, SelfTest, game_addr, game_read, game_write, hook_asm, rva, shadow,
};
use crate::race::noted_calls::{Part, both, copy};
use crate::service::harness::{Region, capture, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("race::credits_race", rva(0x463666), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL,
    ungated
);

pub const SET_UP: u32 = 0;
pub const LOG_WRITE: u32 = 3;
pub const LOG_INT: u32 = 4;
pub const LOG_INT_NL: u32 = 5;
pub const ROLL: u32 = 7;
/// The callees by place, as IDA shows them.
const CALLEES: [usize; 10] = [
    0x463711, 0x4468FB, 0x446D2E, 0x401427, 0x401472, 0x401476, 0x444040, 0x415FC2, 0x444CEE,
    0x444AC7,
];
const PLACES: [(usize, u32); 10] = [
    (0x463711, 0),
    (0x4468FB, 1),
    (0x446D2E, 2),
    (0x401427, 3),
    (0x401472, 4),
    (0x401476, 5),
    (0x444040, 6),
    (0x415FC2, 7),
    (0x444CEE, 8),
    (0x444AC7, 9),
];

const FIRST: [(usize, u32); 5] = [
    (0x5C5CE0, 0),
    (0x5BEC94, 0), // g_holdCarRevs
    (0x60519C, 0x1234_5678),
    (0x60101C, 2),
    (0x621514, 0),
];
const IN_CREDITS: usize = 0x71C3E0; // dword_71C3E0
const AFTER_CREDITS: usize = 0x63DB84; // dword_63DB84
const STAGE: usize = 0x623850; // g_stageNumber
const CAR_KINDS: usize = 0x623858; // g_raceCarKinds
const WORDS: [usize; 2] = [0x4DDFB8, 0x4DDFCD]; // "INIT CREDITS - track", "car"

fn p(x: usize, v: u32) {
    unsafe { game_write(rva(x), v) }
}

/// The steps on `r`.
pub fn credits_race_in(r: &mut Regs, part: Part) -> Result<(), Refused> {
    for (at, v) in FIRST {
        p(at, v);
    }
    for place in 0..3 {
        part(place, r)?;
    }
    p(IN_CREDITS, 1);
    let g = |x: usize| unsafe { game_read::<u32>(rva(x)) };
    let word = |n: usize| game_addr(rva(WORDS[n])) as u32;
    for (place, eax) in [
        (LOG_WRITE, word(0)),
        (LOG_INT, g(STAGE)),
        (LOG_WRITE, word(1)),
        (LOG_INT_NL, g(CAR_KINDS)),
    ] {
        let mut c = Regs { eax, ..*r };
        part(place, &mut c)?;
    }
    part(6, r)?;
    r.eax = 0;
    part(ROLL, r)?;
    p(AFTER_CREDITS, 0);
    part(8, r)?;
    p(IN_CREDITS, 0);
    part(9, r)
}

/// The calls in the game: the set-up and the logs Rust, the rest at their hooks.
pub fn credits_part(place: u32, r: &mut Regs) -> Result<(), Refused> {
    use crate::logfile::{log_int::log_int, log_write::log_write};
    unsafe {
        match place {
            SET_UP => crate::race::set_up_credits_race::set_up_credits_race(r)?.apply(r),
            LOG_WRITE => log_write(r.eax as *const u8, false, 1),
            LOG_INT | LOG_INT_NL => log_int(r.eax as i32, place == LOG_INT_NL, 1),
            _ => crate::calls::call(rva(CALLEES[place as usize]), r),
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
    match credits_race_in(&mut g, &mut credits_part) {
        Ok(()) => {
            *r = Regs {
                esp: r.esp,
                eflags: r.eflags,
                ..g
            }
        }
        Err(e) => unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) },
    }
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    let mut g = *r;
    match credits_race_in(&mut g, &mut credits_part) {
        Ok(()) => {
            *r = Regs {
                esp: r.esp,
                eflags: r.eflags,
                ..g
            }
        }
        Err(e) => return Err(e),
    }
    Ok(())
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let copy = match copy(
        0x463666,
        0xAB,
        &[0xC7, 0x05, 0xE0, 0x5C, 0x5C, 0x00, 0x00, 0x00, 0x00, 0x00],
        &[
            0x32, 0x37, 0x3C, 0x52, 0x5E, 0x6B, 0x77, 0x7D, 0x87, 0x96, 0xA5,
        ],
        &PLACES,
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    let a = |x: usize| (game_addr(rva(x)) as u32, 4);
    let mut all: Vec<Region> = FIRST.iter().map(|&(x, _)| a(x)).collect();
    all.extend([a(IN_CREDITS), a(AFTER_CREDITS)]);
    let mut kept = all.clone();
    kept.extend([a(STAGE), a(CAR_KINDS)]);
    let saved = capture(&kept);
    for (stage, kinds) in [(3u32, 7u32), (0x2A, 0)] {
        both(
            &mut t,
            &format!("stage {stage:#x}, car kinds {kinds}"),
            &copy,
            Regs::SENTINEL,
            &|| {
                restore(&kept, &saved);
                for &(x, _) in &FIRST {
                    p(x, 0xAAAA_AAAA);
                }
                p(IN_CREDITS, 0xAAAA_AAAA);
                p(AFTER_CREDITS, 0xAAAA_AAAA);
                p(STAGE, stage);
                p(CAR_KINDS, kinds);
            },
            &all,
            &credits_race_in,
        );
    }
    restore(&kept, &saved);
    drop(copy);
    t.finish()
}
