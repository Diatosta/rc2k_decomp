//! The advanced page's "delete all saved data": the player's own files (`var` and `savedata`,
//! what stays is in `rc2k_core::saves::wipe`) removed, and the program kept from writing them
//! back while it closes. Meant for the platforms where the folders are not at hand (a phone, the
//! browser's storage, a handheld).
//!
//! The deletion is made when the question is answered yes, and once more as the program
//! ends: a file written in between (the options, the window's mode, the debug window's layout)
//! would bring the settings back.

use crate::errors::Refused;
use crate::owners::Owners;
use rc2k_core::saves::wipe::{Entry, Kind, ListError, Outcome, Tree, delete_player_data};
use rc2k_core::strings::own::id;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};

/// Seconds the question keeps its Yes out of reach, so that a key pressed in passing cannot
/// confirm it.
const YES_HELD_SECS: u32 = 5;

/// The data was deleted: nothing may save again.
static DELETED: AtomicBool = AtomicBool::new(false);

/// Whether the player's data was deleted in this run (the program is closing and saves nothing).
pub fn deleted() -> bool {
    DELETED.load(Ordering::Relaxed)
}

/// The folders on the disk.
struct Disk;

impl Tree for Disk {
    fn list(&mut self, dir: &Path) -> Result<Vec<Entry>, ListError> {
        let entries = match std::fs::read_dir(dir) {
            Ok(entries) => entries,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Err(ListError::Missing),
            Err(_) => return Err(ListError::Unreadable),
        };
        Ok(entries
            .map(|entry| match entry {
                Ok(entry) => {
                    // A link is looked at as a link, never followed.
                    let kind = match std::fs::symlink_metadata(entry.path()) {
                        Ok(meta) if meta.file_type().is_dir() => Kind::Folder,
                        Ok(_) => Kind::File,
                        Err(_) => Kind::Unreadable,
                    };
                    Entry {
                        name: entry.file_name(),
                        kind,
                    }
                }
                Err(_) => Entry {
                    name: Default::default(),
                    kind: Kind::Unreadable,
                },
            })
            .collect())
    }

    fn remove_file(&mut self, path: &Path) -> bool {
        std::fs::remove_file(path).is_ok()
    }

    fn remove_folder(&mut self, path: &Path) -> bool {
        std::fs::remove_dir(path).is_ok()
    }
}

/// The player's files removed now.
fn delete_now() -> Outcome {
    let out = delete_player_data(
        &mut Disk,
        &crate::files::host_path(b"var"),
        &crate::files::host_path(b"savedata"),
    );
    crate::program_log::log(&format!(
        "saves: all saved data deleted ({} files removed, {} could not be)",
        out.removed, out.failed
    ));
    out
}

/// What the program does as it ends after a deletion: the files made since are removed too.
pub fn delete_again_at_end() {
    if deleted() {
        delete_now();
    }
}

/// The row chosen: the question (No is chosen to begin with and is Escape's answer; Yes opens after a few seconds), then the
/// deletion, a line saying so, and the program closes as when the window is closed.
pub fn choose(o: Owners<'static>) -> Result<(), Refused> {
    if !crate::error_dialog::ask_yes(id::DELETE_TITLE, id::DELETE_ASK, YES_HELD_SECS) {
        return Ok(());
    }
    DELETED.store(true, Ordering::Relaxed);
    let out = delete_now();
    let said = if out.failed == 0 {
        id::DELETE_DONE
    } else {
        id::DELETE_FAILED
    };
    crate::error_dialog::notice(&crate::error_dialog::text_with(said, ""));
    o.system.session.flags.quitting.set(true);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn scratch(name: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("rc2k_wipe_{name}_{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    /// The players' files on a real disk go, the game's stay, and the folders the program
    /// expects remain.
    #[test]
    fn a_disk_tree_is_emptied_as_the_rule_says() {
        let root = scratch("tree");
        for dir in [
            "var",
            "files/d",
            "savedata/savegame",
            "savedata/replay/deep",
        ] {
            fs::create_dir_all(root.join(dir)).unwrap();
        }
        let player = [
            "var/ral.cfg",
            "savedata/savegame/svgame00",
            "savedata/replay/deep/x",
        ];
        let kept = ["var/ral.zog", "var/INSTALL.CFG", "files/d/jade.bun"];
        for f in player.iter().chain(&kept) {
            fs::write(root.join(f), b"x").unwrap();
        }
        let out = delete_player_data(&mut Disk, &root.join("var"), &root.join("savedata"));
        assert_eq!(
            out,
            Outcome {
                removed: player.len(),
                failed: 0
            }
        );
        for f in player {
            assert!(!root.join(f).exists(), "{f} was not deleted");
        }
        for f in kept {
            assert!(root.join(f).exists(), "{f} was deleted");
        }
        assert!(root.join("savedata/savegame").is_dir());
        assert!(!root.join("savedata/replay/deep").exists());
        fs::remove_dir_all(&root).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn a_link_is_removed_but_not_followed() {
        let root = scratch("link");
        fs::create_dir_all(root.join("var")).unwrap();
        fs::create_dir_all(root.join("outside")).unwrap();
        fs::write(root.join("outside/keep.txt"), b"x").unwrap();
        fs::create_dir_all(root.join("savedata")).unwrap();
        std::os::unix::fs::symlink(root.join("outside"), root.join("savedata/link")).unwrap();
        let out = delete_player_data(&mut Disk, &root.join("var"), &root.join("savedata"));
        assert_eq!(
            out,
            Outcome {
                removed: 1,
                failed: 0
            }
        );
        assert!(root.join("outside/keep.txt").exists());
        assert!(!root.join("savedata/link").exists());
        fs::remove_dir_all(&root).unwrap();
    }
}
