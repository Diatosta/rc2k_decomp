//! `install::script_name` (0x41370C): a name copied from the script (`[ebx]` on) to
//! `byte_95EAF8` (`[esi]`), byte by byte, until a byte below a space (a control byte or the
//! end); a NUL written at the end.
//!
//! Takes the script cursor, where to write.
//! Caller `install::run_script` (three times).
//!
//! Conversion phase 3 (2026-10-23): plain Rust with typed arguments.

use crate::errors::Refused;
use crate::install::installer::wb;
use rc2k_core::install::script;

/// The name at the script's `cursor` written at `to`, NUL-ended. Returns the
/// cursor at the stop byte (not past it) and where the NUL went.
pub fn script_name(
    cursor: crate::place::Place,
    to: impl Into<crate::place::Place>,
) -> Result<(crate::place::Place, crate::place::Place), Refused> {
    // Converted: `rc2k_core::install::script::script_name` (read whole, then written; a
    // name with no end is refused before anything is written).
    let words = cursor.game_ptr().bytes("the installer's script")?;
    let (name, _) = script::script_name(&words)?;
    let mut at = to.into();
    for &c in &name {
        wb(at, c);
        at = at.add(1);
    }
    wb(at, 0);
    Ok((cursor.add(name.len() as u32), at))
}
