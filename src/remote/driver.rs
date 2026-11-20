//! The test driver behind the `remote` door (cargo feature `remote`; shipped builds have none,
//! see `off.rs`). `RC2K_REMOTE` names a folder. `tools/remote.py` renames a file of commands to `cmd` in
//! it; each is run at the next pump and answered with a line in `reply`, `<id> <answer>`.
//! Keys and buttons are SDL events, so they take the path real ones take.
//!
//!     <id> key <scancode name> <ms>      a press of ms (a letter, digit or space also typed into a field being edited)
//!     <id> down|up <scancode name>       held until told
//!     <id> pad <button name> <ms>        a gamepad button (the menus' side)
//!     <id> joy attach                    a plain stand-in stick plugged in (4 axes, 17
//!                                        buttons, 1 hat; USB ids 0x1209 / 0x5253)
//!     <id> joy axis <n> <value>          its axis n set (-32768 .. 32767)
//!     <id> joy button <n> <0|1>          its button n let go or pressed
//!     <id> joy detach                    the stick unplugged
//!     <id> shot <name>                   the next frame to <name>.raw, BGRA; answers its size
//!     <id> window <w> <h>                the window's size set (its frames are shown scaled in it)
//!     <id> wshot <name>                  the window's own pixels (what the player sees) to <name>.raw, BGRA
//!     <id> state                         frames, race flag, menu, highlighted item
//!     <id> quit                          as closing the window
//!
//! `RC2K_SHOT_CLOCKS` (car 0's replay clocks, comma-separated): playing a replay, the frame
//! drawn after each of those ticks is saved as `replay_<clock>.raw` (BGRA; its size in
//! `reply`). From the replay's stage load on, the game's clock is the one it keeps for
//! recording a replay as pictures (`VIEWER_PANEL.picture_clock`), 10 ms a drawn frame
//! (`shots_loading`, `shot_frame_drawn`), and the random numbers start from one state, so the
//! picture is the same from run to run (the picture check, `scratchpad/shots.sh`).

use crate::gfx::owner::Gfx;
use crate::menu::owner::Menus;
use crate::owners::Owners;
use crate::platform::ffi::sdl_input;
use crate::race::owner::Race;
use crate::race::owner_course::Course;
use sdl3_sys::everything::*;
use std::borrow::Cow;
use std::io::Write;
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};
use std::time::Instant;

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
    /// The keys and buttons to let go, with the game's clock value (`tick_count`) they are due at.
    releases: Vec<(u32, Release)>,
    shot: Option<(String, String)>,
    /// The window's picture asked for: the command's id and the file's name.
    window_shot: Option<(String, String)>,
}

fn pending() -> std::sync::MutexGuard<'static, Pending> {
    static PENDING: OnceLock<Mutex<Pending>> = OnceLock::new();
    PENDING
        .get_or_init(Default::default)
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

/// The stand-in camera's frame (set by `camframe`).
fn fake_frame() -> std::sync::MutexGuard<'static, Option<crate::platform::ffi::sdl_camera::Frame>> {
    static FRAME: Mutex<Option<crate::platform::ffi::sdl_camera::Frame>> = Mutex::new(None);
    FRAME
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

/// Whether the save code page has a stand-in camera (`RC2K_FAKE_CAMERA` set): a test cannot
/// have a real one.
pub fn fake_camera() -> bool {
    std::env::var_os("RC2K_FAKE_CAMERA").is_some()
}

/// How many stand-in cameras there are (`RC2K_FAKE_CAMERA_COUNT`, at least one): the odd ones
/// face the user.
pub fn fake_camera_count() -> usize {
    std::env::var("RC2K_FAKE_CAMERA_COUNT")
        .ok()
        .and_then(|n| n.parse().ok())
        .unwrap_or(1)
        .max(1)
}

/// The stand-in camera's frame.
pub fn fake_camera_frame() -> Option<crate::platform::ffi::sdl_camera::Frame> {
    fake_frame()
        .as_ref()
        .map(|f| crate::platform::ffi::sdl_camera::Frame {
            width: f.width,
            height: f.height,
            rgba: f.rgba.clone(),
            facing_user: f.facing_user,
        })
}

/// A binary PGM ("P5 w h 255" and the grey bytes) as an RGBA frame.
fn load_pgm(path: &str) -> Option<crate::platform::ffi::sdl_camera::Frame> {
    let data = std::fs::read(path).ok()?;
    let mut at = 0;
    let mut words = Vec::new();
    while words.len() < 4 {
        while data.get(at)?.is_ascii_whitespace() {
            at += 1;
        }
        let start = at;
        while !data.get(at)?.is_ascii_whitespace() {
            at += 1;
        }
        words.push(std::str::from_utf8(&data[start..at]).ok()?.to_owned());
    }
    at += 1;
    let (w, h): (usize, usize) = (words[1].parse().ok()?, words[2].parse().ok()?);
    let grey = data.get(at..at.checked_add(w.checked_mul(h)?)?)?;
    let rgba = match fake_layout() {
        // The picture as a camera of that format would send it, with padded rows, read back
        // by the conversion the real camera's frames go through.
        Some(layout) => {
            let (bytes, pitch) = encode_grey(layout, w, h, grey);
            rc2k_core::gfx::camera_frame::to_rgba(layout, w, h, pitch, &bytes)?
        }
        None => grey.iter().flat_map(|&g| [g, g, g, 255]).collect(),
    };
    Some(crate::platform::ffi::sdl_camera::Frame {
        width: w,
        height: h,
        rgba,
        facing_user: std::env::var_os("RC2K_FAKE_CAMERA_FACING_USER").is_some(),
    })
}

