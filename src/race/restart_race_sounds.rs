//! `race::restart_race_sounds` (0x40D34A): the sound commands waited for, the race's
//! sounds reset (`race::reset_race_sounds`), let play (`sound::let_sounds_play`);
//! `dword_5124E4` 1.
//!
//! In:  nothing
//! Out: every register preserved
//!
//! See `race/sound_tables.rs`. One caller, `race::set_up_race_sound_objects`. Its calls
//! begin at +1: private copy, the three calls relocated, manual patch. Shadow mode runs
//! the original alone (it sends commands). Self-test: a copy whose three calls go to
//! stand-ins, against the Rust version on the same.

use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva, shadow};
use crate::race::sound_tables::*;
use crate::service::harness::{at, capture, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("race::restart_race_sounds", rva(0x40D34A), thunk, &ORIGINAL)
        .private_copy(0x12)
        .relocate_calls(&[0x1, 0x6, 0xB])
        .manual_patch()
        .self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    if let Err(e) = unsafe { restart_race_sounds(r, &mut sound_part) } {
        unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) };
    }
}

/// The calls by place, as IDA shows them.
const CALLEES: [(usize, u32); 5] = [
    (0x409C1E, WAIT),
    (0x40D378, RESET),
    (0x409B76, LET_PLAY),
    (0x409BCA, COMMAND_4),
    (0, 0),
];

fn self_test() -> (usize, usize) {
    use crate::race::mud::{copy, run_both, test_part};
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let copy = match copy(
        0x40D34A,
        0x12,
        &[0x60, 0xE8, 0xCE, 0xC8, 0xFF, 0xFF],
        &[0x1, 0x6, 0xB],
        &CALLEES[..4],
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    let regions = [at(rva(0x5124E4))];
    let saved = capture(&regions);
    for v in [0u32, 7] {
        run_both(
            &mut t,
            &format!("dword_5124E4 {v}"),
            &copy,
            &regions,
            &|| unsafe { crate::framework::game_write(rva(0x5124E4), v) },
            &|r| unsafe { restart_race_sounds(r, &mut test_part) },
        );
    }
    restore(&regions, &saved);
    t.finish()
}
