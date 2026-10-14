//! `audio::release_eax` (0x40D2F0): the EAX property set released.
//!
//! In:  nothing
//! Out: every register preserved
//!
//! If `g_eaxProperties` holds an object, its `Release` (+8), the variable 0, and the room
//! asked for and the room set (`g_eaxRoom`, `g_eaxRoomSet`) both 6. Callers: `audio::start`
//! (0x40A0D1) and `audio::free_voices`.
//! Shadow mode runs the original alone. Self-test: a redirected copy whose three variables
//! are the test's, with and without a test object.

use crate::audio::com::{self, RELEASE, TestObjects, take_noted};
use crate::audio::voices::*;
use crate::framework::{RawHook, Regs, SelfTest, call_game, game_addr, hook_asm, rva, shadow};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("audio::release_eax", rva(0x40D2F0), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- Rust version ----

/// `vars`: the property set, the room asked for and the room set.
pub unsafe fn release_eax(vars: [u32; 3]) {
    unsafe {
        let object = read(vars[0]);
        if object != 0 {
            com::call(object, RELEASE);
            write(vars[0], 0);
            write(vars[1], 6);
            write(vars[2], 6);
        }
    }
}

pub fn game_vars() -> [u32; 3] {
    [EAX_RVA, EAX_ROOM_RVA, EAX_ROOM_SET_RVA].map(|r| game_addr(r) as u32)
}

// ---- hook ----

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    unsafe { release_eax(game_vars()) };
}

// ---- self-test ----

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let mut vars = Box::new([0u32; 3]);
    let at = vars.as_mut_ptr() as u32;
    let copy = match crate::audio::test_copy::make(
        &HOOK,
        &crate::audio::test_copy::Copy {
            address: 0x40D2F0,
            len: 0x35,
            entry: &[0x60, 0x83, 0x3D, 0x74, 0x6C, 0x50, 0x00, 0x00],
            calls: &[],
            jumps_out: &[],
        },
        &|_| None,
        &[(0x506C74, at), (0x506C78, at + 4), (0x506C7C, at + 8)],
        5,
    ) {
        Ok(copy) => copy,
        Err(why) => return t.skip(&why),
    };
    let objects = TestObjects::new(1, &[(0x08, 0)]);
    for object in [0, objects.get(0)] {
        *vars = [object, 2, 3];
        let mut ro = Regs::SENTINEL;
        unsafe { call_game(&mut ro, copy.addr()) };
        let theirs = (*vars, take_noted());
        *vars = [object, 2, 3];
        unsafe { release_eax([at, at + 4, at + 8]) };
        let mine = (*vars, take_noted());
        t.check(
            theirs == mine && ro.preserved_except(&Regs::SENTINEL, 0),
            || format!("object {object:#x}: original {theirs:x?}; rust {mine:x?}"),
        );
    }
    t.finish()
}
