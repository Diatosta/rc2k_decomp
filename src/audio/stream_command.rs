//! `audio::stream_command` (0x40C194): command 9, the sound stream (`g_soundStream`).
//!
//! In:  eax = the file's path, ecx = the volume, edx = the action
//! Out: every register preserved
//!
//! See `audio/stream_commands.rs`. Callers: `audio::take_commands` (command 9), the
//! thread's turn (action 1), `audio::stop_streams`. Shadow mode runs the original alone.
//! Self-test: a redirected copy whose player is the test's and whose four calls go to
//! stand-ins (`stream_commands::handler`), 180 cases.

use crate::audio::buffer_play::{Leaves, dispatch_leaf};
use crate::audio::stream::*;
use crate::audio::stream_commands::*;
use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, call_game, game_addr, hook_asm, rva};
use crate::install::stand_ins::{set_handler, stand_in};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("audio::stream_command", rva(0x40C194), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

unsafe fn rust(r: &Regs) -> Result<Leaves, Refused> {
    let player = game_addr(SOUND_STREAM_RVA) as u32;
    stream_command(r.eax, r.ecx as i32, r.edx, player, &mut game_part)?;
    Ok(Leaves {
        eax: r.eax,
        carry: None,
        zero: None,
    })
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_leaf(&HOOK, r, call_original, rust);
}

/// The leaves' places by their addresses (and `files::file_exists`).
pub fn stand_in_for(callee: usize) -> Option<usize> {
    [0x40C1EA, 0x40C349, 0x40C468, 0x40C4B2, 0x402EBE]
        .iter()
        .position(|&t| rva(t) == callee)
        .map(|n| stand_in(n as u32))
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let player = Box::new(AudioStream::default());
    let pl = &*player as *const AudioStream as u32;
    let copy = match crate::audio::test_copy::make(
        &HOOK,
        &crate::audio::test_copy::Copy {
            address: 0x40C194,
            len: 0x56,
            entry: &[0x60, 0x8D, 0x35, 0x60, 0x69, 0x50, 0x00],
            calls: &[0x22, 0x2E, 0x3E, 0x45, 0x4F],
            jumps_out: &[],
        },
        &stand_in_for,
        &[(0x506960, pl)],
        1,
    ) {
        Ok(copy) => copy,
        Err(why) => return t.skip(&why),
    };
    set_handler(handler);
    let dummy_path = b"music\0";
    for action in [0u32, 1, 2, 3, 5] {
        for was_playing in [0, 1] {
            for volume in [0x20i32, 0, -1] {
                for path in [0, dummy_path.as_ptr() as u32] {
                    for (start_fails, feed_stops) in [(false, 0), (true, 0), (false, 1)] {
                        let run = |rust: bool| {
                            unsafe {
                                (pl as *mut AudioStream).write_volatile(AudioStream {
                                    playing: was_playing,
                                    volume: 0x11,
                                    ..AudioStream::default()
                                })
                            };
                            begin(Script {
                                start_fails,
                                feed_stops,
                                ..Script::default()
                            });
                            let before = Regs {
                                eax: path,
                                ecx: volume as u32,
                                edx: action,
                                ..Regs::SENTINEL
                            };
                            let mut ro = before;
                            let done = if rust {
                                stream_command(path, volume, action, pl, &mut |p, r| {
                                    handler(p, r);
                                    Ok(())
                                })
                                .is_ok()
                            } else {
                                unsafe { call_game(&mut ro, copy.addr()) };
                                ro.preserved_except(&before, 0)
                            };
                            (done, noted(), unsafe {
                                (pl as *const AudioStream).read_volatile()
                            })
                        };
                        let theirs = run(false);
                        let mine = run(true);
                        t.check(theirs.0 && theirs == mine, || {
                            format!(
                                "action {action}, playing {was_playing}, volume {volume}, path {path:#x}, start fails {start_fails}, feeds stopping {feed_stops}: original {theirs:x?}; rust {mine:x?}"
                            )
                        });
                    }
                }
            }
        }
    }
    drop(player);
    t.finish()
}
