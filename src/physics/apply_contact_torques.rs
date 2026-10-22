//! `physics::apply_contact_torques` (0x43F6A5): when `dword_614148` is 0,
//! `physics::apply_torques` on the 7 records at `unk_614A50` (the ones
//! `physics::apply_contact_forces` pushes with); else nothing.
//!
//! In:  nothing
//! Out: `dword_614148` 0: ebx the count, edi the first record; else nothing changed;
//!      the rest kept; no flags read (the caller's next instruction is a `cmp`)
//!
//! Caller: `sub_43B635` (0x43B888, after `physics::apply_contact_forces`), every physics
//! step.
//! Self-test: a copy of the original (its switch, records and car the test's, its call
//! to a copy of `apply_torques`) against the Rust core, the switch 0 and 1.

use crate::errors::Refused;
use crate::framework::{
    EBX, EDI, RawHook, Regs, SelfTest, call_game, game_addr, game_read, hook_asm, rva,
};
use crate::physics::apply_torques::{Cells, apply_torques, regions, test_copies};
use crate::screens::harness::Left;
use crate::service::harness::{capture, dispatch_state, state_entry};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "physics::apply_contact_torques",
    rva(0x43F6A5),
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

const SWITCH_RVA: usize = rva(0x614148); // dword_614148
const RECORDS_RVA: usize = rva(0x614A50);
const COUNT: u32 = 7;

/// The Rust core, the switch given, `run` doing the records (`apply_torques`).
pub fn apply_contact_torques_with(
    switch: u32,
    first: u32,
    run: impl FnOnce(u32, u32) -> Result<(), Refused>,
) -> Result<Left, Refused> {
    if switch != 0 {
        return Ok(Left::nothing());
    }
    run(first, COUNT)?;
    Ok(Left::nothing().with(EDI, first).with(EBX, COUNT))
}

/// The Rust version.
pub unsafe fn apply_contact_torques() -> Result<Left, Refused> {
    apply_contact_torques_with(
        unsafe { game_read::<u32>(SWITCH_RVA) },
        game_addr(RECORDS_RVA) as u32,
        |first, count| unsafe { apply_torques(first, count) },
    )
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    unsafe { apply_contact_torques() }
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
    let mut switch = vec![0u32; 1];
    let sw = switch.as_mut_ptr();
    let (inner, keep) = match test_copies(c.car, c.work, c.cross) {
        Ok(x) => x,
        Err(why) => return t.skip(&why),
    };
    let copy = match crate::audio::test_copy::make(
        &HOOK,
        &crate::audio::test_copy::Copy {
            address: 0x43F6A5,
            len: 0x1A,
            entry: &[0x83, 0x3D, 0x48, 0x41, 0x61, 0x00, 0x00],
            calls: &[0x14],
            jumps_out: &[],
        },
        &|r| (r == rva(0x43F6BF)).then_some(inner),
        &[(0x614148, sw as u32), (0x614A50, c.recs)],
        2,
    ) {
        Ok(x) => x,
        Err(why) => return t.skip(&why),
    };
    for value in [0u32, 1, 0x8000_0000] {
        unsafe { sw.write_volatile(value) };
        c.fill(COUNT, 5 + value);
        let mut ro = Regs::SENTINEL;
        unsafe { call_game(&mut ro, copy.addr()) };
        let theirs = capture(&c.all());
        c.fill(COUNT, 5 + value);
        let left = apply_contact_torques_with(unsafe { sw.read_volatile() }, c.recs, |_, n| {
            c.run_rust(n);
            Ok(())
        });
        let mine = capture(&c.all());
        let regs_ok = match left {
            Ok(l) => l.matches(&Regs::SENTINEL, &ro),
            Err(_) => false,
        };
        t.check(regs_ok && theirs == mine, || {
            format!("switch {value:#x}: regs_ok={regs_ok}; the memory differs")
        });
    }
    drop((copy, keep, c, switch));
    t.finish()
}
