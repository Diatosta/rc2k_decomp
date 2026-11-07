//! `files::numbered_path` (0x403292): copies a path template to the buffer at 0x94E8B0 and
//! writes a number over its last `!!`, or if there is none its last `00`.
//!
//! Takes template, number. Returns the buffer. The number is written with `format_u32` flags 0x12:
//! two digits, zero-padded (a larger number's last two). A template with neither marker is fatal
//! error 0x80F19001 in the original. The search never looks at the template's first character.
//! Refuses an empty template (the original reads before the buffer) and anything that does not fit
//! the buffer's 0x100 bytes.

use crate::errors::{BadInput, FatalError, Refused};

// ---- game data ----

const BUFFER_LEN: usize = 0x100; // up to the path buffer at 0x94E9B0

// ---- Rust version ----

/// What the buffer holds afterwards, from its start to the last byte written. `template`
/// excludes the NUL.
pub fn numbered_path(template: &[u8], number: u32) -> Result<Vec<u8>, Refused> {
    // Converted: `rc2k_core::files::paths::numbered`.
    let Some(out) = rc2k_core::files::paths::numbered(template, number)? else {
        return Err(FatalError {
            code: 0x80F1_9001,
            message: "",
        }
        .into());
    };
    if out.len() > BUFFER_LEN {
        return Err(BadInput("longer than the buffer: the original writes past it").into());
    }
    Ok(out)
}

/// For Rust callers: writes the buffer (as the original does) and returns the path, both
/// `template` and the path without their NULs.
pub fn numbered_path_bytes(template: &[u8], number: u32) -> Result<Vec<u8>, Refused> {
    let path = numbered_path(template, number)?;
    crate::files::state::PATHS.write_at(0, &path, "the numbered path's buffer")?;
    Ok(path[..path.iter().position(|&b| b == 0).unwrap_or(path.len())].to_vec())
}
