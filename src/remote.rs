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
//!
//! `RC2K_SHOT_CLOCKS` (car 0's replay clocks, comma-separated): playing a replay, the frame
//! drawn after each of those ticks is saved as `replay_<clock>.raw` (BGRA; its size in
//! `reply`). From the replay's stage load on, the game's clock is the one it keeps for
//! recording a replay as pictures (`VIEWER_PANEL.picture_clock`), 10 ms a drawn frame
//! (`shots_loading`, `shot_frame_drawn`), and the random numbers start from one state, so the
//! picture is the same from run to run (the picture check, `scratchpad/shots.sh`).

#![allow(unsafe_code)]

use crate::race::owner::race;
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

/// A file the game opens, listed in `opened` for tests that damage what is read.
pub fn note_open(path: &[u8]) {
    use std::io::Write;
    if let Some(folder) = folder()
        && let Ok(mut list) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(folder.join("opened"))
    {
        let _ = list.write_all(path);
        let _ = list.write_all(b"\n");
    }
}

/// The car `state` reports: the viewed car's block in a race. With several cars the work car
/// holds whichever was stepped last between frames.
fn driven_car() -> &'static crate::physics::car_store::Car {
    let viewed =
        crate::race::blocks_store::car_of(crate::race::view_store::RACE_VIEW.view_blocks[0].get());
    match viewed {
        Some(car) if race().course.numbers.in_race.get() != 0 => car,
        _ => &crate::physics::car_store::CAR,
    }
}

fn state() -> String {
    use crate::menu::loop_store::LOOP_FLAGS;
    use crate::race::checkpoints_store::{CHECKPOINTS, cell_in};
    #[allow(non_snake_case)]
    let CAR = driven_car();
    let at = CAR.checkpoint.get() as usize;
    // The stage line ahead, for `tools/remote.py pilot`.
    let ahead = (at + LOOK_AHEAD).min(CHECKPOINTS.map_x.len() - 1);
    let near = (at + LOOK_NEAR).min(CHECKPOINTS.map_x.len() - 1);
    format!(
        "frames={} screen_frames={} race={} menu={:?} item={} focused={} speed={:.0} time={} splits={} finished={} stuck={} checkpoints={} checkpoint={} map={},{} target={},{} near={},{} channels={} mix={} heading={} gear={} box={} driver={} controls={},{},{},{},{},{} matrix={} cars={} shown={}",
        crate::gfx::screen_state::DRAW_SURFACE.frames.get(),
        screen_frames(),
        race().course.numbers.in_race.get(),
        LOOP_FLAGS.menu.get().ok(),
        LOOP_FLAGS.item.get(),
        crate::session::FLAGS.window_focused.get(),
        CAR.speed.get(),
        CAR.stage_time.get(),
        CAR.splits_passed.get(),
        CAR.finish_flag.get(),
        CAR.stuck_count.get(),
        crate::race::vars_store::RACE_VARS.checkpoint_count.get(),
        at,
        CAR.map_place[0].get() as i32,
        CAR.map_place[1].get() as i32,
        cell_in(&CHECKPOINTS.map_x, ahead).get() as i32,
        cell_in(&CHECKPOINTS.map_y, ahead).get() as i32,
        cell_in(&CHECKPOINTS.map_x, near).get() as i32,
        cell_in(&CHECKPOINTS.map_y, near).get() as i32,
        crate::audio::store::AUDIO.channels_used.get(),
        crate::platform_sound::LAST_PEAK.swap(0, std::sync::atomic::Ordering::Relaxed),
        CAR.car_heading.get() as i32,
        CAR.gear.get() as i32,
        CAR.gearbox_kind.get(),
        CAR.driver_kind.get(),
        crate::physics::tables_store::PHYSICS.input_steer.get() as i32,
        crate::physics::tables_store::PHYSICS.input_throttle.get(),
        crate::physics::tables_store::PHYSICS.input_brake.get(),
        crate::physics::tables_store::PHYSICS.input_handbrake.get(),
        crate::physics::tables_store::PHYSICS.input_gear_up.get(),
        crate::physics::tables_store::PHYSICS.input_gear_down.get(),
        CAR.crash_matrix[..12]
            .iter()
            .map(|cell| format!("{:.2}", cell.get()))
            .collect::<Vec<_>>()
            .join(","),
        // The players' cars and the car the car choice shows, for the menus' checks.
        (0..crate::menu::player::PLAYERS)
            .filter_map(crate::menu::player::player)
            .map(|p| p.get().car_number.to_string())
            .collect::<Vec<_>>()
            .join(","),
        crate::menu::event_store::EVENT_STATE.shown_car.get(),
    )
}

