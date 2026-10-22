//! `race::clear_race_sound_objects` (0x415C6D): `race::silence_race_sounds`;
//! `dword_514F78`, `dword_514F7C`, `dword_514F8C` 0.
//!
//! In:  nothing
//! Out: every register preserved
//!
//! See `race/sound_tables.rs`. Four callers (`sub_4843EC` and menus; a call follows each).
//! Its call at +1: private copy, the call relocated, manual patch. Shadow mode runs the
//! original alone (it sends commands). Self-test: a copy whose call goes to a stand-in,
//! against the Rust version on the same.

use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva, shadow};
use crate::race::sound_tables::*;
use crate::service::harness::{capture, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "race::clear_race_sound_objects",
    rva(0x415C6D),
    thunk,
    &ORIGINAL,
)
.private_copy(0x26)
.relocate_calls(&[0x1])
.manual_patch()
.self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    if let Err(e) = unsafe { clear_race_sound_objects(r, &mut object_part) } {
        unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) };
    }
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    unsafe { clear_race_sound_objects(r, &mut object_part) }?;
    Ok(())
}

fn self_test() -> (usize, usize) {
    use crate::race::mud::{copy, run_both, test_part};
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let copy = match copy(
        0x415C6D,
        0x26,
        &[0x60, 0xE8, 0xF3, 0x76, 0xFF, 0xFF],
        &[0x1],
        &[(0x40D366, SILENCE)],
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    let regions = object_regions();
    let saved = capture(&regions);
    run_both(
        &mut t,
        "counts set",
        &copy,
        &regions,
        &|| {
            for &(a, len) in &regions {
                unsafe { std::ptr::write_bytes(a as *mut u8, 0x5A, len) };
            }
        },
        &|r| unsafe { clear_race_sound_objects(r, &mut test_part) },
    );
    restore(&regions, &saved);
    t.finish()
}
