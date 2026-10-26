//! `race::reload_objects` (0x425B2F): the objects reloaded for the stage, with their lines
//! in the log (`LOADLEVEL_INIT : reloadobjects`).
//!
//! In:  nothing
//! Out: the registers as `race::set_object_kinds` leaves them
//!
//! See `race/objects.rs`. Two callers (`race::loadlevel_init` and one more). Shadow mode
//! runs the original alone (it loads and logs). Self-test: a redirected copy whose eleven
//! calls note themselves.

use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva, shadow};
use crate::race::mud::{copy, run_both, test_part};
use crate::race::objects::*;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("race::reload_objects", rva(0x425B2F), thunk, &ORIGINAL).self_test(self_test);

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
    match unsafe { reload_objects(r, &mut game_part) } {
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
        let g = unsafe { reload_objects(r, &mut game_part) }?;
        *r = Regs {
            esp: r.esp,
            eflags: r.eflags,
            ..g
        }
    }
    Ok(())
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let copy = match copy(
        0x425B2F,
        0x85,
        &[0x60, 0x8D, 0x05, 0xE6, 0xB8, 0x4D, 0x00],
        &[
            0x07, 0x14, 0x1A, 0x26, 0x32, 0x3F, 0x4B, 0x58, 0x64, 0x71, 0x7E,
        ],
        CALLEES,
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    run_both(&mut t, "the calls", &copy, &[], &|| {}, &|r| unsafe {
        reload_objects(r, &mut test_part)
    });
    t.finish()
}
