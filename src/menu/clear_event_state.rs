//! `menu::clear_event_state` (0x486C2C): an event's state cleared at its start.
//!
//! In:  nothing
//! Out: every register preserved
//!
//! `dword_755950`, `dword_758714` and the 28 dwords `dword_758758` .. `dword_7587C4` set to
//! 0 (the results screens and the savegame read them; their meaning is not read). Callers:
//! the event loops (`sub_482382` and four more). Self-test: the variables filled with 0xAA,
//! put back afterwards.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_addr, hook_asm, rva};
use crate::screens::harness::Left;
use crate::service::harness::{
    Region, at, capture, check_state, dispatch_state, restore, state_entry,
};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("menu::clear_event_state", rva(0x486C2C), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const FIRST_RVA: usize = rva(0x755950); // dword_755950
const SECOND_RVA: usize = rva(0x758714); // dword_758714
const RUN_RVA: usize = rva(0x758758); // dword_758758 .. dword_7587C4
const RUN_DWORDS: usize = 28;

fn regions() -> [Region; 3] {
    [
        at(FIRST_RVA),
        at(SECOND_RVA),
        (game_addr(RUN_RVA) as u32, RUN_DWORDS * 4),
    ]
}

pub fn clear_event_state() {
    for (addr, len) in regions() {
        unsafe { std::ptr::write_bytes(addr as *mut u8, 0, len) };
    }
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    clear_event_state();
    Ok(Left::nothing())
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
    let regions = regions();
    let saved = capture(&regions);
    check_state(
        &mut t,
        "filled with 0xAA",
        call_original,
        Regs::SENTINEL,
        &|| {
            for (addr, len) in regions {
                unsafe { std::ptr::write_bytes(addr as *mut u8, 0xAA, len) };
            }
        },
        &regions,
        &rust,
    );
    restore(&regions, &saved);
    t.finish()
}
