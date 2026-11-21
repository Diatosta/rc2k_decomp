//! The program's joysticks and wheels: the backend of DirectInput's objects made
//! in Rust (`input::di_objects`) on SDL's joysticks and haptics.
//!
//! - The devices: SDL's joysticks in the order first seen (a device unplugged keeps its place and
//!   reads at rest, its handles closed), each opened when seen: its name, a product GUID (SDL's),
//!   an instance GUID made from SDL's and its instance number, force feedback when SDL opens a
//!   haptic device for it. A gamepad SDL knows (an Xbox pad and the like) as Windows' DirectInput
//!   shows an XInput pad, which is what the game was made for: x, y the left stick, z both triggers
//!   on one axis (the left one up, the right one down, at rest in the middle), rx, ry the right
//!   stick; ten buttons (A, B, X, Y, the shoulders, back, start, the sticks' presses), the pad's
//!   cross as a hat. Another joystick as SDL has it: its first six axes as x, y, z, rx, ry, rz, its
//!   buttons (up to 32) and hats (up to 4).
//! - A reading: SDL's axes (-32768 .. 32767), buttons and hats (as DirectInput's
//!   hundredths of degrees, clockwise from up; centred `u32::MAX`). SDL updates them on the
//!   main thread's event pump.
//! - The effects as SDL's: a constant force, a ramp, the periodic ones (square, sine,
//!   triangle, saws), the conditions (spring, damper, inertia, friction), on the x axis
//!   (the game's effects have one axis, x, cartesian); their gain (SDL's effects have none)
//!   applied to their levels; DirectInput's microseconds as SDL's milliseconds (for ever as
//!   SDL's infinity); the device's gain and self-centring as SDL's (where it has them).
//!
//! What SDL's haptics cannot do (an effect it does not have) is logged once and skipped:
//! the game's own effects go on without it.
//!
//! A pad SDL can rumble but has no force-feedback effects for (an Xbox pad, a DualSense; SDL
//! may open a haptic device for it that has only the two motors) reports force feedback too:
//! the game's effects are kept here and, on each reading, those running are turned into the
//! motors' levels (`rc2k_core::input::rumble`).

use crate::input::di_objects::{Backend, DeviceInfo, Effect, Kind, ProfileKind, Reading};
use crate::platform::ffi::sdl_input::{self, Gamepad, Haptic, Joystick};
use crate::program_log::log;
use sdl3_sys::everything::*;
use std::collections::HashMap;
use std::sync::Mutex;

/// Set when SDL reports a joystick added or removed, for the input thread to list the new ones
/// (`take_devices_changed`).
static DEVICES_CHANGED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// Whether a joystick came or went since the last call.
pub fn take_devices_changed() -> bool {
    DEVICES_CHANGED.swap(false, std::sync::atomic::Ordering::AcqRel)
}

/// The gamepads `pad_added` opened for the menus, by SDL's id.
static MENU_PADS: Mutex<Vec<(SDL_JoystickID, Gamepad)>> = Mutex::new(Vec::new());

/// A joystick has appeared (SDL's event, also for those there at the start): open as a gamepad
/// if SDL knows it as one, so its buttons reach the menus (`input::menu_pad`) whether or not
/// the game has asked for its devices yet.
pub(super) fn pad_added(id: SDL_JoystickID) {
    let name = sdl_input::joystick_name_for_id(id, "a joystick");
    #[cfg(handheld)]
    if crate::platform::handheld::is_built_in_pad(&name) {
        let guid = sdl_input::joystick_guid_text_for_id(id);
        sdl_input::add_gamepad_mapping(crate::platform::handheld::pad_mapping(&guid));
    }
    // A gamepad opened stays open until it goes (`pad_removed`): SDL counts the opens, so the
    // game's own open of it is another.
    let pad = if sdl_input::is_gamepad(id) {
        Gamepad::open(id)
    } else {
        None
    };
    let opened = pad.is_some();
    if let Some(pad) = pad {
        MENU_PADS
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .push((id, pad));
    }
    log(&format!(
        "platform: {name} appeared; a gamepad for the menus: {opened}"
    ));
    DEVICES_CHANGED.store(true, std::sync::atomic::Ordering::Release);
}

/// A joystick has gone (SDL's event): the menus' gamepad of it closed.
pub(super) fn pad_removed(id: SDL_JoystickID) {
    let mut pads = MENU_PADS
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    // A pad taken out of the list is dropped, which closes it.
    pads.retain(|(pad_id, _)| *pad_id != id);
    DEVICES_CHANGED.store(true, std::sync::atomic::Ordering::Release);
}

