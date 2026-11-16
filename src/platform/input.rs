//! The standalone program's joysticks and wheels: the backend of DirectInput's objects made
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

use crate::input::di_objects::{Backend, DeviceInfo, Effect, Kind, Reading};
use crate::logfile::program_log::log;
use crate::platform::ffi::sdl_input::{self, Gamepad, Haptic, Joystick};
use sdl3_sys::everything::*;
use std::collections::HashMap;
use std::sync::Mutex;

/// The gamepads `pad_added` opened for the menus, by SDL's id.
static MENU_PADS: Mutex<Vec<(SDL_JoystickID, Gamepad)>> = Mutex::new(Vec::new());

/// A joystick has appeared (SDL's event, also for those there at the start): open as a gamepad
/// if SDL knows it as one, so its buttons reach the menus (`input::menu_pad`) whether or not
/// the game has asked for its devices yet.
pub fn pad_added(id: SDL_JoystickID) {
    let name = sdl_input::joystick_name_for_id(id, "a joystick");
    #[cfg(all(target_os = "linux", target_arch = "aarch64"))]
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
}

/// A joystick has gone (SDL's event): the menus' gamepad of it closed.
pub fn pad_removed(id: SDL_JoystickID) {
    let mut pads = MENU_PADS
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    pads.retain(|&(pad_id, pad)| {
        if pad_id == id {
            pad.close();
        }
        pad_id != id
    });
}

struct Device {
    sdl_id: SDL_JoystickID,
    joystick: Option<Joystick>,
    /// Read as a gamepad (DirectInput's XInput layout), when SDL knows it as one.
    gamepad: Option<Gamepad>,
    haptic: Option<Haptic>,
    features: u32,
    info: DeviceInfo,
    /// The game's effects (their object numbers) as SDL's.
    effects: HashMap<u32, SDL_HapticEffectID>,
    /// Which of the sticks' four axes have left their rest (`stick_axis`).
    stick_moved: [bool; 4],
}

impl Device {
    /// The device's handles closed (it has gone): it reads at rest and plays no effect.
    fn close(&mut self) {
        // The handles `add` opened, each closed once (taken); the joystick of a gamepad is
        // the gamepad's, closed with it.
        if let Some(haptic) = self.haptic.take() {
            haptic.close();
        }
        if let Some(gamepad) = self.gamepad.take() {
            gamepad.close();
        } else if let Some(joystick) = self.joystick {
            joystick.close();
        }
        self.joystick = None;
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
        #[cfg(all(target_os = "linux", target_arch = "aarch64"))]
        {
            sdl_input::set_handheld_joystick_hints();
        }
        if !sdl_input::init_joystick_subsystems() {
            log("platform: SDL's joysticks do not start: no joysticks");
            return;
        }
        #[cfg(feature = "remote")]
        crate::remote::stick_at_start();
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
            if device.joystick.is_some() && !ids.contains(&device.sdl_id) {
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
        let joystick = match gamepad {
            None => Joystick::open(id),
            Some(pad) => pad.joystick(),
        };
        let Some(joystick) = joystick else {
            return;
        };
        let name = joystick.name();
        let product = joystick.guid();
        let mut instance = product;
        for (n, b) in id.0.to_le_bytes().iter().enumerate() {
            instance[n] ^= b;
        }
        instance[15] ^= 0x5A;
        let axes = if gamepad.is_none() {
            joystick.axis_count().clamp(0, 6) as u32
        } else {
            5
        };
        let haptic = joystick.open_haptic();
        let features = haptic.map_or(0, Haptic::features);
        let info = DeviceInfo {
            name,
            instance,
            product,
            axes: (1 << axes) - 1,
            buttons: if gamepad.is_none() {
                joystick.button_count().clamp(0, 32) as u32
            } else {
                PAD_BUTTONS.len() as u32
            },
            hats: if gamepad.is_none() {
                joystick.hat_count().clamp(0, 4) as u32
            } else {
                1
            },
            force: haptic.is_some(),
        };
        log(&format!(
            "platform: joystick {}: \"{}\"{}, {} axes, {} buttons, {} hats, force feedback {} (SDL's effects {features:#x})",
            self.devices.len(),
            info.name,
            if gamepad.is_none() {
                ""
            } else {
                " (a gamepad)"
            },
            axes,
            info.buttons,
            info.hats,
            info.force
        ));
        self.devices.push(Device {
            sdl_id: id,
            joystick: Some(joystick),
            gamepad,
            haptic,
            features,
            info,
            effects: HashMap::new(),
            stick_moved: [false; 4],
        });
    }
}

/// A gamepad's buttons in DirectInput's order for an XInput pad.
const PAD_BUTTONS: [SDL_GamepadButton; 10] = [
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

fn pad_reading(pad: Gamepad, stick_moved: &mut [bool; 4]) -> Reading {
    let mut r = Reading::default();
    if !pad.connected() {
        return r;
    }
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
        let Some(joystick) = d.joystick.filter(|j| j.connected()) else {
            if d.joystick.is_some() {
                let gone = format!("joystick {device}: \"{}\" has gone", d.info.name);
                self.warn(gone);
            }
            return r;
        };
        if let Some(pad) = d.gamepad {
            return pad_reading(pad, &mut d.stick_moved);
        }
        for (n, axis) in r
            .axes
            .iter_mut()
            .enumerate()
            .take(d.info.axes.count_ones() as usize)
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
        let Some(haptic) = d.haptic else {
            return;
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
        if let Some(sdl_id) = d.effects.remove(&id)
            && let Some(haptic) = d.haptic
        {
            haptic.destroy_effect(sdl_id);
        }
    }

    fn gain(&mut self, device: usize, gain: u32) {
        if let Some(d) = self.devices.get(device)
            && d.features & SDL_GAIN != 0
            && let Some(haptic) = d.haptic
        {
            haptic.set_gain((gain / 100) as i32);
        }
    }

    fn centring(&mut self, device: usize, on: bool) {
        if let Some(d) = self.devices.get(device)
            && d.features & SDL_AUTOCENTER != 0
            && let Some(haptic) = d.haptic
        {
            haptic.set_autocenter(if on { 100 } else { 0 });
        }
    }

    fn acquired(&mut self, device: usize, on: bool) {
        // Given back, a device's effects stop (as DirectInput's exclusive devices do).
        if !on
            && let Some(d) = self.devices.get(device)
            && let Some(haptic) = d.haptic
        {
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
