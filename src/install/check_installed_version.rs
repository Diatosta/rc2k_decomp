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
//! Caller `install::run_installer`.
//!
//! Conversion phase 3 (2026-10-23): plain Rust with typed arguments; the registers above
//! are the original's.

use crate::bridge::{Raw, read_through};
use crate::errors::Refused;
use crate::files::scratch::SCRATCH;
use crate::install::installer::{rb, set_cd, wb};
use crate::install::vars::INSTALLER_NAMES;
use crate::install::vars::INSTALLER_PATHS;
use crate::install::vars::INSTALLER_RUN;
use crate::system::start_texts::START_TEXTS;
use rc2k_core::install::version::{self, Scan};

const READ_TARGET: usize = 0x100; // into the scratch
const TAIL: usize = 0xFF00;
const SCAN_LEN: u32 = version::CHUNK;

pub fn check_installed_version() -> Result<(), Refused> {
    INSTALLER_RUN.same_version.set(0);
    if INSTALLER_NAMES.status.get() != 1 || INSTALLER_NAMES.source.get(0) == 0 {
        return Ok(());
    }
    // normalise the source into byte_95E7F8, cd there
    let written = unsafe {
        crate::install::normalise_path::normalise_path(
            INSTALLER_NAMES.source.addr() as u32 as *const u8,
            INSTALLER_PATHS.installed.addr() as u32 as *mut u8,
        )
    }?;
    if !written || !set_cd(INSTALLER_PATHS.installed.addr() as u32) {
        return Ok(());
    }
    unsafe {
        crate::system::zero_memory::zero_memory(SCRATCH.buffer.addr() as u32 as *mut u8, 0x10000)
    }?;
    // The standalone checks its own program (the same version text is in its built-in
    // image). files::open_by_mode, mode 0.
    let program = crate::install::run_after_exit::RC2K.as_ptr();
    if let Some(file) = unsafe { crate::files::handle::open_by_mode(program, 0) }? {
        loop {
            // the previous tail carried to the front (`rep movsd`, 0x40 dwords)
            for n in 0..0x100u32 {
                wb(
                    SCRATCH.buffer.addr() as u32 + n,
                    rb(SCRATCH.buffer.addr() as u32 + TAIL as u32 + n),
                );
            }
            let (read, ok) = unsafe {
                crate::files::handle::read(
                    file,
                    (SCRATCH.buffer.addr() + READ_TARGET) as *mut u8,
                    SCAN_LEN,
                )
            };
            if !ok || read == 0 {
                break;
            }
            // Converted: `rc2k_core::install::version::scan`.
            let buffer = unsafe { Raw::new(SCRATCH.buffer.addr() as u32 as *const u8) };
            let copyright = unsafe { Raw::new(START_TEXTS.copyright.addr() as u32 as *const u8) };
            if let Scan::Stopped { same, .. } = read_through(version::scan(&buffer, &copyright)) {
                if same {
                    INSTALLER_RUN.same_version.set(1);
                }
                break;
            }
        }
        unsafe { crate::files::handle::close(file) };
    }
    set_cd(INSTALLER_PATHS.game_dir.addr() as u32);
    Ok(())
}
