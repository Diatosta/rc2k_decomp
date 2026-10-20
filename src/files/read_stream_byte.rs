//! `files::read_stream_byte` (0x402D16): the next byte of a buffered stream.
//!
//! In:  esi = the stream; eax as it comes
//! Out: al = the byte, carry clear; or carry set with ah = 0 at the end of the file (eax =
//!      0) and ah = 1 after a read error; the zero flag is clear in every case; everything
//!      but eax preserved
//! While the buffer has bytes left it hands out the next one and only al changes. Otherwise
//! it reads up to 0x2000 bytes into the buffer (`files::read`), and eax becomes the number
//! read: with at least one, the first is handed out in al; with none, that is the end of the
//! file. After a failed read the stream is left as it was.
//! Seven call sites: `read_stream_block` and the picture readers, which call it per byte.
//! Shadow mode runs the original alone (the file's position moves) and checks its result
//! against the stream as it was, or as the refill left it.
//! The name is a reading.

use crate::errors::BadInput;
use crate::files::handle::{close, open_by_mode};
use crate::files::stream::{
    BUFFER_LEN, LastPath, Stream, TEST_LEN, TEST_PATH, make_test_file, remove_test_file,
};
use crate::framework::{EAX, RawHook, Regs, SelfTest, hook_asm, rva, shadow};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("files::read_stream_byte", rva(0x402D16), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- Rust version ----

pub use rc2k_core::files::io::ByteRead;

/// `eax` is that register at entry.
pub unsafe fn read_stream_byte(stream: *mut Stream, eax: u32) -> Result<ByteRead, BadInput> {
    // Converted: `rc2k_core::files::io::read_stream_byte`.
    unsafe {
        Ok(rc2k_core::files::io::read_stream_byte(
            &mut crate::files::handle::GameFiles::new(),
            crate::files::stream::core_stream(stream),
            eax,
        )?)
    }
}

// ---- hook ----

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    let stream = r.esi as *mut Stream;
    if !shadow() {
        let left = match unsafe { read_stream_byte(stream, r.eax) } {
            Ok(left) => left,
            Err(e) => return unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) },
        };
        r.eax = left.eax;
        r.set_carry(left.carry);
        r.set_zero(false);
        return;
    }
    let (position, filled) = unsafe { ((*stream).position, (*stream).filled) };
    let waiting = (position < filled && (position as usize) < BUFFER_LEN)
        .then(|| unsafe { (*stream).buffer[position as usize] });
    let mut o = *r;
    unsafe { call_original(&mut o) };
    let (position_o, filled_o) = unsafe { ((*stream).position, (*stream).filled) };
    let as_rust = match waiting {
        // From the buffer: only al changes.
        Some(byte) => {
            !o.carry()
                && o.eax == r.eax & !0xFF | byte as u32
                && position_o == position.wrapping_add(1)
                && filled_o == filled
        }
        // After a refill: the first byte of the new buffer, or the end, or an error.
        None if position >= filled => {
            if o.carry() {
                (o.eax == 0 && filled_o == 0 && position_o == 0)
                    || (o.eax & 0xFF00 == 0x100 && position_o == position && filled_o == filled)
            } else {
                filled_o >= 1
                    && position_o == 1
                    && o.eax == filled_o & !0xFF | unsafe { (*stream).buffer[0] } as u32
            }
        }
        // A position past the buffer's length: not predicted.
        None => true,
    };
    if !as_rust || !o.preserved_except(r, EAX) {
        HOOK.mismatch(|| {
            format!(
                "position={position} filled={filled} eax={:#x}: the original left eax={:#x} carry={} position={position_o} filled={filled_o}",
                r.eax,
                o.eax,
                o.carry()
            )
        });
    }
    r.eax = o.eax;
    r.eflags = o.eflags;
}

// ---- self-test: the test file read to its end a byte at a time through both, each with a
// stream and a handle of its own; then a stream whose handle is 0 ----

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    if !make_test_file() {
        return t.skip("cannot create a test file in the game's var folder");
    }
    let saved = LastPath::save();
    let mut theirs = Stream::filler();
    let mut ours = Stream::filler();
    let files = unsafe {
        (
            open_by_mode(TEST_PATH.as_ptr(), 0),
            open_by_mode(TEST_PATH.as_ptr(), 0),
        )
    };
    if let (Ok(Some(file_o)), Ok(Some(file_r))) = files {
        for (stream, file) in [(&mut theirs, file_o), (&mut ours, file_r)] {
            stream.mode = 0;
            stream.file = file;
            stream.position = 0;
            stream.filled = 0;
        }
        // Three calls past the end as well.
        for n in 0..TEST_LEN + 3 {
            let eax = 0x1234_5600 ^ (n as u32) << 12;
            let before = Regs {
                eax,
                esi: &mut *theirs as *mut Stream as u32,
                ..Regs::SENTINEL
            };
            let mut ro = before;
            unsafe { call_original(&mut ro) };
            let mine = unsafe { read_stream_byte(&mut *ours, eax) };
            let zero_o = ro.eflags & 0x40 != 0;
            let same_stream = (theirs.position, theirs.filled) == (ours.position, ours.filled)
                && theirs.buffer == ours.buffer;
            let preserved = ro.preserved_except(&before, EAX);
            let ok =
                mine == Ok(ByteRead {
                    eax: ro.eax,
                    carry: ro.carry(),
                }) && !zero_o
                    && same_stream
                    && preserved;
            t.check(ok, || {
                format!(
                    "byte {n} of {TEST_LEN}: original eax={:#x} carry={} zero={zero_o} position={} filled={} preserved={preserved}; rust {mine:x?} position={} filled={} same buffer={}",
                    ro.eax,
                    ro.carry(),
                    theirs.position,
                    theirs.filled,
                    ours.position,
                    ours.filled,
                    theirs.buffer == ours.buffer
                )
            });
        }
        unsafe {
            close(file_o);
            close(file_r);
        }
    } else {
        saved.restore();
        remove_test_file();
        return t.skip("cannot open the test file");
    }

    // A read that fails.
    for position in [0u32, 5] {
        let mut theirs = Stream::filler();
        let mut ours = Stream::filler();
        for stream in [&mut theirs, &mut ours] {
            stream.mode = 0;
            stream.file = 0;
            stream.position = position;
            stream.filled = position;
        }
        let before = Regs {
            esi: &mut *theirs as *mut Stream as u32,
            ..Regs::SENTINEL
        };
        let mut ro = before;
        unsafe { call_original(&mut ro) };
        let mine = unsafe { read_stream_byte(&mut *ours, before.eax) };
        let same_stream = (theirs.position, theirs.filled) == (ours.position, ours.filled)
            && theirs.buffer == ours.buffer;
        let ok =
            mine == Ok(ByteRead {
                eax: ro.eax,
                carry: ro.carry(),
            }) && mine.is_ok_and(|m| m.failed())
                && same_stream
                && ro.preserved_except(&before, EAX);
        t.check(ok, || {
            format!(
                "a handle of 0, position {position}: original eax={:#x} carry={} position={} filled={}; rust {mine:x?} position={} filled={}",
                ro.eax,
                ro.carry(),
                theirs.position,
                theirs.filled,
                ours.position,
                ours.filled
            )
        });
    }
    saved.restore();
    remove_test_file();
    t.finish()
}
