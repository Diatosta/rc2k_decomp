//! `audio::set_master_volume` (0x40A740): the primary buffer's volume.
//!
//! In:  eax = the volume, 0 .. 0x40
//! Out: every register preserved
//!
//! Nothing with A3D (`g_useA3d` 1: `nullsub_46`) or without a primary buffer
//! (`g_primaryBuffer`); else its `SetVolume` (+0x3C) with `g_volumeTable` at the volume,
//! held to 0x40 (unsigned). Callers: `sub_40A54B` (the mode's set-up) and
//! `audio::volume_command`.
//! Shadow mode runs the original alone. Self-test: a redirected copy whose switch and
//! buffer are the test's, 72 cases.

use crate::audio::com::{self, TestObjects, take_noted};
use crate::audio::voices::*;
use crate::framework::{
    RawHook, Regs, SelfTest, call_game, game_addr, game_read, hook_asm, rva, shadow,
};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("audio::set_master_volume", rva(0x40A740), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const SET_VOLUME: u32 = 0x3C;

// ---- Rust version ----

/// `primary`: where the primary buffer is kept.
pub unsafe fn set_master_volume(volume: u32, a3d: bool, primary: u32) {
    unsafe {
        if a3d {
            return;
        }
        let buffer = read(primary);
        if buffer == 0 {
            return;
        }
        // Converted: `rc2k_core::audio::levels::master_volume`.
        let level =
            rc2k_core::audio::levels::master_volume(volume, crate::audio::buffer_settings::volumes);
        com::call1(buffer, SET_VOLUME, level);
    }
}

pub unsafe fn set_master_volume_in_game(volume: u32) {
    unsafe {
        set_master_volume(
            volume,
            game_read::<u32>(A3D_RVA) == 1,
            game_addr(PRIMARY_RVA) as u32,
        )
    }
}

// ---- hook ----

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    unsafe { set_master_volume_in_game(r.eax) };
}

// ---- self-test ----

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let mut vars = Box::new([0u32; 2]);
    let at = vars.as_mut_ptr() as u32;
    let copy = match crate::audio::test_copy::make(
        &HOOK,
        &crate::audio::test_copy::Copy {
            address: 0x40A740,
            len: 0x35,
            entry: &[0x83, 0x3D, 0x60, 0x6D, 0x50, 0x00, 0x01],
            calls: &[],
            jumps_out: &[0x7],
        },
        &|_| None,
        &[(0x506D60, at), (0x506A8C, at + 4)],
        3,
    ) {
        Ok(copy) => copy,
        Err(why) => return t.skip(&why),
    };
    let objects = TestObjects::new(1, &[(0x3C, 1)]);
    for switch in [0u32, 1, 2] {
        for buffer in [0, objects.get(0)] {
            for volume in [
                0u32,
                1,
                0x20,
                0x3F,
                0x40,
                0x41,
                0x8000_0000,
                0xFFFF_FFFF,
                7,
                0x30,
                0x10,
                0x200,
            ] {
                *vars = [switch, buffer];
                let before = Regs {
                    eax: volume,
                    ..Regs::SENTINEL
                };
                let mut ro = before;
                unsafe { call_game(&mut ro, copy.addr()) };
                let theirs = take_noted();
                unsafe { set_master_volume(volume, switch == 1, at + 4) };
                let mine = take_noted();
                t.check(theirs == mine && ro.preserved_except(&before, 0), || {
                    format!(
                        "A3D {switch}, buffer {buffer:#x}, volume {volume:#x}: original {theirs:x?}; rust {mine:x?}"
                    )
                });
            }
        }
    }
    t.finish()
}
