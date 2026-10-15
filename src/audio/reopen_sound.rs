//! `audio::reopen_sound` (0x40A52A): DirectSound opened again when the mode needs another.
//!
//! In:  nothing
//! Out: carry = `audio::open_sound`'s, when it ran; every register preserved
//!
//! See `audio/sound_device.rs`: when `g_soundMode` and `g_openedSoundMode` differ in their
//! 0xF000 bits: `audio::close_sound` and `audio::open_sound` with eax 0. One caller,
//! `audio::set_mode`. Shadow mode runs the original alone. Self-test: a redirected copy
//! whose two calls go to stand-ins, 8 cases.

use crate::audio::buffer_play::{Leaves, dispatch_leaf};
use crate::audio::sound_device::*;
use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, call_game, game_addr, hook_asm, rva};
use crate::install::stand_ins::set_handler;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("audio::reopen_sound", rva(0x40A52A), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

unsafe fn rust(r: &Regs) -> Result<Leaves, Refused> {
    let carry = unsafe {
        reopen_sound(
            game_addr(crate::audio::voices::SOUND_MODE_RVA) as u32,
            game_addr(OPENED_MODE_RVA) as u32,
            &mut game_part,
        )
    }?;
    Ok(Leaves {
        eax: r.eax,
        carry: Some(carry),
        zero: None,
    })
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_leaf(&HOOK, r, call_original, rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let mut vars = Box::new([0u32; 2]);
    let at = vars.as_mut_ptr() as u32;
    let c = match copy(
        &HOOK,
        0x40A52A,
        0x21,
        &[0x60, 0xA1, 0xBC, 0x82, 0x4F, 0x00],
        &[0x13, 0x1A],
        &[],
        &[(0x4F82BC, at), (0x4F82C0, at + 4)],
        2,
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(&why),
    };
    set_handler(handler);
    for (mode, opened) in [
        (0x11221u32, 0x11221u32),
        (0x11221, 0x10221),
        (0x11221, 0x12221),
        (0x2441220, 0x2441221),
    ] {
        for open_fails in [false, true] {
            let run = |rust: bool| {
                unsafe { (at as *mut [u32; 2]).write_volatile([mode, opened]) };
                *PART_CARRIES.lock().unwrap() = [false, false, false, open_fails];
                noted();
                let before = Regs::SENTINEL;
                let mut ro = before;
                let carry = if rust {
                    unsafe { reopen_sound(at, at + 4, &mut test_part) }.ok()
                } else {
                    unsafe { call_game(&mut ro, c.addr()) };
                    ro.preserved_except(&before, 0).then(|| ro.carry())
                };
                (carry, noted())
            };
            let theirs = run(false);
            let mine = run(true);
            t.check(theirs.0.is_some() && theirs == mine, || {
                format!("mode {mode:#x}, opened {opened:#x}, open fails {open_fails}: original {theirs:x?}; rust {mine:x?}")
            });
        }
    }
    *PART_CARRIES.lock().unwrap() = [false; 4];
    drop(vars);
    t.finish()
}
