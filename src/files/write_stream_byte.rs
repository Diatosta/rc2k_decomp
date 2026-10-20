//! `files::write_stream_byte` (0x402D53): a byte into a buffered stream opened to write
//! (the block `files/stream.rs` describes): a full buffer (position 0x2000 or more) first
//! written out (`files::write`, the bytes in it) and emptied; the byte at the position,
//! position and count one more.
//!
//! In:  esi the stream, al the byte
//! Out: carry set when the write failed (or was short; the byte not stored); else carry
//!      clear and ZF of `test eax, eax` (the whole of eax); every register kept
//!
//! Callers: `files::write_stream_block`, `gfx::write_screen_pcx` (six). Shadow mode runs
//! the original alone (it writes a file). Self-test: two files of the test's in `var`
//! (removed after), 0x4567 bytes through either a byte at a time; the flags, the streams
//! and the files compared; a stream with no file failing.

use crate::errors::BadInput;
use crate::files::handle::{close, open_by_mode};
use crate::files::stream::{BUFFER_LEN, LastPath, Stream, TEST_LEN, test_content};
use crate::framework::{CARRY_FLAG, RawHook, Regs, SelfTest, ZERO_FLAG, hook_asm, rva, shadow};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("files::write_stream_byte", rva(0x402D53), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// The Rust version: whether it worked (carry clear). `eax` the register (al the byte).
pub unsafe fn write_stream_byte(stream: *mut Stream, eax: u32) -> Result<bool, BadInput> {
    // Converted: `rc2k_core::files::io::write_stream_byte`.
    unsafe {
        Ok(rc2k_core::files::io::write_stream_byte(
            &mut crate::files::handle::GameFiles::new(),
            crate::files::stream::core_stream(stream),
            eax as u8,
        )?)
    }
}

/// The flags it leaves for `eax`: carry, or the zero flag of `test eax, eax`.
pub fn flags(ok: bool, eax: u32, r: &mut Regs) {
    r.set_carry(!ok);
    if ok {
        r.set_zero(eax == 0);
    }
}

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    let ok = match unsafe { write_stream_byte(r.esi as *mut Stream, r.eax) } {
        Ok(ok) => ok,
        Err(e) => return unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) },
    };
    flags(ok, r.eax, r);
}

// ---- self-test ----

const FILE_A: &[u8] = b"var\\rc2k_hook_write_a.bin\0";
const FILE_B: &[u8] = b"var\\rc2k_hook_write_b.bin\0";

fn name(path: &[u8]) -> &str {
    std::str::from_utf8(&path[..path.len() - 1]).unwrap_or("")
}

/// Two streams opened to write the test's two files; None if they cannot be made.
pub fn test_streams() -> Option<(Box<Stream>, Box<Stream>)> {
    let mut streams = (Stream::filler(), Stream::filler());
    for (stream, path) in [(&mut streams.0, FILE_A), (&mut streams.1, FILE_B)] {
        let file = unsafe { open_by_mode(path.as_ptr(), 1) }.ok()??;
        stream.mode = 1;
        stream.file = file;
        stream.position = 0;
        stream.filled = 0;
    }
    Some(streams)
}

/// The two streams flushed and closed, the files read and removed: whether they are the
/// same, and the first's length.
pub fn finish_streams(mut streams: (Box<Stream>, Box<Stream>)) -> (bool, usize) {
    for stream in [&mut streams.0, &mut streams.1] {
        unsafe {
            if stream.filled != 0 {
                crate::files::handle::write(stream.file, stream.buffer.as_ptr(), stream.filled);
            }
            close(stream.file);
        }
    }
    let (a, b) = (
        std::fs::read(name(FILE_A)).ok(),
        std::fs::read(name(FILE_B)).ok(),
    );
    let _ = std::fs::remove_file(name(FILE_A));
    let _ = std::fs::remove_file(name(FILE_B));
    (a.is_some() && a == b, a.map_or(0, |f| f.len()))
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
    for (n, &byte) in content.iter().enumerate() {
        let eax = if n % 7 == 0 {
            byte as u32
        } else {
            0x1234_5600 | byte as u32
        };
        let before = Regs {
            eax,
            esi: &mut *theirs as *mut Stream as u32,
            ..Regs::SENTINEL
        };
        let mut ro = before;
        unsafe { call_original(&mut ro) };
        let done = unsafe { write_stream_byte(&mut *ours, eax) };
        let ok = done == Ok(true);
        let mut mine = before;
        flags(ok, eax, &mut mine);
        let same = done.is_ok()
            && (theirs.position, theirs.filled) == (ours.position, ours.filled)
            && theirs.buffer == ours.buffer
            && (ro.eflags ^ mine.eflags) & (CARRY_FLAG | ZERO_FLAG) == 0
            && ro.preserved_except(&before, 0);
        t.check(same, || {
            format!(
                "byte {n} of {TEST_LEN}: original flags {:#x} position {} filled {}; rust ok {ok} flags {:#x} position {} filled {}",
                ro.eflags & 0x41,
                theirs.position,
                theirs.filled,
                mine.eflags & 0x41,
                ours.position,
                ours.filled
            )
        });
    }
    let (same, len) = finish_streams((theirs, ours));
    t.check(same, || format!("the files differ (the first {len} bytes)"));
    // A full buffer and no file: the write fails.
    let (mut theirs, mut ours) = (Stream::filler(), Stream::filler());
    for stream in [&mut theirs, &mut ours] {
        stream.mode = 1;
        stream.file = 0;
        stream.position = BUFFER_LEN as u32;
        stream.filled = BUFFER_LEN as u32;
    }
    let before = Regs {
        eax: 0x55,
        esi: &mut *theirs as *mut Stream as u32,
        ..Regs::SENTINEL
    };
    let mut ro = before;
    unsafe { call_original(&mut ro) };
    let done = unsafe { write_stream_byte(&mut *ours, 0x55) };
    let ok = done == Ok(true);
    t.check(
        ro.carry()
            && done == Ok(false)
            && (theirs.position, theirs.filled) == (ours.position, ours.filled),
        || format!("no file: original carry {}, rust ok {ok}", ro.carry()),
    );
    saved.restore();
    t.finish()
}
