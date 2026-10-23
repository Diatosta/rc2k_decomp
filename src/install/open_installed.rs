//! `install::open_installed` (0x413629): the installed folder (`g_installSource` normalised
//! into `byte_95E7F8`) made the current directory and `g_installing` set; if either the
//! normalise or the change fails, nothing is set.
//!
//! In:  nothing
//! Out: every register kept (`pusha` / `popa`)
//! A list action (`off_51DD48` for the update script). Caller `install::run_page`. Its two
//! calls through `part`. Shadow mode runs the original alone (it changes directory).
//! Self-test: a copy whose two calls go to stand-ins, against the Rust version.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::install::installer::{INSTALLED, INSTALLING, NORMALISE, Part, SET_CD, a, at, p};
use crate::race::viewer::regs_entry;

const SOURCE_PATH: usize = 0x51BB8D; // g_installSource

pub fn open_installed(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut r = Regs {
        esi: a(SOURCE_PATH),
        edx: a(INSTALLED),
        ..*entry
    };
    part(at(NORMALISE), &mut r)?;
    if r.carry() {
        return Ok(*entry);
    }
    r.edx = a(INSTALLED);
    part(at(SET_CD), &mut r)?;
    if r.carry() {
        return Ok(*entry);
    }
    p(INSTALLING, 1);
    Ok(*entry)
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    regs_entry(r, &|r| {
        open_installed(r, &mut crate::install::installer::game_part)
    })
}
