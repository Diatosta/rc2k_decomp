//! `physics::apply_forces` (0x43F566): ecx force records of 0x20 bytes from esi, each
//! by `physics::apply_force` on the work car (`g_car`), inside `pusha` / `popa`.
//!
//! In:  esi the first record, ecx their number
//! Out: every register kept; no flags read
//!
//! Callers: `physics::apply_wheel_forces`, `physics::apply_contact_forces`.
//! Refuses (before anything) ecx 0 (the original would loop 2^32 times) or above 0x100.
//! Self-test: a copy of the original (its car the test's, its call to a copy of
//! `apply_force`) on test records, counts 1 .. 7.

use crate::errors::{BadInput, Refused};
use crate::framework::{RawHook, Regs, SelfTest, call_game, game_addr, hook_asm, rva};
use crate::physics::apply_force::{BLOCK_LEN, apply_at, apply_force, fill};
use crate::screens::harness::Left;
use crate::service::harness::{capture, dispatch_state};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("physics::apply_forces", rva(0x43F566), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// Refused before anything?
// Converted: `rc2k_core::physics::pushes::record_count`.
pub fn refusal(count: u32) -> Option<&'static str> {
    rc2k_core::physics::pushes::record_count(count, "a force record count of 0 or above 0x100")
        .err()
        .map(|e| e.0)
}

/// The Rust version: `count` records from `first` on the work car.
pub unsafe fn apply_forces(first: u32, count: u32) -> Result<(), Refused> {
    if let Some(why) = refusal(count) {
        return Err(BadInput(why).into());
    }
    let car = game_addr(rva(0x719FD8)) as u32;
    for k in 0..count {
        unsafe { apply_force(first + 0x20 * k, car) };
    }
    Ok(())
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    unsafe { apply_forces(r.esi, r.ecx) }?;
    Ok(Left::nothing())
}

/// What `apply_force` writes on the work car.
pub fn regions() -> Vec<crate::service::harness::Region> {
    vec![
        (game_addr(rva(0x614B30)) as u32, 12),
        (game_addr(rva(0x62400C)) as u32, 4),
        (game_addr(rva(0x719FD8 + 0x198)) as u32, 0x18),
    ]
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &regions(), &rust);
}

/// Copies for a test: this one (its `g_car` at `car`, its call to a copy of
/// `apply_force` with its work vector and scratch at `work`, `scratch`).
pub fn test_copies(
    car: u32,
    work: u32,
    scratch: u32,
) -> Result<(usize, Vec<crate::redirect::RedirectedCopy>), String> {
    let inner = crate::physics::apply_force::test_copy(work, scratch)?;
    let target = inner.addr();
    let own = crate::audio::test_copy::make(
        &HOOK,
        &crate::audio::test_copy::Copy {
            address: 0x43F566,
            len: 0x14,
            entry: &[0x60, 0x8D, 0x3D, 0xD8, 0x9F, 0x71, 0x00],
            calls: &[7],
            jumps_out: &[],
        },
        &|r| (r == rva(0x45FAE1)).then_some(target),
        &[(0x719FD8, car)],
        1,
    )?;
    Ok((own.addr(), vec![own, inner]))
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let mut cells = vec![0u32; (BLOCK_LEN as usize) / 4 + 8 * 8 + 4];
    let car = cells.as_mut_ptr() as u32;
    let recs = car + BLOCK_LEN;
    let work = recs + 0x20 * 8;
    let scratch = work + 12;
    let (copy, keep) = match test_copies(car, work, scratch) {
        Ok(c) => c,
        Err(why) => return t.skip(&why),
    };
    let all = [(car, 4 * cells.len())];
    for count in 1..=7u32 {
        let fill_all = || {
            for k in 0..8 {
                fill(recs + 0x20 * k, car, count * 3 + k);
            }
        };
        fill_all();
        let before = Regs {
            esi: recs,
            ecx: count,
            ..Regs::SENTINEL
        };
        let mut ro = before;
        unsafe { call_game(&mut ro, copy) };
        let theirs = capture(&all);
        fill_all();
        for k in 0..count {
            unsafe { apply_at(recs + 0x20 * k, car, work, scratch) };
        }
        let mine = capture(&all);
        let regs_ok = ro.preserved_except(&before, 0);
        t.check(regs_ok && theirs == mine, || {
            format!("count {count}: regs_ok={regs_ok}; the memory differs")
        });
    }
    t.check(refusal(0).is_some(), || "count 0 not refused".into());
    drop((keep, cells));
    t.finish()
}
