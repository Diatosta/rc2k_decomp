//! `install::make_folders` (0x413A97): the folders of a path made, segment by segment. A
//! drive path (`X:`) first changes to the drive's root; a UNC path (`\\`) skips the server
//! and share. Then for each `\`-separated segment the path is cut there and changed into
//! (`files::set_current_directory`); where that fails the folder is made
//! (`files::make_directory`) and entered. The path's bytes are cut with a NUL and put back
//! as the walk goes. A failed change or make ends it.
//!
//! In:  edx the path (normalised: ending in `\\`)
//! Out: every register kept (`pusha` / `popa`); carry set when a folder could not be gone
//!      into or made (the failing call's flags), else zero set and carry clear
//! A path without its closing `\\` (as `install_files` gives every folder) is scanned on
//! past its NUL to the next `\\`, as the original does. Refuses (as met) a scan of 0x400
//! bytes from the path's start, and a UNC path without its share's `\\`.
//! Callers `install::install_files`, `install::install_clio`.
//!
//! Conversion phase 3 (2026-10-23): plain Rust with typed arguments; the registers above
//! are the original's.

use crate::errors::{BadInput, Refused};
use crate::install::installer::{make_dir, rb, rw, set_cd, wb};

const LIMIT: u32 = 0x400;

/// A path that does not end in `\\`: the original looks for the next one past its end.
const PAST_END: BadInput =
    BadInput("a path that does not end in \\ (the original reads past its end)");

/// The folders of `path` (edx) gone into, made where missing; whether it worked (the
/// original's carry clear).
pub fn make_folders(path: impl Into<crate::place::Place>) -> Result<bool, Refused> {
    let path: u32 = path.into().game();
    use crate::install::folders::{Folders, Native};
    <Native as Folders>::make(path)
}

/// The original's walk (`install::folders`' Windows rules).
#[cfg_attr(not(windows), allow(dead_code))] // reason: the Windows rules' and the tests'
pub fn walk(path: u32) -> Result<bool, Refused> {
    let mut at = path;
    let start = rw(at);
    if (start >> 8) as u8 == b':' {
        at += 2;
        // xor cl, cl; xchg cl, [esi+1]
        let kept = rb(at + 1);
        wb(at + 1, 0);
        let done = set_cd(path);
        wb(at + 1, kept);
        if !done {
            return Ok(false);
        }
    } else if start == 0x5C5C {
        at += 1;
        for _ in 0..2 {
            loop {
                at += 1;
                match rb(at) {
                    b'\\' => break,
                    0 => return Err(PAST_END.into()),
                    _ => {}
                }
            }
        }
        wb(at, 0);
        let done = set_cd(path);
        wb(at, b'\\');
        if !done {
            return Ok(false);
        }
    }
    // loc_413AD6. A path without its closing `\\` (the installer's folders, as the script
    // lists them: `install_files`) has the scan go on past its NUL to the next `\\` in what
    // follows, as the original's does: each further segment only goes into the same path
    // again (`notes/bugs.md`). Bounded: 0x400 bytes from the path's start.
    while rb(at + 1) != 0 {
        loop {
            at += 1;
            if at.wrapping_sub(path) > LIMIT {
                return Err(BadInput("a path with no end in 0x400 bytes").into());
            }
            if rb(at) == b'\\' {
                break;
            }
        }
        // Each call with the path cut at the `\\`, put back after it.
        let cut = |call: fn(u32) -> bool| {
            wb(at, 0);
            let done = call(path);
            wb(at, b'\\');
            done
        };
        if cut(set_cd) {
            continue;
        }
        if !cut(make_dir) || !cut(set_cd) {
            return Ok(false);
        }
    }
    Ok(true)
}
