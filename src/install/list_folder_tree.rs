//! `install::list_folder_tree` (0x41371C): a folder and everything under it added to the
//! tables, breadth first. The skip mask (`FindFileData+0x13E`) is set to 0 for the walk
//! (and put back at the end). The folder (eax) is resolved; if not empty it goes into the
//! folders table (`dword_51BB54`). Then, starting with that folder and working through every
//! folder recorded from `dword_51BB50` to `dword_51BB54`, each is searched (`<folder>\*.*`
//! built in `byte_94E8B0`): a sub-folder (not `.`/`..`) is appended to the folders table
//! behind its parent, a file to the files table (`dword_51BB58`); the folders table is the
//! queue. Overflow of either table is fatal error 0x80AF2201 or 0x80AF2202.
//!
//! In:  eax the folder
//! Out: every register kept (`pusha` / `popa`)
//! Caller `install::run_script` (opcode 4).
//!
//! Conversion phase 3 (2026-10-23): plain Rust with typed arguments; the registers above
//! are the original's.

use crate::errors::Refused;
use crate::install::installer::{
    append_byte, close_search, fatal, find_first, find_next, rb, resolve_path, str_append,
};
use crate::install::name_tables::{FILES, FOLDERS, made_name, text_at};
use crate::install::vars::INSTALLER_RUN;

/// FindFileData.cFileName (the search record: `files::search`).
fn find_name() -> u32 {
    crate::files::search::found_name_place()
}
/// FindFileData.dwFileAttributes.
fn find_attributes() -> u32 {
    crate::files::search::found_attributes()
}
const DIRS_ROOM: u32 = 0x10000;
const FILES_ROOM: u32 = 0x40000;

fn same_name(a1: impl Into<crate::place::Place>, a2: impl Into<crate::place::Place>) -> bool {
    let a1: u32 = a1.into().game();
    let a2: u32 = a2.into().game();
    crate::menu::event_flow::refuse_here(
        crate::strings::same_text::same_text(
            crate::pointers::GamePtr::new(a1),
            crate::pointers::GamePtr::new(a2),
        ),
        "install::list_folder_tree: a name",
    )
}

/// The found file's path (the searched folder's) and name.
fn found_path(search: u32) -> Result<Vec<u8>, Refused> {
    made_name(|at| {
        let (cursor, _) = crate::files::append_directory::append_directory(
            crate::pointers::GamePtr::new(search),
            crate::pointers::GamePtr::new(at),
        )?;
        Ok(append_byte(str_append(find_name(), cursor.addr()), 0).game())
    })
}

fn walk(folder: u32) -> Result<(), Refused> {
    let folder = resolve_path(folder)?;
    let mut folder = text_at(folder)?;
    if !folder.is_empty() {
        let (used, scan) = FOLDERS.with(|t| (t.push(folder.clone()), t.names.len() as u32))?;
        INSTALLER_RUN.dir_cursor.set(used);
        INSTALLER_RUN.dir_scan.set(scan);
        INSTALLER_RUN
            .dir_count
            .set(INSTALLER_RUN.dir_count.get().wrapping_add(1));
    }
    loop {
        // list the folder
        let search = crate::files::numbered_path::buffer(); // byte_94E8B0
        search.write_bytes(&folder, "the search path")?;
        let base = search.addr();
        let mut cursor = base.wrapping_add(folder.len() as u32);
        if rb(cursor.wrapping_sub(1)) != b'\\' {
            cursor = append_byte(cursor, b'\\').game();
        }
        append_byte(
            str_append(&crate::install::installer::INSTALL_TEXTS.star, cursor),
            0,
        );
        if find_first(base) {
            loop {
                if find_attributes() & 0x10 != 0 {
                    // "." and ".." passed over
                    let real =
                        !same_name(&crate::install::installer::INSTALL_TEXTS.dot, find_name())
                            && !same_name(
                                &crate::install::installer::INSTALL_TEXTS.dotdot,
                                find_name(),
                            );
                    if real {
                        let name = found_path(base)?;
                        let used = FOLDERS.with(|t| t.push(name))?;
                        INSTALLER_RUN.dir_cursor.set(used);
                        INSTALLER_RUN
                            .dir_count
                            .set(INSTALLER_RUN.dir_count.get().wrapping_add(1));
                        // `sub ebx, g_dirNames`
                        if used >= DIRS_ROOM {
                            return Err(fatal(0x80AF_2201));
                        }
                    }
                } else {
                    let name = found_path(base)?;
                    let used = FILES.with(|t| t.push(name))?;
                    INSTALLER_RUN.file_cursor.set(used);
                    INSTALLER_RUN
                        .file_count
                        .set(INSTALLER_RUN.file_count.get().wrapping_add(1));
                    // `sub ebx, g_fileNames`
                    if used >= FILES_ROOM {
                        return Err(fatal(0x80AF_2202));
                    }
                }
                if !find_next() {
                    break;
                }
            }
        }
        close_search();
        // the next folder from the scan (its number in the folders' table)
        let scan = INSTALLER_RUN.dir_scan.get();
        let Some(next) = FOLDERS.with(|t| t.names.get(scan as usize).cloned())? else {
            break;
        };
        if next.is_empty() {
            break;
        }
        folder = next;
        INSTALLER_RUN.dir_scan.set(scan.wrapping_add(1));
    }
    Ok(())
}