/// The handle that keeps a device open (closed when dropped).
enum Opened {
    Pad(Gamepad),
    Stick(Joystick),
}

impl Opened {
    /// The device's joystick (a gamepad's is its own), borrowed from the open handle.
    fn joystick(&self) -> Option<&sdl_input::JoystickRef> {
        match self {
            Opened::Pad(pad) => pad.joystick(),
            Opened::Stick(stick) => Some(stick),
        }
    }

    /// The gamepad, when SDL knows the device as one.
    fn gamepad(&self) -> Option<&Gamepad> {
        match self {
            Opened::Pad(pad) => Some(pad),
            Opened::Stick(_) => None,
        }
    }
}

struct Device {
    sdl_id: SDL_JoystickID,
    /// What SDL has open for the device (none once it has gone): read as a gamepad
    /// (DirectInput's XInput layout) when SDL knows it as one, else as a plain joystick.
    opened: Option<Opened>,
    haptic: Option<Haptic>,
    features: u32,
    info: DeviceInfo,
    /// The game's effects (their object numbers) as SDL's.
    effects: HashMap<u32, SDL_HapticEffectID>,
    /// The game's effects as it last set them, each with whether it runs: kept while the
    /// device is away, to make them again on its return.
    known: HashMap<u32, (Effect, bool)>,
    /// The device's gain the game set (its force feedback strength), 0 .. 10000: applied to
    /// each effect by us when SDL has no gain for the device (many wheels).
    gain: u32,
    /// Which of the sticks' four axes have left their rest (`stick_axis`).
    stick_moved: [bool; 4],
    /// With no haptic device of use and SDL able to rumble it: the game's effects for its
    /// motors.
    rumble: Option<Rumble>,
}

/// A rumbled device's effects: each as the game last set it, and since when it runs.
#[derive(Default)]
struct Rumble {
    effects: HashMap<u32, (Effect, Option<std::time::Instant>)>,
    /// The device's gain, 0 .. 10000.
    gain: u32,
    /// When the game last set an effect: in a race it sets them every frame, so a game that
    /// has stopped setting them (a pause, a question) has its pad still.
    set_at: Option<std::time::Instant>,
    /// The motors' levels last sent, and when.
    sent: (u16, u16),
    sent_at: Option<std::time::Instant>,
}

/// How long a rumble lasts unless sent again, and how often a running one is sent again (ms).
const RUMBLE_FOR_MS: u32 = 1000;
const RUMBLE_AGAIN_MS: u128 = 400;
/// How long after the game's last setting the motors go still (ms).
const RUMBLE_STALE_MS: u128 = 250;

impl Rumble {
    fn new() -> Self {
        Self {
            gain: 10_000,
            ..Self::default()
        }
    }

    /// The motors set for the effects running now (sent when they change, and again before a
    /// running rumble ends).
    fn update(&mut self, joystick: &sdl_input::JoystickRef) {
        use rc2k_core::input::rumble::{Form, Running, motors};
        let now = std::time::Instant::now();
        let running = self.effects.values().filter_map(|(e, since)| {
            let since = (*since)?;
            let form = match e.kind {
                Kind::Constant => Form::Constant,
                Kind::Ramp => Form::Ramp,
                Kind::Square | Kind::Sine | Kind::Triangle | Kind::SawUp | Kind::SawDown => {
                    Form::Periodic
                }
                Kind::Spring | Kind::Damper | Kind::Inertia | Kind::Friction => Form::Condition,
            };
            Some(Running {
                form,
                params: &e.params,
                gain: e.gain,
                length_us: e.duration,
                envelope: e.envelope,
                elapsed_us: now.duration_since(since).as_micros() as u64,
            })
        });
        let fresh = self
            .set_at
            .is_some_and(|at| now.duration_since(at).as_millis() < RUMBLE_STALE_MS);
        let levels = if fresh {
            motors(running, self.gain)
        } else {
            (0, 0)
        };
        let due = levels != (0, 0)
            && self
                .sent_at
                .is_none_or(|at| now.duration_since(at).as_millis() >= RUMBLE_AGAIN_MS);
        if levels != self.sent || due {
            joystick.rumble(levels.0, levels.1, RUMBLE_FOR_MS);
            self.sent = levels;
            self.sent_at = Some(now);
        }
    }

