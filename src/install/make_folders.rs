//! The folders of a path made, segment by segment: a drive path first changes to the drive's root,
//! a UNC path skips the server and share; then each `\`-separated segment is entered
//! (`files::set_current_directory`) and, where that fails, made (`files::make_directory`). A failed
//! change or make ends it. A path without its closing `\` has its last folder made at its NUL.
//! Refuses a scan of 0x400 bytes from the path's start and a UNC path without its share's `\`.

use crate::errors::{BadInput, Refused};
use crate::install::installer::{make_dir, rb, rw, set_cd, wb};
use crate::memory::place::BytePlace;

const LIMIT: u32 = 0x400;

/// A path that does not end in `\\`: the walk looks for the next one past its end.
const PAST_END: BadInput =
    BadInput("a path that does not end in \\ (the original reads past its end)");

/// The folders of `path` gone into, made where missing; whether it worked.
pub fn make_folders(path: impl Into<BytePlace>) -> Result<bool, Refused> {
    use crate::install::folders::{Folders, Native};
    <Native as Folders>::make(path.into())
}

/// The Windows walk (`install::folders`' rules); `at` the byte the walk is on, as an offset
/// from the path's start.
#[cfg_attr(not(windows), allow(dead_code))] // reason: the Windows rules' and the tests'
pub fn walk(path: BytePlace) -> Result<bool, Refused> {
    let byte = |n: u32| rb(path.add(n));
    let put = |n: u32, b: u8| wb(path.add(n), b);
    let mut at = 0u32;
    let start = rw(path);
    if (start >> 8) as u8 == b':' {
        at += 2;
        // The byte after is taken out and a NUL put in its place.
        let kept = byte(at + 1);
        put(at + 1, 0);
        let done = set_cd(path);
        put(at + 1, kept);
        if !done {
            return Ok(false);
        }
    } else if start == 0x5C5C {
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
        put(at, 0);
        let done = set_cd(path);
        put(at, b'\\');
        if !done {
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
                // at the NUL, without reading past the path's own buffer.
                0 => return Ok(set_cd(path) || (make_dir(path) && set_cd(path))),
                _ => {}
            }
        }
        // Each call with the path cut at the `\\`, put back after it.
        let cut = |call: fn(BytePlace) -> bool| {
            put(at, 0);
            let done = call(path);
            put(at, b'\\');
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
