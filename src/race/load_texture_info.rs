//! `race::load_texture_info` (0x4398FA): the course's texture and mark info read (`load texture info`).
//!
//! In:  nothing
//! Out: esi, edi, ecx as `copy_texture_info_tail` leaves them; the rest preserved
//!
//! See `race/level_init.rs`. One caller, `sub_44426F` (`race::load_stage`'s
//! "loadlevel_init"). Shadow mode runs the original alone. Self-test: a redirected copy
//! whose variables and memory are the test's and whose calls note themselves.

use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva, shadow};
use crate::race::level_init::*;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("race::load_texture_info", rva(0x4398FA), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

fn rust(r: &Regs, v: &Vars) -> Result<Regs, crate::errors::Refused> {
    unsafe { load_texture_info(r, v, &mut game_part) }
}

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    match rust(r, &Vars::game()) {
        Ok(left) => {
            *r = Regs {
                esp: r.esp,
                eflags: r.eflags,
                ..left
            }
        }
        Err(e) => unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) },
    }
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    {
        let left = rust(r, &Vars::game())?;
        *r = Regs {
            esp: r.esp,
            eflags: r.eflags,
            ..left
        }
    }
    Ok(())
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let m = TestMemory::new();
    let copy = match copy_of(
        &m,
        0x4398FA,
        0x7B,
        &[0x8B, 0x35, 0xA0, 0x6F, 0x5C, 0x00],
        &[0x14, 0x34, 0x54, 0x75],
        4,
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(&why),
    };
    let v = m.vars();
    let put = |at: u32, value: u32| unsafe { (at as *mut u32).write_unaligned(value) };
    let _ = (&v, &put);
    let info = m.info.as_ptr() as u32;
    both(
        &mut t,
        "the test's memory",
        &copy,
        &m,
        &|| put(v.course_info, info),
        &|r| unsafe { load_texture_info(r, &v, &mut test_part) },
    );
    t.finish()
}
