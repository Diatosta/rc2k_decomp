//! `audio::close_way` (0x40AC58): the turn's way given up by the sound mode.
//!
//! In:  nothing
//! Out: carry as the way chosen leaves it (none chosen: set); every register preserved
//!
//! See `audio/mixer_set_up.rs`: bit 0x10000 (the voices' own buffers): a `jnz` into
//! `audio::clear_channels`; bit 0x40000 (the mixer): a `jnz` into `audio::close_primary_mixer`; else `stc`.
//! Shadow mode runs the original alone. Self-test: a redirected copy that runs on into
//! `clear_channels`' first bytes, where a jump to a stand-in is written (the short `jnz`
//! cannot be moved), the near one pointed at a stand-in too, 3 modes x 2 carries.

use crate::audio::buffer_play::{Leaves, dispatch_leaf};
use crate::audio::mixer_set_up::*;
use crate::audio::restore::{CARRIES, game_part, handler, noted, test_part};
use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, call_game, game_read, hook_asm, rva};
use crate::install::stand_ins::{set_handler, stand_in};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("audio::close_way", rva(0x40AC58), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

unsafe fn rust(r: &Regs) -> Result<Leaves, Refused> {
    let mode = unsafe { game_read(crate::audio::voices::SOUND_MODE_RVA) };
    let carry = close_way(mode, &mut game_part)?;
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
    let mut vars = Box::new([0u32; 1]);
    let at = vars.as_mut_ptr() as u32;
    // From the entry to `clear_channels`' (0x40ACA1) first five bytes.
    let len = 0x40ACA1 - 0x40AC58 + 5;
    let c = match crate::audio::test_copy::make(
        &HOOK,
        &crate::audio::test_copy::Copy {
            address: 0x40AC58,
            len,
            entry: &[0xF7, 0x05, 0xBC, 0x82, 0x4F, 0x00, 0x00, 0x00, 0x01, 0x00],
            calls: &[],
            jumps_out: &[],
        },
        &|_| None,
        &[(0x4F82BC, at)],
        // Two of its own, two of `update_voices`' (0x40AC77), which the copy runs past but
        // never into.
        4,
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(&why),
    };
    unsafe {
        // At `clear_channels`' entry: `jmp` to place 6's stand-in.
        let entry = len - 5;
        c.patch_u32(entry, 0xE9);
        c.retarget(entry + 1, stand_in(6));
        // The near `jnz` (0F 85 at +0x16) to the mixer's part: its stand-in.
        c.retarget(0x16 + 2, stand_in(10));
    }
    set_handler(handler);
    for mode in [0x11221u32, 0x2441220, 0x100000] {
        for fails in [false, true] {
            let run = |rust: bool| {
                unsafe { (at as *mut u32).write_volatile(mode) };
                {
                    let mut c = CARRIES.lock().unwrap();
                    *c = [false; 12];
                    c[6] = fails;
                    c[10] = fails;
                }
                noted();
                let before = Regs::SENTINEL;
                let mut ro = before;
                let carry = if rust {
                    close_way(mode, &mut test_part).ok()
                } else {
                    unsafe { call_game(&mut ro, c.addr()) };
                    ro.preserved_except(&before, 0).then(|| ro.carry())
                };
                (carry, noted())
            };
            let theirs = run(false);
            let mine = run(true);
            t.check(theirs.0.is_some() && theirs == mine, || {
                format!(
                    "mode {mode:#x}, the way's carry {fails}: original {theirs:x?}; rust {mine:x?}"
                )
            });
        }
    }
    *CARRIES.lock().unwrap() = [false; 12];
    drop(vars);
    t.finish()
}
