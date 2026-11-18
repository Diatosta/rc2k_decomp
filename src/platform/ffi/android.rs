//! Android's own calls: SDL's storage paths and the system log.

#![allow(unsafe_code)]

use std::ffi::{CStr, c_char, c_int};
use std::path::PathBuf;

unsafe extern "C" {
    fn __android_log_write(priority: c_int, tag: *const c_char, text: *const c_char) -> c_int;
}

/// The log priority of an informational line.
const LOG_INFO: c_int = 4;

/// One line in the system log (`adb logcat -s rc2k`).
pub fn log_line(line: &str) {
    let Ok(text) = std::ffi::CString::new(line.replace('\0', " ")) else {
        return;
    };
    // SAFETY: two NUL-ended texts that live past the call, which copies them.
    unsafe {
        __android_log_write(LOG_INFO, c"rc2k".as_ptr(), text.as_ptr());
    }
}

fn path_from(raw: *const c_char) -> Option<PathBuf> {
    if raw.is_null() {
        return None;
    }
    // SAFETY: SDL's NUL-ended text (null checked) that lives as long as the program.
    let text = unsafe { CStr::from_ptr(raw) }.to_str().ok()?;
    Some(PathBuf::from(text))
}

/// The app's folder on shared storage (`Android/data/<package>/files`), which `adb push`
/// reaches and the player's file manager shows.
pub fn external_storage() -> Option<PathBuf> {
    // SAFETY: SDL's call, valid once the activity has started SDL.
    path_from(unsafe { sdl3_sys::everything::SDL_GetAndroidExternalStoragePath() })
}

/// The app's private folder.
pub fn internal_storage() -> Option<PathBuf> {
    // SAFETY: as `external_storage`.
    path_from(unsafe { sdl3_sys::everything::SDL_GetAndroidInternalStoragePath() })
}

/// Standard error and standard output sent to the system log, line by line: the program's
/// first words (before its own log is open) would otherwise go nowhere.
pub fn forward_standard_streams() {
    use std::io::{BufRead, BufReader};
    use std::os::fd::FromRawFd;
    let mut ends = [0 as c_int; 2];
    // SAFETY: `pipe` fills the two descriptors; both are valid when it returns 0.
    if unsafe { libc::pipe(ends.as_mut_ptr()) } != 0 {
        return;
    }
    // SAFETY: the pipe's write end replaces descriptors 1 and 2 (valid, open in every
    // process); the read end is owned by the reader thread below, the write end by the two.
    let reader = unsafe {
        libc::dup2(ends[1], 1);
        libc::dup2(ends[1], 2);
        libc::close(ends[1]);
        std::fs::File::from_raw_fd(ends[0])
    };
    std::thread::spawn(move || {
        for line in BufReader::new(reader).lines().map_while(Result::ok) {
            log_line(&line);
        }
    });
}
