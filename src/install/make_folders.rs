//! `install::make_folders` (0x413A97): the folders of a path made, segment by segment. A
//! drive path (`X:`) first changes to the drive's root; a UNC path (`\\`) skips the server
//! and share. Then for each `\`-separated segment the path is cut there and changed into
//! (`files::set_current_directory`); where that fails the folder is made
//! (`files::make_directory`) and entered. The path's bytes are cut with a NUL and put back
//! as the walk goes. A failed change or make ends it.
//!
//! In:  edx the path (normalised: ending in `\\`)
//! Out: every register kept (`pusha` / `popa`); carry set when a folder could not be gone
//!      into or made (the failing call's flags), else zero set and carry clear
//! A path without its closing `\\` (as `install_files` gives every folder) is scanned on
//! past its NUL to the next `\\`, as the original does. Refuses (as met) a scan of 0x400
//! bytes from the path's start, and a UNC path without its share's `\\`.
//! Callers `install::install_files`, `install::install_clio`. Its calls through `part`
//! (`files::set_current_directory`, `files::make_directory`, both edx the path). Ungated.
//! Shadow mode runs the original alone (folders are made). Self-test: a copy whose two
//! calls go to stand-ins (their success scripted), against the Rust version, on paths of
//! the test's; the path buffer is compared (it is put back).
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::install::installer::{MAKE_DIR, SET_CD, at, rb, rw, wb};

const LIMIT: u32 = 0x400;

/// A path that does not end in `\\`: the original looks for the next one past its end.
const PAST_END: crate::errors::BadInput =
    crate::errors::BadInput("a path that does not end in \\ (the original reads past its end)");

/// The Rust version, register for register as the original: the registers its `popa`
/// leaves (the entry's) with the flags of its end: carry set when a folder could not be
/// gone into or made (the failing call's flags), else the end's `cmp byte ptr [esi+1], 0`
/// (zero, carry clear).
pub fn make_folders(entry: &Regs) -> Result<Regs, Refused> {
    let mut r = *entry;
    let failed = |r: &Regs| -> Result<Regs, Refused> {
        Ok(Regs {
            eflags: r.eflags,
            ..*entry
        })
    };
    r.esi = r.edx;
    r.eax = (r.eax & !0xFFFF) | rw(r.esi) as u32;
    if (r.eax >> 8) as u8 == b':' {
        r.esi += 2;
        // xor cl, cl; xchg cl, [esi+1]
        r.ecx = (r.ecx & !0xFF) | rb(r.esi + 1) as u32;
        wb(r.esi + 1, 0);
        crate::install::installer::game_part(at(SET_CD), &mut r)?;
        wb(r.esi + 1, r.ecx as u8);
        if r.carry() {
            return failed(&r);
        }
    } else if r.eax as u16 == 0x5C5C {
        r.esi += 1;
        loop {
            r.esi += 1;
            match rb(r.esi) {
                b'\\' => break,
                0 => return Err(PAST_END.into()),
                _ => {}
            }
        }
        loop {
            r.esi += 1;
            match rb(r.esi) {
                b'\\' => break,
                0 => return Err(PAST_END.into()),
                _ => {}
            }
        }
        wb(r.esi, 0);
        crate::install::installer::game_part(at(SET_CD), &mut r)?;
        wb(r.esi, b'\\');
        if r.carry() {
            return failed(&r);
        }
    }
    // loc_413AD6. A path without its closing `\\` (the installer's folders, as the script
    // lists them: `install_files`) has the scan go on past its NUL to the next `\\` in what
    // follows, as the original's does: each further segment only goes into the same path
    // again (`notes/bugs.md`). Bounded: 0x400 bytes from the path's start.
    while rb(r.esi + 1) != 0 {
        loop {
            r.esi += 1;
            if r.esi.wrapping_sub(entry.edx) > LIMIT {
                return Err(crate::errors::BadInput("a path with no end in 0x400 bytes").into());
            }
            if rb(r.esi) == b'\\' {
                break;
            }
        }
        wb(r.esi, 0);
        crate::install::installer::game_part(at(SET_CD), &mut r)?;
        wb(r.esi, b'\\');
        if !r.carry() {
            continue;
        }
        wb(r.esi, 0);
        crate::install::installer::game_part(at(MAKE_DIR), &mut r)?;
        wb(r.esi, b'\\');
        if r.carry() {
            return failed(&r);
        }
        wb(r.esi, 0);
        crate::install::installer::game_part(at(SET_CD), &mut r)?;
        wb(r.esi, b'\\');
        if r.carry() {
            return failed(&r);
        }
    }
    let mut out = *entry;
    out.set_carry(false);
    out.set_zero(true);
    Ok(out)
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    crate::race::viewer::regs_entry(r, &|r| make_folders(r))
}
