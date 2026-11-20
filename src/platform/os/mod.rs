//! The operating system's services the game asks for, on std and SDL3: the clock, the local time,
//! the command line, the current folder, message boxes. Nothing here is Windows-only; the game's
//! texts go in and out as Windows-1252 bytes, as the Win32 `A` functions took them. The system's
//! own facts (its version, names, drives, memory) are each system's file: `windows.rs`, `unix.rs`.

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

/// Opens files so that other processes may open them as `share` allows (the Windows share
/// flags, 0 for none) and with the normal attributes; where files carry no share mode, nothing.
pub fn share_mode(options: &mut std::fs::OpenOptions, share: u32) -> &mut std::fs::OpenOptions {
    #[cfg(windows)]
    {
        use std::os::windows::fs::OpenOptionsExt;
        const FILE_ATTRIBUTE_NORMAL: u32 = 0x80;
        options.share_mode(share).attributes(FILE_ATTRIBUTE_NORMAL);
    }
    #[cfg(not(windows))]
    let _ = share;
    options
}

/// Starts `command` without a console window where a window would otherwise open (Windows).
pub fn spawn_hidden(command: &mut std::process::Command) -> std::io::Result<std::process::Child> {
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        command.creation_flags(CREATE_NO_WINDOW);
    }
    command.spawn()
}

/// Milliseconds on a steady clock, wrapping as `GetTickCount` did.
pub fn ticks() -> u32 {
    let (start, base) = *clock_start();
    let elapsed = crate::remote::scaled_millis(start.elapsed());
    base.wrapping_add(elapsed as u32)
}

/// The local date and time, as `GetLocalTime` gave them (SDL's calendar).
pub fn local_time() -> rc2k_core::system::time::LocalTime {
    let Some(dt) = crate::platform::ffi::sdl::local_date_time() else {
        crate::fatal::raise::end(
            "os::local_time",
            crate::errors::BadInput("the system gave no local time"),
        );
    };
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

/// The words the program was started with, its own name first. Android passes none the way
/// other systems do: `platform::android` keeps the ones its launcher reads.
pub fn program_args() -> Vec<String> {
    #[cfg(target_os = "android")]
    return crate::platform::android::args();
    #[cfg(not(target_os = "android"))]
    std::env::args().collect()
}

/// The program's command line as one text (Windows-1252, no NUL): the arguments joined by
/// spaces, each with a space or a tab in it (or empty) quoted, as a shell's line reads.
pub fn command_line() -> Vec<u8> {
    let words: Vec<String> = program_args()
        .into_iter()
        .map(|a| {
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
    #[cfg(web)]
    let folder = crate::platform::web::with_mount(&folder);
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
    use sdl3_sys::messagebox::{SDL_MESSAGEBOX_INFORMATION, SDL_MESSAGEBOX_WARNING};
    let c = |b: &[u8]| {
        std::ffi::CString::new(crate::files::ansi_text(b))
            .unwrap_or_else(|_| c"(a text with a NUL inside)".to_owned())
    };
    let (caption, text) = (c(caption), c(text));
    let flags = match icon {
        Icon::Information => SDL_MESSAGEBOX_INFORMATION,
        Icon::Warning => SDL_MESSAGEBOX_WARNING,
    };
    crate::platform::ffi::sdl::show_message_box(flags, &caption, &text);
}

/// A text for the terminal the program was started from (`--help`, a switch it does not know):
/// standard output; on Windows, where a windowed program has one only when its output was sent
/// to a pipe or a file, else the console of the program that started it (`CONOUT$` once
/// attached). False when there is nowhere to write to.
pub fn print_to_terminal(text: &str) -> bool {
    use std::io::Write;
    #[cfg(windows)]
    {
        if crate::platform::ffi::win32::has_standard_output() {
            let mut out = std::io::stdout().lock();
            return out
                .write_all(text.as_bytes())
                .and_then(|()| out.flush())
                .is_ok();
        }
        if !crate::platform::ffi::win32::attach_parent_console() {
            return false;
        }
        // The console's line ends, and a fresh line after the shell's prompt.
        let text = format!("\n{}", text.replace('\n', "\r\n"));
        std::fs::OpenOptions::new()
            .write(true)
            .open("CONOUT$")
            .and_then(|mut out| out.write_all(text.as_bytes()))
            .is_ok()
    }
    #[cfg(not(windows))]
    {
        let mut out = std::io::stdout().lock();
        out.write_all(text.as_bytes())
            .and_then(|()| out.flush())
            .is_ok()
    }
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
