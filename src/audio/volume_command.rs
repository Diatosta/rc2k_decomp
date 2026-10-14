//! `audio::volume_command` (0x40B016): command 8: the master volume.
//!
//! In:  eax = the volume (the sender's eax)
//! Out: every register preserved
//!
//! Nothing for a negative volume; else it is kept in `g_masterVolume` and
//! `audio::set_master_volume` called with it. One caller, `audio::take_commands`.
//! Manual patch with a private copy (a short `js` in its first bytes; its call relocated).
//! Shadow mode runs the original alone. Self-test: a redirected copy whose variable is the
//! test's and whose call notes eax, 8 cases.

use crate::audio::set_master_volume::set_master_volume_in_game;
use crate::audio::voices::*;
use crate::framework::{RawHook, Regs, SelfTest, call_game, game_addr, hook_asm, rva, shadow};
use crate::install::stand_ins::{set_handler, stand_in};
use std::sync::Mutex;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new("audio::volume_command", rva(0x40B016), thunk, &ORIGINAL)
    .private_copy(0x11)
    .relocate_calls(&[0x0A])
    .manual_patch()
    .self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- Rust version ----

/// `master`: where the volume is kept; `set`: the master volume's setting.
pub unsafe fn volume_command(volume: u32, master: u32, set: &mut dyn FnMut(u32)) {
    if (volume as i32) < 0 {
        return;
    }
    unsafe { write(master, volume) };
    set(volume);
}

// ---- hook ----

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    unsafe {
        volume_command(r.eax, game_addr(MASTER_VOLUME_RVA) as u32, &mut |v| {
            set_master_volume_in_game(v)
        })
    };
}

// ---- self-test ----

static NOTED: Mutex<Vec<u32>> = Mutex::new(Vec::new());

fn handler(_: u32, r: &mut Regs) {
    NOTED.lock().unwrap().push(r.eax);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let mut var = Box::new(0u32);
    let at = &mut *var as *mut u32 as u32;
    let copy = match crate::audio::test_copy::make(
        &HOOK,
        &crate::audio::test_copy::Copy {
            address: 0x40B016,
            len: 0x11,
            entry: &[0x60, 0x85, 0xC0, 0x78, 0x0A],
            calls: &[0x0A],
            jumps_out: &[],
        },
        &|callee| (callee == rva(0x40A740)).then(|| stand_in(0)),
        &[(0x4F82D8, at)],
        1,
    ) {
        Ok(copy) => copy,
        Err(why) => return t.skip(&why),
    };
    set_handler(handler);
    for volume in [
        0u32,
        1,
        0x40,
        0x41,
        0x7FFF_FFFF,
        0x8000_0000,
        0xFFFF_FFFF,
        0x20,
    ] {
        *var = 0x5555;
        NOTED.lock().unwrap().clear();
        let before = Regs {
            eax: volume,
            ..Regs::SENTINEL
        };
        let mut ro = before;
        unsafe { call_game(&mut ro, copy.addr()) };
        let theirs = (*var, std::mem::take(&mut *NOTED.lock().unwrap()));
        *var = 0x5555;
        unsafe { volume_command(volume, at, &mut |v| NOTED.lock().unwrap().push(v)) };
        let mine = (*var, std::mem::take(&mut *NOTED.lock().unwrap()));
        t.check(theirs == mine && ro.preserved_except(&before, 0), || {
            format!("volume {volume:#x}: original {theirs:x?}; rust {mine:x?}")
        });
    }
    t.finish()
}
