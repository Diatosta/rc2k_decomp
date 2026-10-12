//! `files::swap_missing_extension` (0x4032DB): if a path ends in `.png` or `.avi` and there
//! is no such file, changes the ending to `.pcx` or `.mpg`, in place.
//!
//! In:  eax = path, which is written to
//! Out: every register preserved
//! Paths of fewer than five characters are left alone, and so is every other ending (also
//! `.pcx` and `.mpg`, and upper case). Whether the file is there is asked of
//! `files::file_exists`. Two callers: the picture loader and the film player `sub_4161CD`.
//! Manual patch with a private copy (0x35 bytes, both calls relocated): the first call
//! starts at the fourth byte, inside what the patch overwrites.
//! Refuses what `text_length` and `file_exists` refuse. The name is a reading.

use crate::errors::BadInput;
use crate::files::file_exists::file_exists;
use crate::files::stream::LastPath;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva, shadow};
use crate::strings::text_length::text_length;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "files::swap_missing_extension",
    rva(0x4032DB),
    thunk,
    &ORIGINAL,
)
.private_copy(0x35)
.relocate_calls(&[0x03, 0x25])
.manual_patch()
.self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// aPngPcxAviMpg (0x4E0744): an ending, and what it becomes.
const PARTNERS: [(&[u8; 4], &[u8; 4]); 2] = [(b".png", b".pcx"), (b".avi", b".mpg")];

// ---- Rust version ----

/// Where the path's last four characters are, if it has at least five.
unsafe fn ending(path: *mut u8) -> Result<Option<*mut [u8; 4]>, BadInput> {
    let length = unsafe { text_length(path) }? as usize;
    Ok((length >= 5).then(|| path.wrapping_add(length - 4).cast()))
}

pub unsafe fn swap_missing_extension(path: *mut u8) -> Result<(), BadInput> {
    unsafe {
        let Some(ending) = ending(path)? else {
            return Ok(());
        };
        let Some((_, partner)) = PARTNERS.iter().find(|(from, _)| ending.read() == **from) else {
            return Ok(());
        };
        if !file_exists(path)? {
            ending.write(**partner);
        }
        Ok(())
    }
}

// ---- hook ----

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    let path = r.eax as *mut u8;
    if !shadow() {
        if let Err(e) = unsafe { swap_missing_extension(path) } {
            unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) };
        }
        return;
    }
    // The Rust version first, on the path as it came; then the path is put back and the
    // original runs on it.
    let place = unsafe { ending(path) }.ok().flatten();
    let before = place.map(|at| unsafe { at.read() });
    let mine = unsafe { swap_missing_extension(path) };
    let after = place.map(|at| unsafe { at.read() });
    if let (Some(at), Some(bytes)) = (place, before) {
        unsafe { at.write(bytes) };
    }
    let mut o = *r;
    unsafe { call_original(&mut o) };
    let theirs = place.map(|at| unsafe { at.read() });
    if mine.is_ok() && (theirs != after || !o.preserved_except(r, 0)) {
        HOOK.mismatch(|| {
            format!(
                "path={:?}: original left {theirs:x?}, rust {after:x?}",
                unsafe { std::ffi::CStr::from_ptr(path.cast()) }
            )
        });
    }
}

// ---- self-test: paths to a file that is there and to ones that are not ----

const THERE: &str = "var\\rc2k_hook_swap_test.png";

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    if std::fs::write(THERE, b"x").is_err()
        || std::fs::write(THERE.replace(".png", ".avi"), b"x").is_err()
    {
        return t.skip("cannot create a test file in the game's var folder");
    }
    let saved = LastPath::save();
    let paths: [&[u8]; 12] = [
        b"var\\rc2k_hook_swap_test.png",
        b"var\\rc2k_hook_swap_test.avi",
        b"var\\rc2k_hook_swap_none.png",
        b"var\\rc2k_hook_swap_none.avi",
        b"var\\rc2k_hook_swap_none.pcx",
        b"var\\rc2k_hook_swap_none.mpg",
        b"var\\rc2k_hook_swap_none.PNG",
        b"var\\rc2k_hook_swap_none.txt",
        b"v.png",
        b".png",
        b"png",
        b"",
    ];
    for path in paths {
        let run = |rust: bool| {
            let mut buffer = path.to_vec();
            buffer.extend_from_slice(b"\0\xAA\xAA\xAA\xAA");
            let before = Regs {
                eax: buffer.as_mut_ptr() as u32,
                ..Regs::SENTINEL
            };
            let ok = if rust {
                unsafe { swap_missing_extension(buffer.as_mut_ptr()) }.is_ok()
            } else {
                let mut ro = before;
                unsafe { call_original(&mut ro) };
                ro.preserved_except(&before, 0)
            };
            // Zero after a file that is there; after one that is not, a pointer that may be
            // into this run's own buffer.
            (ok, buffer, LastPath::now() != 0)
        };
        let (theirs, mine) = (run(false), run(true));
        t.check(theirs == mine && theirs.0, || {
            format!(
                "path={:?}: original ok={} left {:?} last path set={}; rust ok={} left {:?} last path set={}",
                String::from_utf8_lossy(path),
                theirs.0,
                String::from_utf8_lossy(&theirs.1[..path.len()]),
                theirs.2,
                mine.0,
                String::from_utf8_lossy(&mine.1[..path.len()]),
                mine.2
            )
        });
    }
    saved.restore();
    let _ = std::fs::remove_file(THERE);
    let _ = std::fs::remove_file(THERE.replace(".png", ".avi"));
    t.finish()
}
