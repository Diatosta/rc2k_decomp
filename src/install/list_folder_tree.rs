//! `install::list_folder_tree` (0x41371C): a folder and everything under it added to the
//! tables, breadth first. The skip mask (`FindFileData+0x13E`) is set to 0 for the walk
//! (and put back at the end). The folder is resolved; if not empty it goes into the
//! folders table (`dir_cursor` its end). Then, starting with that folder and working through every
//! folder recorded from `dir_scan` to `dir_cursor`, each is searched (`<folder>\*.*`
//! built in the numbered paths' buffer): a sub-folder (not `.`/`..`) is appended to the folders table
//! behind its parent, a file to the files table (`file_cursor` its end); the folders table is the
//! queue. Overflow of either table is fatal error 0x80AF2201 or 0x80AF2202.
//!
//! Takes the folder.
//! Caller `install::run_script` (opcode 4).

use crate::errors::Refused;
use crate::install::installer::{
    close_search, fatal, find_first, find_next, resolve_path, text_at,
};
use crate::install::name_tables::{FILES, FOLDERS, found_name};

/// FindFileData.cFileName (the search record: `files::search`), up to its NUL.
fn find_name() -> Vec<u8> {
    crate::files::search::found_name()
}
/// FindFileData.dwFileAttributes.
fn find_attributes() -> u32 {
    crate::files::search::found_attributes()
}
const DIRS_ROOM: u32 = 0x10000;
const FILES_ROOM: u32 = 0x40000;

fn same_name(a1: impl Into<crate::place::BytePlace>, name: &[u8]) -> bool {
    let mut b = name.to_vec();
    b.push(0);
    crate::menu::event_flow::refuse_here(
        a1.into()
            .bytes("a name")
            .and_then(|a| crate::strings::same_text::same_text(&a, &b)),
        "install::list_folder_tree: a name",
    )
}

/// The found file's path (the searched folder's) and name.
fn found_path(search: &[u8]) -> Result<Vec<u8>, Refused> {
    found_name(search, &find_name())
}

fn walk(
    install: &'static crate::install::owner::Install,
    folder: crate::place::BytePlace,
) -> Result<(), Refused> {
    let mut folder = resolve_path(folder)?;
    if !folder.is_empty() {
        let (used, scan) = FOLDERS.with(|t| (t.push(folder.clone()), t.names.len() as u32))?;
        install.run.dir_cursor.set(used);
        install.run.dir_scan.set(scan);
        install
            .run
            .dir_count
            .set(install.run.dir_count.get().wrapping_add(1));
    }
    loop {
        // list the folder
        let mut search_text = folder.clone();
        // The original tests the byte before its cursor; for an empty folder that is the
        // byte before the search buffer (not the path's): here an empty folder gets its `\\`.
        if folder.last() != Some(&b'\\') {
            search_text.push(b'\\');
        }
        search_text.extend(text_at(&install.texts.star));
        // The search path, kept in the search buffer with its NUL (the numbered paths read
        // it later).
        let mut kept = search_text.clone();
        kept.push(0);
        // The numbered paths' buffer.
        crate::files::state::PATHS.write_at(0, &kept, "the search path")?;
        if find_first(&search_text) {
            loop {
                if find_attributes() & 0x10 != 0 {
                    // "." and ".." passed over
                    let name = find_name();
                    let real = !same_name(&install.texts.dot, &name)
                        && !same_name(&install.texts.dotdot, &name);
                    if real {
                        let name = found_path(&search_text)?;
                        let used = FOLDERS.with(|t| t.push(name))?;
                        install.run.dir_cursor.set(used);
                        install
                            .run
                            .dir_count
                            .set(install.run.dir_count.get().wrapping_add(1));
                        // The room used in the folders table.
                        if used >= DIRS_ROOM {
                            return Err(fatal(0x80AF_2201));
                        }
                    }
                } else {
                    let name = found_path(&search_text)?;
                    let used = FILES.with(|t| t.push(name))?;
                    install.run.file_cursor.set(used);
                    install
                        .run
                        .file_count
                        .set(install.run.file_count.get().wrapping_add(1));
                    // The room used in the files table.
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
        let scan = install.run.dir_scan.get();
        let Some(next) = FOLDERS.with(|t| t.names.get(scan as usize).cloned())? else {
            break;
        };
        if next.is_empty() {
            break;
        }
        folder = next;
        install.run.dir_scan.set(scan.wrapping_add(1));
    }
    Ok(())
}

/// `folder` and every folder under it added to the folders' table, their files to
/// the files'.
pub fn list_folder_tree(
    install: &'static crate::install::owner::Install,
    folder: impl Into<crate::place::BytePlace>,
) -> Result<(), Refused> {
    let folder = folder.into();
    use crate::files::state::SKIPPED_ATTRIBUTES;
    let saved_skip = SKIPPED_ATTRIBUTES.get();
    SKIPPED_ATTRIBUTES.set(0);
    let result = walk(install, folder);
    SKIPPED_ATTRIBUTES.set(saved_skip);
    result
}

#[cfg(test)]
mod tests {
    use crate::install::name_tables::{FILES, FOLDERS, NameTable};

    #[test]
    fn a_tree_is_listed_into_the_tables() {
        let _lock = crate::test_support::test_lock();
        let figures = crate::memory::figures();
        let kept: Vec<u32> = [
            &crate::install::owner::install().run.dir_count,
            &crate::install::owner::install().run.file_count,
            &crate::install::owner::install().run.dir_scan,
            &crate::install::owner::install().run.dir_cursor,
            &crate::install::owner::install().run.file_cursor,
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
        let memory = crate::memory::ListCells::empty();
        let list = &memory;
        FOLDERS.take(list, 0x10100, NameTable::default()).unwrap();
        FILES.take(list, 0x40100, NameTable::default()).unwrap();
        for d in [
            &crate::install::owner::install().run.dir_count,
            &crate::install::owner::install().run.file_count,
            &crate::install::owner::install().run.dir_scan,
            &crate::install::owner::install().run.dir_cursor,
            &crate::install::owner::install().run.file_cursor,
        ] {
            d.set(0);
        }
        let path_buffer: &'static crate::session::Bytes<1024> =
            Box::leak(Box::new(crate::session::Bytes::new([0; 1024])));
        for (n, &b) in path.iter().enumerate() {
            path_buffer.set(n, b);
        }
        super::list_folder_tree(crate::install::owner::install(), path_buffer).unwrap();
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
        assert_eq!(crate::install::owner::install().run.dir_count.get(), 3);
        assert_eq!(crate::install::owner::install().run.file_count.get(), 3);
        assert_eq!(
            crate::install::owner::install().run.dir_cursor.get(),
            folder_bytes
        );
        FOLDERS.give_back();
        FILES.give_back();
        crate::memory::with_figures(|f| *f = figures);
        for (d, v) in [
            &crate::install::owner::install().run.dir_count,
            &crate::install::owner::install().run.file_count,
            &crate::install::owner::install().run.dir_scan,
            &crate::install::owner::install().run.dir_cursor,
            &crate::install::owner::install().run.file_cursor,
        ]
        .iter()
        .zip(kept)
        {
            d.set(v);
        }
        std::fs::remove_dir_all(dir).unwrap();
    }
}
