//! `sound::spot_sounds` (0x40E611): the shared spots' sounds for the turn.
//!
//! In:  nothing
//! Out: every register preserved (`pusha` / `popa`)
//! esi the request; then `sound::match_spot_voices`, `free_spot_voices`,
//! `spot_voice_distances`, `merge_spot_voices_all`, `thin_spot_voices_all`,
//! `exclusive_spot_voices`, `repeat_spot_voices`, `play_spot_voices`, each getting the
//! registers the one before left. See `sound/spot_voices.rs`. One caller,
//! `sound::race_turn`.
//! What the callees refuse puts back what was written (sounds already sent stay sent).
//! Shadow mode runs the original alone (it sends sounds).
//! Self-test: a copy whose eight calls go to stand-ins, against the Rust version.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_addr, hook_asm, rva};
use crate::sound::turn_parts::{Part, REQUEST, both, copy_of, dispatch_turn, run_copy};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("sound::spot_sounds", rva(0x40E611), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const LEN: usize = 0x31;
const OUTS: [usize; 8] = [0x07, 0x0C, 0x11, 0x16, 0x1B, 0x20, 0x25, 0x2A];
const ENTRY: [u8; 7] = [0x60, 0x8D, 0x35, 0xBC, 0x24, 0x51, 0x00];
const STEPS: [u32; 8] = [
    0x40E642, 0x40E6F4, 0x40E733, 0x40E7CA, 0x40E97D, 0x40EAF3, 0x40EB44, 0x40EB9E,
];

// ---- Rust version ----

pub fn spot_sounds_in(r: &mut Regs, part: Part) -> Result<(), Refused> {
    let mut c = Regs {
        esi: game_addr(rva(REQUEST)) as u32,
        ..*r
    };
    for step in STEPS {
        part(step, &mut c)?;
    }
    Ok(())
}

// ---- hook ----

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_turn(&HOOK, r, call_original, spot_sounds_in, None);
}

// ---- self-test ----

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let copy = match copy_of(0x40E611, LEN, &ENTRY, &OUTS, &STEPS) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    both(
        &mut t,
        "the calls",
        &run_copy(&copy),
        Regs::SENTINEL,
        &|| {},
        &[],
        spot_sounds_in,
        false,
    );
    t.finish()
}
