//! `menu::run_a8_championship` (0x4AA4D7): a whole A8 championship (event type 5,
//! `g_eventMode` 0x13), from its start or a saved game. `menu::run_championship`'s
//! driver (`run_championship::run_event`) with these differences: `clear_players_284`
//! before `dword_755254` 0; `g_eventMode` 0x13 (so the welcome is `menu::a8_welcome`) and
//! `dword_75AB40` 1; the section to start at is `dword_7554C4`; `dword_755970` is not set
//! back to 0; `menu::reset_single_event` where the other calls `menu::begin_event` (the
//! welcome record, the next section); no overall standings, overall stage or rally results
//! (only the class passes); the races are run even when `dword_755250` is 1;
//! `dword_755254` is not cleared after a stage; no `best_overall_scores` at the end; an
//! unknown record kind is fatal error 0x8070DBA9 "Invalid Level Type(A8)".
//!
//! In:  nothing
//! Out: every register as `menu::show_in_game_options` leaves the entry's (all kept)
//!
//! Callers: `saves::load_savegame` (`dword_7556D0` 0x13) and the five starters
//! (`menu::start_a8_*`). Ungated (a whole event). Refuses what its callees refuse, when
//! they are reached (the original then runs from the start). Shadow mode runs the
//! original alone. Self-test: `run_championship::test_event`.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva, shadow};
use crate::menu::event_flow::{Call, Flow, game_call};
use crate::menu::run_championship::{VARIABLES, run_event, test_event};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("menu::run_a8_championship", rva(0x4AA4D7), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL,
    ungated
);

pub unsafe fn run_a8_championship(entry: &Regs, call: Call) -> Result<Regs, Refused> {
    unsafe { run_event(entry, call, true) }
}

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    match unsafe { run_a8_championship(r, &mut game_call) } {
        Ok(left) => *r = Regs { esp: r.esp, ..left },
        Err(e) => unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) },
    }
}

const CALLS: &[usize] = &[
    0x27, 0x2C, 0x31, 0x36, 0x3B, 0x40, 0x4F, 0x54, 0x9D, 0xA4, 0xA9, 0x126, 0x159, 0x15E, 0x17B,
    0x180, 0x185, 0x18A, 0x18F, 0x194, 0x1CB, 0x1DD, 0x22D, 0x289, 0x2A1, 0x2B2, 0x2CA, 0x2DB,
    0x2F3, 0x33F, 0x344, 0x349, 0x361, 0x36F, 0x374, 0x379, 0x391, 0x3AE, 0x3D3, 0x3D8, 0x3F0,
    0x40E, 0x413, 0x421, 0x439, 0x457, 0x493, 0x4AB, 0x4E8, 0x4EF, 0x52B, 0x539, 0x53E, 0x54C,
    0x552,
];

fn self_test() -> (usize, usize) {
    let Some(t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let flow = Flow {
        address: 0x4AA4D7,
        len: 0x558,
        // pusha; cmp dword_755970, 1
        entry: &[0x60, 0x83, 0x3D, 0x70, 0x59, 0x75, 0x00, 0x01],
        calls: CALLS,
        fatal_jumps: &[0x283],
        writes: &VARIABLES,
    };
    test_event(t, &flow, true)
}
