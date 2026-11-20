//! The folders of a path made, segment by segment: a drive path first changes to the drive's root,
//! a UNC path skips the server and share; then each `\`-separated segment is entered
//! (`files::set_current_directory`) and, where that fails, made (`files::make_directory`). A failed
//! change or make ends it. A path without its closing `\` has its last folder made at its NUL.
//! Refuses a scan of 0x400 bytes from the path's start and a UNC path without its share's `\`.

use crate::errors::{BadInput, Refused};
use crate::files::directories::{make_directory, set_current_directory};

const LIMIT: usize = 0x400;

/// A path that does not end in `\\`: the walk looks for the next one past its end.
const PAST_END: BadInput =
    BadInput("a path that does not end in \\ (the original reads past its end)");

/// The folders of `path` (its text, no NUL) gone into, made where missing; whether it worked.
pub fn make_folders(path: &[u8]) -> Result<bool, Refused> {
    use crate::install::folders::{Folders, Native};
    <Native as Folders>::make(path)
}

/// The Windows walk (`install::folders`' rules) over the path's text; `at` the byte the walk is
/// on.
#[cfg_attr(not(windows), allow(dead_code))] // reason: the Windows rules' and the tests'
pub fn walk(path: &[u8]) -> Result<bool, Refused> {
    // The text ends at its NUL: every byte past it reads as 0.
    let byte = |n: usize| path.get(n).copied().unwrap_or(0);
    // The path cut before byte `n`, as the folder calls take it.
    let cut_at = |n: usize| &path[..n.min(path.len())];
    let mut at = 0usize;
    if byte(1) == b':' {
        at += 2;
        if !set_current_directory(cut_at(at + 1)) {
            return Ok(false);
        }
    } else if byte(0) == b'\\' && byte(1) == b'\\' {
        at += 1;
        for _ in 0..2 {
            loop {
                at += 1;
                match byte(at) {
                    b'\\' => break,
                    0 => return Err(PAST_END.into()),
                    _ => {}
                }
            }
        }
        if !set_current_directory(cut_at(at)) {
            return Ok(false);
        }
    }
    // Bounded: 0x400 bytes from the path's start.
    while byte(at + 1) != 0 {
        loop {
            at += 1;
            if at > LIMIT {
                return Err(BadInput("a path with no end in 0x400 bytes").into());
            }
            match byte(at) {
                b'\\' => break,
                // The path's end without its closing `\`: the last folder is gone into or made
                // at the end, without reading past the path's own buffer.
                0 => {
                    return Ok(set_current_directory(path)
                        || (make_directory(path) && set_current_directory(path)));
                }
                _ => {}
            }
        }
        // Each call with the path cut before the `\\`.
        let folder = cut_at(at);
        if set_current_directory(folder) {
            continue;
        }
        if !make_directory(folder) || !set_current_directory(folder) {
            return Ok(false);
        }
    }
    Ok(true)
}
