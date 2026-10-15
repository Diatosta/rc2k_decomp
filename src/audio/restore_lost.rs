//! `audio::restore_lost` (0x40A7C9): the lost buffers restored, when the window is back.
//!
//! In:  nothing
//! Out: every register preserved
//!
//! See `audio/restore.rs`: `Sleep(1000)`; with `g_windowActive`: `g_soundBufferLost` 0,
//! `audio::restore_buffer` of the primary and of each voice's buffer (each voice then
//! `audio::fill_buffer`), `audio::play_primary`, `audio::replay_looping`; a failure ends
//! it. Refuses A3D (0x40CB0A). One caller, the audio thread's turn. Shadow mode runs the
//! original alone. Self-test: a redirected copy whose calls go to stand-ins (`Sleep` to one
//! that waits for nothing), 12 cases.

use crate::audio::buffer_play::{Leaves, dispatch_leaf};
use crate::audio::restore::*;
use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, call_game, hook_asm, rva};
use crate::install::stand_ins::set_handler;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("audio::restore_lost", rva(0x40A7C9), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

unsafe fn rust(r: &Regs) -> Result<Leaves, Refused> {
    unsafe { restore_lost(&RestoreVars::game(), &mut game_part) }?;
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
    if crate::audio::buffer_settings::a3d() {
        return t.skip("A3D is on");
    }
    // [0] lost, [1] primary, [2] voice count, [3] window active; then three voices.
    let mut memory = vec![0u32; 4 + 3 * 0x1C];
    let at = memory.as_mut_ptr() as u32;
    let v = RestoreVars {
        lost: at,
        primary: at + 4,
        voice_count: at + 8,
        voices: at + 16,
        window_active: at + 12,
    };
    let c = match copy(
        &HOOK,
        &crate::audio::test_copy::Copy {
            address: 0x40A7C9,
            len: 0x70,
            entry: &[0x83, 0x3D, 0x60, 0x6D, 0x50, 0x00, 0x01],
            calls: &[0x13, 0x31, 0x4B, 0x52, 0x5F, 0x64, 0x69],
            jumps_out: &[0x07],
        },
        &[
            (0x4F82CC, v.lost),
            (0x506A8C, v.primary),
            (0x4F8348, v.voice_count),
            (0x518374, v.window_active),
            (0x4F834C, v.voices),
        ],
        5,
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(&why),
    };
    set_handler(handler);
    for window in [1u32, 0] {
        for count in [0u32, 3] {
            for failing in [None, Some(0usize), Some(1)] {
                let run = |rust: bool| {
                    unsafe {
                        let m = at as *mut u32;
                        for n in 0..memory.len() {
                            m.add(n).write_volatile(0);
                        }
                        m.write_volatile(1);
                        m.add(2).write_volatile(count);
                        m.add(3).write_volatile(window);
                    }
                    {
                        let mut c = CARRIES.lock().unwrap();
                        *c = [false; 12];
                        if let Some(p) = failing {
                            c[p] = true;
                        }
                    }
                    noted();
                    let before = Regs::SENTINEL;
                    let mut ro = before;
                    let done = if rust {
                        unsafe { restore_lost(&v, &mut test_part) }.is_ok()
                    } else {
                        unsafe { call_game(&mut ro, c.addr()) };
                        ro.preserved_except(&before, 0)
                    };
                    (done, noted(), unsafe {
                        (at as *const [u32; 4]).read_volatile()
                    })
                };
                let theirs = run(false);
                let mine = run(true);
                t.check(theirs.0 && theirs == mine, || {
                    format!("window {window}, {count} voices, failing {failing:?}: original {theirs:x?}; rust {mine:x?}")
                });
            }
        }
    }
    *CARRIES.lock().unwrap() = [false; 12];
    drop(memory);
    t.finish()
}
