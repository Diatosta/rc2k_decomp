//! `d3d::unlock_texture` (0x41AB02): system texture eax unlocked: with a surface (+0x14)
//! that is locked (+0x60 1), its `Unlock` (no rectangle; a failure is fatal error
//! 0x8021A173) and +0x60 0.
//!
//! In:  eax the texture
//! Out: the flags as the original leaves them (unlocked: zero set, carry clear; no
//!      surface: those of `cmp [+0x14], 0`; not locked: of `cmp [+0x60], 1`); every
//!      register kept
//!
//! Callers: those of `d3d::lock_texture`. The unlock is the surface's own method. Refuses
//! a texture past the pool's 0x140, before anything. Shadow mode runs the original alone.
//! Self-test: `texture_lock::test_slot` with a surface of the test's.

use crate::d3d::texture_lock::*;
use crate::errors::{BadInput, Refused};
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva, shadow};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("d3d::unlock_texture", rva(0x41AB02), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// The Rust version (see the header): the registers it leaves.
pub unsafe fn unlock_texture(entry: &Regs) -> Result<Regs, Refused> {
    if entry.eax >= 0x140 {
        return Err(BadInput("a texture past the pool").into());
    }
    let r = record(entry.eax);
    let surface = get(r + 0x14);
    let mut g = *entry;
    let (carry, zero) = if surface == 0 {
        (false, true)
    } else if get(r + 0x60) != 1 {
        compared_with_one(get(r + 0x60))
    } else {
        if unsafe { method(surface, 0x80, &[0]) } != 0 {
            return Err(fatal(0x8021A173).into());
        }
        put(r + 0x60, 0);
        (false, true)
    };
    g.set_carry(carry);
    g.set_zero(zero);
    Ok(g)
}

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    match unsafe { unlock_texture(r) } {
        Ok(left) => *r = Regs { esp: r.esp, ..left },
        Err(e) => unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) },
    }
}

fn self_test() -> (usize, usize) {
    let Some(t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    test_slot(
        t,
        call_original,
        &[
            (false, 1, 0x100),
            (true, 0, 0x100),
            (true, 1, 0x100),
            (true, 2, 0x40),
        ],
        &|r| unsafe { unlock_texture(r) },
    )
}
