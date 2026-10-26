//! `race::start_race` (0x4278B7): `dword_621870` 0, then `race::run_race`.
//!
//! In:  nothing
//! Out: the registers as `race::run_race` leaves them
//!
//! See `race/race_end.rs`. One caller, `race::race_from_menu` (a `retn` follows).
//! Shadow mode runs the original alone (it runs the race). Self-test: a copy whose call
//! goes to a stand-in, against the Rust version on the same.

use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva, shadow};
use crate::race::race_end::*;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("race::start_race", rva(0x4278B7), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL,
    // It runs a whole race: a gated thunk would count the game's thread as inside a
    // hook throughout, and every test's pause wait out its 2 s (2026-10-18).
    ungated
);

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    match unsafe { start_race(r, &mut start_part) } {
        Ok(g) => {
            *r = Regs {
                esp: r.esp,
                eflags: r.eflags,
                ..g
            }
        }
        Err(e) => unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) },
    }
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    {
        let g = unsafe { start_race(r, &mut start_part) }?;
        *r = Regs {
            esp: r.esp,
            eflags: r.eflags,
            ..g
        }
    }
    Ok(())
}

fn self_test() -> (usize, usize) {
    use crate::race::mud::{copy, run_both, test_part};
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let copy = match copy(
        0x4278B7,
        0x10,
        &[0xC7, 0x05, 0x70, 0x18, 0x62, 0x00, 0x00, 0x00, 0x00, 0x00],
        &[0xA],
        &[(0x443F0B, 1)],
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    let regions = [crate::service::harness::at(rva(0x621870))];
    let saved = crate::service::harness::capture(&regions);
    run_both(
        &mut t,
        "the calls",
        &copy,
        &regions,
        &|| unsafe { crate::framework::game_write(rva(0x621870), 0x5A5A_5A5Au32) },
        &|r| unsafe { start_race(r, &mut test_part) },
    );
    crate::service::harness::restore(&regions, &saved);
    t.finish()
}
