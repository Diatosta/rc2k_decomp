//! Copies a path template to the numbered-path buffer and writes a number over its last `!!`,
//! or if there is none its last `00`: two digits, zero-padded (a larger number's last two). A
//! template with neither marker is fatal error 0x80F19001. The search never looks at the
//! template's first character. Refuses an empty template and anything that does not fit the
//! buffer's 0x100 bytes.

use crate::errors::{BadInput, FatalError, Refused};

const BUFFER_LEN: usize = 0x100; // up to the resolved-path buffer after it

/// What the buffer holds afterwards, from its start to the last byte written. `template`
/// excludes the NUL.
pub fn numbered_path(template: &[u8], number: u32) -> Result<Vec<u8>, Refused> {
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

/// Writes the buffer and returns the path; `template` and the path are without NULs.
pub fn numbered_path_bytes(template: &[u8], number: u32) -> Result<Vec<u8>, Refused> {
    let path = numbered_path(template, number)?;
    crate::files::state::PATHS.write_at(0, &path, "the numbered path's buffer")?;
    Ok(path[..path.iter().position(|&b| b == 0).unwrap_or(path.len())].to_vec())
}
