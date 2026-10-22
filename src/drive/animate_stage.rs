//! `drive::animate_stage` (0x45E9CA): `drive::animate_lit_faces`, then
//! `physics::move_slots`.
//!
//! In:  nothing
//! Out: esi past the slots, ecx 8 (`move_slots`); eax, ebx, edx as `animate_lit_faces`
//!      leaves them; no flags read (callers go on with calls)
//!
//! Callers: race init's loop (`sub_4608BB`, 0x46093E) and `sub_460B2F` (0x460BE2).
//! Refuses what the two refuse; what was written before a refusal is put back first.
//! Self-test: in a race only: a copy of the original (its two calls the game's) against
//! the Rust version on the game's own state, with the lit faces' list as the stage has
//! it; everything written saved and put back.

use crate::drive::animate_lit_faces::animate_lit_faces;
use crate::errors::Refused;
use crate::framework::{
    EAX, EBX, ECX, EDX, ESI, RawHook, Regs, SelfTest, call_game, game_addr, hook_asm, rva,
};
use crate::physics::move_slots::move_slots;
use crate::screens::harness::Left;
use crate::service::harness::{Region, capture, dispatch_state, restore, state_entry};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("drive::animate_stage", rva(0x45E9CA), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// Everything the two write.
pub fn regions() -> Vec<Region> {
    let mut r = crate::drive::animate_lit_faces::regions();
    r.extend(crate::physics::move_slot::regions());
    r.push((game_addr(rva(0x71BAC8)) as u32, 0x3C8));
    r
}

/// The Rust version: `r` as the original leaves it.
pub unsafe fn animate_stage(r: &mut Regs) -> Result<(), Refused> {
    unsafe {
        let all = regions();
        let saved = capture(&all);
        let result = (|| {
            animate_lit_faces(r)?;
            let left = move_slots(r)?;
            left.apply(r);
            Ok(())
        })();
        if result.is_err() {
            restore(&all, &saved);
        }
        result
    }
}

fn left_of(g: &Regs) -> Left {
    Left::nothing()
        .with(EAX, g.eax)
        .with(EBX, g.ebx)
        .with(ECX, g.ecx)
        .with(EDX, g.edx)
        .with(ESI, g.esi)
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    let mut g = *r;
    unsafe { animate_stage(&mut g) }?;
    Ok(left_of(&g))
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
    if !crate::physics::move_slot::in_race() {
        return t.skip("not in a race (no stage's faces)");
    }
    let copy = match crate::audio::test_copy::make(
        &HOOK,
        &crate::audio::test_copy::Copy {
            address: 0x45E9CA,
            len: 0xB,
            entry: &[0xE8, 0x37, 0x01, 0x00, 0x00],
            calls: &[0, 5],
            jumps_out: &[],
        },
        &|_| None,
        &[],
        0,
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(&why),
    };
    let all = regions();
    let saved = capture(&all);
    let mut ro = Regs::SENTINEL;
    unsafe { call_game(&mut ro, copy.addr()) };
    let theirs = capture(&all);
    restore(&all, &saved);
    let mut g = Regs::SENTINEL;
    let mine = unsafe { animate_stage(&mut g) };
    let state = capture(&all);
    restore(&all, &saved);
    let regs_ok = mine.is_ok() && left_of(&g).matches(&Regs::SENTINEL, &ro);
    t.check(regs_ok && theirs == state, || {
        let first = theirs.iter().zip(&state).position(|(a, b)| a != b);
        format!("regs_ok={regs_ok} {mine:?}, first difference at byte {first:?}")
    });
    drop(copy);
    t.finish()
}
