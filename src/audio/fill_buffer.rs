//! `audio::fill_buffer` (0x40B82B): a voice's sample data copied into its buffer.
//!
//! In:  esi = the voice
//! Out: carry = failed; every register preserved
//!
//! Only for a voice with a buffer of its own (none shared): `Lock` (+0x2C) from 0 for its
//! length into `g_lockedParts` (0x506A98: two pointers and their sizes), the data copied
//! into the first part and, if there is one, on into the second, `Unlock` (+0x4C) with
//! them. A failing `Lock` or `Unlock` sets the carry, and with `DSERR_BUFFERLOST`
//! (0x88780096) `g_soundBufferLost`. Callers: command 3 and `sub_40A7C9` (the lost
//! buffers' restoring).
//! Refuses A3D (the chunk at 0x40CDE8).
//! Shadow mode runs the original alone. Self-test: a redirected copy whose parts and flag
//! are the test's, on the test's DirectSound, 15 cases.

use crate::audio::buffer_settings::a3d;
use crate::audio::voices::*;
use crate::errors::{BadInput, Refused};
use crate::framework::{RawHook, Regs, SelfTest, call_game, game_addr, hook_asm, rva, shadow};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("audio::fill_buffer", rva(0x40B82B), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

pub const LOCKED_PARTS_RVA: usize = rva(0x506A98); // g_lockedParts
const BUFFER_LOST: i32 = 0x8878_0096u32 as i32;

/// `parts`: the four dwords the parts go to; `lost`: the lost buffer's flag. True for the
/// carry clear.
pub unsafe fn fill_buffer(v: &Voice, parts: u32, lost: u32) -> Result<bool, Refused> {
    if a3d() {
        return Err(BadInput("A3D (the chunk at 0x40CDE8)").into());
    }
    if v.buffer == 0 || v.source != 0 {
        return Ok(true);
    }
    unsafe {
        let failed = |r: i32| {
            if r == BUFFER_LOST {
                write(lost, 1);
            }
            false
        };
        let lock: unsafe extern "system" fn(u32, u32, u32, u32, u32, u32, u32, u32) -> i32 =
            std::mem::transmute(crate::audio::com::method(v.buffer, 0x2C));
        let r = lock(
            v.buffer,
            0,
            v.length,
            parts,
            parts + 4,
            parts + 8,
            parts + 12,
            0,
        );
        if r != 0 {
            return Ok(failed(r));
        }
        let [p1, n1, p2, n2] = (parts as *const [u32; 4]).read_unaligned();
        std::ptr::copy(v.data as *const u8, p1 as *mut u8, n1 as usize);
        if p2 != 0 && n2 != 0 {
            std::ptr::copy((v.data + n1) as *const u8, p2 as *mut u8, n2 as usize);
        }
        let unlock: unsafe extern "system" fn(u32, u32, u32, u32, u32) -> i32 =
            std::mem::transmute(crate::audio::com::method(v.buffer, 0x4C));
        let r = unlock(v.buffer, p1, n1, p2, n2);
        if r != 0 {
            return Ok(failed(r));
        }
    }
    Ok(true)
}

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    let parts = game_addr(LOCKED_PARTS_RVA) as u32;
    let lost = game_addr(BUFFER_LOST_RVA) as u32;
    match unsafe { fill_buffer(voice(r.esi), parts, lost) } {
        Ok(done) => r.set_carry(!done),
        Err(e) => unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) },
    }
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    if a3d() {
        return t.skip("A3D is on");
    }
    let mut vars = Box::new([0u32; 5]);
    let at = vars.as_mut_ptr() as u32;
    let mut a3d_var = Box::new(0u32);
    let copy = match crate::audio::test_copy::make(
        &HOOK,
        &crate::audio::test_copy::Copy {
            address: 0x40B82B,
            len: 0xA6,
            entry: &[0x83, 0x3D, 0x60, 0x6D, 0x50, 0x00, 0x01],
            calls: &[],
            jumps_out: &[0x7],
        },
        &|_| None,
        &[
            (0x506D60, &mut *a3d_var as *mut u32 as u32),
            (0x506A98, at),
            (0x506A9C, at + 4),
            (0x506AA0, at + 8),
            (0x506AA4, at + 12),
            (0x4F82CC, at + 16),
        ],
        14,
    ) {
        Ok(copy) => copy,
        Err(why) => return t.skip(&why),
    };
    let mut sound = crate::audio::test_sound::TestSound::new();
    let data: Vec<u8> = (0..0x200u32).map(|i| (i * 13) as u8).collect();
    let mut case = 0;
    for (buffer, shared) in [(true, false), (false, false), (true, true)] {
        for (lock, unlock, parts) in [
            (0, 0, (0x100u32, 0u32)),
            (0, 0, (0x80, 0x80)),
            (BUFFER_LOST, 0, (0x100, 0)),
            (-1, 0, (0x100, 0)),
            (0, BUFFER_LOST, (0x40, 0xC0)),
        ] {
            case += 1;
            let v = Voice {
                buffer: if buffer { sound.buffer(2) } else { 0 },
                source: if shared { 0x1234 } else { 0 },
                data: data.as_ptr() as u32,
                length: 0x100,
                ..Voice::default()
            };
            let results = crate::audio::test_sound::Results {
                lock,
                unlock,
                parts,
                ..Default::default()
            };
            let mut run = |rust: bool| {
                *vars = [7, 7, 7, 7, 0];
                sound.memory.fill(0xAA);
                sound.begin(results);
                let before = Regs {
                    esi: &v as *const Voice as u32,
                    ..Regs::SENTINEL
                };
                let mut ro = before;
                let done = if rust {
                    unsafe { fill_buffer(&v, at, at + 16) }.map(|d| (d, true))
                } else {
                    unsafe { call_game(&mut ro, copy.addr()) };
                    Ok((!ro.carry(), ro.preserved_except(&before, 0)))
                };
                (done, *vars, sound.memory.to_vec(), sound.noted())
            };
            let theirs = run(false);
            let mine = run(true);
            let ok = matches!(theirs.0, Ok((_, true)))
                && theirs.0.as_ref().ok().map(|d| d.0) == mine.0.as_ref().ok().map(|d| d.0)
                && theirs.1 == mine.1
                && theirs.2 == mine.2
                && theirs.3 == mine.3;
            t.check(ok, || {
                format!(
                    "case {case}: buffer {buffer}, shared {shared}, results {lock:#x} {unlock:#x} {parts:x?}: original {:?} {:x?} {:x?}; rust {:?} {:x?} {:x?}",
                    theirs.0, theirs.1, theirs.3, mine.0, mine.1, mine.3
                )
            });
        }
    }
    t.finish()
}
