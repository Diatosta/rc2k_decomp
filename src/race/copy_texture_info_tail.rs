//! `race::copy_texture_info_tail` (0x439AE5): 0x800 bytes of the course info moved on (from +0x17E800 to +0x17F000).
//!
//! In:  nothing
//! Out: esi and edi past the bytes, ecx 0; the rest preserved
//!
//! See `race/level_init.rs`. One caller, `sub_44426F` (`race::load_stage`'s
//! "loadlevel_init"). Shadow mode runs the original alone. Self-test: a redirected copy
//! whose variables and memory are the test's and whose calls note themselves.

use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva, shadow};
use crate::race::level_init::*;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "race::copy_texture_info_tail",
    rva(0x439AE5),
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

fn rust(r: &Regs, v: &Vars) -> Result<Regs, crate::errors::Refused> {
    Ok(unsafe { copy_texture_info_tail(r, v) })
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
    let copy = match copy_of(&m, 0x439AE5, 0x1E, &[0xBE, 0x00, 0xE8, 0x17, 0x00], &[], 2) {
        Ok(c) => c,
        Err(why) => return t.skip(&why),
    };
    let v = m.vars();
    let put = |at: u32, value: u32| unsafe { (at as *mut u32).write_unaligned(value) };
    let _ = (&v, &put);
    let info = m.info.as_ptr() as u32;
    for fill in [0x11u8, 0xEE] {
        both(
            &mut t,
            &format!("fill {fill:#x}"),
            &copy,
            &m,
            &|| unsafe {
                put(v.course_info, info);
                for (n, b) in std::slice::from_raw_parts_mut(info as *mut u8, 0x18_0000)
                    .iter_mut()
                    .enumerate()
                {
                    *b = fill ^ n as u8;
                }
            },
            &|r| Ok(unsafe { copy_texture_info_tail(r, &v) }),
        );
    }
    t.finish()
}
