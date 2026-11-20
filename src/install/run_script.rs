//! The install script (a byte program) run to build the folder- and file-name tables. A name is
//! assembled in `script_path` as nested folders. Opcodes:
//! - 1 n: read a name, add the whole path as a folder, append `\`, and push the level;
//! - 2: pop a level (refused with nothing pushed);
//! - 3 n: read a name and add the matching files, path unchanged;
//! - 4 n: read a name and add the folder tree;
//! - 8 n: if the installation kind (always the full one, 3) is below n, skip to the matching 9
//!   (they nest);
//! - 9, 6: nothing; any other byte (0): the end. An unbalanced stack is fatal error 0x80AF3011.

use crate::errors::{BadInput, Refused};
use crate::install::installer::{FULL_INSTALLATION, fatal};
use rc2k_core::install::script::{self, Op};

/// The script bytes run to build the name tables; `script` is the script's whole buffer.
pub(super) fn run_script(
    install: &'static crate::install::owner::Install,
    script: &[u8],
) -> Result<(), Refused> {
    use crate::install as i;
    // Bounded by the buffer the script is in: the uninstall script (`names.script`) holds the
    // mark and the rest of the script after its first 17 bytes (the end at the mark, 5, or at
    // the end of the buffer); the other scripts end inside their own fields.
    let mut cursor = 0usize;
    let byte = |at: usize| {
        script
            .get(at)
            .copied()
            .ok_or(BadInput("a script read past its buffer"))
    };
    // The path being built, kept in `script_path` as its text for the scans that read it.
    let mut path: Vec<u8> = Vec::new();
    let show = |path: &[u8]| {
        let mut text = path.to_vec();
        text.push(0);
        install
            .paths
            .script_path
            .write_at(0, &text, "a script path")
    };
    show(&path)?;
    // The path's length before each folder entered.
    let mut levels: Vec<usize> = Vec::new();
    // Every installation is a full one: no part of a script is left out.
    let media = FULL_INSTALLATION as u8;
    let mut guard = 0usize;
    loop {
        guard += 1;
        if guard > script::LIMIT {
            return Err(BadInput("a script with no end in 0x10000 bytes").into());
        }
        let op = byte(cursor)?;
        cursor += 1;
        match Op::of(op) {
            Op::Folder => {
                let (name, _) = script::script_name(&script[cursor.min(script.len())..])?;
                cursor += name.len();
                levels.push(path.len());
                path.extend_from_slice(&name);
                show(&path)?;
                i::list_folder::list_folder(install, &install.paths.script_path)?;
                path.push(b'\\');
                show(&path)?;
            }
            Op::Up => {
                let Some(level) = levels.pop() else {
                    return Err(BadInput(
                        "a pop with nothing pushed (the original pops its caller's stack)",
                    )
                    .into());
                };
                path.truncate(level);
                show(&path)?;
            }
            Op::Files | Op::Tree => {
                let (name, _) = script::script_name(&script[cursor.min(script.len())..])?;
                cursor += name.len();
                let level = path.len();
                path.extend_from_slice(&name);
                show(&path)?;
                if op == 3 {
                    i::list_files::list_files(install, &install.paths.script_path)?;
                } else {
                    i::list_folder_tree::list_folder_tree(install, &install.paths.script_path)?;
                }
                path.truncate(level);
                show(&path)?;
            }
            Op::Media => {
                let n = byte(cursor)?;
                cursor += 1;
                if !script::skipped(n, media) {
                    continue;
                }
                let rest = &script[cursor.min(script.len())..];
                cursor += script::skip(rest, 0, &mut guard)?;
            }
            Op::Nothing => {}
            Op::End => {
                if !levels.is_empty() {
                    return Err(fatal(script::UNBALANCED));
                }
                break;
            }
        }
    }
    Ok(())
}
