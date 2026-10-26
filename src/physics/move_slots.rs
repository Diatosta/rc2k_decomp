//! `physics::move_slots` (0x462206): `dword_71BE88` = the number of the eight
//! `g_raceSlots` (0x78 bytes each) in use (+0 not 0), each moved by
//! `physics::move_slot` (inside `pusha` / `popa`).
//!
//! In:  nothing
//! Out: esi past the slots, ecx 8; the rest kept; no flags read (`retn`, then `sub_45E9CA`
//!      returns)
//!
//! Caller: `sub_45E9CA` (0x45E9CF), from race init's loop and `sub_460B2F`.
//! Refuses (before anything) what `move_slot` refuses for a slot in use.
//! Self-test: in a race only: a copy of the original whose slots and count are the
//! test's (its calls to the game's `move_slot`) against the Rust version, slots in use
//! and not; every game variable `move_slot` writes saved and put back.

use crate::errors::Refused;
use crate::framework::{ECX, ESI, RawHook, Regs, SelfTest, call_game, game_addr, hook_asm, rva};
use crate::physics::move_slot::{
    SLOT_LEN, fill_slots, in_race, move_slot_at, regions, slot_refusal,
};
use crate::screens::harness::Left;
use crate::service::harness::{capture, dispatch_state, restore, state_entry};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("physics::move_slots", rva(0x462206), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const SLOTS_RVA: usize = rva(0x71BAC8); // g_raceSlots
const COUNT_RVA: usize = rva(0x71BE88); // dword_71BE88

/// The Rust core: the slots at `slots`, the count at `count`.
pub unsafe fn move_slots_at(slots: u32, count: u32, r: &Regs) -> Result<Left, Refused> {
    unsafe {
        for k in 0..8 {
            let s = slots + SLOT_LEN * k;
            if (s as *const u32).read_unaligned() != 0
                && let Some(why) = slot_refusal(s)
            {
                return Err(crate::errors::BadInput(why).into());
            }
        }
        let mut n = 0u32;
        for k in 0..8 {
            let s = slots + SLOT_LEN * k;
            if (s as *const u32).read_unaligned() != 0 {
                n += 1;
                (count as *mut u32).write_unaligned(n);
                let mut g = Regs {
                    esi: s,
                    ecx: k,
                    ..*r
                };
                move_slot_at(&mut g)?;
            }
        }
        (count as *mut u32).write_unaligned(n);
        Ok(Left::nothing().with(ESI, slots + 8 * SLOT_LEN).with(ECX, 8))
    }
}

/// The Rust version.
pub unsafe fn move_slots(r: &Regs) -> Result<Left, Refused> {
    unsafe { move_slots_at(game_addr(SLOTS_RVA) as u32, game_addr(COUNT_RVA) as u32, r) }
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    unsafe { move_slots(r) }
}

extern "C" fn dispatch(r: &mut Regs) {
    let mut regions = regions();
    regions.push((game_addr(SLOTS_RVA) as u32, 8 * SLOT_LEN as usize + 8));
    dispatch_state(&HOOK, r, call_original, &regions, &rust);
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
    if !in_race() {
        return t.skip("not in a race (no stage's faces)");
    }
    let mut cells = vec![0u32; (8 * SLOT_LEN as usize + 8) / 4];
    let slots = cells.as_mut_ptr() as u32;
    let count = slots + 8 * SLOT_LEN;
    let copy = match crate::audio::test_copy::make(
        &HOOK,
        &crate::audio::test_copy::Copy {
            address: 0x462206,
            len: 0x31,
            entry: &[0xC7, 0x05, 0x88, 0xBE, 0x71, 0x00, 0x00, 0x00, 0x00, 0x00],
            calls: &[0x21],
            jumps_out: &[],
        },
        &|_| None,
        &[(0x71BE88, count), (0x71BAC8, slots)],
        3,
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(&why),
    };
    let mut all = regions();
    all.push((slots, 8 * SLOT_LEN as usize + 8));
    let saved = capture(&all);
    for seed in 1..=6u32 {
        fill_slots(slots, 8, seed);
        for k in 0..8u32 {
            if (k + seed) % 3 == 0 {
                unsafe { ((slots + SLOT_LEN * k) as *mut u32).write(0) };
            }
        }
        let input = capture(&all);
        let mut ro = Regs::SENTINEL;
        unsafe { call_game(&mut ro, copy.addr()) };
        let theirs = capture(&all);
        restore(&all, &input);
        let mine = unsafe { move_slots_at(slots, count, &Regs::SENTINEL) };
        let state = capture(&all);
        restore(&all, &saved);
        let regs_ok = mine.as_ref().is_ok_and(|l| l.matches(&Regs::SENTINEL, &ro));
        t.check(regs_ok && theirs == state, || {
            let first = theirs.iter().zip(&state).position(|(a, b)| a != b);
            format!("seed {seed}: regs_ok={regs_ok} {mine:?}, first difference at byte {first:?}")
        });
    }
    drop((copy, cells));
    t.finish()
}
