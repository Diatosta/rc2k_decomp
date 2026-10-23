//! `install::check_folder_name` (0x413A6C): whether a folder path (from its third byte on,
//! past a drive letter and colon) holds a character that is not allowed in a folder name
//! (`unk_51DE18`: `"`, `+`, `*`, `,`, `;`, `=`, `/`, `:`, `<`, `>`, `?`, `\`, `{`, `}`,
//! `[`, `]`, `|`, 0x7F) or a double backslash.
//!
//! In:  edx the path
//! Out: carry set if a bad character or `\\` is found, clear otherwise; every register kept
//! Caller `install::install_files`. Private copy (0x2B bytes): the loop's `jnz` at +0x12 goes
//! back to +4, inside the patched bytes. Shadow mode compares. Self-test: paths of the test's
//! against the original.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::bridge::{Raw, read_through};
use crate::errors::Refused;
use crate::framework::Regs;
use crate::race::viewer::regs_entry;

pub fn check_folder_name(entry: &Regs) -> Regs {
    let mut r = *entry;
    // Converted: `rc2k_core::install::paths::bad_folder_name` (the characters, RAL.EXE's
    // `unk_51DE18`, as its constant).
    let path = unsafe { Raw::new(r.edx as *const u8) };
    r.set_carry(read_through(rc2k_core::install::paths::bad_folder_name(
        &path,
    )));
    r
}

fn rust(r: &Regs) -> Result<Regs, Refused> {
    Ok(check_folder_name(r))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    regs_entry(r, &rust)
}
