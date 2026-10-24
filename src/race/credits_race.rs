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
//! Callers: `menu::play_films` (with ebp kept round it), which calls the Rust version directly,
//! and `sub_443104`. Its calls by place: 0 `set_up_credits_race`, 1 `set_up_arcade_cars`, 2
//! `set_start_states`, 3 `log_write_l1`, 4 `log_int_l1`, 5 `log_int_l1_nl`, 6 `load_stage`, 7
//! `roll_credits`, 8 `play_race`, 9 `tidy_everything`.

use crate::errors::Refused;
use crate::framework::{game_addr, game_read, game_write, rva};
use crate::logfile::{log_int::log_int, log_write::log_write};

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

/// The Rust version: the steps in order. (The original keeps ebp, ecx and edi and leaves
/// esi and edx; no step reads a register of the caller or of an earlier step.)
pub fn credits_race() -> Result<(), Refused> {
    crate::framework::log("race::credits_race: the demonstration starts");
    for (at, v) in FIRST {
        p(at, v);
    }
    crate::race::set_up_credits_race::set_up_credits_race()?;
    unsafe {
        crate::race::race_cars::set_up_arcade_cars()?;
        crate::race::race_shell::set_start_states()?;
    }
    p(IN_CREDITS, 1);
    let g = |x: usize| unsafe { game_read::<u32>(rva(x)) };
    let word = |n: usize| game_addr(rva(WORDS[n])) as *const u8;
    unsafe {
        log_write(word(0), false, 1);
        log_int(g(STAGE) as i32, false, 1);
        log_write(word(1), false, 1);
        log_int(g(CAR_KINDS) as i32, true, 1);
        crate::race::stage_loading::load_stage()?;
        crate::race::frame_text::roll_credits(
            0,
            &mut crate::text::draw_text::GameCanvas,
            &mut crate::race::frame_text::credits_tune,
        )?;
    }
    p(AFTER_CREDITS, 0);
    unsafe {
        crate::drive::play_race::play_race()?;
        p(IN_CREDITS, 0);
        crate::race::race_end::tidy_everything(&crate::race::loadlevel::LoadVars::game())
    }
}
