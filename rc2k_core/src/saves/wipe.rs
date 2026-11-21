//! Deleting what the player has made: everything under `savedata` (saved games, car set-ups,
//! replays, screenshots, videos) and everything in `var` except the installation's records and
//! the game's log. What is left alone is the program and the game's own data (`files`, the
//! language files, the CD's folders), which neither folder holds.

use std::ffi::OsString;
use std::path::{Path, PathBuf};

/// What a folder's entry is, links not followed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    /// A folder.
    Folder,
    /// A file, or a link (removed as a link).
    File,
    /// An entry that could not be read.
    Unreadable,
}

/// One entry of a folder.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Entry {
    /// The entry's name in its folder.
    pub name: OsString,
    /// What it is.
    pub kind: Kind,
}

/// Why a folder could not be listed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ListError {
    /// There is no such folder: nothing to remove.
    Missing,
    /// It is there and cannot be read.
    Unreadable,
}

/// The files the deletion walks, as the program's side gives them.
pub trait Tree {
    /// The entries of `dir`.
    fn list(&mut self, dir: &Path) -> Result<Vec<Entry>, ListError>;
    /// Removes a file or a link; false when it could not.
    fn remove_file(&mut self, path: &Path) -> bool;
    /// Removes an empty folder; false when it could not.
    fn remove_folder(&mut self, path: &Path) -> bool;
}

/// The files of `var` that stay: the installation's two records (without them the program
/// takes the folder for a download and runs the installer) and the log of the run in progress
/// (it is open, and it is not the player's).
pub(crate) const KEPT_IN_VAR: [&str; 3] = ["ral.zog", "install.cfg", "logfile.txt"];

/// What a deletion did.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Outcome {
    /// Files (and links) removed.
    pub removed: usize,
    /// Files or folders that could not be removed.
    pub failed: usize,
}

/// Whether `name` (a file of `var`) stays: compared without regard to case, as the game finds
/// its files.
pub(crate) fn is_kept(name: &str) -> bool {
    KEPT_IN_VAR.iter().any(|k| k.eq_ignore_ascii_case(name))
}

/// Everything under `savedata` and the files of `var` that are not kept removed. The folders
/// `savedata` and the ones directly in it stay (the program expects them); deeper folders go.
/// A missing folder is nothing to remove. A link is removed as a link, never followed.
pub fn delete_player_data(tree: &mut dyn Tree, var: &Path, savedata: &Path) -> Outcome {
    let mut out = Outcome::default();
    empty(tree, var, 0, &mut out, &|name| is_kept(name));
    empty(tree, savedata, 0, &mut out, &|_| false);
    out
}