    /// Everything stopped, the motors still.
    fn stop(&mut self, joystick: Option<&sdl_input::JoystickRef>) {
        for (_, since) in self.effects.values_mut() {
            *since = None;
        }
        if let Some(joystick) = joystick
            && self.sent != (0, 0)
        {
            joystick.rumble(0, 0, 0);
        }
        self.sent = (0, 0);
        self.sent_at = None;
    }
}

/// SDL's haptic effects the game's effects can be (`sdl_effect`): a haptic device with none of
/// them (only the two motors) is of no use, the pad is rumbled instead.
const GAME_EFFECTS: u32 = SDL_HAPTIC_CONSTANT.0 as u32
    | SDL_HAPTIC_RAMP.0 as u32
    | SDL_HAPTIC_SQUARE.0 as u32
    | SDL_HAPTIC_SINE.0 as u32
    | SDL_HAPTIC_TRIANGLE.0 as u32
    | SDL_HAPTIC_SAWTOOTHUP.0 as u32
    | SDL_HAPTIC_SAWTOOTHDOWN.0 as u32
    | SDL_HAPTIC_SPRING.0 as u32
    | SDL_HAPTIC_DAMPER.0 as u32
    | SDL_HAPTIC_INERTIA.0 as u32
    | SDL_HAPTIC_FRICTION.0 as u32;

/// The joystick's haptic device if it has effects the game can use, else none (closed); and
/// whether the pad is rumbled instead.
fn haptic_or_rumble(joystick: &sdl_input::JoystickRef) -> (Option<Haptic>, bool) {
    let haptic = joystick.open_haptic();
    let haptic = match haptic {
        Some(h) if h.features() & GAME_EFFECTS == 0 => {
            h.close();
            None
        }
        h => h,
    };
    let rumble = haptic.is_none() && joystick.has_rumble();
    (haptic, rumble)
}

impl Device {
    /// The device's handles closed (it has gone): it reads at rest and plays no effect.
    fn close(&mut self) {
        // The handles `add` opened, each closed once (taken); the joystick of a gamepad is
        // the gamepad's, closed with it.
        if let Some(haptic) = self.haptic.take() {
            haptic.close();
        }
        // A rumbled device keeps its effects as the game set them (they run again from its
        // return); only the motors are marked still.
        if let Some(rumble) = self.rumble.as_mut() {
            rumble.sent = (0, 0);
            rumble.sent_at = None;
        }
        self.opened = None;
        self.features = 0;
        self.effects.clear();
    }
}

pub struct SdlBackend {
    devices: Vec<Device>,
    /// SDL's joysticks when last looked at (the list is looked over when they change).
    seen: Vec<SDL_JoystickID>,
    warned: Vec<String>,
}

impl SdlBackend {
    /// SDL's joysticks and haptics started (on the main thread, before the game's threads).
    pub fn start() {
        // The handheld's dummy window never has the focus; its pad has no x and y axes, so
        // SDL does not take its event device for a joystick: named, by its joystick node.
        #[cfg(handheld)]
        {
            sdl_input::set_handheld_joystick_hints();
        }
        if !sdl_input::init_joystick_subsystems() {
            log("platform: SDL's joysticks do not start: no joysticks");
            return;
        }
        #[cfg(web)]
        crate::platform::web_pads::stop_sdl_driver();
        crate::remote::stick_at_start();
        #[cfg(phone)]
        crate::platform::touch::attach_at_start();
        crate::input::di_objects::set_backend(Box::new(SdlBackend {
            devices: Vec::new(),
            seen: Vec::new(),
            warned: Vec::new(),
        }));
    }

    fn warn(&mut self, what: String) {
        if !self.warned.contains(&what) {
            log(&format!("platform: {what}"));
            self.warned.push(what);
        }
    }

    /// New joysticks added to the list; those gone closed (they keep their places).
    fn look(&mut self) {
        let Some(ids) = sdl_input::joystick_ids() else {
            return;
        };
        if ids == self.seen {
            return;
        }
        for (n, device) in self.devices.iter_mut().enumerate() {
            if device.opened.is_some() && !ids.contains(&device.sdl_id) {
                log(&format!(
                    "platform: joystick {n}: \"{}\" has gone",
                    device.info.name
                ));
                device.close();
            }
        }
        for &id in &ids {
            if !self.devices.iter().any(|d| d.sdl_id == id) {
                self.add(id);
            }
        }
        self.seen = ids;
    }

