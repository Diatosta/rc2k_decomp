//! `install::value_by_number` (0x4148B7): an item of a value (kind 3) set by a number key to
//! its least plus (most - least) x ecx / 8 (`imul`, `idiv` by 8, towards 0).
//!
//! In:  esi the item (+8 the value's byte, +0xC the least, +0xD the most), ecx the number
//! Out: eax kept; ebx the least, ecx 8, edx the value's address; zero set, carry clear
//! Callers `install::key_number` (ecx the key less 2: 0 .. 8). Shadow mode compares.
//! Self-test: items of the test's against the original.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.
//! Conversion phase 3 (2026-10-23): plain Rust with typed arguments; the registers above
//! are the original's.

use crate::errors::Refused;
use crate::install::installer::{rb, rd, wb};

/// `number` the eighths (ecx).
pub fn value_by_number(item: u32, number: u32) -> Result<(), Refused> {
    // Converted: `rc2k_core::install::items::value_by_number`.
    let value = rc2k_core::install::items::value_by_number(rb(item + 0xC), rb(item + 0xD), number)?;
    wb(rd(item + 8), value);
    Ok(())
}
