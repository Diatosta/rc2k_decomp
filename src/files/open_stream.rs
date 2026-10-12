//! `files::open_stream` (0x402CDD): opens a file as a buffered stream.
//!
//! In:  eax = path, ebx = file mode, esi = the stream
//! Out: carry = the file could not be opened; every register preserved
//! Stores the mode in the stream, opens the file with `open_by_mode` and, if that worked,
//! stores the handle and an empty buffer (position 0, 0 bytes). After a failure only the
//! mode is stored. Five callers, the picture readers.
//! Shadow mode runs the original alone (two opens would leave two handles) and checks what
//! it stored. Manual patch with a private copy (0x1D bytes, the call relocated): the call
//! starts at the fourth byte, inside what the patch overwrites.
//! Refuses what `open_by_mode` refuses. The name is a reading.

use crate::errors::BadInput;
use crate::files::handle::{NO_HANDLE, close, open_by_mode};
use crate::files::stream::{
    LastPath, MISSING_PATH, Stream, TEST_PATH, make_test_file, remove_test_file,
};
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva, shadow};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new("files::open_stream", rva(0x402CDD), thunk, &ORIGINAL)
    .private_copy(0x1D)
    .relocate_calls(&[0x03])
    .manual_patch()
    .self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- Rust version ----

/// Returns whether the file was opened.
pub unsafe fn open_stream(
    stream: *mut Stream,
    path: *const u8,
    mode: u32,
) -> Result<bool, BadInput> {
    unsafe {
        // Checked first, so that nothing is stored when the input is refused.
        let file = open_by_mode(path, mode)?;
        (&raw mut (*stream).mode).write_unaligned(mode);
        let Some(file) = file else {
            return Ok(false);
        };
        (&raw mut (*stream).file).write_unaligned(file);
        (&raw mut (*stream).position).write_unaligned(0);
        (&raw mut (*stream).filled).write_unaligned(0);
        Ok(true)
    }
}

// ---- hook ----

unsafe fn head(stream: *const Stream) -> [u32; 4] {
    unsafe { (stream as *const [u32; 4]).read_unaligned() }
}

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    let stream = r.esi as *mut Stream;
    if !shadow() {
        match unsafe { open_stream(stream, r.eax as *const u8, r.ebx) } {
            Ok(opened) => {
                r.set_carry(!opened);
                r.set_zero(false);
            }
            Err(e) => unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) },
        }
        return;
    }
    let mut o = *r;
    unsafe { call_original(&mut o) };
    let [mode, file, position, filled] = unsafe { head(stream) };
    let as_rust =
        mode == r.ebx && (o.carry() || (file != NO_HANDLE && position == 0 && filled == 0));
    if !as_rust || !o.preserved_except(r, 0) {
        HOOK.mismatch(|| {
            format!(
                "mode={}: the original left carry={} mode={mode} file={file:#x} position={position} filled={filled}",
                r.ebx,
                o.carry()
            )
        });
    }
    r.eflags = o.eflags;
}

// ---- self-test: the test file and a missing one, with the modes that do not create ----

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    if !make_test_file() {
        return t.skip("cannot create a test file in the game's var folder");
    }
    let saved = LastPath::save();
    for (path, mode) in [
        (TEST_PATH, 0u32),
        (TEST_PATH, 3),
        (MISSING_PATH, 0),
        (MISSING_PATH, 3),
    ] {
        let mut theirs = Stream::filler();
        let mut ours = Stream::filler();
        let before = Regs {
            eax: path.as_ptr() as u32,
            ebx: mode,
            esi: &mut *theirs as *mut Stream as u32,
            ..Regs::SENTINEL
        };
        let mut ro = before;
        unsafe { call_original(&mut ro) };
        let last_o = LastPath::now();
        // The two handles differ; each is closed, and what else was stored is compared.
        let opened_o = !ro.carry();
        let closed_o = opened_o && unsafe { close(theirs.file) };
        let mine = unsafe { open_stream(&mut *ours, path.as_ptr(), mode) };
        let last_r = LastPath::now();
        let closed_r = mine == Ok(true) && unsafe { close(ours.file) };
        let fields = |s: &Stream| (s.mode, s.position, s.filled, s.file == 0xAAAA_AAAA);
        let preserved = ro.preserved_except(&before, 0);
        let ok = mine == Ok(opened_o)
            && fields(&theirs) == fields(&ours)
            && theirs.buffer == ours.buffer
            && closed_o == closed_r
            && closed_o == opened_o
            && last_o == last_r
            && preserved;
        t.check(ok, || {
            format!(
                "path={:?} mode={mode}: original opened={opened_o} closed={closed_o} {:x?} last={last_o:#x} preserved={preserved}; rust {mine:?} closed={closed_r} {:x?} last={last_r:#x}",
                String::from_utf8_lossy(&path[..path.len() - 1]),
                fields(&theirs),
                fields(&ours)
            )
        });
    }
    saved.restore();
    remove_test_file();
    t.finish()
}
