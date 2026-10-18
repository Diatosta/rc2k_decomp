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

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_read, game_write, hook_asm, rva};
use crate::service::harness::{at, capture, restore};
use crate::sound::turn_parts::{
    Part, STEP_CLOCK, TEST_NOT_DUE, both, copy_of, dispatch_turn, run_copy,
};
use std::sync::atomic::{AtomicUsize, Ordering};

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("sound::race_turn", rva(0x40D5F3), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL,
    ungated
);

const LEN: usize = 0x43;
/// The quit jump (left where it went), then the eight calls.
const OUTS: [usize; 9] = [0x08, 0x17, 0x1E, 0x23, 0x28, 0x2D, 0x32, 0x37, 0x3C];
const ENTRY: [u8; 8] = [0x60, 0x83, 0x3D, 0x7C, 0x83, 0x51, 0x00, 0x01];
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
    #[cfg(not(feature = "standalone"))]
    unsafe {
        call_original(r)
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

// ---- hook ----

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_turn(&HOOK, r, call_original, race_turn_in, None);
}

// ---- self-test ----

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    if g(QUITTING) == 1 {
        return t.skip("quitting");
    }
    let mut places = vec![STEP_CLOCK];
    places.extend(TURN);
    places.push(EFFECTS_CLOCK);
    let copy = match copy_of(0x40D5F3, LEN, &ENTRY, &OUTS, &places) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    let run = run_copy(&copy);
    let all = [at(rva(STARTED))];
    let saved = capture(&all);
    for started in [1u32, 0, 2] {
        for not_due in [0u32, 1] {
            let fill = || {
                unsafe { game_write(rva(STARTED), started) };
                TEST_NOT_DUE.store(not_due, Ordering::SeqCst);
            };
            both(
                &mut t,
                &format!("started {started}, not due {not_due}"),
                &run,
                Regs::SENTINEL,
                &fill,
                &all,
                race_turn_in,
                false,
            );
        }
    }
    restore(&all, &saved);
    t.finish()
}
