//! `physics::note_time_mark` (0x440C38): when `flt_71AEE0` is not below `flt_5C5D5C`
//! (60.0; `fcomp`, C0 clear: also not unordered) and `dword_6140EC` is 0:
//! `dword_6140EC` = `dword_719FE0` (the work copy's +8, a step count by `sub_460835`'s
//! `inc`) less `dword_6140F0`. By the look of it the step at which a figure first
//! reached 60 (a reading).
//!
//! In:  nothing
//! Out: eax the difference when it is stored, else kept; no flags read
//!
//! Caller: `sub_43B635` (0x43B9A4), every physics step.
//! Self-test: `physics/copy_hooks_test.rs` (the converted version), 12 cases (figures
//! below, at, above, NaN); a compare of floats only: exact at any precision.

use crate::errors::Refused;
use crate::framework::{EAX, RawHook, Regs, game_addr, game_read, hook_asm, rva};
use crate::physics::copy_hooks_test::{lot, test_moves};
use crate::screens::harness::Left;
use crate::service::harness::{dispatch_state, state_entry};
use rc2k_core::physics::motion::time_mark;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("physics::note_time_mark", rva(0x440C38), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// The figure, the mark, the count, the start (IDA addresses).
const VARIABLES: [usize; 4] = [0x71AEE0, 0x6140EC, 0x719FE0, 0x6140F0];
const LIMIT_RVA: usize = rva(0x5C5D5C); // flt_5C5D5C, 60.0

fn game() -> Vec<u32> {
    VARIABLES
        .iter()
        .map(|&x| game_addr(rva(x)) as u32)
        .collect()
}

/// The converted version (`rc2k_core::physics::motion`) on the variables at `v`; `r` as
/// the original leaves it.
// Converted: `rc2k_core::physics::motion::time_mark`.
unsafe fn core(v: &[u32], r: &mut Regs) {
    unsafe {
        let get = |k: usize| (v[k] as *const u32).read_unaligned();
        let limit = f32::from_bits(game_read(LIMIT_RVA));
        // The mark, count and start read only where the original reads them.
        if let Some(mark) = time_mark(
            f32::from_bits(get(0)),
            limit,
            || get(1),
            || (get(2), get(3)),
        ) {
            r.eax = mark;
            (v[1] as *mut u32).write_unaligned(mark);
        }
    }
}

/// The Rust version (a compare of floats only: the same at any precision); `r` as the
/// original leaves it.
pub unsafe fn note_time_mark(r: &mut Regs) {
    unsafe { core(&game(), r) }
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    let mut g = *r;
    unsafe { note_time_mark(&mut g) };
    Ok(Left::nothing().with(EAX, g.eax))
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &[(game()[1], 4)], &rust);
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}

fn self_test() -> (usize, usize) {
    test_moves(
        &HOOK,
        0x440C38,
        0x2E,
        &[0xD9, 0x05, 0xE0, 0xAE, 0x71, 0x00],
        &VARIABLES,
        5,
        12,
        &|seed, out| {
            lot(seed, out);
            out[0] = [59.9f32, 60.0, 61.0, f32::NAN, -1.0, 1000.0][(seed % 6) as usize].to_bits();
            if seed >= 6 {
                out[1] = 0;
            }
        },
        &|v, r| {
            unsafe { core(v, r) };
            Left::nothing().with(EAX, r.eax)
        },
    )
}
