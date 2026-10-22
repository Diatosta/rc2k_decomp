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
use crate::sound::turn_parts::{Part, STEP_CLOCK, turn_entry};

/// g_quitting.
const QUITTING: usize = 0x51837C;
/// dword_5124E4: 1 once the race's sounds are started.
const STARTED: usize = 0x5124E4;
/// The quit jump's chunk: its place for the Rust callers' part.
pub const QUIT: u32 = 0x4154B5;
/// With a turn due, in the original's order.
const TURN: [u32; 6] = [0x40D75A, 0x40D7AB, 0x40ED13, 0x40E2A9, 0x40E611, 0x409BE0];
const EFFECTS_CLOCK: u32 = 0x40F257;

fn g(x: usize) -> u32 {
    unsafe { game_read(rva(x)) }
}

/// The quit (the game is closing): in the game the original, as it is: its jump goes into
/// `system::main_loop`'s Rust version, which sees its stack kept and leaves for `start`'s
/// end (the Rust frames it jumps over are never returned to; the program ends). In the
/// standalone, where there is no original to jump into, the program is ended the same
/// way from here.
pub fn quit_in_game(r: &mut Regs) {
    #[cfg(feature = "standalone")]
    {
        let _ = r;
        crate::standalone::leave_from_race();
    }
    // (its hook is gone: the address is the original)
    #[cfg(not(feature = "standalone"))]
    unsafe {
        crate::framework::call_game(r, crate::framework::game_addr(rva(0x40D5F3)))
    };
}

// ---- Rust version ----

pub fn race_turn_in(r: &mut Regs, part: Part) -> Result<(), Refused> {
    if g(QUITTING) == 1 {
        let mut c = *r;
        return part(QUIT, &mut c);
    }
    if g(STARTED) != 1 {
        return Ok(());
    }
    let mut c = *r;
    part(STEP_CLOCK, &mut c)?;
    if !c.carry() {
        for step in TURN {
            part(step, &mut c)?;
        }
    }
    part(EFFECTS_CLOCK, &mut c)
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    turn_entry(r, race_turn_in)
}
