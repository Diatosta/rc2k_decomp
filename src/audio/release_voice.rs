//! `audio::release_voice` (0x40B767): a voice's buffers released and the voice made empty.
//!
//! In:  esi = the voice
//! Out: every register preserved
//!
//! With A3D (`g_useA3d` 1, the chunk at 0x40CD47): its buffer, if any, is stopped (A3D's
//! +0x38, `sub_40CEBB`) and released. Otherwise a buffer is stopped (`Stop`, +0x48,
//! `sub_40B963`), then its 3D buffer and the buffer released (`Release`, +8) and cleared.
//! Then the voice: `flags`, `data_key` .. `length`, `format`, `frames`, `priority`,
//! `kind`, `state`, `dirty`, `channel`, `next` 0; `own_frequency` 44100, `scale` and
//! `volume_factor` 0x40, `field_50` 1.0. Callers: `audio::start` (0x40A0DB) and
//! `audio::free_voices`.
//! Shadow mode runs the original alone (a buffer is released once). Self-test: a
//! redirected copy whose A3D switch is the test's, voices with test objects, 12 cases.

use crate::audio::com::{self, RELEASE, TestObjects, take_noted};
use crate::audio::voices::*;
use crate::framework::{RawHook, Regs, SelfTest, call_game, game_read, hook_asm, rva, shadow};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("audio::release_voice", rva(0x40B767), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const STOP: u32 = 0x48;
const A3D_STOP: u32 = 0x38;

// ---- Rust version ----

pub unsafe fn release_voice(at: u32, a3d: bool) {
    unsafe {
        let v = voice(at);
        if a3d {
            if v.buffer != 0 {
                com::call(v.buffer, A3D_STOP);
                if v.buffer != 0 {
                    com::call(v.buffer, RELEASE);
                    v.buffer = 0;
                }
            }
        } else if v.buffer != 0 {
            com::call(v.buffer, STOP);
            if v.buffer_3d != 0 {
                com::call(v.buffer_3d, RELEASE);
                v.buffer_3d = 0;
            }
            if v.buffer != 0 {
                com::call(v.buffer, RELEASE);
                v.buffer = 0;
            }
        }
        v.flags = 0;
        v.data_key = 0;
        v.source = 0;
        v.next_copy = 0;
        v.data = 0;
        v.length = 0;
        v.own_frequency = 44100;
        v.format = 0;
        v.frames = 0;
        v.scale = 0x40;
        v.priority = 0;
        v.kind = 0;
        v.state = 0;
        v.dirty = 0;
        v.channel = 0;
        v.next = 0;
        v.volume_factor = 0x40;
        v.field_50 = 1.0f32.to_bits();
    }
}

/// With the game's A3D switch.
pub unsafe fn release_voice_in_game(at: u32) {
    unsafe { release_voice(at, game_read::<u32>(A3D_RVA) == 1) }
}

// ---- hook ----

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    unsafe { release_voice_in_game(r.esi) };
}

// ---- self-test ----

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let mut a3d = Box::new(0u32);
    let a3d_at = &mut *a3d as *mut u32 as u32;
    let copy = match crate::audio::test_copy::make(
        &HOOK,
        &crate::audio::test_copy::Copy {
            address: 0x40B767,
            len: 0xC4,
            entry: &[0x83, 0x3D, 0x60, 0x6D, 0x50, 0x00, 0x01],
            calls: &[0x14],
            jumps_out: &[0x7],
        },
        &|_| None,
        &[(0x506D60, a3d_at)],
        1,
    ) {
        Ok(copy) => copy,
        Err(why) => return t.skip(&why),
    };
    let objects = TestObjects::new(2, &[(0x08, 0), (0x38, 0), (0x48, 0)]);
    let mut v = Box::new(Voice::default());
    let at = &mut *v as *mut Voice as u32;
    for switch in [0u32, 1, 2] {
        for (buffer, buffer_3d) in [(0, 0), (1, 0), (0, 1), (1, 1)] {
            let prepare = |v: &mut Voice| {
                let words = v as *mut Voice as *mut u32;
                for i in 0..0x1C {
                    unsafe { words.add(i).write(0xB000_0000 | i as u32) };
                }
                v.buffer = if buffer == 1 { objects.get(0) } else { 0 };
                v.buffer_3d = if buffer_3d == 1 { objects.get(1) } else { 0 };
            };
            *a3d = switch;
            prepare(&mut v);
            let before = Regs {
                esi: at,
                ..Regs::SENTINEL
            };
            let mut ro = before;
            unsafe { call_game(&mut ro, copy.addr()) };
            let theirs = (*v, take_noted());
            prepare(&mut v);
            unsafe { release_voice(at, switch == 1) };
            let mine = (*v, take_noted());
            t.check(theirs == mine && ro.preserved_except(&before, 0), || {
                format!(
                    "A3D {switch}, buffer {buffer}, 3D buffer {buffer_3d}: original {theirs:x?}; rust {mine:x?}"
                )
            });
        }
    }
    t.finish()
}