/// Whether the computer drives the players' cars (`drive on`).
static AUTODRIVE: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// Whether the computer drives the players' cars, as it drives its own
/// (`drive::read_car_controls`).
pub fn autodrive() -> bool {
    AUTODRIVE.load(std::sync::atomic::Ordering::Relaxed)
}

/// The steps without a new checkpoint after which the computer at a player's wheel asks
/// for the reset (about 10 s), and how long it holds the reset key (the game resets after
/// 100 steps held while slow, `drive::reset_request`).
const STALLED_STEPS: u32 = 1000;
const RESET_HELD_STEPS: u32 = 150;

/// The computer's progress at a player's wheel: the furthest checkpoint, the steps since
/// it, the steps the reset key is still held.
static PROGRESS: std::sync::Mutex<(u32, u32, u32)> = std::sync::Mutex::new((0, 0, 0));

/// The keys the computer at a player's wheel adds this step, the car at `checkpoint`: the
/// reset key (`RESET_KEY`) while it holds it, after `STALLED_STEPS` without progress (the
/// computer's own reversing does not always free a car, and its fourth switch, which
/// would ask for more, is read by nothing).
pub fn autodrive_keys(checkpoint: u32) -> u32 {
    let mut p = PROGRESS
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let (best, since, held) = &mut *p;
    if checkpoint > *best || checkpoint + 50 < *best {
        // On, or a new stage (the checkpoint far behind).
        (*best, *since) = (checkpoint, 0);
    } else {
        *since += 1;
    }
    if *held > 0 {
        *held -= 1;
        if *held == 0 {
            crate::framework::log(&format!(
                "remote: the reset key let go at checkpoint {checkpoint} (stage time {})",
                crate::physics::car_store::CAR.stage_time.get()
            ));
        }
        return rc2k_core::drive::controls::RESET_KEY;
    }
    if *since >= STALLED_STEPS {
        crate::framework::log(&format!(
            "remote: stalled at checkpoint {checkpoint}: the reset key held"
        ));
        (*since, *held) = (0, RESET_HELD_STEPS);
        return rc2k_core::drive::controls::RESET_KEY;
    }
    0
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
        // The game's own computer driver at the player's car's wheel (`tools/remote.py pilot`).
        ("drive", [switch @ ("on" | "off")]) => {
            AUTODRIVE.store(*switch == "on", std::sync::atomic::Ordering::Relaxed);
            Some("ok".to_string())
        }
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

/// The screen now (the menu and the race flag) and the frame count when it began.
static SCREEN: std::sync::Mutex<(Option<crate::menu::menus_store::MenuId>, u32, u32)> =
    std::sync::Mutex::new((None, 0, 0));

/// The screen's identity: its menu (none outside the menus) and the race flag.
fn screen_now() -> (Option<crate::menu::menus_store::MenuId>, u32) {
    let menu = crate::menu::loop_store::LOOP_FLAGS.menu.get().ok();
    (menu, race().course.numbers.in_race.get())
}

/// A change of screen noted at its first frame.
fn note_screen() {
    let (menu, race) = screen_now();
    let frames = crate::gfx::screen_state::DRAW_SURFACE.frames.get();
    let mut s = SCREEN
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    if (s.0, s.1) != (menu, race) {
        *s = (menu, race, frames);
    }
}

/// The frames drawn since the screen (menu or race) changed: a test presses keys once its
/// screen has drawn a few, not by waiting a fixed time.
fn screen_frames() -> u32 {
    let s = SCREEN
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    crate::gfx::screen_state::DRAW_SURFACE
        .frames
        .get()
        .wrapping_sub(s.2)
}

/// At each pump: the releases due, then the commands waiting.
pub fn poll() {
    let Some(folder) = folder() else { return };
    note_screen();
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

/// The replay clocks to save the frame after (`RC2K_SHOT_CLOCKS`).
fn shot_clocks() -> &'static [u32] {
    static CLOCKS: OnceLock<Vec<u32>> = OnceLock::new();
    CLOCKS.get_or_init(|| {
        std::env::var("RC2K_SHOT_CLOCKS")
            .map(|v| v.split(',').filter_map(|c| c.trim().parse().ok()).collect())
            .unwrap_or_default()
    })
}

/// A replay's tick played, car 0's clock `clock`: a shot asked for when it is one of
/// `RC2K_SHOT_CLOCKS`.
pub fn replay_tick(clock: u32) {
    if folder().is_some() && shot_clocks().contains(&clock) {
        pending().shot = Some(("replay".to_string(), format!("replay_{clock}")));
    }
}

/// Whether shots are asked for (`RC2K_SHOT_CLOCKS` in a remote-driven run).
pub fn shots_on() -> bool {
    folder().is_some() && !shot_clocks().is_empty()
}

/// The replay's stage about to load (`race::viewer_load_stage`): with `RC2K_SHOT_CLOCKS`
/// set, the game's clock held as when it records a replay as pictures (`tick_count` reads
/// `picture_clock`), from one value, and the random numbers (the camera's shake, the
/// replay's cameras, some sounds) from one state: the menus before took a number of them
/// that depends on the timing (the effects' generator builds the snow's tables at load).
/// The viewer then plays at once (`shots_on`): no frame waits for a key.
pub fn shots_loading() {
    use crate::race::viewer_screen_store::VIEWER_PANEL;
    if !shots_on() {
        return;
    }
    VIEWER_PANEL.pictures_started.set_bits(1u32);
    VIEWER_PANEL.picture_clock.set(0x10000);
    reseed();
    // The scene's clock as the menus left it (their time on screen) is the snow's first
    // step's start: the held clock's from here.
    crate::physics::tables_store::PHYSICS
        .scene_clock
        .set(0x10000u32);
    // The frame rate's text measures the host: not drawn.
    race().frame.counter.show.set(0);
}

/// Both random generators (the game's and the effects' `g_marksSeed`) from one state.
fn reseed() {
    crate::math::next_random::with_random(|r| r.state = 0x1234_5678);
    crate::effects::work_store::MARKS_WORK.seed.set(0x8765_4321);
}

/// A race frame drawn (`drive::race_frame`): the held clock 10 ms on, unless the game is
/// recording pictures itself (`race::replay_screenshot` moves it then).
pub fn shot_frame_drawn() {
    use crate::race::viewer_screen_store::VIEWER_PANEL;
    if !shots_on() {
        return;
    }
    if VIEWER_PANEL.pictures_started.bits() == 1 && VIEWER_PANEL.replay_recording.bits() != 1 {
        VIEWER_PANEL
            .picture_clock
            .set(VIEWER_PANEL.picture_clock.bits().wrapping_add(10));
    }
}

/// Whether the next frame is to be saved: the GPU's picture is then wanted in the canvas.
pub fn shot_wanted() -> bool {
    pending().shot.is_some()
}

/// After a frame is shown: the screenshot asked for.
pub fn frame(pixels: &[u8], width: u32, height: u32) {
    let Some(folder) = folder() else { return };
    let Some((id, name)) = pending().shot.take() else {
        return;
    };
    let answer = match std::fs::write(folder.join(format!("{name}.raw")), pixels) {
        Ok(()) => format!("{width} {height}"),
        Err(e) => format!("error: {e}"),
    };
    reply(&id, &answer);
}

/// A race message as it is to go out: with `RC2K_MANGLE` set (a seed), some of its dwords
/// replaced, for trying what a hostile machine can do to the others' games.
pub fn mangled(data: &[u8]) -> Vec<u8> {
    use std::sync::atomic::{AtomicU64, Ordering::Relaxed};
    static STATE: AtomicU64 = AtomicU64::new(0);
    static SEED: OnceLock<Option<u64>> = OnceLock::new();
    let mut out = data.to_vec();
    let Some(seed) = *SEED.get_or_init(|| {
        std::env::var("RC2K_MANGLE")
            .ok()
            .and_then(|s| s.parse().ok())
    }) else {
        return out;
    };
    // a sequence byte, then the tag: 'PGSM' and its like
    if !matches!(data.get(2..5), Some(b"GSM")) {
        return out;
    }
    let next = || {
        let mut x = STATE.load(Relaxed) ^ seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        STATE.store(x, Relaxed);
        x
    };
    for dword in out[5..].as_chunks_mut::<4>().0 {
        let r = next();
        // a seed from 100: rarely, for single bad values that pass the receiver's checks
        if r & (if seed < 100 { 7 } else { 0x7F }) == 0 {
            let value: u32 = match (r >> 8) & 7 {
                0 => 0,
                1 => u32::MAX,
                2 => 0x7FFF_FFFF,
                3 => 0x8000_0000,
                4 => 0x7FC0_0000,
                5 => 0x7F80_0000,
                _ => (r >> 16) as u32,
            };
            dword.copy_from_slice(&value.to_le_bytes());
        }
    }
    out
}