/// The pixel layout `RC2K_FAKE_CAMERA` names (`rgba`, `bgra`, `rgb24`, `yuy2`, `yvyu`, `uyvy`,
/// `nv12`, `nv21`, `i420`, `yv12`); none for another value (frames then go in as RGBA).
fn fake_layout() -> Option<rc2k_core::gfx::camera_frame::Layout> {
    use rc2k_core::gfx::camera_frame::Layout;
    let name = std::env::var("RC2K_FAKE_CAMERA").ok()?;
    Some(match name.to_ascii_lowercase().as_str() {
        "rgba" => Layout::Packed32 { r: 0, g: 1, b: 2 },
        "bgra" => Layout::Packed32 { r: 2, g: 1, b: 0 },
        "rgb24" => Layout::Rgb24 { r: 0, g: 1, b: 2 },
        "yuy2" => Layout::Yuy2,
        "yvyu" => Layout::Yvyu,
        "uyvy" => Layout::Uyvy,
        "nv12" => Layout::Nv12,
        "nv21" => Layout::Nv21,
        "i420" => Layout::I420,
        "yv12" => Layout::Yv12,
        _ => return None,
    })
}

/// A grey picture in `layout`, rows padded by 8 bytes (as a camera's often are), and the pitch.
fn encode_grey(
    layout: rc2k_core::gfx::camera_frame::Layout,
    w: usize,
    h: usize,
    grey: &[u8],
) -> (Vec<u8>, usize) {
    use rc2k_core::gfx::camera_frame::Layout;
    let luma = |g: u8| (16 + u32::from(g) * 219 / 255) as u8;
    let (even_w, even_h) = (w.div_ceil(2) * 2, h.div_ceil(2) * 2);
    let at = |x: usize, y: usize| grey[y.min(h - 1) * w + x.min(w - 1)];
    match layout {
        Layout::Rgb24 { .. } | Layout::Packed32 { .. } => {
            let bytes = if matches!(layout, Layout::Rgb24 { .. }) {
                3
            } else {
                4
            };
            let pitch = w * bytes + 8;
            let mut out = vec![0x55; pitch * h];
            for y in 0..h {
                for x in 0..w {
                    out[y * pitch + x * bytes..y * pitch + x * bytes + 3].fill(at(x, y));
                }
            }
            (out, pitch)
        }
        Layout::Yuy2 | Layout::Yvyu | Layout::Uyvy => {
            let pitch = even_w * 2 + 8;
            let mut out = vec![0x55; pitch * h];
            for y in 0..h {
                for x in (0..even_w).step_by(2) {
                    let (a, b) = (luma(at(x, y)), luma(at(x + 1, y)));
                    let q = match layout {
                        Layout::Uyvy => [128, a, 128, b],
                        _ => [a, 128, b, 128],
                    };
                    out[y * pitch + x * 2..y * pitch + x * 2 + 4].copy_from_slice(&q);
                }
            }
            (out, pitch)
        }
        _ => {
            let pitch = even_w + 8;
            let cp = pitch.div_ceil(2);
            let chroma_rows = even_h / 2;
            let total = match layout {
                Layout::Nv12 | Layout::Nv21 => pitch * h + pitch * chroma_rows,
                _ => pitch * h + 2 * cp * chroma_rows,
            };
            let mut out = vec![128; total];
            for y in 0..h {
                for x in 0..w {
                    out[y * pitch + x] = luma(grey[y * w + x]);
                }
            }
            (out, pitch)
        }
    }
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
    sdl_input::push_key_event(scancode, down);
}

/// The character a key named `name` types (letters in lower case, digits, space, the slash), if any.
fn typed_char(name: &str) -> Option<char> {
    let mut chars = name.chars();
    match (chars.next(), chars.next()) {
        (Some(c), None) if c.is_ascii_alphanumeric() => Some(c.to_ascii_lowercase()),
        _ if name.eq_ignore_ascii_case("space") => Some(' '),
        _ if name == "/" => Some('/'),
        _ => None,
    }
}

fn pad(button: SDL_GamepadButton, down: bool) {
    sdl_input::push_gamepad_button_event(button, down);
}

/// The stand-in stick (`joy attach`): its id and its handle, open for setting its values.
static STICK: Mutex<Option<(SDL_JoystickID, sdl_input::Joystick)>> = Mutex::new(None);

/// `RC2K_REMOTE_STICK`: the stand-in stick plugged in as SDL's joysticks start, so the game
/// lists it with the others at its start (`joy attach` later is a pad plugged in while it runs).
pub fn stick_at_start() {
    if std::env::var_os("RC2K_REMOTE_STICK").is_some() && folder().is_some() {
        let answer = stick(&["attach"]);
        crate::program_log::log(&format!(
            "remote: the stand-in stick plugged in at the start: {}",
            answer.unwrap_or_else(|| "failed".to_string())
        ));
    }
}

fn stick(words: &[&str]) -> Option<String> {
    let mut held = STICK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    match words {
        ["attach"] if held.is_none() => {
            let id = sdl_input::attach_virtual_stick(4, 17, 1, 0x1209, 0x5253)?;
            let joystick = sdl_input::Joystick::open(id)?;
            *held = Some((id, joystick));
            Some("ok".to_string())
        }
        ["axis", n, value] => {
            let (_, joystick) = held.as_ref()?;
            joystick
                .set_virtual_axis(n.parse().ok()?, value.parse().ok()?)
                .then(|| "ok".to_string())
        }
        ["button", n, down @ ("0" | "1")] => {
            let (_, joystick) = held.as_ref()?;
            joystick
                .set_virtual_button(n.parse().ok()?, *down == "1")
                .then(|| "ok".to_string())
        }
        ["rumble"] => {
            let v = sdl_input::VIRTUAL_RUMBLE.load(std::sync::atomic::Ordering::Relaxed);
            Some(format!("{} {}", v >> 16, v & 0xFFFF))
        }
        ["detach"] => {
            let (id, joystick) = held.take()?;
            drop(joystick);
            sdl_input::detach_virtual_stick(id).then(|| "ok".to_string())
        }
        _ => None,
    }
}

