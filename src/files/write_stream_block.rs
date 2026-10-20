//! `files::write_stream_block` (0x402DC9): ecx bytes from ebx into a buffered stream, one
//! by one (`files::write_stream_byte`), until one fails.
//!
//! In:  esi the stream, ebx the bytes, ecx their count
//! Out: eax the bytes written; carry set when one failed; else carry clear and ZF of
//!      `test eax, eax` with al the last byte (ZF set for a count of 0); ebx, ecx, edx kept
//!
//! Callers: `gfx::write_screen_pcx`, 0x403D27, 0x403D89, 0x403D9B. Shadow mode runs the
//! original alone (it writes a file). Self-test: as `write_stream_byte`'s, blocks of 0 ..
//! 0x2345 bytes; a stream with no file failing part way.

use crate::errors::BadInput;
use crate::files::stream::{BUFFER_LEN, LastPath, Stream, test_content};
use crate::files::write_stream_byte::{finish_streams, test_streams};
use crate::framework::{
    CARRY_FLAG, EAX, RawHook, Regs, SelfTest, ZERO_FLAG, hook_asm, rva, shadow,
};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("files::write_stream_block", rva(0x402DC9), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// The Rust version: (eax, carry, ZF) as it leaves them. `eax` the register at entry.
pub unsafe fn write_stream_block(
    stream: *mut Stream,
    bytes: *const u8,
    count: u32,
    eax: u32,
) -> Result<(u32, bool, bool), BadInput> {
    // Converted: `rc2k_core::files::io::write_stream_block`.
    unsafe {
        Ok(rc2k_core::files::io::write_stream_block(
            &mut crate::files::handle::GameFiles::new(),
            crate::files::stream::core_stream(stream),
            std::slice::from_raw_parts(bytes, count as usize),
            eax,
        )?)
    }
}

/// For Rust callers: whether all were written.
pub unsafe fn write_stream_all(stream: *mut Stream, bytes: &[u8]) -> Result<bool, BadInput> {
    Ok(!unsafe { write_stream_block(stream, bytes.as_ptr(), bytes.len() as u32, 0) }?.1)
}

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    let (eax, carry, zero) =
        match unsafe { write_stream_block(r.esi as *mut Stream, r.ebx as *const u8, r.ecx, r.eax) }
        {
            Ok(left) => left,
            Err(e) => return unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) },
        };
    r.eax = eax;
    r.set_carry(carry);
    if !carry {
        r.set_zero(zero);
    }
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let saved = LastPath::save();
    let Some((mut theirs, mut ours)) = test_streams() else {
        saved.restore();
        return t.skip("cannot make the test files in the game's var folder");
    };
    let content = test_content();
    let mut at = 0usize;
    for (n, len) in [0usize, 1, 0x10, 0x2345, 0x7, 0x1FF0, 0x40]
        .into_iter()
        .enumerate()
    {
        let len = len.min(content.len() - at);
        let eax = 0x1234_5600 | n as u32;
        let bytes = content[at..].as_ptr();
        let before = Regs {
            eax,
            ebx: bytes as u32,
            ecx: len as u32,
            esi: &mut *theirs as *mut Stream as u32,
            ..Regs::SENTINEL
        };
        let mut ro = before;
        unsafe { call_original(&mut ro) };
        let done = unsafe { write_stream_block(&mut *ours, bytes, len as u32, eax) };
        let mine = done.unwrap_or_default();
        let flags_ok = done.is_ok()
            && ro.carry() == mine.1
            && (mine.1 || (ro.eflags & ZERO_FLAG != 0) == mine.2);
        let same = (theirs.position, theirs.filled) == (ours.position, ours.filled)
            && theirs.buffer == ours.buffer
            && ro.eax == mine.0
            && flags_ok
            && ro.preserved_except(&before, EAX);
        t.check(same, || {
            format!(
                "block {n} of {len:#x} bytes: original eax {:#x} flags {:#x}; rust {mine:?}",
                ro.eax,
                ro.eflags & (CARRY_FLAG | ZERO_FLAG)
            )
        });
        at += len;
    }
    let (same, len) = finish_streams((theirs, ours));
    t.check(same, || format!("the files differ (the first {len} bytes)"));
    // No file, the buffer nearly full: fails part way.
    let (mut theirs, mut ours) = (Stream::filler(), Stream::filler());
    for stream in [&mut theirs, &mut ours] {
        stream.mode = 1;
        stream.file = 0;
        stream.position = BUFFER_LEN as u32 - 3;
        stream.filled = BUFFER_LEN as u32 - 3;
    }
    let before = Regs {
        ebx: content.as_ptr() as u32,
        ecx: 9,
        esi: &mut *theirs as *mut Stream as u32,
        ..Regs::SENTINEL
    };
    let mut ro = before;
    unsafe { call_original(&mut ro) };
    let done = unsafe { write_stream_block(&mut *ours, content.as_ptr(), 9, before.eax) };
    let mine = done.unwrap_or_default();
    t.check(
        done.is_ok() && ro.carry() && mine.1 && ro.eax == mine.0 && theirs.buffer == ours.buffer,
        || {
            format!(
                "no file: original eax {:#x} carry {}; rust {mine:?}",
                ro.eax,
                ro.carry()
            )
        },
    );
    saved.restore();
    t.finish()
}
