//! `race::reset_refresh` (0x45C53B): `dword_63E174` and `dword_63E178` 0x80000000 (`refresh init`).
//!
//! In:  nothing
//! Out: every register preserved
//!
//! See `race/level_init.rs`. One caller, `sub_44426F` (`race::load_stage`'s
//! "loadlevel_init"). Shadow mode runs the original alone. Self-test: a redirected copy
//! whose variables and memory are the test's and whose calls note themselves.

use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva, shadow};
use crate::race::level_init::*;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("race::reset_refresh", rva(0x45C53B), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

fn rust(r: &Regs, v: &Vars) -> Result<Regs, crate::errors::Refused> {
    unsafe { reset_refresh(v) };
    Ok(*r)
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

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let m = TestMemory::new();
    let copy = match copy_of(
        &m,
        0x45C53B,
        0x15,
        &[0xC7, 0x05, 0x74, 0xE1, 0x63, 0x00, 0x00, 0x00, 0x00, 0x80],
        &[],
        2,
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(&why),
    };
    let v = m.vars();
    let put = |at: u32, value: u32| unsafe { (at as *mut u32).write_unaligned(value) };
    let _ = (&v, &put);
    both(
        &mut t,
        "from 0",
        &copy,
        &m,
        &|| {
            put(v.refresh[0], 0);
            put(v.refresh[1], 0);
        },
        &|r| {
            unsafe { reset_refresh(&v) };
            Ok(*r)
        },
    );
    t.finish()
}
