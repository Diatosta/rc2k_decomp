//! The Android build's start. The activity (`tools/android/`) loads the program as a shared
//! library and calls `main` on the thread SDL makes for it. The game's folder is the app's
//! `game` folder in its private storage (tests put the game's files there with `run-as`, see
//! `tools/android/README.md`; the first-run page's zip is unpacked there too), where the program also keeps
//! its log, options and saved games, as on any system. Words the launcher passes (tests
//! use them) are read from `rc2k_args.txt` in the storage folder, split at spaces.

use crate::platform::ffi::android;
use std::path::PathBuf;
use std::sync::OnceLock;

static ARGS: OnceLock<Vec<String>> = OnceLock::new();

/// The program's words, its own name first.
pub fn args() -> Vec<String> {
    ARGS.get()
        .cloned()
        .unwrap_or_else(|| vec!["rc2k".to_string()])
}

/// The folder the app's files live in: the app's private storage (shared storage's app folder
/// is out of the player's file managers' reach on current Android, and a file put there by
/// `adb push` belongs to the shell, which the app may not read).
fn storage() -> Option<PathBuf> {
    android::internal_storage().or_else(android::external_storage)
}

/// The program's first steps, from the activity's call: the standard streams, the program's
/// words, the game's folder entered, the home for the program's own files. The code to end with
/// when one fails.
pub fn prepare() -> Result<(), i32> {
    android::forward_standard_streams();
    let Some(storage) = storage() else {
        eprintln!("rc2k: the app has no storage folder");
        return Err(1);
    };
    let mut words = vec!["rc2k".to_string()];
    if let Ok(text) = std::fs::read_to_string(storage.join("rc2k_args.txt")) {
        words.extend(text.split_whitespace().map(str::to_string));
    }
    let _ = ARGS.set(words);
    let game = storage.join("game");
    if let Err(e) = std::fs::create_dir_all(&game).and_then(|()| std::env::set_current_dir(&game)) {
        eprintln!(
            "rc2k: the game's folder {} cannot be entered: {e}",
            game.display()
        );
        return Err(1);
    }
    eprintln!("rc2k: the game's folder is {}", game.display());
    // The home the program keeps its own files under, where it looks for one.
    // SAFETY: set before the game's threads start; nothing else reads the environment yet.
    unsafe {
        std::env::set_var("HOME", &storage);
    }
    Ok(())
}
