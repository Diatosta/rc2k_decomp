//! `sound::race_turn_if` (0x421BCF): `sound::race_turn` while the race's sounds are on.
//!
//! In:  nothing
//! Out: ebx 0x5317707 when it called (a leftover constant); everything else preserved
//! Only with `dword_6218F0` not 0. One caller, `sound::race_sound_tick`.
//! Ungated: `race_turn`'s quit jump never comes back.
//! Shadow mode runs the original alone (sounds are sent).
//! Self-test: a copy whose call goes to a stand-in, against the Rust version.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_read, game_write, hook_asm, rva};
use crate::service::harness::{at, capture, restore};
use crate::sound::turn_parts::{Part, both, copy_of, dispatch_turn, run_copy};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("sound::race_turn_if", rva(0x421BCF), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL,
    ungated
);

const LEN: usize = 0x14;
const OUTS: [usize; 1] = [0x09];
const ENTRY: [u8; 7] = [0x83, 0x3D, 0xF0, 0x18, 0x62, 0x00, 0x00];
/// dword_6218F0: the race's sounds on.
const ON: usize = 0x6218F0;
pub const RACE_TURN: u32 = 0x40D5F3;

// ---- Rust version ----

pub fn race_turn_if_in(r: &mut Regs, part: Part) -> Result<(), Refused> {
    if unsafe { game_read::<u32>(rva(ON)) } == 0 {
        return Ok(());
    }
    part(RACE_TURN, r)?;
    r.ebx = 0x531_7707;
    Ok(())
}

// ---- hook ----

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_turn(&HOOK, r, call_original, race_turn_if_in, None);
}

// ---- self-test ----

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let copy = match copy_of(0x421BCF, LEN, &ENTRY, &OUTS, &[RACE_TURN]) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    let run = run_copy(&copy);
    let all = [at(rva(ON))];
    let saved = capture(&all);
    for on in [0u32, 1, 0x8000_0000] {
        both(
            &mut t,
            &format!("on {on:#x}"),
            &run,
            Regs::SENTINEL,
            &|| unsafe { game_write(rva(ON), on) },
            &all,
            race_turn_if_in,
            false,
        );
    }
    restore(&all, &saved);
    t.finish()
}
