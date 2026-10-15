//! `audio::release_voice_manager` (0x40C74C): DirectSound's voice manager (its `IKsPropertySet`) released.
//!
//! In:  nothing
//! Out: every register preserved
//!
//! If `g_voiceManager` holds an object, its `Release` (+8), and the variable 0. Callers:
//! `audio::start` (0x40A0D6) and `audio::free_voices`.
//! Shadow mode runs the original alone. Self-test: a redirected copy whose variable is the
//! test's, with and without a test object.

use crate::audio::com::{self, RELEASE, TestObjects, take_noted};
use crate::audio::voices::*;
use crate::framework::{RawHook, Regs, SelfTest, call_game, game_addr, hook_asm, rva, shadow};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "audio::release_voice_manager",
    rva(0x40C74C),
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

// ---- Rust version ----

pub unsafe fn release_voice_manager(at: u32) {
    unsafe {
        let object = read(at);
        if object != 0 {
            com::call(object, RELEASE);
            write(at, 0);
        }
    }
}

// ---- hook ----

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    unsafe { release_voice_manager(game_addr(VOICE_MANAGER_RVA) as u32) };
}

// ---- self-test ----

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let mut var = Box::new(0u32);
    let at = &mut *var as *mut u32 as u32;
    let copy = match crate::audio::test_copy::make(
        &HOOK,
        &crate::audio::test_copy::Copy {
            address: 0x40C74C,
            len: 0x21,
            entry: &[0x60, 0x83, 0x3D, 0x58, 0x6C, 0x50, 0x00, 0x00],
            calls: &[],
            jumps_out: &[],
        },
        &|_| None,
        &[(0x506C58, at)],
        3,
    ) {
        Ok(copy) => copy,
        Err(why) => return t.skip(&why),
    };
    let objects = TestObjects::new(1, &[(0x08, 0)]);
    for object in [0, objects.get(0)] {
        *var = object;
        let mut ro = Regs::SENTINEL;
        unsafe { call_game(&mut ro, copy.addr()) };
        let theirs = (*var, take_noted());
        *var = object;
        unsafe { release_voice_manager(at) };
        let mine = (*var, take_noted());
        t.check(
            theirs == mine && ro.preserved_except(&Regs::SENTINEL, 0),
            || format!("object {object:#x}: original {theirs:x?}; rust {mine:x?}"),
        );
    }
    t.finish()
}
