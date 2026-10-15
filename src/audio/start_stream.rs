//! `audio::start_stream` (0x40C1EA): a stream player started on a WAV file.
//!
//! In:  eax = the file's path, esi = the player (`AudioStream`, its volume set)
//! Out: carry = failed (the player stopped); every register preserved
//!
//! See `audio/stream.rs`. No `g_directSound`: fail. `g_streamHeader` (0xC00 bytes on
//! `g_streamHeaderMemory`) is taken the first time; zeroed; `audio::read_sample` into it
//! (the original swaps it in for `g_sampleBuffer` and its size, and back); `open_by_mode`
//! mode 0 (file, open 1); `seek` to the data; `audio::describe_wave_format`;
//! `g_bufferDescription` size 0x14, flags 0x80 (volume), the player's bytes,
//! `g_waveFormat`; `CreateSoundBuffer` (+0x0C) into the player; `SetVolume`; write place
//! `step`, play cursor 0, tail `step`, the format; `Play` looping; playing 1. Any failure
//! stops the player (`audio::stop_stream`). Callers: the two commands.
//! Refuses A3D (0x40CF6B) and a volume past the table, before anything is done; and what
//! `allocate`, `read_sample` and `open_by_mode` refuse.
//! Shadow mode runs the original alone. Self-test: a redirected copy whose DirectSound,
//! header and its list are the test's, on the test's WAV files, 36 cases.

