//! The operating system's services the game asks for, on std and SDL3 (phase 11,
//! `notes/conversion.md`): the clock, the local time, the command line, the current folder,
//! message boxes. Nothing here is Windows-only; the game's texts go in and out as
//! Windows-1252 bytes, as the Win32 `A` functions took them. The system's own facts (its
//! version, names, drives, memory) are each system's file: `windows.rs`, `unix.rs`.

#![allow(unsafe_code)]

#[cfg(windows)]
mod windows;
#[cfg(windows)]
pub use windows::*;
#[cfg(unix)]
mod unix;
#[cfg(unix)]
pub use unix::*;

use std::sync::OnceLock;
use std::time::Instant;

/// The clock's start: the moment of the first reading, and the count it began at.
fn clock_start() -> &'static (Instant, u32) {
    static START: OnceLock<(Instant, u32)> = OnceLock::new();
    START.get_or_init(|| {
        // `GetTickCount` counted from the machine's start: never 0 when the game ran, and
        // small. The game compares some times by the sign of their difference from 0 (the
        // menu sounds' gap, `sound::play_menu_sound`), so a count with its top bit set kept
        // them silent: the count begins as a machine a minute up would show.
        (Instant::now(), 0x0001_0000)
    })
}

/// Milliseconds on a steady clock, wrapping as `GetTickCount` did.
pub fn ticks() -> u32 {
    let (start, base) = *clock_start();
    base.wrapping_add(start.elapsed().as_millis() as u32)
}

/// The local date and time, as `GetLocalTime` gave them (SDL's calendar).
pub fn local_time() -> rc2k_core::system::time::LocalTime {
    use sdl3_sys::time::{SDL_DateTime, SDL_GetCurrentTime, SDL_TimeToDateTime};
    let mut now = 0;
    let mut dt = SDL_DateTime::default();
    // SAFETY: two out-parameters of documented calls that need no SDL start-up.
    let read =
        unsafe { SDL_GetCurrentTime(&raw mut now) && SDL_TimeToDateTime(now, &raw mut dt, true) };
    if !read {
        crate::fatal::raise::end(
            "os::local_time",
            crate::errors::BadInput("the system gave no local time"),
        );
    }
    rc2k_core::system::time::LocalTime {
        year: dt.year as u16,
        month: dt.month as u16,
        weekday: dt.day_of_week as u16,
        day: dt.day as u16,
        hour: dt.hour as u16,
        minute: dt.minute as u16,
        second: dt.second as u16,
        milliseconds: (dt.nanosecond / 1_000_000) as u16,
    }
}

/// The program's command line as one text (Windows-1252, no NUL): the arguments joined by
/// spaces, each with a space or a tab in it (or empty) quoted, as a shell's line reads.
pub fn command_line() -> Vec<u8> {
    let words: Vec<String> = std::env::args_os()
        .map(|a| {
            let a = a.to_string_lossy().into_owned();
            if a.is_empty() || a.contains([' ', '\t']) {
                format!("\"{a}\"")
            } else {
                a
            }
        })
        .collect();
    crate::files::ansi_bytes(&words.join(" "))
}

/// The current folder (Windows-1252), or None if it cannot be had.
pub fn current_folder() -> Option<Vec<u8>> {
    let folder = std::env::current_dir().ok()?;
    Some(crate::files::ansi_bytes(&folder.to_string_lossy()))
}

/// What a message box shows besides its text.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Icon {
    Information,
    Warning,
}

/// A message box with one button, over no window, waited for (`MessageBoxA` with
/// MB_TASKMODAL; the texts Windows-1252).
pub fn message_box(icon: Icon, caption: &[u8], text: &[u8]) {
    use sdl3_sys::messagebox::{
        SDL_MESSAGEBOX_INFORMATION, SDL_MESSAGEBOX_WARNING, SDL_ShowSimpleMessageBox,
    };
    let c = |b: &[u8]| {
        std::ffi::CString::new(crate::files::ansi_text(b))
            .unwrap_or_else(|_| c"(a text with a NUL inside)".to_owned())
    };
    let (caption, text) = (c(caption), c(text));
    let flags = match icon {
        Icon::Information => SDL_MESSAGEBOX_INFORMATION,
        Icon::Warning => SDL_MESSAGEBOX_WARNING,
    };
    // SAFETY: both texts are NUL-ended and live over the call; no window.
    unsafe {
        SDL_ShowSimpleMessageBox(flags, caption.as_ptr(), text.as_ptr(), std::ptr::null_mut())
    };
}

#[cfg(test)]
mod tests {
    #[test]
    fn the_clock_runs_forward_and_never_starts_at_0() {
        let a = super::ticks();
        std::thread::sleep(std::time::Duration::from_millis(20));
        let b = super::ticks();
        assert_ne!(a, 0);
        assert!(b.wrapping_sub(a) >= 20 && b.wrapping_sub(a) < 2_000);
    }

    #[test]
    fn the_local_time_is_a_calendar_one() {
        let t = super::local_time();
        assert!(t.year >= 2024 && (1..=12).contains(&t.month) && (1..=31).contains(&t.day));
        assert!(t.weekday < 7 && t.hour < 24 && t.minute < 60 && t.milliseconds < 1000);
    }

    #[test]
    fn the_command_line_starts_with_the_program() {
        let line = super::command_line();
        assert!(!line.is_empty() && !line.contains(&0));
    }
}