    fn add(&mut self, id: SDL_JoystickID) {
        let gamepad = if sdl_input::is_gamepad(id) {
            Gamepad::open(id)
        } else {
            None
        };
        let opened = match gamepad {
            None => Joystick::open(id).map(Opened::Stick),
            Some(pad) => Some(Opened::Pad(pad)),
        };
        let Some(opened) = opened else {
            return;
        };
        let Some(joystick) = opened.joystick() else {
            return;
        };
        let is_pad = opened.gamepad().is_some();
        let name = joystick.name();
        let serial = joystick.serial();
        let product = joystick.guid();
        // The same device plugged back in takes the place it left, with its instance: the
        // game's controller entry and the players' choice of it hold. The same one is the one
        // of its serial number when it has one (two pads of a model told apart), else the first
        // of its kind that has gone.
        let gone_same =
            |d: &Device| d.opened.is_none() && d.info.product == product && d.info.name == name;
        let back = self
            .devices
            .iter()
            .position(|d| gone_same(d) && serial.is_some() && d.info.serial == serial)
            .or_else(|| {
                self.devices
                    .iter()
                    .position(|d| gone_same(d) && serial.is_none() && d.info.serial.is_none())
            });
        if let Some(n) = back {
            let (haptic, rumble) = haptic_or_rumble(joystick);
            let d = &mut self.devices[n];
            d.rumble = rumble.then(|| d.rumble.take().unwrap_or_default());
            d.sdl_id = id;
            d.features = haptic.as_ref().map_or(0, Haptic::features);
            d.haptic = haptic;
            d.opened = Some(opened);
            d.stick_moved = [false; 4];
            log(&format!(
                "platform: joystick {n}: \"{}\" is back",
                d.info.name
            ));
            // The effects the game had set are made again on the returned device.
            let gain = d.gain;
            let mut known: Vec<(u32, (Effect, bool))> = d.known.drain().collect();
            known.sort_by_key(|(id, _)| *id);
            self.gain(n, gain);
            for (effect_id, (effect, running)) in known {
                self.effect(n, effect_id, &effect, running, running);
            }
            return;
        }
        let mut instance = product;
        for (n, b) in id.0.to_le_bytes().iter().enumerate() {
            instance[n] ^= b;
        }
        instance[15] ^= 0x5A;
        let axes = if !is_pad {
            joystick.axis_count().clamp(0, 6) as u32
        } else {
            5
        };
        let (haptic, rumble) = haptic_or_rumble(joystick);
        let features = haptic.as_ref().map_or(0, Haptic::features);
        let kind = if is_pad {
            ProfileKind::Pad
        } else if joystick.is_wheel() {
            ProfileKind::Wheel
        } else {
            ProfileKind::Joystick
        };
        let info = DeviceInfo {
            kind,
            icon_family: is_pad.then(|| sdl_input::pad_family(id)),
            profile_axes: if is_pad {
                6
            } else {
                joystick
                    .axis_count()
                    .clamp(0, rc2k_core::input::profile::AXES as i32) as u32
            },
            name,
            serial,
            instance,
            product,
            axes: (1 << axes) - 1,
            buttons: if !is_pad {
                joystick.button_count().clamp(0, 32) as u32
            } else {
                PAD_BUTTONS.len() as u32
            },
            hats: if !is_pad {
                joystick.hat_count().clamp(0, 4) as u32
            } else {
                1
            },
            force: haptic.is_some() || rumble,
        };
        log(&format!(
            "platform: joystick {}: \"{}\"{}, {} axes, {} buttons, {} hats, force feedback {}{} (SDL's effects {features:#x}), serial number {}",
            self.devices.len(),
            info.name,
            if !is_pad { "" } else { " (a gamepad)" },
            axes,
            info.buttons,
            info.hats,
            info.force,
            if rumble { " (its rumble motors)" } else { "" },
            info.serial.as_deref().unwrap_or("none")
        ));
        self.devices.push(Device {
            sdl_id: id,
            opened: Some(opened),
            haptic,
            features,
            info,
            effects: HashMap::new(),
            known: HashMap::new(),
            gain: 10_000,
            stick_moved: [false; 4],
            rumble: rumble.then(Rumble::new),
        });
    }
}

/// A gamepad's buttons in DirectInput's order for an XInput pad.
/// A gamepad's buttons in the order the profiles number them (as DirectInput numbered an XInput
/// pad's).
pub const PAD_BUTTONS: [SDL_GamepadButton; 10] = [
    SDL_GAMEPAD_BUTTON_SOUTH,
    SDL_GAMEPAD_BUTTON_EAST,
    SDL_GAMEPAD_BUTTON_WEST,
    SDL_GAMEPAD_BUTTON_NORTH,
    SDL_GAMEPAD_BUTTON_LEFT_SHOULDER,
    SDL_GAMEPAD_BUTTON_RIGHT_SHOULDER,
    SDL_GAMEPAD_BUTTON_BACK,
    SDL_GAMEPAD_BUTTON_START,
    SDL_GAMEPAD_BUTTON_LEFT_STICK,
    SDL_GAMEPAD_BUTTON_RIGHT_STICK,
];

