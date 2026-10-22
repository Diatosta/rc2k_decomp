//! `physics::clear_contacts` (0x43F3D6): `physics::clear_records` on 7 records of 0x20 bytes at `unk_614A50` (by the look of
//! it the step's contacts, a reading).
//!
//! In:  nothing
//! Out: edi past the records, ecx 0; the rest kept; no flags read
//!
//! Caller: `sub_43B635` (0x43B85B), every physics step.
//! Self-test: a copy of the original (its records a test buffer, its call kept) against
//! the Rust version.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, call_game, game_addr, hook_asm, rva};
use crate::physics::clear_records::clear_records;
use crate::screens::harness::Left;
use crate::service::harness::{capture, dispatch_state, state_entry};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("physics::clear_contacts", rva(0x43F3D6), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const RECORDS_RVA: usize = rva(0x614A50);
const COUNT: u32 = 7;

/// The Rust version.
pub unsafe fn clear_contacts() -> Result<Left, Refused> {
    unsafe { clear_records(game_addr(RECORDS_RVA) as u32, COUNT) }
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    unsafe { clear_contacts() }
}

extern "C" fn dispatch(r: &mut Regs) {
    let at = game_addr(RECORDS_RVA) as u32;
    dispatch_state(
        &HOOK,
        r,
        call_original,
        &[(at, 0x20 * COUNT as usize)],
        &rust,
    );
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
    let mut buffer = vec![0u32; 8 * COUNT as usize + 8];
    let b = buffer.as_mut_ptr() as u32;
    let copy = match crate::audio::test_copy::make(
        &HOOK,
        &crate::audio::test_copy::Copy {
            address: 0x43F3D6,
            len: 0x11,
            entry: &[0x8D, 0x3D, 0x50, 0x4A, 0x61, 0x00],
            calls: &[0xB],
            jumps_out: &[],
        },
        &|_| None,
        &[(0x614A50, b)],
        1,
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(&why),
    };
    let all = [(b, buffer.len() * 4)];
    let fill = || {
        for k in 0..(8 * COUNT + 8) {
            unsafe { ((b + 4 * k) as *mut u32).write(0x5A5A_0000 | k) };
        }
    };
    fill();
    let mut ro = Regs::SENTINEL;
    unsafe { call_game(&mut ro, copy.addr()) };
    let theirs = capture(&all);
    fill();
    let mine = unsafe { clear_records(b, COUNT) };
    let state = capture(&all);
    let ok = mine.as_ref().is_ok_and(|l| l.matches(&Regs::SENTINEL, &ro)) && theirs == state;
    t.check(ok, || format!("{mine:?}, the records differ"));
    drop((copy, buffer));
    t.finish()
}
