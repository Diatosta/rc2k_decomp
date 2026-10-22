//! `race::clear_race_tables` (0x4632D4): entries 0 .. 9 of thirteen ten-entry tables
//! (`dword_71C010`, `71C038`, `71BFA8`, `71BFD0`, `71C088`, `71C0E8` .. `71C0F4` (four,
//! a dword apart), `71C188` .. `71C194` (four)) set to 0.
//!
//! In:  nothing
//! Out: every register preserved (`pusha` / `popa`)
//!
//! See `race/race_cars.rs`. One caller, `race::set_up_mode_2_cars`. Shadow mode: the Rust
//! version, the tables put back, the original's, compared. Self-test: the game's tables
//! filled with a pattern, both versions, put back.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::race::race_cars::*;
use crate::screens::harness::Left;
use crate::service::harness::{capture, check_state, dispatch_state, restore, state_entry};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("race::clear_race_tables", rva(0x4632D4), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

fn rust(_: &Regs) -> Result<Left, Refused> {
    unsafe { clear_race_tables() };
    Ok(Left::nothing())
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &race_table_regions(), &rust);
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
    let regions = race_table_regions();
    let saved = capture(&regions);
    for fill in [0x5Au8, 0xFF] {
        check_state(
            &mut t,
            &format!("filled {fill:#x}"),
            call_original,
            Regs::SENTINEL,
            &|| {
                for &(a, len) in &regions {
                    unsafe { std::ptr::write_bytes(a as *mut u8, fill, len) };
                }
            },
            &regions,
            &rust,
        );
    }
    restore(&regions, &saved);
    t.finish()
}
