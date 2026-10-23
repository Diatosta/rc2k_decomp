//! `sound::race_turn` (0x40D5F3): the race's sounds, once a frame.
//!
//! In:  nothing
//! Out: every register preserved (`pusha` / `popa`)
//! Quitting (`g_quitting` 1, the game closing): the original's jump (chunk 0x4154B5:
//! esp = `g_mainLoopStack` and on into `system::main_loop`, or to 0x401116 when that is
//! 0) is run as it is, by the original through its hook; in the standalone the program
//! is ended as `start` ends it (`standalone::leave_from_race`). Owner, 2026-10-18. Else, once the sounds are
//! started (`dword_5124E4` 1): `sound::step_race_clock`; with a turn due (carry clear)
//! `sound::mark_car_voices`, `car_sounds_all`, `play_cue_queues`, `stage_sounds`,
//! `spot_sounds`, `send_end_of_turn`; and either way `input::step_effects_clock`.
//! One caller, `sub_421BCF` (`sound::race_turn_if`).
//! The thunk is ungated: the quit jump never comes back to it.
//! What the callees refuse: the original runs from the start (a sound already sent by
//! the Rust version is sent again).
//! Shadow mode runs the original alone (it sends sounds).
//! Self-test: a copy whose calls go to stand-ins (the clock's carry the test's), against
//! the Rust version; not quitting only (the jump leaves the copy for good).
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{Regs, game_read, rva};
use crate::sound::turn_parts::turn_entry;

/// g_quitting.
const QUITTING: usize = 0x51837C;
/// dword_5124E4: 1 once the race's sounds are started.
const STARTED: usize = 0x5124E4;

fn g(x: usize) -> u32 {
    unsafe { game_read(rva(x)) }
}

// ---- Rust version ----

pub fn race_turn_in(r: &mut Regs) -> Result<(), Refused> {
    if g(QUITTING) == 1 {
        // The original's quit jump (into `system::main_loop`'s way out): the program
        // ended as `start` ends it.
        crate::standalone::leave_from_race();
    }
    if g(STARTED) != 1 {
        return Ok(());
    }
    let mut c = *r;
    // `step_race_clock` leaves no register, the carry set when no turn is due.
    let due = crate::sound::turn_parts::step_clock();
    c.set_carry(!due);
    c.set_zero(false);
    if due {
        use crate::sound::*;
        mark_car_voices::mark_car_voices_in(&mut c)?;
        car_sounds_all::car_sounds_all_in(&mut c)?;
        play_cue_queues::play_cue_queues_in(&mut c)?;
        stage_sounds::stage_sounds_in(&mut c)?;
        spot_sounds::spot_sounds_in(&mut c)?;
        send_end_of_turn::send_end_of_turn_in(&mut c)?;
    }
    crate::input::step_effects_clock::step_effects_clock_in(&mut c)
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    turn_entry(r, race_turn_in)
}
