//! `race::clear_player_absences` (0x449D44): the eight car blocks' +0xB0 (the player
//! absent, `race::drop_absent_player`'s test) 0.
//!
//! In:  nothing
//! Out: every register preserved
//!
//! See `race/race_shell.rs`. Callers: `sub_442FEF`, `sub_44306B`, `sub_443104` and
//! `sub_446EBE` (none reads flags after it). Shadow mode: the Rust version, the blocks put
//! back, the original's, compared. Self-test: the game's eight values set, put back.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::race::race_shell::*;
use crate::screens::harness::Left;
use crate::service::harness::{capture, check_state, dispatch_state, restore, state_entry};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "race::clear_player_absences",
    rva(0x449D44),
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

fn rust(_: &Regs) -> Result<Left, Refused> {
    unsafe { clear_player_absences() };
    Ok(Left::nothing())
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &absence_regions(), &rust);
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
    let regions = absence_regions();
    let saved = capture(&regions);
    for fill in [0u8, 1, 0xFF] {
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
