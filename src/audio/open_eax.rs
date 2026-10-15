//! `audio::open_eax` (0x40D27F): EAX 1's reverb opened through a 3D buffer.
//!
//! In:  ebx = where the 3D buffer is (a voice's `buffer_3d`)
//! Out: carry = not opened; every register preserved
//!
//! See `audio/property_sets.rs`: nothing with `g_ignoreEaxSetting` or the set already there
//! (`g_eaxProperties`); "AUD: Using EAX" logged, `g_hasEax` 1. One caller,
//! `audio::make_buffer` with bit 0x1000000, where ebx is not always the holder (the game's
//! bug, `notes/bugs.md`: refused when no pointer). Shadow mode runs the original alone.
//! Self-test: a redirected copy whose three variables are the test's and whose log call
//! notes eax, 9 cases.

use crate::audio::buffer_play::{Leaves, dispatch_leaf};
use crate::audio::property_sets::*;
use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("audio::open_eax", rva(0x40D27F), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// True for the carry clear.
pub unsafe fn open_eax(holder: u32) -> Result<bool, Refused> {
    unsafe { open_property_set(&Opening::eax(), holder, &mut game_log) }
}

unsafe fn rust(r: &Regs) -> Result<Leaves, Refused> {
    let clear = unsafe { open_eax(r.ebx) }?;
    Ok(Leaves {
        eax: r.eax,
        carry: Some(!clear),
        zero: None,
    })
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_leaf(&HOOK, r, call_original, rust);
}

fn self_test() -> (usize, usize) {
    let Some(t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    test_opening(
        t,
        &HOOK,
        &crate::audio::test_copy::Copy {
            address: 0x40D27F,
            len: 0x71,
            entry: &[0x60, 0x83, 0x3D, 0x6C, 0x6C, 0x50, 0x00, 0x00],
            calls: &[0x5A],
            jumps_out: &[],
        },
        &Opening::eax(),
        [0x506C6C, 0x506C74, 0x506C70],
        5,
    )
}
