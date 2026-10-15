//! `audio::restore_buffer` (0x40A862): a lost buffer's memory restored.
//!
//! In:  eax = where the buffer's pointer is
//! Out: carry = `Restore` failed; edx = the status (when there is a buffer and `Restore`
//!      did not run; after it, as it leaves edx), eax = `Restore`'s result (when it ran);
//!      ecx as the COM calls leave it; the rest preserved
//!
//! See `audio/restore.rs`. No buffer: nothing. `GetStatus` (+0x24); with
//! `DSBSTATUS_BUFFERLOST` (2): `Restore` (+0x50), whose failure sets `g_soundBufferLost`.
//! Refuses A3D (0x40CB0A). Callers: `audio::restore_lost`. Shadow mode runs the original
//! alone. Self-test: a redirected copy on the test DirectSound's buffer, 12 cases.

use crate::audio::restore::*;
use crate::audio::test_sound::{Results, TestSound};
use crate::framework::{
    EAX, ECX, EDX, RawHook, Regs, SelfTest, call_game, game_addr, hook_asm, rva, shadow,
};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("audio::restore_buffer", rva(0x40A862), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

fn apply(r: &mut Regs, left: Restored) {
    if let Some(eax) = left.eax {
        r.eax = eax;
    }
    if let Some(edx) = left.edx {
        r.edx = edx;
    }
    r.set_carry(left.carry);
}

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    let lost = game_addr(crate::audio::voices::BUFFER_LOST_RVA) as u32;
    match unsafe { restore_buffer(r.eax, lost) } {
        Ok(left) => apply(r, left),
        Err(e) => unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) },
    }
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    if crate::audio::buffer_settings::a3d() {
        return t.skip("A3D is on");
    }
    // [0] the buffer's pointer, [1] the lost flag.
    let mut vars = Box::new([0u32; 2]);
    let at = vars.as_mut_ptr() as u32;
    let c = match crate::audio::test_copy::make(
        &HOOK,
        &crate::audio::test_copy::Copy {
            address: 0x40A862,
            len: 0x42,
            entry: &[0x83, 0x3D, 0x60, 0x6D, 0x50, 0x00, 0x01],
            calls: &[],
            jumps_out: &[0x07],
        },
        &|_| None,
        &[(0x4F82CC, at + 4)],
        1,
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(&why),
    };
    let sound = TestSound::new();
    for buffer in [0, sound.buffer(1)] {
        for status in [0u32, 2, 3] {
            for result in [0, 0x8878_0096u32 as i32] {
                let run = |rust: bool| {
                    unsafe { (at as *mut [u32; 2]).write_volatile([buffer, 0]) };
                    sound.begin(Results {
                        status,
                        restore: result,
                        ..Results::default()
                    });
                    let before = Regs {
                        eax: at,
                        ..Regs::SENTINEL
                    };
                    let mut r = before;
                    let ok = if rust {
                        match unsafe { restore_buffer(at, at + 4) } {
                            Ok(left) => {
                                apply(&mut r, left);
                                true
                            }
                            Err(_) => false,
                        }
                    } else {
                        unsafe { call_game(&mut r, c.addr()) };
                        r.preserved_except(&before, EAX | ECX | EDX)
                    };
                    // After `Restore` edx is the COM call's: not compared (the test's stub
                    // and DirectSound leave different things there).
                    let restored = buffer != 0 && status & BUFFER_LOST != 0;
                    let edx = if restored { None } else { Some(r.edx) };
                    (ok, r.carry(), r.eax, edx, sound.noted(), unsafe {
                        (at as *const [u32; 2]).read_volatile()
                    })
                };
                let theirs = run(false);
                let mine = run(true);
                t.check(theirs.0 && theirs == mine, || {
                    format!("buffer {buffer:#x}, status {status}, Restore {result:#x}: original {theirs:x?}; rust {mine:x?}")
                });
            }
        }
    }
    drop(vars);
    t.finish()
}
