//! `install::script_name` (0x41370C): a name copied from the script (`[ebx]` on) to
//! `byte_95EAF8` (`[esi]`), byte by byte, until a byte below a space (a control byte or the
//! end); a NUL written at the end.
//!
//! In:  ebx the script cursor, esi where to write
//! Out: al the stop byte, ebx at it (not past), esi where the NUL went; the flags of
//!      `cmp al, 20h` (no caller reads them)
//! Caller `install::run_script` (three times). A loop back to its first byte: private copy,
//! manual patch. Shadow mode compares. Self-test: scripts of the test's against the
//! original.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::bridge::Raw;
use crate::errors::Refused;
use crate::framework::Regs;
use crate::install::installer::wb;
use crate::race::viewer::regs_entry;
use rc2k_core::install::script;

pub fn script_name(entry: &Regs) -> Result<Regs, Refused> {
    let mut r = *entry;
    // Converted: `rc2k_core::install::script::script_name` (read whole, then written; a
    // name with no end is refused before anything is written).
    let script = unsafe { Raw::new(r.ebx as *const u8) };
    let (name, stop) = script::script_name(&script)?;
    for &c in &name {
        wb(r.esi, c);
        r.esi = r.esi.wrapping_add(1);
    }
    wb(r.esi, 0);
    r.ebx = r.ebx.wrapping_add(name.len() as u32);
    r.eax = r.eax & !0xFF | stop as u32;
    r.set_carry(true);
    r.set_zero(false);
    Ok(r)
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    regs_entry(r, &script_name)
}