fn scancode(name: &str) -> Option<SDL_Scancode> {
    // The commands' words are split at spaces: an underscore stands for one ("Keypad_0").
    sdl_input::scancode_from_name(&name.replace('_', " "))
}

fn button(name: &str) -> Option<SDL_GamepadButton> {
    sdl_input::gamepad_button_from_name(name)
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
fn driven_car<'a>(race: &'a Race, course: &Course) -> &'a crate::physics::car_store::Car {
    let viewed = race
        .car_blocks()
        .of(race.shared.view.get().view_blocks[0].get());
    match viewed {
        Some(car) if course.numbers.get().in_race.is_set() => car,
        _ => race.shared.car(),
    }
}

fn state(o: &Owners<'static>, course: &Course) -> String {
    let race = o.race;
    let screens = &o.menus.screens;
    #[allow(non_snake_case)]
    let CAR = driven_car(race, course);
    let at = CAR.checkpoint.get() as usize;
    // The stage line ahead, for `tools/remote.py pilot`.
    let ahead = (at + LOOK_AHEAD).min(race.checkpoints().map_x.len() - 1);
    let near = (at + LOOK_NEAR).min(race.checkpoints().map_x.len() - 1);
    format!(
        "renderer={:?} frames={} screen_frames={} race={} menu={:?} item={} focused={} speed={:.0} time={} splits={} finished={} stuck={} checkpoints={} checkpoint={} map={},{} target={},{} near={},{} channels={} mix={} heading={} gear={} box={} driver={} controls={},{},{},{},{},{} matrix={} cars={} shown={}",
        crate::render::running_wish(),
        o.gfx.drawing.draw_surface.frames.get(),
        screen_frames(o.gfx),
        u32::from(course.numbers.get().in_race.is_set()),
        screens.loop_flags.menu.get().ok(),
        screens.loop_flags.item.get(),
        o.system.session.flags.window_focused.get(),
        CAR.speed.get(),
        CAR.stage_time.get(),
        CAR.splits_passed.get(),
        CAR.finish_flag.get(),
        CAR.stuck_count.get(),
        race.shared.vars.get().checkpoint_count.get(),
        at,
        CAR.map_place[0].get() as i32,
        CAR.map_place[1].get() as i32,
        race.checkpoints().map_x[ahead].get() as i32,
        race.checkpoints().map_y[ahead].get() as i32,
        race.checkpoints().map_x[near].get() as i32,
        race.checkpoints().map_y[near].get() as i32,
        o.audio.state.channels_used.get(),
        LAST_PEAK.swap(0, std::sync::atomic::Ordering::Relaxed),
        CAR.car_heading.get() as i32,
        CAR.gear.get(),
        CAR.gearbox_kind.get(),
        CAR.driver_kind.get(),
        race.shared.physics_tables.get().input_steer.get() as i32,
        race.shared.physics_tables.get().input_throttle.get(),
        race.shared.physics_tables.get().input_brake.get(),
        race.shared.physics_tables.get().input_handbrake.get(),
        race.shared.physics_tables.get().input_gear_up.get(),
        race.shared.physics_tables.get().input_gear_down.get(),
        CAR.crash_matrix[..12]
            .iter()
            .map(|cell| format!("{:.2}", cell.get()))
            .collect::<Vec<_>>()
            .join(","),
        // The players' cars and the car the car choice shows, for the menus' checks.
        (0..crate::menu::player::PLAYERS)
            .filter_map(|p| crate::menu::player::player(o.menus, p))
            .map(|p| p.get().car_number.to_string())
            .collect::<Vec<_>>()
            .join(","),
        screens.event.shown.car.get(),
    )
}

/// The cars of the race in progress.
fn race_cars(race: &Race) -> &[crate::physics::car_store::Car] {
    let n = (race.shared.view.get().race_cars.get() as usize).min(8);
    &race.car_blocks().cars[..n]
}

