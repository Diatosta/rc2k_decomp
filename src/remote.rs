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
        .unwrap_or_else(std::sync::PoisonError::into_inner)
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
        SDL_PushEvent(&raw mut event);
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
        SDL_PushEvent(&raw mut event);
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

/// The car `state` reports. With several cars the work car holds the last one stepped between
/// frames: in the arcade none of them, in split screen player 2's.
fn driven_car() -> &'static crate::physics::car_store::Car {
    let viewed = crate::race::view_store::RACE_VIEW
        .view_block(0)
        .wrapping_sub(crate::race::blocks_store::blocks_addr()) as usize
        / size_of::<crate::physics::car_store::Car>();
    match crate::race::blocks_store::CAR_BLOCKS.cars.get(viewed) {
        Some(car)
            if crate::race::vars_store::RACE_VARS.run_kind.bits() == 2
                || crate::menu::event_store::EVENT_STATE.player_count.get() > 1 =>
        {
            car
        }
        _ => &crate::physics::car_store::CAR,
    }
}

fn state() -> String {
    use crate::menu::loop_store::LOOP_FLAGS;
    use crate::race::checkpoints_store::{CHECKPOINTS, nth};
    #[allow(non_snake_case)]
    let CAR = driven_car();
    let at = CAR.checkpoint.get() as usize;
    // The stage line ahead, for `tools/remote.py pilot`.
    let ahead = (at + LOOK_AHEAD).min(CHECKPOINTS.map_x.len() - 1);
    let near = (at + LOOK_NEAR).min(CHECKPOINTS.map_x.len() - 1);
    format!(
        "frames={} race={} menu={:?} item={} focused={} speed={:.0} time={} splits={} checkpoint={} map={},{} target={},{} near={},{} channels={} mix={} heading={} matrix={}",
        crate::gfx::screen_state::DRAW_SURFACE.frames.get(),
        crate::race::numbers_store::NUMBERS.in_race.get(),
        LOOP_FLAGS.menu.get().ok(),
        LOOP_FLAGS.item.get(),
        crate::session::FLAGS.window_focused.get(),
        CAR.speed.get(),
        CAR.stage_time.get(),
        CAR.splits_passed.get(),
        at,
        CAR.map_place[0].get() as i32,
        CAR.map_place[1].get() as i32,
        nth(&CHECKPOINTS.map_x[0], ahead).get() as i32,
        nth(&CHECKPOINTS.map_y[0], ahead).get() as i32,
        nth(&CHECKPOINTS.map_x[0], near).get() as i32,
        nth(&CHECKPOINTS.map_y[0], near).get() as i32,
        crate::audio::store::AUDIO.channels_used.get(),
        crate::platform_sound::LAST_PEAK.swap(0, std::sync::atomic::Ordering::Relaxed),
        CAR.car_heading.get() as i32,
        CAR.crash_matrix[..12]
            .iter()
            .map(|cell| format!("{:.2}", cell.get()))
            .collect::<Vec<_>>()
            .join(","),
    )
}

/// How many checkpoints ahead `state`'s target is.
const LOOK_AHEAD: usize = 12;
/// And its near target, for a slow car in a sharp turn.
const LOOK_NEAR: usize = 4;

fn run(line: &str) {
    let words: Vec<&str> = line.split_whitespace().collect();
    let [id, command, rest @ ..] = &words[..] else {
        return;
    };
    let ms =
        |w: Option<&&str>| Duration::from_millis(w.and_then(|m| m.parse().ok()).unwrap_or(100));
    let answer = match (*command, rest) {
        // The arcade's countdown, so a test can reach a level's end whatever the pilot's pace.
        ("time", [n]) => n.parse().ok().map(|n: u32| {
            driven_car().time_left.set(n);
            "ok".to_string()
        }),
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
                SDL_PushEvent(&raw mut event);
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
