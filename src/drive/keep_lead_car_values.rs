//! `drive::keep_lead_car_values` (0x4564F5): eight fields of car block 0 (0x70FA70 ..,
//! `drive/lead_car_values.rs`) into 0x64165C .. 0x641678 (`push`/`pop`). The marks read
//! `dword_64165C` (`marks::stage_record`).
//!
//! In:  nothing
//! Out: every register kept; no flags read
//!
//! Caller: `drive::share_all_cars` (0x456484), first.
//! Self-test: `physics/copy_hooks_test.rs` (a copy on test variables; the game's never
//! written), 8 fillings.

use crate::drive::lead_car_values::{PAIRS, move_pairs};
use crate::errors::Refused;
use crate::framework::{RawHook, Regs, game_addr, hook_asm, rva};
use crate::physics::copy_hooks_test::{lot, test_moves};
use crate::screens::harness::Left;
use crate::service::harness::{dispatch_state, state_entry};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "drive::keep_lead_car_values",
    rva(0x4564F5),
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
    PAIRS.iter().map(|&(f, t)| (a(f), a(t))).collect()
}

/// The Rust version.
pub unsafe fn keep_lead_car_values() {
    unsafe { move_pairs(&game()) }
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    unsafe { keep_lead_car_values() };
    Ok(Left::nothing())
}

extern "C" fn dispatch(r: &mut Regs) {
    let regions: Vec<_> = game().iter().map(|&(_, t)| (t, 4)).collect();
    dispatch_state(&HOOK, r, call_original, &regions, &rust);
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}

fn self_test() -> (usize, usize) {
    let vars: Vec<usize> = PAIRS
        .iter()
        .map(|p| p.0)
        .chain(PAIRS.iter().map(|p| p.1))
        .collect();
    test_moves(
        &HOOK,
        0x4564F5,
        0x61,
        &[0xFF, 0x35, 0x70, 0xFA, 0x70, 0x00],
        &vars,
        16,
        8,
        &lot,
        &|v, _| {
            let pairs: Vec<(u32, u32)> = (0..8).map(|k| (v[k], v[k + 8])).collect();
            unsafe { move_pairs(&pairs) };
            Left::nothing()
        },
    )
}
