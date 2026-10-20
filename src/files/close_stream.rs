//! `files::close_stream` (0x402CFA): closes a buffered stream.
//!
//! In:  esi = the stream
//! Out: carry = the handle could not be closed (zero flag = it could); every register
//!      preserved
//! A stream opened with mode 1 has what is in its buffer written out first (`sub_402D8E`:
//! `filled` bytes, nothing if that is 0; whether the write worked is not looked at). Then
//! the handle is closed and -1 stored in its place. Five callers.
//! Shadow mode runs the original alone. Manual patch with a private copy (0x1C bytes, both
//! calls relocated): a conditional jump sits in the first bytes. The name is a reading.

use crate::errors::BadInput;
use crate::files::handle::{NO_HANDLE, open_by_mode};
use crate::files::stream::{LastPath, Stream, TEST_PATH, make_test_file, remove_test_file};
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva, shadow};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new("files::close_stream", rva(0x402CFA), thunk, &ORIGINAL)
    .private_copy(0x1C)
    .relocate_calls(&[0x06, 0x0E])
    .manual_patch()
    .self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- Rust version ----

/// Returns whether the handle was closed.
pub unsafe fn close_stream(stream: *mut Stream) -> Result<bool, BadInput> {
    // Converted: `rc2k_core::files::io::close_stream`.
    unsafe {
        Ok(rc2k_core::files::io::close_stream(
            &mut crate::files::handle::GameFiles::new(),
            crate::files::stream::core_stream(stream),
            NO_HANDLE,
        )?)
    }
}

// ---- hook ----

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    let stream = r.esi as *mut Stream;
    if !shadow() {
        let closed = match unsafe { close_stream(stream) } {
            Ok(closed) => closed,
            Err(e) => return unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) },
        };
        r.set_carry(!closed);
        r.set_zero(closed);
        return;
    }
    let mut o = *r;
    unsafe { call_original(&mut o) };
    let file = unsafe { (*stream).file };
    if file != NO_HANDLE || !o.preserved_except(r, 0) {
        HOOK.mismatch(|| format!("the original left file={file:#x}"));
    }
    r.eflags = o.eflags;
}

// ---- self-test: streams read from and written to, and one whose handle is 0 ----

const WRITTEN_FILE: &str = "var\\rc2k_hook_stream_written.bin";
const WRITTEN_PATH: &[u8] = b"var\\rc2k_hook_stream_written.bin\0";

/// What one side leaves behind.
#[derive(PartialEq)]
struct Closed {
    closed: bool,
    zero: bool,
    preserved: bool,
    file: u32,
    position: u32,
    filled: u32,
    last_path: u32,
    /// The file a stream of mode 1 was written to, read back.
    written: Option<Vec<u8>>,
}

impl std::fmt::Debug for Closed {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "closed={} zero={} preserved={} file={:#x} position={} filled={} last={:#x} written={:?}",
            self.closed,
            self.zero,
            self.preserved,
            self.file,
            self.position,
            self.filled,
            self.last_path,
            self.written.as_ref().map(Vec::len)
        )
    }
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    if !make_test_file() {
        return t.skip("cannot create a test file in the game's var folder");
    }
    let saved = LastPath::save();
    // (path, mode, bytes waiting in the buffer, a real handle)
    for (path, mode, filled, real) in [
        (TEST_PATH, 0u32, 0u32, true),
        (TEST_PATH, 0, 40, true),
        (WRITTEN_PATH, 1, 0, true),
        (WRITTEN_PATH, 1, 300, true),
        (TEST_PATH, 0, 0, false),
    ] {
        // Each side gets a stream of its own; what it leaves in the file is read back.
        let run = |original: bool| {
            let mut stream = Stream::filler();
            stream.mode = mode;
            stream.position = filled;
            stream.filled = filled;
            for (n, byte) in stream.buffer.iter_mut().enumerate() {
                *byte = (n * 13 + 5) as u8;
            }
            stream.file = if real {
                match unsafe { open_by_mode(path.as_ptr(), mode) } {
                    Ok(Some(file)) => file,
                    _ => return None,
                }
            } else {
                0
            };
            let before = Regs {
                esi: &mut *stream as *mut Stream as u32,
                ..Regs::SENTINEL
            };
            let (closed, zero, preserved) = if original {
                let mut ro = before;
                unsafe { call_original(&mut ro) };
                (
                    !ro.carry(),
                    ro.eflags & 0x40 != 0,
                    ro.preserved_except(&before, 0),
                )
            } else {
                let closed = unsafe { close_stream(&mut *stream) } == Ok(true);
                (closed, closed, true)
            };
            let written = (mode == 1).then(|| std::fs::read(WRITTEN_FILE).unwrap_or_default());
            Some(Closed {
                closed,
                zero,
                preserved,
                file: stream.file,
                position: stream.position,
                filled: stream.filled,
                last_path: LastPath::now(),
                written,
            })
        };
        let (theirs, mine) = (run(true), run(false));
        let ok = theirs.is_some() && theirs == mine && theirs.as_ref().is_some_and(|c| c.preserved);
        t.check(ok, || {
            format!("mode={mode} filled={filled} real={real}: original {theirs:x?}; rust {mine:x?}")
        });
    }
    saved.restore();
    remove_test_file();
    let _ = std::fs::remove_file(WRITTEN_FILE);
    t.finish()
}
