//! `physics::apply_contact_forces` (0x43F555): `physics::apply_forces` on the 7 records at `unk_614A50` (`physics::clear_contacts`').
//!
//! In:  nothing
//! Out: esi the first record, ecx the count; the rest kept; no flags read
//!
//! Caller: `sub_43B635` (0x43B883), every physics step.
//! Self-test: a copy of the original (its records and car the test's, its call to a
//! copy of `apply_forces`) against the Rust core.

use crate::errors::Refused;
use crate::framework::{ECX, ESI, RawHook, Regs, SelfTest, call_game, game_addr, hook_asm, rva};
use crate::physics::apply_force::{BLOCK_LEN, apply_at, fill};
use crate::physics::apply_forces::{apply_forces, regions, test_copies};
use crate::screens::harness::Left;
use crate::service::harness::{capture, dispatch_state};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "physics::apply_contact_forces",
    rva(0x43F555),
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

const RECORDS_RVA: usize = rva(0x614A50);
const COUNT: u32 = 7;

/// The Rust version.
pub unsafe fn apply_contact_forces() -> Result<Left, Refused> {
    let first = game_addr(RECORDS_RVA) as u32;
    unsafe { apply_forces(first, COUNT) }?;
    Ok(Left::nothing().with(ESI, first).with(ECX, COUNT))
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    unsafe { apply_contact_forces() }
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &regions(), &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let mut cells = vec![0u32; (BLOCK_LEN as usize) / 4 + 8 * COUNT as usize + 4];
    let car = cells.as_mut_ptr() as u32;
    let recs = car + BLOCK_LEN;
    let work = recs + 0x20 * COUNT;
    let scratch = work + 12;
    let (inner, keep) = match test_copies(car, work, scratch) {
        Ok(c) => c,
        Err(why) => return t.skip(&why),
    };
    let copy = match crate::audio::test_copy::make(
        &HOOK,
        &crate::audio::test_copy::Copy {
            address: 0x43F555,
            len: 0x11,
            entry: &[0x8D, 0x35, 0x50, 0x4A, 0x61, 0x00],
            calls: &[0xB],
            jumps_out: &[],
        },
        &|r| (r == rva(0x43F566)).then_some(inner),
        &[(0x614A50, recs)],
        1,
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(&why),
    };
    let all = [(car, 4 * cells.len())];
    let fill_all = || {
        for k in 0..COUNT {
            fill(recs + 0x20 * k, car, 5 + k);
        }
    };
    fill_all();
    let mut ro = Regs::SENTINEL;
    unsafe { call_game(&mut ro, copy.addr()) };
    let theirs = capture(&all);
    fill_all();
    for k in 0..COUNT {
        unsafe { apply_at(recs + 0x20 * k, car, work, scratch) };
    }
    let mine = capture(&all);
    let regs_ok = Left::nothing()
        .with(ESI, recs)
        .with(ECX, COUNT)
        .matches(&Regs::SENTINEL, &ro);
    t.check(regs_ok && theirs == mine, || {
        format!("regs_ok={regs_ok}; the memory differs")
    });
    drop((copy, keep, cells));
    t.finish()
}