use crate::audio::buffer_play::{Leaves, dispatch_leaf};
use crate::audio::stream::*;
use crate::audio::test_sound::{Results, TestSound};
use crate::errors::Refused;
use crate::files::stream::LastPath;
use crate::framework::{RawHook, Regs, SelfTest, call_game, game_addr, hook_asm, rva};
use crate::service::harness::{capture, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("audio::start_stream", rva(0x40C1EA), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

unsafe fn rust(r: &Regs) -> Result<Leaves, Refused> {
    let started = unsafe { start_stream(stream(r.esi), r.eax as *const u8, &StartVars::game()) }?;
    Ok(Leaves {
        eax: r.eax,
        carry: Some(!started),
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
    // [0] the DirectSound, [1] the header, [2..6] its list.
    let mut vars = Box::new([0u32; 6]);
    // Used through `at` only from here on.
    let at = vars.as_mut_ptr() as u32;
    let v = StartVars {
        direct_sound: at,
        header: at + 4,
        header_memory: at + 8,
    };
    let copy = match crate::audio::test_copy::make(
        &HOOK,
        &crate::audio::test_copy::Copy {
            address: 0x40C1EA,
            len: 0x15F,
            entry: &[0x83, 0x3D, 0x60, 0x6D, 0x50, 0x00, 0x01],
            calls: &[0x33, 0x5F, 0x64, 0x80, 0xA6, 0xB6, 0xC8, 0x157],
            jumps_out: &[0x07],
        },
        &|_| None,
        &[
            (0x506A88, v.direct_sound),
            (0x5069D8, v.header),
            (0x5069DC, v.header_memory),
        ],
        6,
    ) {
        Ok(copy) => copy,
        Err(why) => return t.skip(&why),
    };
    // What it writes of the game's: g_sampleBuffer and its size, g_sampleFile,
    // g_bufferDescription, g_waveFormat, g_lastFilePath, the memory figures.
    let a = |r| game_addr(rva(r)) as u32;
    let globals = [
        (a(0x506930), 0x28),
        (a(0x506B1C), 0x14),
        (a(0x506B30), 0x12),
        (
            game_addr(crate::files::read_file::LAST_FILE_PATH_RVA) as u32,
            4,
        ),
        (game_addr(crate::memory::MOST_HELD_RVA) as u32, 12),
    ];
    let saved = capture(&globals);
    let last = LastPath::save();
    let sound = TestSound::new();
    let mut own_header = vec![0u8; HEADER_LEN as usize];
    for (name, file) in [
        ("16 stereo", Some(false)),
        ("8 mono", Some(true)),
        ("missing", None),
    ] {
        if let Some(eight) = file
            && !make_test_file(eight)
        {
            return t.skip("the test's file cannot be written");
        }
        let path = if file.is_some() {
            TEST_PATH
        } else {
            MISSING_PATH
        };
        for direct_sound in [true, false] {
            for preset in [true, false] {
                for fail in 0..3 {
                    let results = Results {
                        create: if fail == 1 { 0x8878_000Au32 as i32 } else { 0 },
                        play: if fail == 2 { 0x8878_0096u32 as i32 } else { 0 },
                        ..Results::default()
                    };
                    let run = |original: bool, own: &mut Vec<u8>| {
                        own.fill(0xAA);
                        // Through the address only: the copy and the Rust version write
                        // there behind any reference's back.
                        unsafe {
                            (at as *mut [u32; 6]).write_volatile([
                                if direct_sound { sound.sound() } else { 0 },
                                if preset { own.as_ptr() as u32 } else { 0 },
                                0,
                                0,
                                0,
                                0,
                            ])
                        };
                        restore(&globals, &saved);
                        let mut s = AudioStream {
                            volume: if fail == 0 { 0x40 } else { 0x18 },
                            file: 0x5555,
                            bytes: 0x400,
                            step: 0x100,
                            ..AudioStream::default()
                        };
                        let before = Regs {
                            eax: path.as_ptr() as u32,
                            esi: &mut s as *mut AudioStream as u32,
                            ..Regs::SENTINEL
                        };
                        sound.begin(results);
                        let mut r = before;
                        let carry = if original {
                            unsafe { call_game(&mut r, copy.addr()) };
                            Ok(r.carry())
                        } else {
                            unsafe { rust_on(&r, &v) }
                        };
                        let header = unsafe { (v.header as *const u32).read_volatile() };
                        let total =
                            unsafe { ((v.header_memory + 4) as *const u32).read_volatile() };
                        // g_sampleFile.data points into the header: as an offset, since
                        // the two runs' headers may be taken at different places.
                        let mut g = capture(&globals);
                        let data = u32::from_le_bytes(g[8..12].try_into().unwrap());
                        g[8..12].copy_from_slice(&data.wrapping_sub(header).to_le_bytes());
                        let seen = (
                            carry.ok(),
                            r.preserved_except(&before, 0),
                            without_file(&s),
                            sound.noted(),
                            if preset { own.clone() } else { Vec::new() },
                            (header != 0, total),
                            g,
                            if s.open == 1 { position(s.file) } else { None },
                        );
                        if s.open == 1 {
                            unsafe { crate::files::handle::close(s.file) };
                        }
                        if !preset && header != 0 {
                            let _ = unsafe {
                                crate::memory::free_list::free_list(
                                    v.header_memory as *mut crate::memory::MemoryList,
                                )
                            };
                        }
                        seen
                    };
                    let theirs = run(true, &mut own_header);
                    let mine = run(false, &mut own_header);
                    t.check(theirs.1 && theirs == mine, || {
                        let differ = [
                            theirs.0 != mine.0,
                            !theirs.1 || !mine.1,
                            theirs.2 != mine.2,
                            theirs.3 != mine.3,
                            theirs.4 != mine.4,
                            theirs.5 != mine.5,
                            theirs.6 != mine.6,
                            theirs.7 != mine.7,
                        ];
                        let words = |b: &[u8]| -> Vec<u32> {
                            b.chunks(4).map(|c| c.iter().rev().fold(0, |a, &x| a << 8 | x as u32)).collect()
                        };
                        format!(
                            "file {name}, DirectSound {direct_sound}, header there {preset}, failing {fail}: fields differing {differ:?} (carry, registers, player, calls, header, header var and list total, globals, file place); original {:x?} {:x?} {:x?} {:x?} {:x?}; rust {:x?} {:x?} {:x?} {:x?} {:x?}",
                            theirs.0, theirs.2, theirs.5, words(&theirs.6), theirs.7,
                            mine.0, mine.2, mine.5, words(&mine.6), mine.7
                        )
                    });
                }
            }
        }
    }
    restore(&globals, &saved);
    last.restore();
    remove_test_file();
    t.finish()
}

/// The Rust version on the test's variables: the carry.
unsafe fn rust_on(r: &Regs, v: &StartVars) -> Result<bool, Refused> {
    unsafe { start_stream(stream(r.esi), r.eax as *const u8, v) }.map(|started| !started)
}