#[cfg_attr(
    not(phone),
    allow(dead_code, reason = "used by the phones' touch screen")
)]
/// The touch screen's virtual gamepad: the button a touch button presses, or the trigger for a pedal; matched
/// to the default gamepad profile (`Profile::new` for a pad), which is what the game reads.
pub fn touch_pad_button(
    button: rc2k_core::input::touch::Button,
) -> Result<SDL_GamepadButton, SDL_GamepadAxis> {
    use rc2k_core::input::touch::Button;
    match button {
        Button::Throttle => Err(SDL_GAMEPAD_AXIS_RIGHT_TRIGGER),
        Button::Brake => Err(SDL_GAMEPAD_AXIS_LEFT_TRIGGER),
        Button::Handbrake => Ok(SDL_GAMEPAD_BUTTON_EAST),
        Button::GearUp => Ok(SDL_GAMEPAD_BUTTON_SOUTH),
        Button::GearDown => Ok(SDL_GAMEPAD_BUTTON_WEST),
        Button::Camera => Ok(SDL_GAMEPAD_BUTTON_NORTH),
        Button::Lights => Ok(SDL_GAMEPAD_BUTTON_BACK),
        Button::Pause => Ok(SDL_GAMEPAD_BUTTON_START),
    }
}

/// A gamepad read as DirectInput reads an XInput pad (see the header).
/// A stick's axis as the game should see it. SDL can report a stick that has not moved yet at
/// the end of its range (seen on the Anbernic's pad, whose kernel says 0: until the first move
/// the left stick read full left): an axis counts as at rest until it first leaves that end.
fn stick_axis(raw: i32, moved: &mut bool) -> i32 {
    if raw != i16::MIN as i32 {
        *moved = true;
    }
    if *moved { raw } else { 0 }
}

fn pad_reading(pad: &Gamepad, stick_moved: &mut [bool; 4]) -> Reading {
    let mut r = Reading::default();
    if !pad.connected() {
        return r;
    }
    r.present = true;
    let axis = |a| pad.axis(a) as i32;
    // the triggers 0 .. 32767 each: the left one up, the right one down
    let z = axis(SDL_GAMEPAD_AXIS_LEFT_TRIGGER) - axis(SDL_GAMEPAD_AXIS_RIGHT_TRIGGER);
    let moved = stick_moved;
    r.axes = [
        stick_axis(axis(SDL_GAMEPAD_AXIS_LEFTX), &mut moved[0]),
        stick_axis(axis(SDL_GAMEPAD_AXIS_LEFTY), &mut moved[1]),
        z.clamp(-32768, 32767),
        stick_axis(axis(SDL_GAMEPAD_AXIS_RIGHTX), &mut moved[2]),
        stick_axis(axis(SDL_GAMEPAD_AXIS_RIGHTY), &mut moved[3]),
        0,
    ];
    // The profiles' axes: the sticks, then each trigger as a pedal (0 .. 32767 as SDL has it
    // made -32768 at rest .. 32767 pressed).
    let pedal = |a| (axis(a) * 2 - 32768).clamp(-32768, 32767);
    r.profile_axes[..6].copy_from_slice(&[
        r.axes[0],
        r.axes[1],
        r.axes[3],
        r.axes[4],
        pedal(SDL_GAMEPAD_AXIS_LEFT_TRIGGER),
        pedal(SDL_GAMEPAD_AXIS_RIGHT_TRIGGER),
    ]);
    for (n, &b) in PAD_BUTTONS.iter().enumerate() {
        if pad.button(b) {
            r.buttons |= 1 << n;
        }
    }
    let held = |b| pad.button(b) as u8;
    let bits = held(SDL_GAMEPAD_BUTTON_DPAD_UP)
        | held(SDL_GAMEPAD_BUTTON_DPAD_RIGHT) << 1
        | held(SDL_GAMEPAD_BUTTON_DPAD_DOWN) << 2
        | held(SDL_GAMEPAD_BUTTON_DPAD_LEFT) << 3;
    r.hats[0] = hat_direction(bits);
    r
}

