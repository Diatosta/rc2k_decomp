//! `audio::close_primary_mixer` (0x40BBCB): the mixer's way given up.
//!
//! In:  nothing
//! Out: carry = `audio::make_primary`'s, when it ran; every register preserved
//!
//! See `audio/restore.rs`: `audio::clear_channels`; in the mixer's mode (0x40000) with a
//! DirectSound: `audio::release_primary`, cooperative level 3 (exclusive) again,
//! `audio::make_primary`. Callers: `audio::close_way` (by `jnz`), the mixer's set-up when it
//! fails. Shadow mode runs the original alone. Self-test: a redirected copy on the test
//! DirectSound whose three calls go to stand-ins, 8 cases.

use crate::audio::buffer_play::{Leaves, dispatch_leaf};
use crate::audio::restore::*;
use crate::audio::test_sound::{Results, TestSound};
use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, call_game, game_addr, hook_asm, rva};
use crate::install::stand_ins::set_handler;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "audio::close_primary_mixer",
    rva(0x40BBCB),
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

unsafe fn rust(r: &Regs) -> Result<Leaves, Refused> {
    let carry = unsafe {
        close_primary_mixer(
            game_addr(crate::audio::voices::SOUND_MODE_RVA) as u32,
            game_addr(crate::audio::make_buffer::DIRECT_SOUND_RVA) as u32,
            game_addr(crate::audio::sound_device::WINDOW_RVA) as u32,
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
    // [0] mode, [1] DirectSound, [2] window.
    let mut vars = Box::new([0u32; 3]);
    let at = vars.as_mut_ptr() as u32;
    let c = match copy(
        &HOOK,
        &crate::audio::test_copy::Copy {
            address: 0x40BBCB,
            len: 0x3A,
            entry: &[0x60, 0xE8, 0xD0, 0xF0, 0xFF, 0xFF],
            calls: &[0x01, 0x1B, 0x33],
            jumps_out: &[],
        },
        &[(0x4F82BC, at), (0x506A88, at + 4), (0x5184A4, at + 8)],
        4,
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(&why),
    };
    set_handler(handler);
    let sound = TestSound::new();
    for mode in [0x2441220u32, 0x11221] {
        for ds in [true, false] {
            for make_fails in [false, true] {
                let run = |rust: bool| {
                    unsafe {
                        (at as *mut [u32; 3]).write_volatile([
                            mode,
                            if ds { sound.sound() } else { 0 },
                            0x1234,
                        ])
                    };
                    {
                        let mut c = CARRIES.lock().unwrap();
                        *c = [false; 12];
                        c[8] = make_fails;
                    }
                    sound.begin(Results::default());
                    noted();
                    let before = Regs::SENTINEL;
                    let mut ro = before;
                    let carry = if rust {
                        unsafe { close_primary_mixer(at, at + 4, at + 8, &mut test_part) }.ok()
                    } else {
                        unsafe { call_game(&mut ro, c.addr()) };
                        ro.preserved_except(&before, 0).then(|| ro.carry())
                    };
                    (carry, noted(), sound.noted())
                };
                let theirs = run(false);
                let mine = run(true);
                t.check(theirs.0.is_some() && theirs == mine, || {
                    format!("mode {mode:#x}, DirectSound {ds}, make fails {make_fails}: original {theirs:x?}; rust {mine:x?}")
                });
            }
        }
    }
    *CARRIES.lock().unwrap() = [false; 12];
    drop(vars);
    t.finish()
}
