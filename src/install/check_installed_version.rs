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

use crate::errors::Refused;
use crate::files::scratch::SCRATCH;
use crate::install::installer::set_cd;
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
    let normalised = crate::install::normalise_path::normalised_bytes(
        &INSTALLER_NAMES.source.text(),
        &mut || crate::install::normalise_path::current_directory(),
    )?;
    let Some(installed) = normalised else {
        return Ok(());
    };
    INSTALLER_PATHS
        .installed
        .write_at(0, &installed, "the installed folder")?;
    if !set_cd(&INSTALLER_PATHS.installed) {
        return Ok(());
    }
    SCRATCH
        .buffer
        .write(&[0; crate::files::scratch::Scratch::LEN]);
    // The standalone checks its own program (the same version text is in its built-in
    // image). files::open_by_mode, mode 0.
    let program = crate::install::run_after_exit::RC2K;
    if let Some(file) = crate::files::handle::open_by_mode(program, 0)? {
        let buffer = &SCRATCH.buffer;
        loop {
            // the previous tail carried to the front (`rep movsd`, 0x40 dwords)
            for n in 0..0x100 {
                buffer.set(n, buffer.get(TAIL + n));
            }
            let mut bytes = vec![0u8; SCAN_LEN as usize];
            let (read, ok) = crate::files::handle::read_slice(file, &mut bytes);
            buffer.write_at(READ_TARGET, &bytes[..read as usize], "the program's bytes")?;
            if !ok || read == 0 {
                break;
            }
            // Converted: `rc2k_core::install::version::scan` (the copyright compared up
            // to its NUL, inside its 91 bytes).
            let scanned =
                buffer.read_at(0, crate::files::scratch::Scratch::LEN, "the scan buffer")?;
            let copyright = START_TEXTS.copyright.read();
            if let Scan::Stopped { same, .. } = version::scan(&scanned[..], &copyright[..])? {
                if same {
                    INSTALLER_RUN.same_version.set(1);
                }
                break;
            }
        }
        crate::files::handle::close(file);
    }
    set_cd(&INSTALLER_PATHS.game_dir);
    Ok(())
}
