//! `install::check_installed_version` (0x413B0B): whether the already-installed `ral.exe` is
//! this same build (`dword_51B9C8`). Only when installed (`g_installStatus` 1) with a source
//! (`g_installSource`): the source is normalised into `byte_95E7F8` and made current, then
//! `.\ral.exe` is read in 0xFF00-byte chunks (`g_pngInfo` the buffer, the last 0x100 bytes of
//! each chunk carried to the front of the next so a match can span a boundary). Each chunk is
//! scanned for `$VER: Rally 5`; on a hit the bytes are skipped to the next NUL and compared
//! against the copyright line (`byte_4E001C`, up to its first `.`); a full match sets
//! `dword_51B9C8`. The file is closed and the current directory set to `byte_95E6F8`.
//!
//! In:  nothing
//! Out: every register kept (`pusha` / `popa`)
//! Caller `install::run_installer`. Its directory and file calls through `part`; the scan
//! here. Shadow mode runs the original alone (it reads a file). Self-test: a copy whose
//! directory/file calls go to the fake file system, against the Rust version, with a
//! matching ral.exe, a non-matching one, and no installation.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.
//! Conversion phase 3 (2026-10-23): plain Rust with typed arguments; the registers above
//! are the original's.

use crate::bridge::{Raw, read_through};
use crate::errors::Refused;
use crate::install::installer::{INSTALLED, STATUS, a, gb, p, rb, set_cd, wb};
use rc2k_core::install::version::{self, Scan};

const SAME_VERSION: usize = 0x51B9C8; // dword_51B9C8
const SOURCE: usize = 0x51BB8D; // g_installSource
const BASE_A: usize = 0x95E6F8; // byte_95E6F8: the folder to go back to
const BUF: usize = 0x93E8B0; // g_pngInfo
const READ_TARGET: usize = BUF + 0x100;
const TAIL: usize = BUF + 0xFF00;
const SCAN_LEN: u32 = version::CHUNK;
const COPYRIGHT: usize = 0x4E001C; // byte_4E001C

pub fn check_installed_version() -> Result<(), Refused> {
    p(SAME_VERSION, 0);
    if gb(STATUS) != 1 || gb(SOURCE) == 0 {
        return Ok(());
    }
    // normalise the source into byte_95E7F8, cd there
    let written = unsafe {
        crate::install::normalise_path::normalise_path(
            a(SOURCE) as *const u8,
            a(INSTALLED) as *mut u8,
        )
    }?;
    if !written || !set_cd(a(INSTALLED)) {
        return Ok(());
    }
    unsafe { crate::system::zero_memory::zero_memory(a(BUF) as *mut u8, 0x10000) }?;
    // The standalone checks its own program (the same version text is in its built-in
    // image). files::open_by_mode, mode 0.
    let program = crate::install::run_after_exit::RC2K.as_ptr();
    if let Some(file) = unsafe { crate::files::handle::open_by_mode(program, 0) }? {
        loop {
            // the previous tail carried to the front (`rep movsd`, 0x40 dwords)
            for n in 0..0x100u32 {
                wb(a(BUF) + n, rb(a(TAIL) + n));
            }
            let (read, ok) =
                unsafe { crate::files::handle::read(file, a(READ_TARGET) as *mut u8, SCAN_LEN) };
            if !ok || read == 0 {
                break;
            }
            // Converted: `rc2k_core::install::version::scan`.
            let buffer = unsafe { Raw::new(a(BUF) as *const u8) };
            let copyright = unsafe { Raw::new(a(COPYRIGHT) as *const u8) };
            if let Scan::Stopped { same, .. } = read_through(version::scan(&buffer, &copyright)) {
                if same {
                    p(SAME_VERSION, 1);
                }
                break;
            }
        }
        unsafe { crate::files::handle::close(file) };
    }
    set_cd(a(BASE_A));
    Ok(())
}
