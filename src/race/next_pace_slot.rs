//! `race::next_pace_slot` (0x441A4B): the pace notes' slot (`dword_614D34`) on by one;
//! at 8 or past (unsigned) back to 0 and ebp (the checkpoint) on by one.
//!
//! In:  ebp the checkpoint, `dword_614D34` the slot
//! Out: ebp; the rest kept; no flags read (the callers `jmp` or `cmp` next)
//!
//! Callers: `race::init_pace_notes` (four times). See `race/pace_notes.rs`.
//! Self-test: the original and the Rust version on the game's slot (put back): slots 0,
//! 6, 7, 8 and past.

use crate::framework::{EBP, RawHook, Regs, SelfTest, game_write, hook_asm, rva};
use crate::race::pace_notes::{Course, Machine, SLOT_RVA};
use crate::screens::harness::Left;
use crate::service::harness::{at, capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("race::next_pace_slot", rva(0x441A4B), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// The Rust version: the slot written, ebp left.
pub fn next_pace_slot(r: &Regs) -> Result<Left, crate::errors::Refused> {
    let mut m = Machine::load(r, Course::game());
    m.next_slot();
    unsafe { game_write(SLOT_RVA, m.slot) };
    Ok(Left::nothing().with(EBP, m.r.ebp))
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &[at(SLOT_RVA)], &next_pace_slot);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let regions = [at(SLOT_RVA)];
    let saved = capture(&regions);
    for slot in [0u32, 6, 7, 8, 9, 0xFFFF_FFFF] {
        for ebp in [0u32, 0x41, 0xFFFF_FFFF] {
            check_state(
                &mut t,
                &format!("slot {slot:#x}, ebp {ebp:#x}"),
                call_original,
                Regs {
                    ebp,
                    ..Regs::SENTINEL
                },
                &|| unsafe { game_write(SLOT_RVA, slot) },
                &regions,
                &next_pace_slot,
            );
        }
    }
    restore(&regions, &saved);
    t.finish()
}
