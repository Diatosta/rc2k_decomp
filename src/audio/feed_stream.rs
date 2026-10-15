//! `audio::feed_stream` (0x40C349): a stream's turn, what the play cursor moved over
//! refilled from the file.
//!
//! In:  esi = the player (`AudioStream`)
//! Out: carry = failed (the player stopped); every register preserved
//!
//! See `audio/stream.rs`: no buffer: fail; `GetCurrentPosition` (+0x10) into the play
//! cursor; the bytes moved (round the buffer); none: done; more than `step`: `step`, and the
//! write place put that far ahead of the play cursor; `Lock` (+0x2C) at the write place into
//! `g_lockedParts`, the write place moved on, `audio::fill_stream_part` for both parts,
//! `Unlock` (+0x4C). A failing call stops the player (`audio::stop_stream`). Refuses A3D
//! (0x40D0C0). Callers: the two commands. Shadow mode runs the original alone. Self-test:
//! the original on players of the test's (the test DirectSound's buffer, the test's file),
//! 234 cases.

use crate::audio::buffer_play::{Leaves, dispatch_leaf};
use crate::audio::fill_buffer::LOCKED_PARTS_RVA;
use crate::audio::stream::*;
use crate::audio::test_sound::{Results, TestSound};
use crate::errors::Refused;
use crate::files::stream::LastPath;
use crate::framework::{RawHook, Regs, SelfTest, game_addr, hook_asm, rva};
use crate::service::harness::{capture, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("audio::feed_stream", rva(0x40C349), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

unsafe fn rust(r: &Regs) -> Result<Leaves, Refused> {
    let fed = unsafe { feed_stream(stream(r.esi), game_addr(LOCKED_PARTS_RVA) as u32) }?;
    Ok(Leaves {
        eax: r.eax,
        carry: Some(!fed),
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
    if !make_test_file(false) {
        return t.skip("the test's file cannot be written");
    }
    let parts = [(game_addr(LOCKED_PARTS_RVA) as u32, 16)];
    let saved = capture(&parts);
    let last = LastPath::save();
    let mut sound = TestSound::new();
    let failing = |n: u32| -> Results {
        let mut r = Results::default();
        match n {
            1 => r.position = 0x8878_000Au32 as i32,
            2 => r.lock = 0x8878_0096u32 as i32,
            3 => r.unlock = 0x8878_0096u32 as i32,
            _ => {}
        }
        r
    };
    for with_buffer in [true, false] {
        for old in [0u32, 0x3C0] {
            for moved in [0u32, 0x80, 0x300] {
                for split in [(0x80u32, 0u32), (0x40, 0x40), (0x100, 0x100)] {
                    for offset in [0, DATA_LEN - 0x60, DATA_LEN] {
                        for fail in 0..4 {
                            if !with_buffer && (moved != 0x80 || fail != 0) {
                                continue;
                            }
                            let results = Results {
                                cursor: (old + moved) % 0x400,
                                parts: split,
                                ..failing(fail)
                            };
                            let run = |original: bool, sound: &mut TestSound| {
                                let h = open_test_file(offset).unwrap_or(0);
                                let mut s = AudioStream {
                                    playing: 1,
                                    volume: 0x20,
                                    buffer: if with_buffer { sound.buffer(1) } else { 0 },
                                    open: 1,
                                    file: h,
                                    play_cursor: old,
                                    write_at: (old + 0x100) % 0x400,
                                    format: 0x220,
                                    bytes: 0x400,
                                    step: 0x100,
                                    tail: 0x40,
                                };
                                let before = Regs {
                                    esi: &mut s as *mut AudioStream as u32,
                                    ..Regs::SENTINEL
                                };
                                sound.memory.fill(0xAA);
                                restore(&parts, &saved);
                                sound.begin(results);
                                let mut r = before;
                                let carry = if original {
                                    unsafe { call_original(&mut r) };
                                    Ok(r.carry())
                                } else {
                                    unsafe { rust(&r) }.map(|l| l.carry == Some(true))
                                };
                                let seen = (
                                    carry.ok(),
                                    r.preserved_except(&before, 0),
                                    without_file(&s),
                                    sound.noted(),
                                    sound.memory.to_vec(),
                                    capture(&parts),
                                    position(h),
                                    LastPath::now(),
                                );
                                if position(h).is_some() {
                                    unsafe { crate::files::handle::close(h) };
                                }
                                seen
                            };
                            let theirs = run(true, &mut sound);
                            let mine = run(false, &mut sound);
                            t.check(theirs.1 && theirs == mine, || {
                                format!(
                                    "buffer {with_buffer}, cursor {old:#x} + {moved:#x}, parts {split:x?}, file at {offset:#x}, failing {fail}: original {:x?} {:x?} {:x?} {:x?}; rust {:x?} {:x?} {:x?} {:x?}; preserved {}, same memory {}, parts {:x?} / {:x?}, last path {:x?} / {:x?}",
                                    theirs.0, theirs.2, theirs.3, theirs.6,
                                    mine.0, mine.2, mine.3, mine.6,
                                    theirs.1, theirs.4 == mine.4, theirs.5, mine.5, theirs.7, mine.7
                                )
                            });
                        }
                    }
                }
            }
        }
    }
    restore(&parts, &saved);
    last.restore();
    remove_test_file();
    t.finish()
}
