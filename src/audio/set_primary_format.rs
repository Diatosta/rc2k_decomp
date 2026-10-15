//! `audio::set_primary_format` (0x40A677): the primary buffer's format set.
//!
//! In:  eax = the sound mode (its format bits)
//! Out: carry = no primary buffer, or `SetFormat` failed; every register preserved
//!
//! `audio::describe_wave_format` with the mode and `g_mixRateSetting`, then `SetFormat`
//! (+0x38) with `g_waveFormat`. One caller, `audio::make_primary`. See `audio/primary.rs`.
//! Shadow mode runs the original alone. Self-test: a redirected copy on the test
//! DirectSound's buffer, 12 cases.

use crate::audio::buffer_play::{Leaves, dispatch_leaf};
use crate::audio::primary::*;
use crate::audio::test_sound::{Results, TestSound};
use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, call_game, game_addr, game_read, hook_asm, rva};
use crate::service::harness::{capture, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("audio::set_primary_format", rva(0x40A677), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

unsafe fn rust(r: &Regs) -> Result<Leaves, Refused> {
    let primary = game_addr(crate::audio::voices::PRIMARY_RVA) as u32;
    let ok = unsafe { set_primary_format(primary, r.eax, game_read(MIX_RATE_SETTING_RVA)) }?;
    Ok(Leaves {
        eax: r.eax,
        carry: Some(!ok),
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
    let c = match crate::audio::thread_parts::copy_of(
        &HOOK,
        0x40A677,
        0x2E,
        &[0x60, 0x83, 0x3D, 0x8C, 0x6A, 0x50, 0x00, 0x00],
        &[0x10],
        &[],
        &[(0x506A8C, at), (0x4F82D4, at + 4)],
        3,
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(&why),
    };
    let format = [(
        game_addr(crate::audio::describe_wave_format::WAVE_FORMAT_RVA) as u32,
        0x12,
    )];
    let saved = capture(&format);
    let sound = TestSound::new();
    for primary in [0, sound.buffer(2)] {
        for result in [0, 0x8878_0032u32 as i32] {
            for mode in [0x11221u32, 0x11110, 0x2441220] {
                let run = |rust: bool| {
                    unsafe { (at as *mut [u32; 2]).write_volatile([primary, 44100]) };
                    restore(&format, &saved);
                    sound.begin(Results {
                        format: result,
                        ..Results::default()
                    });
                    let before = Regs {
                        eax: mode,
                        ..Regs::SENTINEL
                    };
                    let mut ro = before;
                    let carry = if rust {
                        unsafe { set_primary_format(at, mode, 44100) }
                            .ok()
                            .map(|ok| !ok)
                    } else {
                        unsafe { call_game(&mut ro, c.addr()) };
                        ro.preserved_except(&before, 0).then(|| ro.carry())
                    };
                    (carry, sound.noted(), capture(&format))
                };
                let theirs = run(false);
                let mine = run(true);
                t.check(theirs.0.is_some() && theirs == mine, || {
                    format!("primary {primary:#x}, SetFormat {result:#x}, mode {mode:#x}: original {theirs:x?}; rust {mine:x?}")
                });
            }
        }
    }
    restore(&format, &saved);
    drop(vars);
    t.finish()
}
