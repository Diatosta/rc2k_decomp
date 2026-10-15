//! `audio::fill_stream_part` (0x40C42C): one locked part of a stream's buffer filled from
//! its file.
//!
//! In:  ebx = the part, ecx = its bytes, esi = the player (`AudioStream`)
//! Out: nothing for no part or no bytes; else eax = the bytes read when all were, or
//!      eax = the silence's dword (0 or 0x80808080), ecx = 0, edi = the part's end
//!
//! See `audio/stream.rs`. One caller, `audio::feed_stream` (twice, inside its `pusha`).
//! Compared in shadow mode: the Rust version first, then the part, the player and the
//! file's place put back and the original run. Self-test: the original on players of the
//! test's reading a WAV of the test's in `var`, 96 cases.

use crate::audio::stream::*;
use crate::framework::{EAX, ECX, EDI, RawHook, Regs, SelfTest, hook_asm, rva, shadow};
use crate::service::harness::{capture, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("audio::fill_stream_part", rva(0x40C42C), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

fn rust(r: &mut Regs) {
    let s = unsafe { stream(r.esi) };
    let (at, bytes) = (r.ebx, r.ecx);
    let f = unsafe { fill_stream_part(s, at, bytes) };
    fill_left(r, at, bytes, s, f);
}

/// The part, the player and the file's place.
type Seen = (Vec<u8>, AudioStream, Option<u32>);

fn seen(r: &Regs, region: &[(u32, usize)]) -> Seen {
    let s = unsafe { *stream(r.esi) };
    (capture(region), s, position(s.file))
}

fn region(r: &Regs) -> Vec<(u32, usize)> {
    if r.ebx != 0 && r.ecx != 0 {
        vec![(r.ebx, r.ecx as usize)]
    } else {
        Vec::new()
    }
}

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if !shadow() {
        rust(r);
        return;
    }
    let region = region(r);
    let before = seen(r, &region);
    let mut mine = *r;
    rust(&mut mine);
    let after_mine = seen(r, &region);
    restore(&region, &before.0);
    unsafe { *stream(r.esi) = before.1 };
    if let Some(p) = before.2 {
        unsafe { crate::files::handle::seek(before.1.file, p as i32, 0) };
    }
    let inputs = (r.ebx, r.ecx);
    unsafe { call_original(r) };
    let theirs = seen(r, &region);
    let same_regs = (mine.eax, mine.ecx, mine.edi) == (r.eax, r.ecx, r.edi);
    if theirs != after_mine || !same_regs {
        HOOK.mismatch(|| {
            format!(
                "part {:#x}, bytes {:#x}: original {:x?} eax {:#x} ecx {:#x} edi {:#x}; rust {:x?} eax {:#x} ecx {:#x} edi {:#x}",
                inputs.0, inputs.1, (theirs.1, theirs.2), r.eax, r.ecx, r.edi,
                (after_mine.1, after_mine.2), mine.eax, mine.ecx, mine.edi
            )
        });
    }
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let mut out = vec![0u8; 0x200];
    for eight in [false, true] {
        if !make_test_file(eight) {
            return t.skip("the test's file cannot be written");
        }
        let Some(h) = open_test_file(0) else {
            remove_test_file();
            return t.skip("the test's file cannot be opened");
        };
        for offset in [0, DATA_LEN - 0x40, DATA_LEN] {
            for (at, bytes) in [(0u32, 0x80u32), (0x10, 0), (0x10, 0x80), (0x20, 0x180)] {
                for tail in [0x100, 0x80, 0x20, -4] {
                    let base = out.as_mut_ptr() as u32;
                    let place = if at == 0 { 0 } else { base + at };
                    let mut s = AudioStream {
                        playing: 1,
                        file: h,
                        format: if eight { 0x110 } else { 0x220 },
                        tail,
                        ..AudioStream::default()
                    };
                    let before = Regs {
                        ebx: place,
                        ecx: bytes,
                        esi: &mut s as *mut AudioStream as u32,
                        ..Regs::SENTINEL
                    };
                    let run = |original: bool, s: &mut AudioStream, out: &mut Vec<u8>| {
                        *s = AudioStream {
                            playing: 1,
                            tail,
                            ..*s
                        };
                        out.fill(0xAA);
                        unsafe { crate::files::handle::seek(h, (DATA_AT + offset) as i32, 0) };
                        let mut r = before;
                        if original {
                            unsafe { call_original(&mut r) };
                        } else {
                            rust(&mut r);
                        }
                        (r, out.clone(), *s, position(h))
                    };
                    let (ro, theirs, ts, tp) = run(true, &mut s, &mut out);
                    let (rm, mine, ms, mp) = run(false, &mut s, &mut out);
                    let ok = theirs == mine
                        && ts == ms
                        && tp == mp
                        && (ro.eax, ro.ecx, ro.edi) == (rm.eax, rm.ecx, rm.edi)
                        && ro.preserved_except(&before, EAX | ECX | EDI);
                    t.check(ok, || {
                        format!(
                            "eight bits {eight}, file at {offset:#x}, part {at:#x} bytes {bytes:#x}, tail {tail}: original {ts:x?} at {tp:x?} eax {:#x} ecx {:#x} edi {:#x}; rust {ms:x?} at {mp:x?} eax {:#x} ecx {:#x} edi {:#x}; first byte differing {:x?}",
                            ro.eax, ro.ecx, ro.edi, rm.eax, rm.ecx, rm.edi,
                            (0..theirs.len()).find(|&i| theirs[i] != mine[i])
                        )
                    });
                }
            }
        }
        unsafe { crate::files::handle::close(h) };
    }
    remove_test_file();
    t.finish()
}
