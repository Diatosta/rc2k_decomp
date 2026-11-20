//! The install script (a byte program) run to build the folder- and file-name tables. A name is
//! assembled in `script_path` as nested folders. Opcodes:
//! - 1 n: read a name, add the whole path as a folder, append `\`, and push the level;
//! - 2: pop a level (refused with nothing pushed);
//! - 3 n: read a name and add the matching files, path unchanged;
//! - 4 n: read a name and add the folder tree;
//! - 8 n: if the media mode is below n, skip to the matching 9 (they nest);
//! - 9, 6: nothing; any other byte (0): the end. An unbalanced stack is fatal error 0x80AF3011.

use crate::errors::{BadInput, Refused};
use crate::install::installer::{fatal, media_mode, rb, wb};
use crate::memory::place::BytePlace;
use rc2k_core::install::script::{self, Op};

/// The script at `script` (a place in the installer's names or variables) run.
pub(super) fn run_script(
    install: &'static crate::install::owner::Install,
    script: BytePlace,
) -> Result<(), Refused> {
    use crate::install as i;
    // Bounded by the buffer the script is in: the uninstall script (`names.script`) holds the
    // mark and the rest of the script after its first 17 bytes (the end at the mark, 5, or at
    // the end of the buffer); the other scripts end inside their own fields.
    let mut cursor = script;
    let mut end = BytePlace::from(&install.paths.script_path);
    wb(end, 0);
    let mut stack: Vec<BytePlace> = Vec::new();
    let media = media_mode() as u8;
    let mut guard = 0usize;
    loop {
        guard += 1;
        if guard > script::LIMIT {
            return Err(BadInput("a script with no end in 0x10000 bytes").into());
        }
        let op = rb(cursor);
        cursor = cursor.add(1);
        match Op::of(op) {
            Op::Folder => {
                stack.push(end);
                (cursor, end) = i::script_name::script_name(cursor, end)?;
                i::list_folder::list_folder(install, &install.paths.script_path)?;
                wb(end, b'\\');
                wb(end.add(1), 0);
                end = end.add(1);
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
                    i::list_files::list_files(install, &install.paths.script_path)?;
                } else {
                    i::list_folder_tree::list_folder_tree(install, &install.paths.script_path)?;
                }
                end = stack
                    .pop()
                    .unwrap_or(BytePlace::from(&install.paths.script_path)); // pushed just above
                wb(end, 0);
            }
            Op::Media => {
                let n = rb(cursor);
                cursor = cursor.add(1);
                if !script::skipped(n, media) {
                    continue;
                }
                let words = cursor.bytes("the installer's script")?;
                let past = script::skip(&words, 0, &mut guard)?;
                cursor = cursor.add(past as u32);
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
