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
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::BadInput;
use crate::framework::Regs;
use crate::strings::same_text::case_table;
use windows_sys::Win32::System::Environment::GetCurrentDirectoryA;

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

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    let (path, to) = (r.esi as *const u8, r.edx as *mut u8);
    let mine = match unsafe { normalised(path, &mut || current_directory()) } {
        Ok(mine) => mine,
        Err(e) => {
            return Err(e.into());
        }
    };
    if let Some(text) = &mine {
        unsafe { std::ptr::copy(text.as_ptr(), to, text.len()) };
    }
    r.set_carry(mine.is_none());
    Ok(())
}

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
