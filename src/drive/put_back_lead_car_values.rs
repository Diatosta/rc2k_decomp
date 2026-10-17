//! `drive::put_back_lead_car_values` (0x456556): the eight values
//! `drive::keep_lead_car_values` kept (0x64165C ..) back into car block 0 (0x70FA70 ..,
//! `drive/lead_car_values.rs`; `push`/`pop`).
//!
//! In:  nothing
//! Out: every register kept; no flags read (`drive::share_all_cars` returns next)
//!
//! Caller: `drive::share_all_cars` (0x456484), last.
//! Self-test: `physics/copy_hooks_test.rs` (a copy on test variables), 8 fillings.

use crate::drive::lead_car_values::{PAIRS, move_pairs};
use crate::errors::Refused;
use crate::framework::{RawHook, Regs, game_addr, hook_asm, rva};
use crate::physics::copy_hooks_test::{lot, test_moves};
use crate::screens::harness::Left;
use crate::service::harness::dispatch_state;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "drive::put_back_lead_car_values",
    rva(0x456556),
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

fn game() -> Vec<(u32, u32)> {
    let a = |x: usize| game_addr(rva(x)) as u32;
    PAIRS.iter().map(|&(f, t)| (a(t), a(f))).collect()
}

/// The Rust version.
pub unsafe fn put_back_lead_car_values() {
    unsafe { move_pairs(&game()) }
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    unsafe { put_back_lead_car_values() };
    Ok(Left::nothing())
}

extern "C" fn dispatch(r: &mut Regs) {
    let regions: Vec<_> = game().iter().map(|&(_, t)| (t, 4)).collect();
    dispatch_state(&HOOK, r, call_original, &regions, &rust);
}

fn self_test() -> (usize, usize) {
    let vars: Vec<usize> = PAIRS
        .iter()
        .map(|p| p.0)
        .chain(PAIRS.iter().map(|p| p.1))
        .collect();
    test_moves(
        &HOOK,
        0x456556,
        0x61,
        &[0xFF, 0x35, 0x5C, 0x16, 0x64, 0x00],
        &vars,
        16,
        8,
        &lot,
        &|v, _| {
            let pairs: Vec<(u32, u32)> = (0..8).map(|k| (v[k + 8], v[k])).collect();
            unsafe { move_pairs(&pairs) };
            Left::nothing()
        },
    )
}
