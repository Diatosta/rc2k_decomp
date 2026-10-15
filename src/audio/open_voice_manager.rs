//! `audio::open_voice_manager` (0x40C6B5): DirectSound's voice manager ("DSV") opened
//! through a 3D buffer.
//!
//! In:  ebx = where the 3D buffer is (a voice's `buffer_3d`)
//! Out: carry = not opened; every register preserved
//!
//! See `audio/property_sets.rs`: nothing with `g_ignoreDsvSetting` or the set already
//! there (`g_voiceManager`); its mode set to
//! 2, "AUD: Using DSV" logged, `g_voiceManagerOn` 1. One caller, `audio::make_buffer` with bit
//! 0x800000. Shadow mode runs the original alone. Self-test: a redirected copy whose three
//! variables are the test's and whose log call notes eax, 9 cases.

use crate::audio::buffer_play::{Leaves, dispatch_leaf};
use crate::audio::property_sets::*;
use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("audio::open_voice_manager", rva(0x40C6B5), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// True for the carry clear.
pub unsafe fn open_voice_manager(holder: u32) -> Result<bool, Refused> {
    unsafe { open_property_set(&Opening::voice_manager(), holder, &mut game_log) }
}

unsafe fn rust(r: &Regs) -> Result<Leaves, Refused> {
    let clear = unsafe { open_voice_manager(r.ebx) }?;
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
            address: 0x40C6B5,
            len: 0x97,
            entry: &[0x60, 0x83, 0x3D, 0x50, 0x6C, 0x50, 0x00, 0x00],
            calls: &[0x80],
            jumps_out: &[],
        },
        &Opening::voice_manager(),
        [0x506C50, 0x506C58, 0x506C54],
        6,
    )
}
