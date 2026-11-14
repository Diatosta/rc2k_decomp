use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;
use std::sync::OnceLock;

/// The program's log (`rc2k.log`), set by `standalone::run`. Nothing is written before that.
pub static LOG_PATH: OnceLock<PathBuf> = OnceLock::new();

pub fn log(msg: &str) {
    if let Some(path) = LOG_PATH.get()
        && let Ok(mut f) = OpenOptions::new().create(true).append(true).open(path)
    {
        // One write per line: with the file opened to append, a line another thread logs at the
        // same moment cannot land inside this one.
        let _ = f.write_all(format!("{msg}\n").as_bytes());
    }
    // The web build: the browser's console shows the log as it goes (the file is in the
    // origin's storage, out of sight).
    #[cfg(target_os = "emscripten")]
    eprintln!("{msg}");
}

/// The carry and zero flags a function leaves, for Rust callers that still read them
/// (a function returns these for callers that read the carry or zero result).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CarryZeroFlags {
    pub carry: bool,
    pub zero: bool,
}

impl CarryZeroFlags {}
