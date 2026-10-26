//! `race::build_texture_infos` (0x4393A0): the 256 textures' records built from their kinds (`initgametexinfbuf`).
//!
//! In:  nothing
//! Out: ecx 0x100, esi the last record, edi and eax the last pass's; with no course info every register preserved
//!
//! See `race/level_init.rs`. One caller, `sub_44426F` (`race::load_stage`'s
//! "loadlevel_init"). Shadow mode runs the original alone. Self-test: a redirected copy
//! whose variables and memory are the test's and whose calls note themselves.

use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva, shadow};
use crate::race::level_init::*;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("race::build_texture_infos", rva(0x4393A0), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

fn rust(r: &Regs, v: &Vars) -> Result<Regs, crate::errors::Refused> {
    unsafe { build_texture_infos(r, v) }
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
        0x4393A0,
        0xAF,
        &[0x83, 0x3D, 0xA0, 0x6F, 0x5C, 0x00, 0x00],
        &[],
        6,
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(&why),
    };
    let v = m.vars();
    let put = |at: u32, value: u32| unsafe { (at as *mut u32).write_unaligned(value) };
    let _ = (&v, &put);
    let info = m.info.as_ptr() as u32;
    for (n, with_info) in [(1u32, true), (7, true), (3, false)] {
        both(
            &mut t,
            &format!("kinds by {n}, info {with_info}"),
            &copy,
            &m,
            &|| unsafe {
                put(v.course_info, if with_info { info } else { 0 });
                for k in 0..0x100u32 {
                    put(v.texture_kinds + 4 * k, k * n * 0x13 % 0x3000);
                }
                for (i, b) in std::slice::from_raw_parts_mut(info as *mut u8, 0x18_0000)
                    .iter_mut()
                    .enumerate()
                {
                    *b = (i * 7 + n as usize) as u8;
                }
            },
            &|r| unsafe { build_texture_infos(r, &v) },
        );
    }
    t.finish()
}