/// `folder` (eax) and every folder under it added to the folders' table, their files to
/// the files'.
pub fn list_folder_tree(folder: impl Into<crate::place::Place>) -> Result<(), Refused> {
    let folder: u32 = folder.into().game();
    use crate::files::state::SKIPPED_ATTRIBUTES;
    let saved_skip = SKIPPED_ATTRIBUTES.load(std::sync::atomic::Ordering::Relaxed);
    SKIPPED_ATTRIBUTES.store(0, std::sync::atomic::Ordering::Relaxed);
    let result = walk(folder);
    SKIPPED_ATTRIBUTES.store(saved_skip, std::sync::atomic::Ordering::Relaxed);
    result
}

#[cfg(test)]
mod tests {
    use crate::install::name_tables::{FILES, FOLDERS, NameTable};
    use crate::install::vars::INSTALLER_RUN;
    use crate::memory::MemoryList;

    #[test]
    fn a_tree_is_listed_into_the_tables() {
        let _lock = crate::store::test_lock();
        let figures = crate::memory::figures();
        let kept: Vec<u32> = [
            &INSTALLER_RUN.dir_count,
            &INSTALLER_RUN.file_count,
            &INSTALLER_RUN.dir_scan,
            &INSTALLER_RUN.dir_cursor,
            &INSTALLER_RUN.file_cursor,
        ]
        .iter()
        .map(|d| d.get())
        .collect();
        let dir = std::env::temp_dir().join(format!("rc2k_tree_{}", std::process::id()));
        std::fs::create_dir_all(dir.join("a").join("b")).unwrap();
        for f in ["z.txt", "a/x.txt", "a/b/y.txt"] {
            std::fs::write(dir.join(f), b"x").unwrap();
        }
        let root = crate::files::ansi_bytes(&dir.to_string_lossy());
        let mut path = root.clone();
        path.push(0);
        let memory = vec![0u8; size_of::<MemoryList>()];
        let list_region = crate::pointers::Lent::new(&memory);
        let list = crate::pointers::GamePtr::new(crate::pointers::game_address(memory.as_ptr()));
        FOLDERS.take(list, 0x10100, NameTable::default()).unwrap();
        FILES.take(list, 0x40100, NameTable::default()).unwrap();
        for d in [
            &INSTALLER_RUN.dir_count,
            &INSTALLER_RUN.file_count,
            &INSTALLER_RUN.dir_scan,
            &INSTALLER_RUN.dir_cursor,
            &INSTALLER_RUN.file_cursor,
        ] {
            d.set(0);
        }
        let path_region = crate::pointers::Lent::new(&path);
        super::list_folder_tree(path_region.addr()).unwrap();
        drop(path_region);
        let joined = |parts: &[&str]| {
            let mut p = root.clone();
            for part in parts {
                p.push(b'\\');
                p.extend(part.as_bytes());
            }
            p
        };
        let (folders, folder_bytes) = FOLDERS.with(|t| (t.names.clone(), t.bytes)).unwrap();
        let (mut files, file_bytes) = FILES.with(|t| (t.names.clone(), t.bytes)).unwrap();
        files.sort();
        assert_eq!(folders, [root.clone(), joined(&["a"]), joined(&["a", "b"])]);
        let mut want = vec![
            joined(&["z.txt"]),
            joined(&["a", "x.txt"]),
            joined(&["a", "b", "y.txt"]),
        ];
        want.sort();
        assert_eq!(files, want);
        let bytes = |names: &[Vec<u8>]| names.iter().map(|n| n.len() as u32 + 1).sum::<u32>();
        assert_eq!((folder_bytes, file_bytes), (bytes(&folders), bytes(&files)));
        assert_eq!(INSTALLER_RUN.dir_count.get(), 3);
        assert_eq!(INSTALLER_RUN.file_count.get(), 3);
        assert_eq!(INSTALLER_RUN.dir_cursor.get(), folder_bytes);
        FOLDERS.give_back();
        FILES.give_back();
        drop(list_region);
        crate::memory::with_figures(|f| *f = figures);
        for (d, v) in [
            &INSTALLER_RUN.dir_count,
            &INSTALLER_RUN.file_count,
            &INSTALLER_RUN.dir_scan,
            &INSTALLER_RUN.dir_cursor,
            &INSTALLER_RUN.file_cursor,
        ]
        .iter()
        .zip(kept)
        {
            d.set(v);
        }
        std::fs::remove_dir_all(dir).unwrap();
    }
}
