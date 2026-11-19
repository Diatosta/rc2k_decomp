//! Deleting what the player has made: everything under `savedata` (saved games, car set-ups,
//! replays, screenshots, videos) and everything in `var` except the installation's records and
//! the game's log. What is left alone is the program and the game's own data (`files`, the
//! language files, the CD's folders), which neither folder holds.

use std::fs;
use std::io;
use std::path::Path;

/// The files of `var` that stay: the installation's two records (without them the program
/// takes the folder for a download and runs the installer) and the log of the run in progress
/// (it is open, and it is not the player's).
pub const KEPT_IN_VAR: [&str; 3] = ["ral.zog", "install.cfg", "logfile.txt"];

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
pub fn is_kept(name: &str) -> bool {
    KEPT_IN_VAR.iter().any(|k| k.eq_ignore_ascii_case(name))
}

/// Everything under `savedata` and the files of `var` that are not kept removed. The folders
/// `savedata` and the ones directly in it stay (the program expects them); deeper folders go.
/// A missing folder is nothing to remove. A link is removed as a link, never followed.
pub fn delete_player_data(var: &Path, savedata: &Path) -> Outcome {
    let mut out = Outcome::default();
    empty(var, 0, &mut out, &|name| is_kept(name));
    empty(savedata, 0, &mut out, &|_| false);
    out
}

/// The entries of `dir` removed except the files `keep` names (at the top only); folders
/// below the first level are removed too.
fn empty(dir: &Path, depth: u32, out: &mut Outcome, keep: &dyn Fn(&str) -> bool) {
    let entries = match fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(e) if e.kind() == io::ErrorKind::NotFound => return,
        Err(_) => {
            out.failed += 1;
            return;
        }
    };
    for entry in entries {
        let Ok(entry) = entry else {
            out.failed += 1;
            continue;
        };
        let path = entry.path();
        let name = entry.file_name();
        let kind = match fs::symlink_metadata(&path) {
            Ok(meta) => meta.file_type(),
            Err(_) => {
                out.failed += 1;
                continue;
            }
        };
        if kind.is_dir() {
            // The folders directly in `savedata` stay, emptied; a folder in `var` or deeper
            // is removed with what it holds.
            if depth == 0 && keep(&name.to_string_lossy()) {
                continue;
            }
            empty(&path, depth + 1, out, &|_| false);
            if depth >= 1
                || dir
                    .file_name()
                    .is_some_and(|n| n.eq_ignore_ascii_case("var"))
            {
                match fs::remove_dir(&path) {
                    Ok(()) => {}
                    Err(_) => out.failed += 1,
                }
            }
        } else if depth == 0 && keep(&name.to_string_lossy()) {
            // Kept.
        } else {
            match fs::remove_file(&path) {
                Ok(()) => out.removed += 1,
                Err(_) => out.failed += 1,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scratch(name: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("rc2k_wipe_{name}_{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn the_players_files_go_and_the_games_stay() {
        let root = scratch("a");
        let (var, save) = (root.join("var"), root.join("savedata"));
        for dir in [
            "var",
            "files/d",
            "savedata/savegame",
            "savedata/replay/deep",
            "savedata/videos",
        ] {
            fs::create_dir_all(root.join(dir)).unwrap();
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
            fs::write(root.join(f), b"x").unwrap();
        }
        let out = delete_player_data(&var, &save);
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
        // The folders the program expects are still there, empty; deeper ones are gone.
        assert!(root.join("savedata/savegame").is_dir());
        assert!(root.join("savedata/videos").is_dir());
        assert!(!root.join("savedata/replay/deep").exists());
        // A second deletion finds nothing.
        assert_eq!(delete_player_data(&var, &save), Outcome::default());
        fs::remove_dir_all(&root).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn a_link_is_removed_but_not_followed() {
        let root = scratch("b");
        fs::create_dir_all(root.join("var")).unwrap();
        fs::create_dir_all(root.join("outside")).unwrap();
        fs::write(root.join("outside/keep.txt"), b"x").unwrap();
        fs::create_dir_all(root.join("savedata")).unwrap();
        std::os::unix::fs::symlink(root.join("outside"), root.join("savedata/link")).unwrap();
        let out = delete_player_data(&root.join("var"), &root.join("savedata"));
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

    #[test]
    fn missing_folders_are_nothing() {
        let root = scratch("c");
        assert_eq!(
            delete_player_data(&root.join("var"), &root.join("savedata")),
            Outcome::default()
        );
        fs::remove_dir_all(&root).unwrap();
    }
}
