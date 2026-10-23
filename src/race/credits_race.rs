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
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{Regs, game_addr, game_read, game_write, rva};

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
pub fn credits_race_in(r: &mut Regs) -> Result<(), Refused> {
    for (at, v) in FIRST {
        p(at, v);
    }
    for place in 0..3 {
        crate::race::credits_race::credits_part(place, r)?;
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
        crate::race::credits_race::credits_part(place, &mut c)?;
    }
    crate::race::credits_race::credits_part(6, r)?;
    r.eax = 0;
    crate::race::credits_race::credits_part(ROLL, r)?;
    p(AFTER_CREDITS, 0);
    crate::race::credits_race::credits_part(8, r)?;
    p(IN_CREDITS, 0);
    crate::race::credits_race::credits_part(9, r)
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

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    let mut g = *r;
    match credits_race_in(&mut g) {
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