/// The lobby chat box's text as the player sees it: its lines joined by `|`.
fn chat_lines(o: &Owners<'static>) -> String {
    let _received = o.net.lock_received();
    let chat = o.net.vars.chat.read();
    let end = chat.iter().position(|&b| b == 0).unwrap_or(chat.len());
    String::from_utf8_lossy(&chat[..end])
        .split('\n')
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join("|")
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
pub fn autodrive_keys(race: &Race, checkpoint: u32) -> u32 {
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
            crate::program_log::log(&format!(
                "remote: the reset key let go at checkpoint {checkpoint} (stage time {})",
                race.shared.car().stage_time.get()
            ));
        }
        return rc2k_core::drive::controls::RESET_KEY;
    }
    if *since >= STALLED_STEPS {
        crate::program_log::log(&format!(
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

fn run(o: &Owners<'static>, line: &str) {
    let course = &o.race.course;
    let words: Vec<&str> = line.split_whitespace().collect();
    let [id, command, rest @ ..] = &words[..] else {
        return;
    };
    // When a key or button is let go: its hold in the game's clock, which a held race counts in
    // frames and `RC2K_SPEED` runs fast, so a hold lasts the same number of game ticks at any speed.
    let due = |w: Option<&&str>| {
        crate::clock::tick_count::tick_count(&o.system.session.picture_clock)
            .wrapping_add(w.and_then(|m| m.parse().ok()).unwrap_or(100))
    };
    let answer = match (*command, rest) {
        // The arcade's countdown, so a test can reach a level's end whatever the pilot's pace.
        ("time", [n]) => n.parse().ok().map(|n: u32| {
            driven_car(o.race, course).time_left.set(n);
            "ok".to_string()
        }),
        ("key", [name, tail @ ..]) => scancode(name).map(|code| {
            key(code, true);
            // A real keyboard's key also gives SDL's text event, which a pushed key does not: a
            // field being edited takes its characters from that.
            if let Some(c) = typed_char(name) {
                crate::input::typed_text::typed_by_keyboard(&c.to_string());
            }
            pending()
                .releases
                .push((due(tail.first()), Release::Key(code)));
            "ok".to_string()
        }),
        // Text as a keyboard's input method or a screen keyboard gives it: the words joined by
        // single spaces, taken by the field being edited (accented letters included).
        ("text", words) if !words.is_empty() => {
            crate::input::typed_text::typed_by_keyboard(&words.join(" "));
            Some("ok".to_string())
        }
        ("down" | "up", [name]) => scancode(name).map(|code| {
            key(code, *command == "down");
            "ok".to_string()
        }),
        ("pad", [name, tail @ ..]) => button(name).map(|code| {
            pad(code, true);
            pending()
                .releases
                .push((due(tail.first()), Release::Pad(code)));
            "ok".to_string()
        }),
        // The right stick's vertical axis of the stand-in gamepad, -32768 .. 32767.
        ("padaxis", ["righty", value]) => value.parse::<i16>().ok().map(|v| {
            sdl_input::push_gamepad_axis_event(SDL_GAMEPAD_AXIS_RIGHTY, v);
            "ok".to_string()
        }),
        ("joy", words) => stick(words),
        // The save code page's camera, clipboard: a frame for the stand-in camera (a PGM file),
        // and the clipboard's text set from, or written to, a file.
        ("camframe", [path]) => load_pgm(path).map(|frame| {
            *fake_frame() = Some(frame);
            "ok".to_string()
        }),
        ("clipset", [path]) => std::fs::read_to_string(path)
            .ok()
            .filter(|text| crate::platform::ffi::sdl_clipboard::set_text(text))
            .map(|_| "ok".to_string()),
        ("clipget", [path]) => crate::platform::ffi::sdl_clipboard::text()
            .filter(|text| std::fs::write(path, text).is_ok())
            .map(|text| format!("ok {}", text.len())),
        ("shot", [name]) if plain_file_name(name) => {
            pending().shot = Some(((*id).to_string(), (*name).to_string()));
            return;
        }
        ("window", [width, height]) => width
            .parse::<usize>()
            .ok()
            .zip(height.parse::<usize>().ok())
            .map(|(w, h)| {
                if let Ok(screen) = crate::platform::SCREEN.lock()
                    && let Some(screen) = screen.as_ref()
                {
                    screen.window.set_size(w, h);
                }
                // The handheld's GL draws on its panel, not on a window of its own.
                #[cfg(not(handheld))]
                if let Some(gl) = crate::render::gles::window() {
                    gl.set_size(w, h);
                }
                "ok".to_string()
            }),
        ("wshot", [name]) if plain_file_name(name) => {
            pending().window_shot = Some(((*id).to_string(), (*name).to_string()));
            WINDOW_SHOT.store(true, std::sync::atomic::Ordering::Relaxed);
            return;
        }
        ("state", []) => Some(state(o, course)),
        // Each race car's checkpoint and place (x, z), for the network bots' checks.
        ("cars", []) => Some(
            race_cars(o.race)
                .iter()
                .enumerate()
                .map(|(n, car)| {
                    format!(
                        "{n}:{}@{},{}",
                        car.checkpoint.get(),
                        car.position[0].get() as i32,
                        car.position[2].get() as i32
                    )
                })
                .collect::<Vec<_>>()
                .join(" "),
        ),
        // The lobby chat box's lines, top to bottom, joined by `|` (for the network chat check).
        ("chat", []) => Some(chat_lines(o)),
        // The game's own computer driver at the player's car's wheel (`tools/remote.py pilot`).
        ("drive", [switch @ ("on" | "off")]) => {
            AUTODRIVE.store(*switch == "on", std::sync::atomic::Ordering::Relaxed);
            Some("ok".to_string())
        }
        // The 3D's renderer swapped in game: `gpu`, `gl` or `software`.
        ("renderer", [name]) => crate::render::choice::parse(name).map(|wish| {
            #[cfg(not(handheld))]
            let done = crate::platform::swap_renderer(o, wish);
            #[cfg(handheld)]
            let done: Result<(), String> = {
                let _ = wish;
                Err("the handheld swaps at its next start".into())
            };
            match done {
                Ok(()) => format!("ok {:?}", crate::render::running_wish()),
                Err(why) => format!("error: {why}; {:?} runs", crate::render::running_wish()),
            }
        }),
        // An invitation to a network game pretended accepted: the first session the TCP/IP
        // connection finds at `host` is joined from the menus (Steam is not involved).
        ("invite", [host]) if host.len() < 64 => {
            let mut field = [0u8; 64];
            field[..host.len()].copy_from_slice(host.as_bytes());
            o.net.setup().host_address.write(&field);
            crate::net::invite::offer(crate::net::invite::Invite {
                provider: crate::net::store::SP_TCPIP,
                instance: None,
            });
            Some("ok".to_string())
        }
        ("quit", []) => {
            sdl_input::push_quit_event();
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
fn screen_now(menus: &Menus, course: &Course) -> (Option<crate::menu::menus_store::MenuId>, u32) {
    let menu = menus.screens.loop_flags.menu.get().ok();
    (menu, u32::from(course.numbers.get().in_race.is_set()))
}

/// A change of screen noted at its first frame.
fn note_screen(menus: &Menus, gfx: &Gfx, course: &Course) {
    let (menu, race) = screen_now(menus, course);
    let frames = gfx.drawing.draw_surface.frames.get();
    let mut s = SCREEN
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    if (s.0, s.1) != (menu, race) {
        *s = (menu, race, frames);
    }
}

/// The frames drawn since the screen (menu or race) changed: a test presses keys once its
/// screen has drawn a few, not by waiting a fixed time.
fn screen_frames(gfx: &Gfx) -> u32 {
    let s = SCREEN
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    gfx.drawing.draw_surface.frames.get().wrapping_sub(s.2)
}

/// Whether `name` is one plain file name (no folder, no `..`, not absolute): a picture is
/// written inside the remote folder under it.
fn plain_file_name(name: &str) -> bool {
    let mut parts = std::path::Path::new(name).components();
    matches!(
        (parts.next(), parts.next()),
        (Some(std::path::Component::Normal(_)), None)
    ) && !name.contains(['/', '\\'])
}

/// At each pump: the releases due, then the commands waiting.
pub fn poll(o: &Owners<'static>, course: &Course) {
    let Some(folder) = folder() else { return };
    send_window_shot(folder);
    note_screen(o.menus, o.gfx, course);
    let now = Instant::now();
    note_slow_frame(o.race, o.gfx, course, now);
    let clock = crate::clock::tick_count::tick_count(&o.system.session.picture_clock);
    pending().releases.retain(|(when, release)| {
        if (clock.wrapping_sub(*when) as i32) < 0 {
            return true;
        }
        match release {
            Release::Key(code) => key(*code, false),
            Release::Pad(code) => pad(*code, false),
        }
        false
    });
    // The file is renamed first: one written under the name after that is a new command file for
    // the next pump, not deleted unread. Read as bytes: a file that is not UTF-8 is still taken
    // (and removed) rather than read again, and failing again, at every pump.
    let path = folder.join("cmd");
    let taken = folder.join("cmd.taken");
    if std::fs::rename(&path, &taken).is_ok() {
        let bytes = std::fs::read(&taken).unwrap_or_default();
        let _ = std::fs::remove_file(&taken);
        String::from_utf8_lossy(&bytes)
            .lines()
            .for_each(|line| run(o, line));
    }
}

/// With `RC2K_SLOW_FRAMES=<ms>`, a log line for each pump in a race that comes more than
/// that many milliseconds after the one before, with the driven car's checkpoint: where on
/// the stage the frame rate drops (the five-second lines smooth the drops away).
fn note_slow_frame(race: &Race, gfx: &Gfx, course: &Course, now: Instant) {
    static LIMIT: OnceLock<Option<u128>> = OnceLock::new();
    static LAST: Mutex<Option<Instant>> = Mutex::new(None);
    let Some(limit) = *LIMIT.get_or_init(|| {
        std::env::var("RC2K_SLOW_FRAMES")
            .ok()
            .and_then(|v| v.trim().parse().ok())
    }) else {
        return;
    };
    let mut last = LAST
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let before = last.replace(now);
    let Some(before) = before else { return };
    let ms = now.duration_since(before).as_millis();
    if ms > limit && course.numbers.get().in_race.is_set() {
        let car = driven_car(race, course);
        crate::program_log::log(&format!(
            "remote: slow frame {ms} ms at checkpoint {} (frame {})",
            car.checkpoint.get(),
            gfx.drawing.draw_surface.frames.get()
        ));
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
pub fn shots_loading(
    race: &Race,
    effects: &crate::effects::owner::Effects,
    clock: &crate::clock::tick_count::PictureClock,
) {
    if !shots_on() {
        return;
    }
    clock.started.set(1);
    clock.clock.set(0x10000);
    reseed(&race.shared.random, effects);
    // The scene's clock as the menus left it (their time on screen) is the snow's first
    // step's start: the held clock's from here.
    race.shared.physics_tables.get().scene_clock.set(0x10000u32);
    // The frame rate's text measures the host: not drawn.
    race.frame.counter().show.set(0);
}

/// The speed asked for with `RC2K_SPEED=N` (1 when unset or not a number from 2 to 100000):
/// the game's clock runs N times the wall clock and its sleeps and frame waits are N times
/// shorter, so scripted runs take a fraction of the wall time. Races that must be exactly
/// repeatable are also held (`RC2K_HELD_RACE`): the simulation then reads the held clock
/// and does not see the speed.
pub fn speed() -> u32 {
    static SPEED: OnceLock<u32> = OnceLock::new();
    *SPEED.get_or_init(|| {
        std::env::var("RC2K_SPEED")
            .ok()
            .and_then(|s| s.parse::<u32>().ok())
            .filter(|n| (2..=100_000).contains(n))
            .unwrap_or(1)
    })
}

/// A wait of `milliseconds` of the game's time, in wall microseconds at the asked speed.
pub fn wall_micros(milliseconds: u32) -> u64 {
    u64::from(milliseconds) * 1000 / u64::from(speed())
}

/// With `RC2K_HELD_RACE` (a check: two runs of a driven race the same tick for tick,
/// `RC2K_SIM_HASH`): the race's clock from its first frame on, 10 ms a frame from a fixed
/// start (the race keeps clock values in its state: a start at the wall clock made two runs
/// differ from the third tick on; 0: not held).
static HELD_CLOCK: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);

/// The held race's first clock value: later than any menu's before it (the game's clock
/// starts at 0x10000), so the clock never runs backwards.
const HELD_START: u32 = 0x0100_0000;

/// The held clock's step for each frame drawn, in milliseconds: 10, or `RC2K_HELD_STEP` (1 to
/// 10) to run frames faster than the ticks (with `RC2K_BLEND_HELD`, frames between ticks).
fn held_step() -> u32 {
    static STEP: OnceLock<u32> = OnceLock::new();
    *STEP.get_or_init(|| {
        std::env::var("RC2K_HELD_STEP")
            .ok()
            .and_then(|s| s.parse::<u32>().ok())
            .filter(|n| (1..=10).contains(n))
            .unwrap_or(10)
    })
}

/// A held race was asked for (`RC2K_HELD_RACE`, remote builds).
pub fn held_race() -> bool {
    static ON: OnceLock<bool> = OnceLock::new();
    folder().is_some() && *ON.get_or_init(|| std::env::var_os("RC2K_HELD_RACE").is_some())
}

/// A race's stage about to load (`race::load_stage`): with `RC2K_HELD_RACE`, the random
/// numbers from one state (the menus before took a number of them that depends on the
/// timing), the computer at the players' wheels (`drive on`), the clock free until the first
/// frame.
pub fn race_loading(race: &Race, effects: &crate::effects::owner::Effects) {
    if !held_race() {
        return;
    }
    reseed(&race.shared.random, effects);
    AUTODRIVE.store(true, std::sync::atomic::Ordering::Relaxed);
    HELD_CLOCK.store(0, std::sync::atomic::Ordering::Relaxed);
}

/// The race's loop is about to run: with `RC2K_HELD_RACE`, the held clock starts here, one frame
/// before `HELD_START` (the first frame drawn brings it there), so that the weather's first
/// frames (the snow's first step, the rain's drift) see the same clock in every run.
pub fn race_loop_begins() {
    if held_race() {
        HELD_CLOCK.store(HELD_START - 10, std::sync::atomic::Ordering::Relaxed);
    }
}

/// The game's clock while a race is held (`clock::tick_count`).
pub fn held_clock() -> Option<u32> {
    let held = HELD_CLOCK.load(std::sync::atomic::Ordering::Relaxed);
    (held != 0).then_some(held)
}

/// Both random generators (the game's and the effects') from one state.
fn reseed(random: &crate::math::next_random::Randoms, effects: &crate::effects::owner::Effects) {
    crate::menu::launch::reseed(random, effects, 0x1234_5678, 0x8765_4321);
}

/// A race frame drawn (`drive::race_frame`): the held clock 10 ms on, unless the game is
/// recording pictures itself (`race::replay_screenshot` moves it then).
pub fn shot_frame_drawn(race: &Race, picture_clock: &crate::clock::tick_count::PictureClock) {
    if held_race() && race.course.numbers.get().in_race.is_set() {
        use std::sync::atomic::Ordering::Relaxed;
        let held = HELD_CLOCK.load(Relaxed);
        let next = if held == 0 {
            HELD_START
        } else {
            held.wrapping_add(held_step())
        };
        HELD_CLOCK.store(next.max(1), Relaxed);
    }
    if !shots_on() {
        return;
    }
    let panel = race.viewer_panel();
    if picture_clock.started.get() == 1 && panel.replay_recording.get() != 1 {
        picture_clock
            .clock
            .set(picture_clock.clock.get().wrapping_add(10));
    }
}

/// Whether the next frame is to be saved: the GPU's picture is then wanted in the canvas.
pub fn shot_wanted() -> bool {
    pending().shot.is_some()
}

/// A window's picture is wanted (`wshot`): the backend that shows the next frame takes it.
static WINDOW_SHOT: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// The window's picture taken by a backend: its width, height and BGRA pixels, top row first.
static WINDOW_PIXELS: Mutex<Option<(u32, u32, Vec<u8>)>> = Mutex::new(None);

/// Whether the frame being shown is to leave the window's picture (`window_taken`).
pub fn window_shot_due() -> bool {
    WINDOW_SHOT.swap(false, std::sync::atomic::Ordering::Relaxed)
}

/// The window's picture, taken from the backend that showed the frame.
pub fn window_taken(width: u32, height: u32, bgra: Vec<u8>) {
    *WINDOW_PIXELS
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner) = Some((width, height, bgra));
}

/// The window's picture written for the command that asked, once a backend has taken it.
fn send_window_shot(folder: &std::path::Path) {
    let Some((width, height, pixels)) = WINDOW_PIXELS
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .take()
    else {
        return;
    };
    let Some((id, name)) = pending().window_shot.take() else {
        return;
    };
    let answer = match std::fs::write(folder.join(format!("{name}.raw")), pixels) {
        Ok(()) => format!("{width} {height}"),
        Err(e) => format!("error: {e}"),
    };
    reply(&id, &answer);
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
pub fn mangled(data: &[u8]) -> Cow<'_, [u8]> {
    use std::sync::atomic::{AtomicU64, Ordering::Relaxed};
    static STATE: AtomicU64 = AtomicU64::new(0);
    static SEED: OnceLock<Option<u64>> = OnceLock::new();
    let Some(seed) = *SEED.get_or_init(|| {
        std::env::var("RC2K_MANGLE")
            .ok()
            .and_then(|s| s.parse().ok())
    }) else {
        return Cow::Borrowed(data);
    };
    // a sequence byte, then the tag: 'PGSM' and its like
    if !matches!(data.get(2..5), Some(b"GSM")) {
        return Cow::Borrowed(data);
    }
    let mut out = data.to_vec();
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
    Cow::Owned(out)
}

/// The loudest sample sent to the device since the remote control last asked.
static LAST_PEAK: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);

/// The samples about to go to the device: their peak noted for `state`.
pub fn note_sound_peak(pcm: &[i16]) {
    if let Some(peak) = pcm.iter().map(|sample| sample.unsigned_abs() as u32).max() {
        LAST_PEAK.fetch_max(peak, std::sync::atomic::Ordering::Relaxed);
    }
}

/// A race is loaded or not (the mix hash starts a fresh count with each race).
pub fn note_racing(racing: bool) {
    crate::audio::mix_hash::set_racing(racing);
}

/// The audio thread has taken a turn's end command.
pub fn turn_end_taken() {
    crate::audio::mix_hash::turn_end_taken();
}

/// The audio thread's turn is done.
pub fn audio_turn_done() {
    crate::audio::mix_hash::audio_turn_done();
}

/// With `RC2K_SOUND_LOG`: the sounds loaded, `voices` in use of `limit` after the list `list`.
pub fn sound_loaded(voices: u32, limit: impl std::fmt::Display, list: &impl std::fmt::Debug) {
    if std::env::var_os("RC2K_SOUND_LOG").is_some() {
        crate::program_log::log(&format!(
            "sound log: {voices} voices of {limit} after loading {list:?}"
        ));
    }
}

/// With `RC2K_SOUND_LOG`: a pace note's call for `car` at `checkpoint`, the note group `group`.
pub fn pace_call(car: u32, checkpoint: u32, group: u32) {
    if std::env::var_os("RC2K_SOUND_LOG").is_some() {
        crate::program_log::log(&format!(
            "pace call: car {car} checkpoint {checkpoint} group {group}"
        ));
    }
}

/// With `RC2K_SOUND_LOG`: the sounds the race starts (command 1) counted, and every 100 turns
/// logged as "sound log: sound x times" in sound order.
pub fn count_sound(racing: bool, command: u32, first_word: u32, is_turn_end: bool) {
    use std::collections::BTreeMap;
    static ON: OnceLock<bool> = OnceLock::new();
    static COUNTS: Mutex<(BTreeMap<u32, u32>, u32)> = Mutex::new((BTreeMap::new(), 0));
    if !*ON.get_or_init(|| std::env::var_os("RC2K_SOUND_LOG").is_some()) || !racing {
        return;
    }
    let Ok(mut counts) = COUNTS.lock() else {
        return;
    };
    if command == 1 {
        *counts.0.entry(first_word & 0xFFFF).or_default() += 1;
    } else if is_turn_end {
        counts.1 += 1;
        if counts.1 % 100 == 0 {
            let text: Vec<String> = counts
                .0
                .iter()
                .map(|(s, n)| format!("{s:#x}x{n}"))
                .collect();
            crate::program_log::log(&format!("sound log, turn {}: {}", counts.1, text.join(" ")));
            counts.0.clear();
        }
    }
}

/// What the driver's trials read: the surface effect asked for, once a second of the race's
/// 40 ms steps (the stand-in stick has no haptics to measure it on).
pub fn note_shake(shake: &rc2k_core::input::feedback::SurfaceEffect, engine_vibration: bool) {
    use std::sync::atomic::{AtomicU32, Ordering::Relaxed};
    static CALLS: AtomicU32 = AtomicU32::new(0);
    if CALLS.fetch_add(1, Relaxed).is_multiple_of(25) {
        crate::program_log::log(&format!(
            "feedback: surface effect row {} at speed {} (steering kind {}), engine vibration {}",
            shake.row,
            shake.speed,
            shake.kind,
            if engine_vibration { "on" } else { "off" }
        ));
    }
}

/// `RC2K_LOSE_FRAMES=n`: every n-th race frame's lock counts as refused, so the skipped-frame
/// path can be run.
pub fn lose_this_frame() -> bool {
    use std::sync::atomic::{AtomicU32, Ordering::Relaxed};
    static EVERY: OnceLock<u32> = OnceLock::new();
    static COUNT: AtomicU32 = AtomicU32::new(0);
    let every = *EVERY.get_or_init(|| {
        std::env::var("RC2K_LOSE_FRAMES")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(0)
    });
    every != 0 && COUNT.fetch_add(1, Relaxed) % every == every - 1
}

/// `RC2K_STEAM_OFF`: Steam's library is kept out entirely.
#[cfg_attr(
    not(steam_support),
    allow(dead_code, reason = "only Steam's start asks")
)]
pub fn steam_off() -> bool {
    std::env::var_os("RC2K_STEAM_OFF").is_some()
}

/// `RC2K_FAIL_RENDERER` (a list of steps): whether renderer step `step` is made to fail, to test
/// the fallbacks.
pub fn renderer_step_fails(step: &str) -> bool {
    let fails =
        std::env::var("RC2K_FAIL_RENDERER").is_ok_and(|steps| steps.split(',').any(|s| s == step));
    if fails {
        crate::program_log::log(&format!(
            "render: step `{step}` is made to fail (RC2K_FAIL_RENDERER)"
        ));
    }
    fails
}

/// `RC2K_FAKE_REFRESH`: a display's refresh rate the scripted run stands in for, or the real one.
pub fn refresh_rate(real: Option<f64>) -> Option<f64> {
    std::env::var("RC2K_FAKE_REFRESH")
        .ok()
        .and_then(|s| s.parse::<f64>().ok())
        .or(real)
}

/// `duration` of the wall clock as milliseconds of the game's clock at the asked speed.
pub fn scaled_millis(duration: std::time::Duration) -> u128 {
    duration.as_micros() * u128::from(speed()) / 1000
}

/// The window's picture, when a `wshot` asked for it and this frame is the one to take it from:
/// read from the renderer.
#[cfg_attr(
    handheld,
    allow(dead_code, reason = "the handheld's panel is not an SDL window")
)]
pub fn window_shot_taken(renderer: crate::platform::ffi::sdl::Renderer) {
    if window_shot_due()
        && let Some((width, height, pixels)) = renderer.read_window()
    {
        window_taken(width, height, pixels);
    }
}

/// The hashes of the stage's stores after loading (`after` "load") or freeing it ("free").
pub fn course_hashes(race: &Race, scene: &crate::scene::owner::Scene, after: &str) {
    crate::race::course_hash::log_hashes(race, scene, after);
}

/// The texture rows' hash started anew for a stage's load.
pub fn reset_texture_hash() {
    crate::race::course_hash::reset_texture_hash();
}

/// A texture row uploaded, for the texture rows' hash.
pub fn texture_row(texture: u32, level: u32, y: u32, row: &[u8]) {
    crate::race::course_hash::note_texture_row(texture, level, y, row);
}

/// The hashes of what the marks' masks leave.
pub fn mask_hashes(
    masks: &crate::effects::mask_store::Masks,
    drops: &crate::effects::windscreen_drop_vars_store::WindscreenDrops,
    effects: &crate::effects::owner::Effects,
) {
    crate::race::course_hash::log_mask_hashes(masks, drops, effects);
}

/// The hashes of what the stage's start leaves in the checkpoints and the race variables.
pub fn started_hashes(race: &Race) {
    crate::race::course_hash::log_started_hashes(race);
}

/// The step of a pixel's value into the running fold of the rescaled picture's pixels.
pub fn fold_pixel(fold: u64, pixel: u32) -> u64 {
    fold.rotate_left(5).wrapping_mul(0x0100_0000_01B3) ^ u64::from(pixel)
}

/// While the file `hold` in the remote folder exists, every loading bar frame lasts as many
/// milliseconds as it holds, so that the loading screen can be photographed; the remote
/// commands are answered meanwhile.
pub fn hold_loading_frame(o: crate::owners::Owners<'static>) {
    let Some(folder) = std::env::var_os("RC2K_REMOTE") else {
        return;
    };
    let held = std::fs::read_to_string(std::path::Path::new(&folder).join("hold"));
    if let Some(ms) = held.ok().and_then(|v| v.trim().parse::<u64>().ok()) {
        // The test commands are answered while the frame is held (a window shot asked for in
        // the hold is taken from the next frame).
        for _ in 0..ms.div_ceil(100) {
            poll(&o, &o.race.course);
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
    }
}

/// With `RC2K_SIM_HASH` (a check, as `RC2K_FAN_HASH` for the drawing): the car blocks and the random state hashed after each tick and logged,
/// so that two runs of a replay can be compared tick by tick (the simulation must stay
/// bit-identical). The blocks are hashed by field path and value (`drive::sim_hash`), so the
/// records' layout does not matter.
pub fn sim_hash_step(race: &Race) {
    use std::sync::OnceLock;
    use std::sync::atomic::{AtomicU32, Ordering::Relaxed};
    static ON: OnceLock<bool> = OnceLock::new();
    static TICKS: AtomicU32 = AtomicU32::new(0);
    if !*ON.get_or_init(|| std::env::var_os("RC2K_SIM_HASH").is_some()) {
        return;
    }
    let mut groups = crate::drive::sim_hash::car_blocks_groups(race.shared.car_blocks.get());
    let staging = race.replay.staging();
    let mut bytes = vec![0; core::mem::size_of_val(staging)];
    if crate::memory::cells::load_cells(staging, 0, &mut bytes).is_err() {
        return;
    }
    groups.push(crate::drive::sim_hash::staging_group(&bytes));
    let tick = TICKS.fetch_add(1, Relaxed);
    if std::env::var_os("RC2K_SIM_FIELDS").is_some() && tick < 40 {
        crate::drive::sim_hash::log_fields(race.shared.car_blocks.get(), tick);
    }
    crate::program_log::log(&format!(
        "sim hash {tick}: {:016x}, random {:08x}",
        crate::drive::sim_hash::total(&groups),
        race.shared.random.state.get()
    ));
    // With `RC2K_SIM_CHUNKS` too: each car block's hash, to find which part of the blocks
    // differs between two runs.
    if std::env::var_os("RC2K_SIM_CHUNKS").is_some() {
        let chunks: Vec<String> = groups
            .iter()
            .map(|(name, hash)| format!("{name}={:04x}", hash & 0xFFFF))
            .collect();
        crate::program_log::log(&format!("sim chunks {tick}: {}", chunks.join(" ")));
    }
}

#[cfg(test)]
mod tests {
    use super::plain_file_name;

    #[test]
    fn picture_names_stay_inside_the_folder() {
        for good in ["menu", "replay_100", "page1edit", "a.b"] {
            assert!(plain_file_name(good), "{good}");
        }
        for bad in ["", ".", "..", "../x", "a/b", "/abs", "a\\b", "x/.."] {
            assert!(!plain_file_name(bad), "{bad}");
        }
    }
}