/// An SDL hat's bits as DirectInput's direction.
fn hat_direction(bits: u8) -> u32 {
    let (up, right, down, left) = (bits & 1 != 0, bits & 2 != 0, bits & 4 != 0, bits & 8 != 0);
    match (up, right, down, left) {
        (true, false, false, false) => 0,
        (true, true, false, false) => 4500,
        (false, true, false, false) => 9000,
        (false, true, true, false) => 13500,
        (false, false, true, false) => 18000,
        (false, false, true, true) => 22500,
        (false, false, false, true) => 27000,
        (true, false, false, true) => 31500,
        _ => u32::MAX,
    }
}

/// DirectInput's -10000 .. 10000 as SDL's -32767 .. 32767, the effect's gain applied.
fn level(v: i32, gain: u32) -> i16 {
    let scaled = v as i64 * 32767 / 10000 * gain as i64 / 10000;
    scaled.clamp(-32767, 32767) as i16
}

/// DirectInput's 0 .. 10000 as SDL's 0 .. 65535.
fn unsigned_level(v: i32) -> u16 {
    (v.clamp(0, 10000) as i64 * 65535 / 10000) as u16
}

/// Microseconds as milliseconds (SDL's 16-bit fields saturate).
fn ms16(us: u32) -> u16 {
    (us / 1000).min(u16::MAX as u32) as u16
}

/// The game's effect as SDL's, or why SDL cannot have it.
fn sdl_effect(e: &Effect, features: u32) -> Result<SDL_HapticEffect, String> {
    let kind = match e.kind {
        Kind::Constant => SDL_HAPTIC_CONSTANT,
        Kind::Ramp => SDL_HAPTIC_RAMP,
        Kind::Square => SDL_HAPTIC_SQUARE,
        Kind::Sine => SDL_HAPTIC_SINE,
        Kind::Triangle => SDL_HAPTIC_TRIANGLE,
        Kind::SawUp => SDL_HAPTIC_SAWTOOTHUP,
        Kind::SawDown => SDL_HAPTIC_SAWTOOTHDOWN,
        Kind::Spring => SDL_HAPTIC_SPRING,
        Kind::Damper => SDL_HAPTIC_DAMPER,
        Kind::Inertia => SDL_HAPTIC_INERTIA,
        Kind::Friction => SDL_HAPTIC_FRICTION,
    };
    if features & kind.0 as u32 == 0 {
        return Err(format!("the device has no {:?} effect in SDL", e.kind));
    }
    let direction = SDL_HapticDirection {
        r#type: SDL_HAPTIC_CARTESIAN,
        dir: [1, 0, 0],
    };
    let length = if e.duration == u32::MAX {
        SDL_HAPTIC_INFINITY
    } else {
        e.duration / 1000
    };
    let p = |n: usize| e.params.get(n).copied().unwrap_or(0);
    let [attack_level, attack_length, fade_level, fade_length] = match e.envelope {
        Some([al, at, fl, ft]) => [
            unsigned_level(al as i32),
            ms16(at),
            unsigned_level(fl as i32),
            ms16(ft),
        ],
        None => [0; 4],
    };
    let mut out = sdl_input::blank_haptic_effect();
    match e.kind {
        Kind::Constant => {
            out.constant = SDL_HapticConstant {
                r#type: kind,
                direction,
                length,
                delay: 0,
                button: 0,
                interval: 0,
                level: level(p(0), e.gain),
                attack_length,
                attack_level,
                fade_length,
                fade_level,
            }
        }
        Kind::Ramp => {
            out.ramp = SDL_HapticRamp {
                r#type: kind,
                direction,
                length,
                delay: 0,
                button: 0,
                interval: 0,
                start: level(p(0), e.gain),
                end: level(p(1), e.gain),
                attack_length,
                attack_level,
                fade_length,
                fade_level,
            }
        }
        Kind::Square | Kind::Sine | Kind::Triangle | Kind::SawUp | Kind::SawDown => {
            // DIPERIODIC: magnitude (0 .. 10000), offset, phase, period (microseconds)
            out.periodic = SDL_HapticPeriodic {
                r#type: kind,
                direction,
                length,
                delay: 0,
                button: 0,
                interval: 0,
                period: ms16(p(3) as u32),
                magnitude: level(p(0), e.gain),
                offset: level(p(1), e.gain),
                phase: (p(2).rem_euclid(36000)) as u16,
                attack_length,
                attack_level,
                fade_length,
                fade_level,
            }
        }
        _ => {
            // DICONDITION: offset, positive and negative coefficients, positive and negative
            // saturations, dead band; one axis
            let mut c = SDL_HapticCondition {
                r#type: kind,
                direction,
                length,
                delay: 0,
                button: 0,
                interval: 0,
                right_sat: [0; 3],
                left_sat: [0; 3],
                right_coeff: [0; 3],
                left_coeff: [0; 3],
                deadband: [0; 3],
                center: [0; 3],
            };
            c.center[0] = level(p(0), 10000);
            c.right_coeff[0] = level(p(1), e.gain);
            c.left_coeff[0] = level(p(2), e.gain);
            c.right_sat[0] = unsigned_level(p(3));
            c.left_sat[0] = unsigned_level(p(4));
            c.deadband[0] = unsigned_level(p(5));
            out.condition = c;
        }
    }
    Ok(out)
}

