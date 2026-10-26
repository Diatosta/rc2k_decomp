//! `race::init_checkpoints` (0x45A98F): the checkpoints listed (`initcheckpoints`).
//!
//! In:  nothing
//! Out: edx the end, ebp the count - 1, ebx and esi the last entry's, al its byte; with fewer than two kinds every register preserved
//!
//! See `race/level_init.rs`. One caller, `sub_44426F` (`race::load_stage`'s
//! "loadlevel_init"). Shadow mode runs the original alone. Self-test: a redirected copy
//! whose variables and memory are the test's and whose calls note themselves.

use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva, shadow};
use crate::race::level_init::*;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("race::init_checkpoints", rva(0x45A98F), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

fn rust(r: &Regs, v: &Vars) -> Result<Regs, crate::errors::Refused> {
    unsafe { init_checkpoints(r, v) }
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
        0x45A98F,
        0x4F,
        &[0xC7, 0x05, 0x30, 0x18, 0x60, 0x00, 0x00, 0x00, 0x00, 0x00],
        &[],
        7,
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(&why),
    };
    let v = m.vars();
    let put = |at: u32, value: u32| unsafe { (at as *mut u32).write_unaligned(value) };
    let _ = (&v, &put);
    let records = m.records.as_ptr() as u32;
    for (kinds, end) in [
        (1u32, 5u32),
        (2, 2),
        (2, 9),
        (3, 0x40),
        (0xFFFF_FFFF, 3),
        (2, 0x300),
    ] {
        both(
            &mut t,
            &format!("kinds {kinds}, end {end}"),
            &copy,
            &m,
            &|| unsafe {
                put(v.checkpoint_kinds, kinds);
                put(v.checkpoint_count, end);
                put(v.checkpoint_records, records);
                put(v.checkpoints_found, 0x55);
                // 0x300 entries over the 0x40 records, a third of them flagged.
                for n in 0..0x300u32 {
                    put(v.checkpoint_table + 4 * n, (n * 5) % 0x40);
                }
                for n in 0..0x40u32 {
                    *((records + n * 0x40 + 0x26) as *mut u8) = (n % 3 == 0) as u8 * 7;
                }
            },
            &|r| unsafe { init_checkpoints(r, &v) },
        );
    }
    t.finish()
}
