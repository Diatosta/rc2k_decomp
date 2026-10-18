//! `audio::read_splash` (0x40C8CD): A3D's splash settings read, and turned off while the game runs.
//!
//! In:  nothing
//! Out: every register preserved
//!
//! See `audio/a3d.rs`: only with `g_a3dSplashKept` 1 (A3D found at the start) does it
//! touch the registry (HKLM\SOFTWARE\Aureal\A3D, "SplashScreen" and "SplashAudio"); the
//! Rust version refuses then (not ported). Shadow mode runs the original alone.
//! Self-test: a redirected copy whose flag is the test's, 2 cases (not 1: that would touch
//! the registry).

use crate::audio::a3d::*;
use crate::audio::buffer_play::{Leaves, dispatch_leaf};
use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, call_game, game_addr, hook_asm, rva};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("audio::read_splash", rva(0x40C8CD), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

unsafe fn rust(r: &Regs) -> Result<Leaves, Refused> {
    unsafe { splash(game_addr(SPLASH_KEPT_RVA) as u32) }?;
    Ok(Leaves {
        eax: r.eax,
        carry: None,
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
    let c = match crate::audio::test_copy::make(
        &HOOK,
        &crate::audio::test_copy::Copy {
            address: 0x40C8CD,
            len: 0xC3,
            entry: &[0x60, 0x83, 0x3D, 0x60, 0x6C, 0x50, 0x00, 0x01],
            // The registry's calls and `write_splash` (paths the tests do not take).
            calls: &[0x1D, 0x46, 0x75, 0x8E, 0xB0],
            jumps_out: &[],
        },
        &|_| None,
        &[(0x506C60, at)],
        1,
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(&why),
    };
    for flag in [0u32, 2] {
        unsafe { (at as *mut u32).write_volatile(flag) };
        let before = Regs::SENTINEL;
        let mut ro = before;
        unsafe { call_game(&mut ro, c.addr()) };
        let preserved = ro.preserved_except(&before, 0);
        let mine = unsafe { splash(at) }.is_ok();
        t.check(mine && preserved, || {
            format!("flag {flag}: original preserved {preserved}; rust {mine}")
        });
    }
    drop(vars);
    t.finish()
}