const SDL_GAIN: u32 = 1 << 16;
const SDL_AUTOCENTER: u32 = 1 << 17;

impl Backend for SdlBackend {
    fn devices(&mut self) -> Vec<DeviceInfo> {
        self.look();
        self.devices.iter().map(|d| d.info.clone()).collect()
    }

    fn read(&mut self, device: usize) -> Reading {
        let Some(d) = self.devices.get_mut(device) else {
            return Reading::default();
        };
        let mut r = Reading::default();
        // The device's joystick, open until it has gone (none then).
        let Some(joystick) = d
            .opened
            .as_ref()
            .and_then(Opened::joystick)
            .filter(|j| j.connected())
        else {
            if d.opened.is_some() {
                let gone = format!("joystick {device}: \"{}\" has gone", d.info.name);
                self.warn(gone);
            }
            return r;
        };
        if let Some(rumble) = d.rumble.as_mut() {
            rumble.update(joystick);
        }
        if let Some(pad) = d.opened.as_ref().and_then(Opened::gamepad) {
            return pad_reading(pad, &mut d.stick_moved);
        }
        r.present = true;
        for (n, axis) in r
            .axes
            .iter_mut()
            .enumerate()
            .take(d.info.axes.count_ones() as usize)
        {
            *axis = joystick.axis(n as i32) as i32;
        }
        for (n, axis) in r
            .profile_axes
            .iter_mut()
            .enumerate()
            .take(d.info.profile_axes as usize)
        {
            *axis = joystick.axis(n as i32) as i32;
        }
        for n in 0..d.info.buttons {
            if joystick.button(n as i32) {
                r.buttons |= 1 << n;
            }
        }
        for (n, hat) in r.hats.iter_mut().enumerate().take(d.info.hats as usize) {
            *hat = hat_direction(joystick.hat(n as i32));
        }
        r
    }

    fn effect(&mut self, device: usize, id: u32, effect: &Effect, start: bool, playing: bool) {
        let Some(d) = self.devices.get_mut(device) else {
            return;
        };
        d.known.insert(id, (effect.clone(), start || playing));
        if let Some(rumble) = d.rumble.as_mut() {
            let since = rumble.effects.get(&id).and_then(|(_, since)| *since);
            let since = if start {
                Some(std::time::Instant::now())
            } else if playing {
                since
            } else {
                None
            };
            rumble.effects.insert(id, (effect.clone(), since));
            rumble.set_at = Some(std::time::Instant::now());
            return;
        }
        let Some(haptic) = d.haptic.as_ref() else {
            return;
        };
        // Without SDL's device gain, the strength is each effect's: its gain and its envelope's
        // levels scaled by it.
        let scaled;
        let effect = if d.features & SDL_GAIN == 0 && d.gain < 10_000 {
            let mut e = effect.clone();
            e.gain = (e.gain as u64 * d.gain as u64 / 10_000) as u32;
            if let Some(envelope) = e.envelope.as_mut() {
                for level in [0, 2] {
                    envelope[level] = (envelope[level] as u64 * d.gain as u64 / 10_000) as u32;
                }
            }
            scaled = e;
            &scaled
        } else {
            effect
        };
        let sdl = match sdl_effect(effect, d.features) {
            Ok(e) => e,
            Err(why) => {
                self.warn(why);
                return;
            }
        };
        let made = d.effects.get(&id).copied();
        let sdl_id = match made {
            Some(sdl_id) => {
                haptic.update_effect(sdl_id, &sdl);
                sdl_id
            }
            None => match haptic.create_effect(&sdl) {
                Ok(sdl_id) => {
                    d.effects.insert(id, sdl_id);
                    sdl_id
                }
                Err(why) => {
                    self.warn(format!("SDL makes no {:?} effect: {why}", effect.kind));
                    return;
                }
            },
        };
        if start {
            haptic.run_effect(sdl_id, 1);
        } else if !playing {
            haptic.stop_effect(sdl_id);
        }
    }

