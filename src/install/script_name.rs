//! A name copied from the script (from the cursor on) to `script_path` until a byte below a
//! space, a NUL written at the end. Called by `install::run_script`.

use crate::errors::Refused;
use crate::install::installer::wb;
use rc2k_core::install::script;

/// The name at the script's `cursor` written at `to`, NUL-ended. Returns the
/// cursor at the stop byte (not past it) and where the NUL went.
pub fn script_name(
    cursor: crate::memory::place::BytePlace,
    to: impl Into<crate::memory::place::BytePlace>,
) -> Result<
    (
        crate::memory::place::BytePlace,
        crate::memory::place::BytePlace,
    ),
    Refused,
> {
    // Read whole, then written; a name with no end is refused before anything is written.
    let words = cursor.bytes("the installer's script")?;
    let (name, _) = script::script_name(&words)?;
    let mut at = to.into();
    for &c in &name {
        wb(at, c);
        at = at.add(1);
    }
    wb(at, 0);
    Ok((cursor.add(name.len() as u32), at))
}
