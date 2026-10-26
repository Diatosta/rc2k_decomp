//! `install::script_name` (0x41370C): a name copied from the script (`[ebx]` on) to
//! `byte_95EAF8` (`[esi]`), byte by byte, until a byte below a space (a control byte or the
//! end); a NUL written at the end.
//!
//! In:  ebx the script cursor, esi where to write
//! Out: al the stop byte, ebx at it (not past), esi where the NUL went; the flags of
//!      `cmp al, 20h` (no caller reads them)
//! Caller `install::run_script` (three times).
//!
//! Conversion phase 3 (2026-10-23): plain Rust with typed arguments; the registers above
//! are the original's.

use crate::errors::Refused;
use crate::install::installer::wb;
use rc2k_core::install::script;

/// The name at the script's `cursor` (ebx) written at `to` (esi), NUL-ended. Returns the
/// cursor at the stop byte (not past it) and where the NUL went.
pub fn script_name(cursor: u32, to: u32) -> Result<(u32, u32), Refused> {
    // Converted: `rc2k_core::install::script::script_name` (read whole, then written; a
    // name with no end is refused before anything is written).
    let words = crate::pointers::GamePtr::new(cursor).bytes("the installer's script")?;
    let (name, _) = script::script_name(&words)?;
    let mut at = to;
    for &c in &name {
        wb(at, c);
        at = at.wrapping_add(1);
    }
    wb(at, 0);
    Ok((cursor.wrapping_add(name.len() as u32), at))
}
