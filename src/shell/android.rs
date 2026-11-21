//! The Android build's entry: the activity (`tools/android/`) loads the program as a shared
//! library and calls `main` on the thread SDL makes for it. `platform::android` prepares the
//! game's folder and words; the game is started from here, where it is known.

/// The program, from the activity's call: enters the game's folder and runs the game, which
/// ends the process when the game ends.
pub fn main() -> i32 {
    if let Err(code) = crate::platform::android::prepare() {
        return code;
    }
    let args = &crate::platform::os::program_args()[1..];
    if let Some(code) = crate::standalone::answer_command_line(args) {
        return code;
    }
    let log = rc2k_core::menu::launch::log_path(args).unwrap_or_else(|| "rc2k.log".into());
    crate::standalone::run(std::path::Path::new(&log), 0)
}
