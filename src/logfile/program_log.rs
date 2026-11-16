//! The program's own log (`rc2k.log`), apart from the game's (`var\logfile.txt`).

use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

/// The program's log (`rc2k.log`), set by `start`. Nothing is written before that.
pub static LOG_PATH: OnceLock<PathBuf> = OnceLock::new();

/// The log, kept open for the run: opening it for every line cost a file open each (and a
/// virus scanner's look at it on Windows).
static LOG_FILE: OnceLock<File> = OnceLock::new();

/// The log begun, empty, at `path`; the first call counts.
pub fn start(path: &Path) {
    let _ = File::create(path);
    let _ = LOG_PATH.set(path.to_path_buf());
    if let Ok(file) = OpenOptions::new().create(true).append(true).open(path) {
        let _ = LOG_FILE.set(file);
    }
}

pub fn log(msg: &str) {
    if let Some(mut file) = LOG_FILE.get() {
        // One write per line, and the file opened to append: a line another thread (or the
        // fault report's own handle) logs at the same moment cannot land inside this one, and
        // a log emptied from outside goes on from its new end.
        let _ = file.write_all(format!("{msg}\n").as_bytes());
    }
    // The web build: the browser's console shows the log as it goes (the file is in the
    // origin's storage, out of sight).
    #[cfg(target_os = "emscripten")]
    eprintln!("{msg}");
}
