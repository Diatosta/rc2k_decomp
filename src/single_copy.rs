//! One copy of the program per game folder, unless `--allow-another` says otherwise. The copy
//! that starts first holds a lock on `rc2k.lock` in the folder for as long as it runs (the
//! system lets go of it when the program ends, however it ends); a copy that finds the lock
//! taken stops with a message that names the switch.
//!
//! A copy let in beside another takes the next free lock (`rc2k.lock.2`, ..) and so a number.
//! The program's log, when it is the default `rc2k.log`, takes the number too (`rc2k-2.log`):
//! the second copy does not empty the first's log. The var and savedata folders, the options
//! file and the game's own log are shared by all copies, and the network's port is the first
//! hosting copy's.
//!
//! `--instance <n>` is the way to run several at once with nothing written by two: instance
//! `n` takes exactly the lock `rc2k.lock[.n]` (and stops if it is taken), writes `rc2k-n.log`
//! and keeps its own `var` and `savedata` folders in `instances/<n>/` (where the options, the
//! best times, the saved games, replays, snapshots and the game's log go). Its first start
//! copies the folder's `var` files into it (so it begins with the same options); its
//! `savedata` begins empty. Shared, read only: the game's `files`, films and locale folders.
//! Shared and still to be told apart by the player: the network port (give each hosting
//! instance its own with `--port`) and the machine's controllers and sound device. Instance 1
//! is the folder's own folders.

use std::fs::File;
use std::path::PathBuf;

/// The most copies in one folder.
const MOST: u32 = 8;

/// A copy's claim on its game folder: held as long as the value is.
pub struct Claim {
    /// This copy's number: 1 for the first in the folder.
    pub number: u32,
    _lock: Option<File>,
}

/// The lock file of copy `number`.
fn lock_path(number: u32) -> PathBuf {
    match number {
        1 => PathBuf::from("rc2k.lock"),
        n => PathBuf::from(format!("rc2k.lock.{n}")),
    }
}

/// Takes the folder's lock for this copy. With `allow_another` a taken lock is passed over
/// for the next number; without it the message that tells the player what to do is returned.
/// A lock file that cannot be made or locked (a read-only folder, a file system without
/// locks) holds nothing back: the copy goes on, as no copy was ever stopped before.
pub fn claim(allow_another: bool) -> Result<Claim, String> {
    for number in 1..=MOST {
        let Ok(file) = File::options()
            .create(true)
            .truncate(false)
            .write(true)
            .open(lock_path(number))
        else {
            return Ok(Claim {
                number,
                _lock: None,
            });
        };
        match file.try_lock() {
            Ok(()) => {
                return Ok(Claim {
                    number,
                    _lock: Some(file),
                });
            }
            Err(std::fs::TryLockError::WouldBlock) if allow_another && number < MOST => {}
            Err(std::fs::TryLockError::WouldBlock) => {
                return Err(if allow_another {
                    format!("{MOST} copies of the program already run in this folder.")
                } else {
                    "Another copy of the program is already running in this folder.\n\
                     Two copies write the same options and saved games; to start this one \
                     beside it anyway, run it with --allow-another."
                        .to_string()
                });
            }
            Err(std::fs::TryLockError::Error(_)) => {
                return Ok(Claim {
                    number,
                    _lock: None,
                });
            }
        }
    }
    unreachable!("the loop returns at its last number");
}

/// The number (above 1) of a copy let in beside another with `--allow-another` and no instance:
/// it shares the folder's `var`, so the files a copy writes while it runs take its number.
static SHARED_COPY: std::sync::OnceLock<u32> = std::sync::OnceLock::new();

/// Notes that this copy is number `number` of a folder it shares with the others (no instance).
pub fn note_shared_copy(number: u32) {
    if number > 1 {
        let _ = SHARED_COPY.set(number);
    }
}

/// The ending of the last race's replay file's name: empty for the folder's first copy, else
/// `.<number>`, so that copies sharing `var` do not write one file.
pub fn last_race_suffix() -> String {
    SHARED_COPY
        .get()
        .map_or_else(String::new, |number| format!(".{number}"))
}

/// Takes the lock of instance `number` (1 ..= `MOST`), and for one above 1 sets its own `var`
/// and `savedata` folders, `files::instance::path`, made and seeded. A taken lock is refused.
pub fn claim_instance(number: u32) -> Result<Claim, String> {
    let taken = || {
        format!(
            "Instance {number} of the program already runs in this folder.\n\
             Start another number with --instance, or none."
        )
    };
    let file = File::options()
        .create(true)
        .truncate(false)
        .write(true)
        .open(lock_path(number));
    let lock = match file {
        Ok(file) => match file.try_lock() {
            Ok(()) => Some(file),
            Err(std::fs::TryLockError::WouldBlock) => return Err(taken()),
            Err(std::fs::TryLockError::Error(_)) => None,
        },
        Err(_) => None,
    };
    if number > 1 {
        crate::files::instance::set(number);
        seed_instance(number);
    }
    Ok(Claim {
        number,
        _lock: lock,
    })
}

/// The instance's `var` folder made, with the files of the folder's own `var` in it the first
/// time (the game's log and the debug window's layout stay out). Failures are logged and leave
/// the instance to begin without them.
fn seed_instance(number: u32) {
    let own = std::path::PathBuf::from(format!("instances/{number}/var"));
    if own.is_dir() {
        return;
    }
    if let Err(e) = std::fs::create_dir_all(&own) {
        eprintln!("rc2k: instance {number}'s folder cannot be made: {e}");
        return;
    }
    let Ok(files) = std::fs::read_dir(crate::files::shared_host_path(b"var")) else {
        return;
    };
    for file in files.flatten() {
        let name = file.file_name();
        let skip = name
            .to_str()
            .is_none_or(|n| n.eq_ignore_ascii_case("logfile.txt") || n.ends_with(".ini"));
        if !skip && file.path().is_file() {
            let _ = std::fs::copy(file.path(), own.join(&name));
        }
    }
}

/// The program's log for copy `number` when the log is the default: the first copy's is
/// `rc2k.log`, the others' `rc2k-<number>.log`; a log the player named stays.
pub fn log_for(named: &std::path::Path, number: u32) -> PathBuf {
    if number > 1 && named == std::path::Path::new("rc2k.log") {
        PathBuf::from(format!("rc2k-{number}.log"))
    } else {
        named.to_path_buf()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn the_default_log_takes_the_copy_number() {
        assert_eq!(log_for(Path::new("rc2k.log"), 1), Path::new("rc2k.log"));
        assert_eq!(log_for(Path::new("rc2k.log"), 3), Path::new("rc2k-3.log"));
        assert_eq!(log_for(Path::new("my.log"), 3), Path::new("my.log"));
    }

    #[test]
    fn a_second_copy_is_refused_or_numbered() {
        let folder = std::env::temp_dir().join(format!("rc2k_single_copy_{}", std::process::id()));
        std::fs::create_dir_all(&folder).unwrap();
        // Locks are the process's own files: take them by path instead of the current folder.
        let first = File::options()
            .create(true)
            .truncate(false)
            .write(true)
            .open(folder.join("rc2k.lock"))
            .unwrap();
        first.try_lock().unwrap();
        let second = File::options()
            .write(true)
            .open(folder.join("rc2k.lock"))
            .unwrap();
        assert!(matches!(
            second.try_lock(),
            Err(std::fs::TryLockError::WouldBlock)
        ));
        drop(first);
        second.try_lock().unwrap();
        let _ = std::fs::remove_dir_all(&folder);
    }
}
