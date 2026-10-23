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

use crate::bridge::{Raw, read_through};
use crate::errors::Refused;
use crate::framework::Regs;
use crate::install::installer::{CLOSE, INSTALLED, OPEN, READ, SET_CD, STATUS, a, at, gb, p, rb};
use rc2k_core::install::version::{self, Scan};

const SAME_VERSION: usize = 0x51B9C8; // dword_51B9C8
const SOURCE: usize = 0x51BB8D; // g_installSource
const BASE_A: usize = 0x95E6F8; // byte_95E6F8: the folder to go back to
const BUF: usize = 0x93E8B0; // g_pngInfo
const READ_TARGET: usize = BUF + 0x100;
const TAIL: usize = BUF + 0xFF00;
const SCAN_LEN: u32 = version::CHUNK;
const COPYRIGHT: usize = 0x4E001C; // byte_4E001C

pub fn check_installed_version(entry: &Regs) -> Result<Regs, Refused> {
    let mut r = *entry;
    p(SAME_VERSION, 0);
    if gb(STATUS) != 1 || gb(SOURCE) == 0 {
        return Ok(*entry);
    }
    // normalise the source into byte_95E7F8, cd there
    let written = unsafe {
        crate::install::normalise_path::normalise_path(
            a(SOURCE) as *const u8,
            a(INSTALLED) as *mut u8,
        )
    }?;
    if !written {
        return Ok(*entry);
    }
    r.esi = a(SOURCE);
    r.edx = a(INSTALLED);
    crate::install::installer::game_part(at(SET_CD), &mut r)?;
    if r.carry() {
        return Ok(*entry);
    }
    unsafe { crate::system::zero_memory::zero_memory(a(BUF) as *mut u8, 0x10000) }?;
    // The standalone checks its own program (the same version text is in its built-in
    // image).
    r.eax = crate::install::run_after_exit::RC2K.as_ptr() as u32;
    r.ebx = 0;
    crate::install::installer::game_part(at(OPEN), &mut r)?;
    if !r.carry() {
        // Register for register: edx the handle; the close gets ebx and ecx where the scan
        // stopped (`loc_413C13`).
        r.edx = r.eax;
        'read: loop {
            // the previous tail carried to the front (`rep movsd`, 0x40 dwords)
            for n in 0..0x100u32 {
                let b = rb(a(TAIL) + n);
                crate::install::installer::wb(a(BUF) + n, b);
            }
            r.esi = a(TAIL) + 0x100;
            r.edi = a(BUF) + 0x100;
            r.eax = r.edx;
            r.ebx = a(READ_TARGET);
            r.ecx = SCAN_LEN;
            crate::install::installer::game_part(at(READ), &mut r)?;
            if r.carry() || r.eax == 0 {
                break;
            }
            // Converted: `rc2k_core::install::version::scan`.
            let buffer = unsafe { Raw::new(a(BUF) as *const u8) };
            let copyright = unsafe { Raw::new(a(COPYRIGHT) as *const u8) };
            match read_through(version::scan(&buffer, &copyright)) {
                Scan::Nothing => {
                    r.ebx = a(BUF) + SCAN_LEN;
                    r.ecx = 0;
                }
                Scan::Stopped { same, at, count } => {
                    if same {
                        p(SAME_VERSION, 1);
                    }
                    r.ebx = a(BUF) + at;
                    r.ecx = count;
                    break 'read;
                }
            }
        }
        r.eax = r.edx;
        crate::install::installer::game_part(at(CLOSE), &mut r)?;
    }
    r.edx = a(BASE_A);
    crate::install::installer::game_part(at(SET_CD), &mut r)?;
    Ok(Regs { ..*entry })
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    crate::race::viewer::regs_entry(r, &|r| check_installed_version(r))
}
