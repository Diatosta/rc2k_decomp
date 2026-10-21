//! `install::normalise_path` (0x4139EC): a directory's path in the installer's one form.
//!
//! In:  esi = the path, edx = where to write it
//! Out: carry clear if it was written, set if not; every register preserved
//! The form: lower case (through `g_lowerCase`), `\` for every `/`, and a `\` at the end.
//! An empty path stands for the current directory (`sub_4030D1`: `GetCurrentDirectoryA`
//! into the place, a `\` added). Otherwise it must start with `\\` or `//`, or with a
//! drive letter and `:`, after which a `\` is put if there is none; anything else is not
//! written. 14 callers, most of them in the installer.
//! Refuses a path of 0xF8 bytes or more, and a current directory that does not fit the
//! 0xFF bytes asked with.

use crate::errors::BadInput;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva, shadow};
use crate::strings::same_text::case_table;
use std::sync::atomic::AtomicUsize;
use windows_sys::Win32::System::Environment::GetCurrentDirectoryA;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("install::normalise_path", rva(0x4139EC), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- Rust version ----

/// `sub_4030D1`'s asking: the current directory, or None if it cannot be had.
pub unsafe fn current_directory() -> Result<Option<Vec<u8>>, BadInput> {
    let mut place = [0u8; 0x100];
    let len = unsafe { GetCurrentDirectoryA(0xFF, place.as_mut_ptr()) } as usize;
    match len {
        0 => Ok(None),
        0xFF.. => Err(BadInput("a current directory longer than the place for it")),
        _ => Ok(Some(place[..len].to_vec())),
    }
}

/// The path as it is written, with its NUL; None if it is not one that is taken.
/// `current` is the current directory, asked for only if the path is empty.
pub unsafe fn normalised(
    path: *const u8,
    current: &mut dyn FnMut() -> Result<Option<Vec<u8>>, BadInput>,
) -> Result<Option<Vec<u8>>, BadInput> {
    let path = unsafe { std::ffi::CStr::from_ptr(path as *const _) }.to_bytes();
    // Converted: `rc2k_core::install::paths::normalise`.
    rc2k_core::install::paths::normalise(path, &case_table(), current)
}

/// For Rust callers: writes the path at `to`. Returns whether it was written.
pub unsafe fn normalise_path(path: *const u8, to: *mut u8) -> Result<bool, BadInput> {
    unsafe {
        let Some(text) = normalised(path, &mut || current_directory())? else {
            return Ok(false);
        };
        std::ptr::copy(text.as_ptr(), to, text.len());
        Ok(true)
    }
}

// ---- hook ----

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    let (path, to) = (r.esi as *const u8, r.edx as *mut u8);
    let mine = match unsafe { normalised(path, &mut || current_directory()) } {
        Ok(mine) => mine,
        Err(e) => {
            unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) };
            return;
        }
    };
    if !shadow() {
        if let Some(text) = &mine {
            unsafe { std::ptr::copy(text.as_ptr(), to, text.len()) };
        }
        r.set_carry(mine.is_none());
        return;
    }
    let mut o = *r;
    unsafe { call_original(&mut o) };
    let same = match &mine {
        Some(text) => {
            !o.carry() && unsafe { std::slice::from_raw_parts(to, text.len()) } == &text[..]
        }
        None => o.carry(),
    };
    if !same || !o.preserved_except(r, 0) {
        HOOK.mismatch(|| {
            format!(
                "path={:?}: original carry={}; rust {:?}",
                unsafe { std::ffi::CStr::from_ptr(path as *const _) },
                o.carry(),
                mine.as_ref()
                    .map(|text| String::from_utf8_lossy(text).into_owned())
            )
        });
    }
    r.eflags = o.eflags;
}

// ---- self-test ----

pub const TEST_PATHS: [&[u8]; 14] = [
    b"\0",
    b"C:\\Games\\RC2k\0",
    b"C:\\Games\\RC2k\\\0",
    b"c:\0",
    b"D:/Mixed/Case/\0",
    b"E:File\0",
    b"\\\\Server\\Share\\Dir\0",
    b"//Server/Share\0",
    b"\\\\\0",
    b"relative\\path\0",
    b"x\0",
    b":\0",
    b"\xC9:\\\xC0b\0",
    b"1:/\0",
];

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    for path in TEST_PATHS {
        let run = |rust: bool| {
            let mut to = [0xAAu8; 0x140];
            let before = Regs {
                esi: path.as_ptr() as u32,
                edx: to.as_mut_ptr() as u32,
                ..Regs::SENTINEL
            };
            let written = if rust {
                unsafe { normalise_path(path.as_ptr(), to.as_mut_ptr()) }.ok()
            } else {
                let mut ro = before;
                unsafe { call_original(&mut ro) };
                ro.preserved_except(&before, 0).then_some(!ro.carry())
            };
            (written, to)
        };
        let (theirs, mine) = (run(false), run(true));
        t.check(theirs == mine && theirs.0.is_some(), || {
            format!(
                "path={:?}: original {:?} {:?}; rust {:?} {:?}",
                String::from_utf8_lossy(path),
                theirs.0,
                crate::strings::utils::bytes_str(&theirs.1[..0x40]),
                mine.0,
                crate::strings::utils::bytes_str(&mine.1[..0x40])
            )
        });
    }
    t.finish()
}
