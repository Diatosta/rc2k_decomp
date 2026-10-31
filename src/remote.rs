//! Remote control for automated test runs (cargo feature `remote`; shipped builds have none).
//! `RC2K_REMOTE` names a folder. `tools/remote.py` renames a file of commands to `cmd` in
//! it; each is run at the next pump and answered with a line in `reply`, `<id> <answer>`.
//! Keys and buttons are SDL events, so they take the path real ones take.
//!
//!     <id> key <scancode name> <ms>      a press of ms
//!     <id> down|up <scancode name>       held until told
//!     <id> pad <button name> <ms>        a gamepad button (the menus' side)
//!     <id> shot <name>                   the next frame to <name>.raw, BGRA; answers its size
//!     <id> state                         frames, race flag, menu, highlighted item
//!     <id> quit                          as closing the window

use sdl3_sys::everything::*;
use std::ffi::CString;
use std::io::Write;
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

fn folder() -> Option<&'static PathBuf> {
    static FOLDER: OnceLock<Option<PathBuf>> = OnceLock::new();
    FOLDER
        .get_or_init(|| std::env::var_os("RC2K_REMOTE").map(PathBuf::from))
        .as_ref()
}

enum Release {
    Key(SDL_Scancode),
    Pad(SDL_GamepadButton),
}

#[derive(Default)]
struct Pending {
    releases: Vec<(Instant, Release)>,
    shot: Option<(String, String)>,
}

fn pending() -> std::sync::MutexGuard<'static, Pending> {
    static PENDING: OnceLock<Mutex<Pending>> = OnceLock::new();
    PENDING
        .get_or_init(Default::default)
        .lock()
        .unwrap_or_else(|e| e.into_inner())
}

fn reply(id: &str, answer: &str) {
    let Some(folder) = folder() else { return };
    if let Ok(mut file) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(folder.join("reply"))
    {
        let _ = writeln!(file, "{id} {answer}");
    }
}

fn key(scancode: SDL_Scancode, down: bool) {
    // SAFETY: an all-zero event is valid; SDL copies it.
    unsafe {
        let mut event: SDL_Event = std::mem::zeroed();
        event.r#type = if down {
            SDL_EVENT_KEY_DOWN.0
        } else {
            SDL_EVENT_KEY_UP.0
        };
        event.key.scancode = scancode;
        event.key.down = down;
        SDL_PushEvent(&mut event);
    }
}

fn pad(button: SDL_GamepadButton, down: bool) {
    // SAFETY: as `key`.
    unsafe {
        let mut event: SDL_Event = std::mem::zeroed();
        event.r#type = if down {
            SDL_EVENT_GAMEPAD_BUTTON_DOWN.0
        } else {
            SDL_EVENT_GAMEPAD_BUTTON_UP.0
        };
        event.gbutton.button = button.0 as u8;
        event.gbutton.down = down;
        SDL_PushEvent(&mut event);
    }
}

fn scancode(name: &str) -> Option<SDL_Scancode> {
    let name = CString::new(name).ok()?;
    // SAFETY: a NUL-ended text, read during the call.
    let code = unsafe { SDL_GetScancodeFromName(name.as_ptr()) };
    (code != SDL_SCANCODE_UNKNOWN).then_some(code)
}

fn button(name: &str) -> Option<SDL_GamepadButton> {
    let name = CString::new(name).ok()?;
    // SAFETY: as `scancode`.
    let code = unsafe { SDL_GetGamepadButtonFromString(name.as_ptr()) };
    (code != SDL_GAMEPAD_BUTTON_INVALID).then_some(code)
}

fn state() -> String {
    use crate::menu::loop_store::LOOP_FLAGS;
    format!(
        "frames={} race={} menu={:?} item={} focused={}",
        crate::gfx::screen_state::DRAW_SURFACE.frames.get(),
        crate::race::numbers_store::NUMBERS.in_race.get(),
        LOOP_FLAGS.menu.get().ok(),
        LOOP_FLAGS.item.get(),
        crate::session::FLAGS.window_focused.get(),
    )
}

fn run(line: &str) {
    let words: Vec<&str> = line.split_whitespace().collect();
    let [id, command, rest @ ..] = &words[..] else {
        return;
    };
    let ms =
        |w: Option<&&str>| Duration::from_millis(w.and_then(|m| m.parse().ok()).unwrap_or(100));
    let answer = match (*command, rest) {
        ("key", [name, tail @ ..]) => scancode(name).map(|code| {
            key(code, true);
            pending()
                .releases
                .push((Instant::now() + ms(tail.first()), Release::Key(code)));
            "ok".to_string()
        }),
        ("down" | "up", [name]) => scancode(name).map(|code| {
            key(code, *command == "down");
            "ok".to_string()
        }),
        ("pad", [name, tail @ ..]) => button(name).map(|code| {
            pad(code, true);
            pending()
                .releases
                .push((Instant::now() + ms(tail.first()), Release::Pad(code)));
            "ok".to_string()
        }),
        ("shot", [name]) => {
            pending().shot = Some(((*id).to_string(), (*name).to_string()));
            return;
        }
        ("state", []) => Some(state()),
        ("quit", []) => {
            // SAFETY: as `key`.
            unsafe {
                let mut event: SDL_Event = std::mem::zeroed();
                event.r#type = SDL_EVENT_QUIT.0;
                SDL_PushEvent(&mut event);
            }
            Some("ok".to_string())
        }
        _ => None,
    };
    reply(
        id,
        answer
            .as_deref()
            .unwrap_or("error: unknown command or name"),
    );
}

/// At each pump: the releases due, then the commands waiting.
pub fn poll() {
    let Some(folder) = folder() else { return };
    let now = Instant::now();
    pending().releases.retain(|(when, release)| {
        if *when > now {
            return true;
        }
        match release {
            Release::Key(code) => key(*code, false),
            Release::Pad(code) => pad(*code, false),
        }
        false
    });
    let path = folder.join("cmd");
    if let Ok(text) = std::fs::read_to_string(&path) {
        let _ = std::fs::remove_file(&path);
        text.lines().for_each(run);
    }
}

/// After a frame is shown: the screenshot asked for.
pub fn frame(pixels: &[u8], width: u32, height: u32) {
    let Some(folder) = folder() else { return };
    let Some((id, name)) = pending().shot.take() else {
        return;
    };
    if let Some(target) = crate::platform_3d::screen() {
        crate::render::bring_back(&target);
    }
    let answer = match std::fs::write(folder.join(format!("{name}.raw")), pixels) {
        Ok(()) => format!("{width} {height}"),
        Err(e) => format!("error: {e}"),
    };
    reply(&id, &answer);
}
