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
//! Callers `install::install_files`, `uninstall`, `install_clio`.
//!
//! Conversion phase 3 (2026-10-23): plain Rust with typed arguments; the registers above
//! are the original's.

use crate::errors::{BadInput, Refused};
use crate::install::installer::{fatal, media_mode, rb, wb};
use crate::install::vars::INSTALLER_PATHS;
use rc2k_core::install::script::{self, Op};

/// The script at `script` (ebx) run.
pub fn run_script(script: u32) -> Result<(), Refused> {
    use crate::install as i;
    let mut cursor = script;
    let mut end = INSTALLER_PATHS.script_path.addr();
    wb(end, 0);
    let mut stack: Vec<u32> = Vec::new();
    let media = media_mode() as u8;
    let mut guard = 0usize;
    // Converted: `rc2k_core::install::script` (`Op`, `skipped`, `skip`).
    loop {
        guard += 1;
        if guard > script::LIMIT {
            return Err(BadInput("a script with no end in 0x10000 bytes").into());
        }
        let op = rb(cursor);
        cursor = cursor.wrapping_add(1);
        match Op::of(op) {
            Op::Folder => {
                stack.push(end);
                (cursor, end) = i::script_name::script_name(cursor, end)?;
                i::list_folder::list_folder(INSTALLER_PATHS.script_path.addr())?;
                wb(end, b'\\');
                wb(end + 1, 0);
                end += 1;
            }
            Op::Up => {
                let Some(level) = stack.pop() else {
                    return Err(BadInput(
                        "a pop with nothing pushed (the original pops its caller's stack)",
                    )
                    .into());
                };
                end = level;
                wb(end, 0);
            }
            Op::Files | Op::Tree => {
                stack.push(end);
                (cursor, _) = i::script_name::script_name(cursor, end)?;
                if op == 3 {
                    i::list_files::list_files(INSTALLER_PATHS.script_path.addr())?;
                } else {
                    i::list_folder_tree::list_folder_tree(INSTALLER_PATHS.script_path.addr())?;
                }
                end = stack.pop().unwrap_or(INSTALLER_PATHS.script_path.addr()); // pushed just above
                wb(end, 0);
            }
            Op::Media => {
                let n = rb(cursor);
                cursor = cursor.wrapping_add(1);
                if !script::skipped(n, media) {
                    continue;
                }
                let words =
                    crate::pointers::GamePtr::new(cursor).bytes("the installer's script")?;
                let past = script::skip(&words, 0, &mut guard)?;
                cursor = cursor.wrapping_add(past as u32);
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
    Ok(())
}
