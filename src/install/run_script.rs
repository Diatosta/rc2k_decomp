//! `install::run_script` (0x41365D): the install script (a byte program at ebx) run to
//! build the folder- and file-name tables. A name is assembled in `byte_95EAF8` as nested
//! folders, with a stack of the segment starts (the real stack in the original). Opcodes:
//! - 1 n: read a name (`install::script_name`), add the whole path as a folder
//!   (`install::list_folder`), append `\`, and push the level;
//! - 2: pop a level (back to the parent; refused with nothing pushed: the original pops
//!   its caller's stack);
//! - 3 n: read a name and add the matching files (`install::list_files`), path unchanged;
//! - 4 n: read a name and add the folder tree (`install::list_folder_tree`);
//! - 8 n: if `g_mediaMode` is below n, skip to the matching 9 (they nest);
//! - 9, 6: nothing;
//! - any other byte (0): the end. An unbalanced stack is fatal error 0x80AF3011.
//!
//! In:  ebx the script
//! Out: every register kept (`pusha` / `popa`)
//! Callers `install::install_files`, `uninstall`, `install_clio`. Its three table calls
//! through `part`; `script_name` is called directly (it only builds the path). Shadow mode
//! runs the original alone. Self-test: a copy whose three table calls go to stand-ins (the
//! path noted), `script_name` kept, against the Rust version; scripts of every opcode, a
//! media-skip and an unbalanced stack (fatal).
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::bridge::Raw;
use crate::errors::{BadInput, Refused};
use crate::framework::Regs;
use crate::install::installer::{
    LIST_FILES, LIST_FOLDER, LIST_FOLDER_TREE, MEDIA_MODE, Part, a, at, fatal, gb, rb, wb,
};
use rc2k_core::install::script::{self, Op};

const NAME_BUF: usize = 0x95EAF8; // byte_95EAF8

pub fn run_script(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut r = *entry;
    let mut esi = a(NAME_BUF);
    wb(esi, 0);
    let mut stack: Vec<u32> = Vec::new();
    let media = gb(MEDIA_MODE);
    let mut guard = 0usize;
    // Converted: `rc2k_core::install::script` (`Op`, `skipped`, `skip`).
    loop {
        guard += 1;
        if guard > script::LIMIT {
            return Err(BadInput("a script with no end in 0x10000 bytes").into());
        }
        let al = rb(r.ebx);
        r.ebx = r.ebx.wrapping_add(1);
        match Op::of(al) {
            Op::Folder => {
                stack.push(esi);
                r.esi = esi;
                r = crate::install::script_name::script_name(&r)?;
                esi = r.esi;
                r.eax = a(NAME_BUF);
                part(at(LIST_FOLDER), &mut r)?;
                wb(esi, b'\\');
                wb(esi + 1, 0);
                esi += 1;
            }
            Op::Up => {
                let Some(level) = stack.pop() else {
                    return Err(BadInput(
                        "a pop with nothing pushed (the original pops its caller's stack)",
                    )
                    .into());
                };
                esi = level;
                wb(esi, 0);
            }
            Op::Files | Op::Tree => {
                stack.push(esi);
                r.esi = esi;
                r = crate::install::script_name::script_name(&r)?;
                r.eax = a(NAME_BUF);
                part(
                    at(if al == 3 {
                        LIST_FILES
                    } else {
                        LIST_FOLDER_TREE
                    }),
                    &mut r,
                )?;
                esi = stack.pop().unwrap_or(a(NAME_BUF)); // pushed just above
                wb(esi, 0);
            }
            Op::Media => {
                let n = rb(r.ebx);
                r.ebx = r.ebx.wrapping_add(1);
                if !script::skipped(n, media) {
                    continue;
                }
                let words = unsafe { Raw::new(r.ebx as *const u8) };
                let past = script::skip(&words, 0, &mut guard)?;
                r.ebx = r.ebx.wrapping_add(past as u32);
            }
            Op::Nothing => {}
            Op::End => {
                if !stack.is_empty() {
                    return Err(fatal(script::UNBALANCED));
                }
                break;
            }
        }
    }
    Ok(Regs { ..*entry })
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    crate::race::viewer::regs_entry(r, &|r| {
        run_script(r, &mut crate::install::installer::game_part)
    })
}
