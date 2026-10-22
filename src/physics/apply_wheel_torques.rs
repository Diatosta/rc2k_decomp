//! `physics::apply_wheel_torques` (0x43F694): `physics::apply_torques` on the 0x17
//! records at `unk_614770` (the ones `physics::apply_wheel_forces` pushes with).
//!
//! In:  nothing
//! Out: ebx the count, edi the first record; the rest kept; no flags read
//!
//! Caller: `sub_43B635` (0x43B822, after `physics::apply_wheel_forces`, before
//! `physics::move_car`), every physics step.
//! Self-test: a copy of the original (its records and car the test's, its call to a
//! copy of `apply_torques`) against the Rust core.

use crate::errors::Refused;
use crate::framework::{EBX, EDI, RawHook, Regs, SelfTest, call_game, game_addr, hook_asm, rva};
use crate::physics::apply_torques::{Cells, apply_torques, regions, test_copies};
use crate::screens::harness::Left;
use crate::service::harness::{capture, dispatch_state, state_entry};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "physics::apply_wheel_torques",
    rva(0x43F694),
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

const RECORDS_RVA: usize = rva(0x614770);
const COUNT: u32 = 0x17;

/// The Rust version.
pub unsafe fn apply_wheel_torques() -> Result<Left, Refused> {
    let first = game_addr(RECORDS_RVA) as u32;
    unsafe { apply_torques(first, COUNT) }?;
    Ok(Left::nothing().with(EDI, first).with(EBX, COUNT))
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    unsafe { apply_wheel_torques() }
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &regions(), &rust);
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let c = Cells::new();
    let (inner, keep) = match test_copies(c.car, c.work, c.cross) {
        Ok(x) => x,
        Err(why) => return t.skip(&why),
    };
    let copy = match crate::audio::test_copy::make(
        &HOOK,
        &crate::audio::test_copy::Copy {
            address: 0x43F694,
            len: 0x11,
            entry: &[0xBB, 0x17, 0x00, 0x00, 0x00],
            calls: &[0xB],
            jumps_out: &[],
        },
        &|r| (r == rva(0x43F6BF)).then_some(inner),
        &[(0x614770, c.recs)],
        1,
    ) {
        Ok(x) => x,
        Err(why) => return t.skip(&why),
    };
    c.fill(COUNT, 5);
    let mut ro = Regs::SENTINEL;
    unsafe { call_game(&mut ro, copy.addr()) };
    let theirs = capture(&c.all());
    c.fill(COUNT, 5);
    c.run_rust(COUNT);
    let mine = capture(&c.all());
    let regs_ok = Left::nothing()
        .with(EDI, c.recs)
        .with(EBX, COUNT)
        .matches(&Regs::SENTINEL, &ro);
    t.check(regs_ok && theirs == mine, || {
        format!("regs_ok={regs_ok}; the memory differs")
    });
    drop((copy, keep, c));
    t.finish()
}
