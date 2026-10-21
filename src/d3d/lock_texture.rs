//! `d3d::lock_texture` (0x41AA4B): system texture eax locked, made the blitters' surface.
//! With no surface (+0x14 0) nothing; locked already (+0x60 1): fatal error 0x8021A171;
//! else its surface's `Lock` (no rectangle, the description `dword_5ABA18`, flags 1): a
//! failure is fatal error 0x8021A172; +0x60 1 and `d3d::texture_surface_info`.
//!
//! In:  eax the texture
//! Out: the flags of `cmp [record + 0x60], 1` (carry: not locked); every register kept
//!
//! Callers: the name and time plates, the mud, the mirror, the marks' masks, ... (most
//! `jb`). The lock is the surface's own method (`d3d::texture_lock`): DirectDraw in the
//! game, `platform_3d`'s in the standalone. Refuses a texture past the pool's 0x140,
//! before anything. Shadow mode runs the original alone (a surface cannot be locked
//! twice). Self-test: `texture_lock::test_slot` with a surface of the test's.

use crate::d3d::texture_lock::*;
use crate::errors::{BadInput, Refused};
use crate::framework::{RawHook, Regs, SelfTest, game_addr, hook_asm, rva, shadow};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("d3d::lock_texture", rva(0x41AA4B), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// The Rust version (see the header): the registers it leaves.
pub unsafe fn lock_texture(entry: &Regs) -> Result<Regs, Refused> {
    use rc2k_core::d3d::textures::{LOCK_FAILED, LOCKED_TWICE, Lock, lock};
    // Converted: `rc2k_core::d3d::textures::lock`.
    let r = record(entry.eax);
    let surface = if entry.eax < 0x140 { get(r + 0x14) } else { 0 };
    match lock(entry.eax, surface, || get(r + 0x60)).map_err(BadInput::from)? {
        Lock::Nothing => {}
        Lock::Twice => return Err(fatal(LOCKED_TWICE).into()),
        Lock::Lock => {
            let d = game_addr(DESCRIPTION_RVA) as u32;
            if unsafe { method(surface, 0x64, &[0, d, 1, 0]) } != 0 {
                return Err(fatal(LOCK_FAILED).into());
            }
            put(r + 0x60, 1);
            crate::d3d::texture_surface_info::texture_surface_info(r)?;
        }
    }
    let mut g = *entry;
    let (carry, zero) = compared_with_one(get(r + 0x60));
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
    match unsafe { lock_texture(r) } {
        Ok(left) => *r = Regs { esp: r.esp, ..left },
        Err(e) => unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) },
    }
}

fn self_test() -> (usize, usize) {
    let Some(t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    // (surface, locked, width): locked already and a failing lock are the original's fatal
    // errors, not run
    test_slot(
        t,
        call_original,
        &[
            (false, 0, 0x100),
            (false, 1, 0x100),
            (true, 0, 0x100),
            (true, 0, 0x40),
            (true, 2, 0x80),
        ],
        &|r| unsafe { lock_texture(r) },
    )
}
