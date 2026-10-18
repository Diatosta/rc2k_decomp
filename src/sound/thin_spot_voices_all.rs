//! `sound::thin_spot_voices_all` (0x40E97D): `sound::thin_spot_voices` for the twelve
//! sounds of which only one voice is heard: 0x1B1 .. 0x1B6, 0x1B9, 0x1BA, 0x1B0, 0x1BE,
//! 0x1C0, 0x1C3.
//!
//! In:  nothing
//! Out: every register preserved (`pusha` / `popa`)
//! Each call with eax the sound and the other registers the one before left. One caller,
//! `sound::spot_sounds`.
//! Shadow mode: both (nothing sent), the voices and variables compared.
//! Self-test: a copy whose calls go to a stand-in, against the Rust version.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::sound::turn_parts::{Part, both, copy_of, dispatch_turn, run_copy};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "sound::thin_spot_voices_all",
    rva(0x40E97D),
    thunk,
    &ORIGINAL,
)
.self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const LEN: usize = 0x7B;
const OUTS: [usize; 12] = [
    0x6, 0x10, 0x1A, 0x24, 0x2E, 0x38, 0x42, 0x4C, 0x56, 0x60, 0x6A, 0x74,
];
const ENTRY: [u8; 6] = [0x60, 0xB8, 0xB1, 0x01, 0x00, 0x00];
/// `sound::thin_spot_voices`.
const EACH: u32 = 0x40E9F8;
const SOUNDS: [u32; 12] = [
    0x1B1, 0x1B2, 0x1B3, 0x1B4, 0x1B5, 0x1B6, 0x1B9, 0x1BA, 0x1B0, 0x1BE, 0x1C0, 0x1C3,
];

// ---- Rust version ----

pub fn thin_spot_voices_all_in(r: &mut Regs, part: Part) -> Result<(), Refused> {
    let mut c = *r;
    for sound in SOUNDS {
        c.eax = sound;
        part(EACH, &mut c)?;
    }
    Ok(())
}

// ---- hook ----

fn regions(r: &Regs) -> Vec<crate::service::harness::Region> {
    crate::sound::thin_spot_voices::thin_regions(r)
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_turn(
        &HOOK,
        r,
        call_original,
        thin_spot_voices_all_in,
        Some(&regions),
    );
}

// ---- self-test ----

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let copy = match copy_of(0x40E97D, LEN, &ENTRY, &OUTS, &[EACH]) {
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
        thin_spot_voices_all_in,
        false,
    );
    t.finish()
}