/// The entries of `dir` removed except the files `keep` names (at the top only); folders
/// below the first level are removed too.
fn empty(
    tree: &mut dyn Tree,
    dir: &Path,
    depth: u32,
    out: &mut Outcome,
    keep: &dyn Fn(&str) -> bool,
) {
    let entries = match tree.list(dir) {
        Ok(entries) => entries,
        Err(ListError::Missing) => return,
        Err(ListError::Unreadable) => {
            out.failed += 1;
            return;
        }
    };
    for entry in entries {
        let path: PathBuf = dir.join(&entry.name);
        match entry.kind {
            Kind::Unreadable => out.failed += 1,
            Kind::Folder => {
                // The folders directly in `savedata` stay, emptied; a folder in `var` or deeper
                // is removed with what it holds.
                if depth == 0 && keep(&entry.name.to_string_lossy()) {
                    continue;
                }
                empty(tree, &path, depth + 1, out, &|_| false);
                if (depth >= 1
                    || dir
                        .file_name()
                        .is_some_and(|n| n.eq_ignore_ascii_case("var")))
                    && !tree.remove_folder(&path)
                {
                    out.failed += 1;
                }
            }
            // Kept.
            Kind::File if depth == 0 && keep(&entry.name.to_string_lossy()) => {}
            Kind::File => {
                if tree.remove_file(&path) {
                    out.removed += 1;
                } else {
                    out.failed += 1;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    /// A tree in memory: folders and files by path.
    #[derive(Default)]
    struct Memory {
        folders: BTreeSet<PathBuf>,
        files: BTreeSet<PathBuf>,
        links: BTreeSet<PathBuf>,
    }

    impl Memory {
        fn add(&mut self, path: &str, is_folder: bool) {
            let path = PathBuf::from(path);
            for parent in path.ancestors().skip(1) {
                if !parent.as_os_str().is_empty() {
                    self.folders.insert(parent.to_owned());
                }
            }
            if is_folder {
                self.folders.insert(path);
            } else {
                self.files.insert(path);
            }
        }
        fn has(&self, path: &str) -> bool {
            let path = Path::new(path);
            self.folders.contains(path) || self.files.contains(path) || self.links.contains(path)
        }
    }

    impl Tree for Memory {
        fn list(&mut self, dir: &Path) -> Result<Vec<Entry>, ListError> {
            if !self.folders.contains(dir) {
                return Err(ListError::Missing);
            }
            let mut entries = Vec::new();
            for (set, kind) in [
                (&self.folders, Kind::Folder),
                (&self.files, Kind::File),
                (&self.links, Kind::File),
            ] {
                for path in set {
                    if path.parent() == Some(dir) {
                        entries.push(Entry {
                            name: path.file_name().unwrap().to_owned(),
                            kind,
                        });
                    }
                }
            }
            Ok(entries)
        }
        fn remove_file(&mut self, path: &Path) -> bool {
            self.files.remove(path) || self.links.remove(path)
        }
        fn remove_folder(&mut self, path: &Path) -> bool {
            self.folders.remove(path)
        }
    }

    #[test]
    fn the_players_files_go_and_the_games_stay() {
        let mut tree = Memory::default();
        for dir in [
            "var",
            "files/d",
            "savedata/savegame",
            "savedata/replay/deep",
            "savedata/videos",
        ] {
            tree.add(dir, true);
        }
        let player = [
            "var/ral.cfg",
            "var/rc2k.toml",
            "var/hst.dat",
            "var/imgui.ini",
            "var/tmprpl",
            "var/RAL.DAT",
            "savedata/savegame/svgame00",
            "savedata/replay/replay00",
            "savedata/replay/deep/x",
            "savedata/videos/a.mp4",
            "savedata/loose.bin",
        ];
        let kept = [
            "var/ral.zog",
            "var/INSTALL.CFG",
            "var/logfile.txt",
            "files/d/jade.bun",
            "rc2k",
        ];
        for f in player.iter().chain(&kept) {
            tree.add(f, false);
        }
        let out = delete_player_data(&mut tree, Path::new("var"), Path::new("savedata"));
        assert_eq!(
            out,
            Outcome {
                removed: player.len(),
                failed: 0
            }
        );
        for f in player {
            assert!(!tree.has(f), "{f} was not deleted");
        }
        for f in kept {
            assert!(tree.has(f), "{f} was deleted");
        }
        // The folders the program expects are still there, empty; deeper ones are gone.
        assert!(tree.has("savedata/savegame"));
        assert!(tree.has("savedata/videos"));
        assert!(!tree.has("savedata/replay/deep"));
        // A second deletion finds nothing.
        assert_eq!(
            delete_player_data(&mut tree, Path::new("var"), Path::new("savedata")),
            Outcome::default()
        );
    }

    #[test]
    fn a_link_is_removed_but_not_followed() {
        let mut tree = Memory::default();
        tree.add("var", true);
        tree.add("outside/keep.txt", false);
        tree.add("savedata", true);
        tree.links.insert("savedata/link".into());
        let out = delete_player_data(&mut tree, Path::new("var"), Path::new("savedata"));
        assert_eq!(
            out,
            Outcome {
                removed: 1,
                failed: 0
            }
        );
        assert!(tree.has("outside/keep.txt"));
        assert!(!tree.has("savedata/link"));
    }

    #[test]
    fn missing_folders_are_nothing() {
        assert_eq!(
            delete_player_data(
                &mut Memory::default(),
                Path::new("var"),
                Path::new("savedata")
            ),
            Outcome::default()
        );
    }

    /// A tree whose removals and listings fail.
    struct Failing;

    impl Tree for Failing {
        fn list(&mut self, dir: &Path) -> Result<Vec<Entry>, ListError> {
            if dir == Path::new("var") {
                Ok(vec![
                    Entry {
                        name: "a".into(),
                        kind: Kind::File,
                    },
                    Entry {
                        name: "b".into(),
                        kind: Kind::Unreadable,
                    },
                    Entry {
                        name: "ral.zog".into(),
                        kind: Kind::File,
                    },
                ])
            } else {
                Err(ListError::Unreadable)
            }
        }
        fn remove_file(&mut self, _: &Path) -> bool {
            false
        }
        fn remove_folder(&mut self, _: &Path) -> bool {
            false
        }
    }

    #[test]
    fn what_cannot_be_removed_or_read_is_counted() {
        let out = delete_player_data(&mut Failing, Path::new("var"), Path::new("savedata"));
        // The file, the unreadable entry, and `savedata` that cannot be listed.
        assert_eq!(
            out,
            Outcome {
                removed: 0,
                failed: 3
            }
        );
    }
}
