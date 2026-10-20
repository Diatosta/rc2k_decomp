//! `audio::music_command` (0x40C4DD): command 0xB, the music (`g_musicStream`).
//!
//! In:  eax = the tune, ebx = the tune after it, ecx = the volume, edx = the action
//! Out: every register preserved
//!
//! See `audio/stream_commands.rs`. Two fixes (section 16, 2026-10-20): after seven plays
//! in a row that stop at their first feed the music stops (the original plays again for
//! ever), and a `g_tuneTurn` past 5 starts tune 0xA's search from 0 (the original's search
//! never ends without a file); the audio thread hung in either. Callers:
//! `audio::take_commands` (command 0xB), the thread's turn (action 1),
//! `audio::stop_streams`. Shadow mode runs the original alone. Self-test: a redirected copy
//! whose player, next tune, `g_noMusic`, tune turn and path are the test's and whose calls
//! go to stand-ins (`stream_commands::handler`; `format_u32` is the real one), 900 cases.

use crate::audio::buffer_play::{Leaves, dispatch_leaf};
use crate::audio::stream::*;
use crate::audio::stream_command::stand_in_for;
use crate::audio::stream_commands::*;
use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, call_game, hook_asm, rva};
use crate::install::stand_ins::set_handler;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("audio::music_command", rva(0x40C4DD), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

unsafe fn rust(r: &Regs) -> Result<Leaves, Refused> {
    music_command(
        r.eax,
        r.ebx,
        r.ecx as i32,
        r.edx,
        &MusicVars::game(),
        &mut game_part,
    )?;
    Ok(Leaves {
        eax: r.eax,
        carry: None,
        zero: None,
    })
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_leaf(&HOOK, r, call_original, rust);
}

const PATH: &[u8; 0x18] = b";3files\\music\\00.wav\0\0\0\0";

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    // Used through their addresses only (the code under test writes them).
    let mut vars = Box::new([0u32; 4]);
    let mut path = Box::new([0u8; 0x18]);
    let mut player = Box::new(AudioStream::default());
    let at = vars.as_mut_ptr() as u32;
    let pa = path.as_mut_ptr() as u32;
    let pl = &mut *player as *mut AudioStream as u32;
    let v = MusicVars {
        player: pl,
        next_tune: at,
        no_music: at + 4,
        path: pa,
        turn: at + 8,
    };
    let copy = match crate::audio::test_copy::make(
        &HOOK,
        &crate::audio::test_copy::Copy {
            address: 0x40C4DD,
            len: 0x10B,
            entry: &[0x60, 0x8D, 0x35, 0x90, 0x69, 0x50, 0x00],
            calls: &[0x31, 0x40, 0xA0, 0xAB, 0xCC, 0xD7, 0xDE, 0xE8, 0xFA],
            jumps_out: &[],
        },
        &stand_in_for,
        &[
            (0x506990, pl),
            (0x50698C, v.next_tune),
            (0x5069BC, v.no_music),
            (0x5180F8, v.turn),
            (0x5069C0, pa),
            (0x5069CE, pa + 0xE),
        ],
        15,
    ) {
        Ok(copy) => copy,
        Err(why) => return t.skip(&why),
    };
    set_handler(handler);
    // (g_noMusic, the tune turn, the tunes whose file is there, start fails, feeds stopping)
    let all = 0xFFFF_FFFF_FFFF_FFFFu64;
    let scenes = [
        (0u32, 0u32, all, false, 0u32),
        (0, 5, 1 << 12, false, 1),
        (0, 2, 1 << 3 | 1 << 8, false, 0),
        (0, 0, all, true, 0),
        (1, 0, all, false, 0),
        (0, 3, all, false, 2),
    ];
    for action in [0u32, 1, 2, 3, 5] {
        for was_playing in [0, 1] {
            for volume in [0x20i32, 0, -1] {
                for (tune, next) in [(0u32, 0u32), (3, 0), (3, 10), (10, 8), (0x105, 0)] {
                    for &(no_music, turn, present, start_fails, feed_stops) in &scenes {
                        let run = |rust: bool| {
                            unsafe {
                                (at as *mut [u32; 4]).write_volatile([0x77, no_music, turn, 0]);
                                (pa as *mut [u8; 0x18]).write_volatile(*PATH);
                                (pl as *mut AudioStream).write_volatile(AudioStream {
                                    playing: was_playing,
                                    volume: 0x11,
                                    ..AudioStream::default()
                                });
                            }
                            begin(Script {
                                start_fails,
                                feed_stops,
                                present,
                                feeds: 0,
                            });
                            let before = Regs {
                                eax: tune,
                                ebx: next,
                                ecx: volume as u32,
                                edx: action,
                                ..Regs::SENTINEL
                            };
                            let mut ro = before;
                            let done = if rust {
                                music_command(tune, next, volume, action, &v, &mut |p, r| {
                                    handler(p, r);
                                    Ok(())
                                })
                                .is_ok()
                            } else {
                                unsafe { call_game(&mut ro, copy.addr()) };
                                ro.preserved_except(&before, 0)
                            };
                            unsafe {
                                (
                                    done,
                                    noted(),
                                    (at as *const [u32; 4]).read_volatile(),
                                    (pa as *const [u8; 0x18]).read_volatile(),
                                    (pl as *const AudioStream).read_volatile(),
                                )
                            }
                        };
                        let theirs = run(false);
                        let mine = run(true);
                        t.check(theirs.0 && theirs == mine, || {
                            format!(
                                "action {action}, playing {was_playing}, volume {volume}, tune {tune:#x} then {next:#x}, no music {no_music}, turn {turn}, files {present:#x}, start fails {start_fails}, feeds stopping {feed_stops}: original {:x?} {:x?} {:?} {:x?}; rust {:x?} {:x?} {:?} {:x?}",
                                theirs.1, theirs.2, crate::strings::utils::bytes_str(&theirs.3), theirs.4,
                                mine.1, mine.2, crate::strings::utils::bytes_str(&mine.3), mine.4
                            )
                        });
                    }
                }
            }
        }
    }
    // The fixes, the Rust version alone (the original never comes back): every tune
    // stopping at its first feed, and a turn past 5 with no file.
    for (turn, present, feed_stops) in [(0u32, all, 1000u32), (9, 0, 1), (9, all, 1000)] {
        unsafe {
            (at as *mut [u32; 4]).write_volatile([0x77, 0, turn, 0]);
            (pa as *mut [u8; 0x18]).write_volatile(*PATH);
            (pl as *mut AudioStream).write_volatile(AudioStream::default());
        }
        begin(Script {
            start_fails: false,
            feed_stops,
            present,
            feeds: 0,
        });
        let done = music_command(8, 0xA, 0x20, 2, &v, &mut |p, r| {
            handler(p, r);
            Ok(())
        });
        let (vars, playing) = unsafe {
            (
                (at as *const [u32; 4]).read_volatile(),
                (pl as *const AudioStream).read_volatile().playing,
            )
        };
        let starts = noted().iter().filter(|n| n[0] == START).count();
        t.check(
            done.is_ok() && vars[0] == 0 && playing == 0 && vars[2] < 6 && starts <= 8,
            || format!("turn {turn}, files {present:#x}, feeds stopping {feed_stops}: {done:?}, vars {vars:x?}, playing {playing}, {starts} starts"),
        );
    }
    drop((vars, path, player));
    t.finish()
}