    fn effect_gone(&mut self, device: usize, id: u32) {
        let Some(d) = self.devices.get_mut(device) else {
            return;
        };
        d.known.remove(&id);
        if let Some(rumble) = d.rumble.as_mut() {
            rumble.effects.remove(&id);
        }
        if let Some(sdl_id) = d.effects.remove(&id)
            && let Some(haptic) = d.haptic.as_ref()
        {
            haptic.destroy_effect(sdl_id);
        }
    }

    fn gain(&mut self, device: usize, gain: u32) {
        if let Some(d) = self.devices.get_mut(device) {
            d.gain = gain.min(10_000);
        }
        if let Some(rumble) = self.devices.get_mut(device).and_then(|d| d.rumble.as_mut()) {
            rumble.gain = gain.min(10_000);
        }
        if let Some(d) = self.devices.get(device)
            && d.features & SDL_GAIN != 0
            && let Some(haptic) = d.haptic.as_ref()
        {
            haptic.set_gain((gain / 100) as i32);
        }
    }

    fn centring(&mut self, device: usize, on: bool) {
        if let Some(d) = self.devices.get(device)
            && d.features & SDL_AUTOCENTER != 0
            && let Some(haptic) = d.haptic.as_ref()
        {
            haptic.set_autocenter(if on { 100 } else { 0 });
        }
    }

    fn acquired(&mut self, device: usize, on: bool) {
        // Given back, a device's effects stop (as DirectInput's exclusive devices do).
        if on {
            return;
        }
        let Some(d) = self.devices.get_mut(device) else {
            return;
        };
        if let Some(rumble) = d.rumble.as_mut() {
            rumble.stop(d.opened.as_ref().and_then(Opened::joystick));
        }
        if let Some(haptic) = d.haptic.as_ref() {
            haptic.stop_effects();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::stick_axis;

    #[test]
    fn a_stick_at_the_end_of_its_range_rests_until_it_moves() {
        let mut moved = false;
        assert_eq!(stick_axis(-32768, &mut moved), 0);
        assert_eq!(stick_axis(-32768, &mut moved), 0);
        assert_eq!(stick_axis(5, &mut moved), 5);
        assert_eq!(stick_axis(-32768, &mut moved), -32768);
    }
}

#[cfg(test)]
mod touch_pad_tests {
    use super::*;
    use rc2k_core::input::device_id::DeviceId;
    use rc2k_core::input::profile::{AxisUse, Digital, Kind, Profile};
    use rc2k_core::input::touch::{Button, PAD_NAME};

    /// Each touch button presses what the default pad profile reads for its action.
    #[test]
    fn the_touch_buttons_press_what_the_pad_profile_reads() {
        let id = DeviceId {
            name: PAD_NAME.into(),
            serial: None,
            nth: 1,
        };
        let b = Profile::new(id, Kind::Pad).bindings;
        let number = |pressed: SDL_GamepadButton| {
            let at = PAD_BUTTONS
                .iter()
                .position(|p| *p == pressed)
                .expect("a profile button");
            Some(Digital::Button { button: at as u8 })
        };
        let pedal = |axis: SDL_GamepadAxis| {
            Some(AxisUse::Pedal {
                axis: axis.0 as u8,
                rest_high: false,
            })
        };
        for button in Button::ALL {
            match (button, touch_pad_button(button)) {
                (Button::Throttle, Err(axis)) => assert_eq!(b.accelerate_axis, pedal(axis)),
                (Button::Brake, Err(axis)) => assert_eq!(b.brake_axis, pedal(axis)),
                (Button::Handbrake, Ok(p)) => assert_eq!(b.handbrake, number(p)),
                (Button::GearUp, Ok(p)) => assert_eq!(b.gear_up, number(p)),
                (Button::GearDown, Ok(p)) => assert_eq!(b.gear_down, number(p)),
                (Button::Camera, Ok(p)) => assert_eq!(b.camera, number(p)),
                (Button::Lights, Ok(p)) => assert_eq!(b.lights, number(p)),
                // The pause is read from Start as the menus' Escape, not from a binding.
                (Button::Pause, Ok(p)) => assert_eq!(p.0, SDL_GAMEPAD_BUTTON_START.0),
                (button, other) => panic!(
                    "{button:?} presses {:?}",
                    other.map(|p| p.0).map_err(|a| a.0)
                ),
            }
        }
    }
}
