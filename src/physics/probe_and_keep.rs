//! `physics::probe_and_keep` (0x45DC12): `physics::probe_ground`, then its three
//! whole-number lists (x, y, z at 0x6D1E14, 0xC0 dwords) copied to 0x6D2C14 and the probe
//! points' faces (`dword_71ADC8`, 0x40 dwords) to 0x6D2F14 (`rep movsd`): by the look of
//! it the step's starting points kept (a reading).
//!
//! In:  nothing
//! Out: ecx 0, esi 0x71ADC8 + 0x100, edi 0x6D2F14 + 0x100; eax, ebx, edx as
//!      `probe_ground` leaves them (with no probe points: kept)
//!
//! Caller: `sub_43B635` (0x43B700), every physics step; next a call (no flags read).
//! Refuses what `probe_ground` refuses (nothing written).
//! Self-test: a copy of the original (its call to a copy of `probe_ground` on the test's
//! ground) against the Rust core, `probe_ground`'s cases.

use crate::errors::Refused;
use crate::framework::{EAX, EBX, ECX, EDI, EDX, ESI, RawHook, Regs, SelfTest, hook_asm, rva};
use crate::physics::ground::*;
use crate::physics::ground_search::test_ground;
use crate::physics::probe_ground::{prepare_probe, probe_ground_at, regions};
use crate::screens::harness::Left;
use crate::service::harness::{dispatch_state, state_entry};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("physics::probe_and_keep", rva(0x45DC12), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// The Rust core; `r` as the original leaves it.
pub unsafe fn probe_and_keep_at(g: &Ground, r: &mut Regs) -> Result<(), Refused> {
    unsafe {
        probe_ground_at(g, r)?;
        for (from, to, n) in [
            (0x6D1E14u32, 0x6D2C14u32, 0xC0u32),
            (0x71ADC8, 0x6D2F14, 0x40),
        ] {
            for k in 0..n {
                g.put(to + 4 * k, g.get(from + 4 * k));
            }
            r.esi = g.a(from) + 4 * n;
            r.edi = g.a(to) + 4 * n;
        }
        r.ecx = 0;
        Ok(())
    }
}

/// The Rust version.
pub unsafe fn probe_and_keep(r: &mut Regs) -> Result<(), Refused> {
    unsafe { probe_and_keep_at(&Ground::game(), r) }
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    let mut g = *r;
    unsafe { probe_and_keep(&mut g) }?;
    Ok(Left::nothing()
        .with(EAX, g.eax)
        .with(EBX, g.ebx)
        .with(ECX, g.ecx)
        .with(EDX, g.edx)
        .with(ESI, g.esi)
        .with(EDI, g.edi))
}

fn all_regions() -> Vec<crate::service::harness::Region> {
    let g = Ground::game();
    let mut r = regions(&g);
    r.extend([(g.a(0x6D2C14), 0x300), (g.a(0x6D2F14), 0x100)]);
    r
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &all_regions(), &rust);
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
    let test = test_ground(1);
    let copies = match test.copies() {
        Ok(c) => c,
        Err(why) => return t.skip(&why),
    };
    let fg = copies.find_ground.addr();
    let probe = match test.copy(PROBE_GROUND, &|a| (a == 0x441E29).then_some(fg)) {
        Ok(c) => c,
        Err(why) => return t.skip(&why),
    };
    let p = probe.addr();
    let copy = match test.copy(PROBE_AND_KEEP, &|a| (a == 0x45D305).then_some(p)) {
        Ok(c) => c,
        Err(why) => return t.skip(&why),
    };
    for (case, count) in [0u32, 1, 4, 6, 9].into_iter().enumerate() {
        let seed = case as u32 * 0x9E37_79B9 + 5;
        check_case(
            &mut t,
            &format!("case {case}, count {count}"),
            copy.addr(),
            &test,
            Regs::SENTINEL,
            &|| prepare_probe(&test, seed, count, case % 2 == 0),
            &|g, r| unsafe { probe_and_keep_at(g, r) }.map(|()| false),
            false,
        );
    }
    drop((copy, probe, copies));
    t.finish()
}
