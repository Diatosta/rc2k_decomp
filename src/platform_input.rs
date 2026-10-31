//! The standalone program's joysticks and wheels: the backend of DirectInput's objects made
//! in Rust (`input::di_objects`) on SDL's joysticks and haptics.
//!
//! - The devices: SDL's joysticks in the order first seen (a device unplugged keeps its
//!   place and reads at rest), each opened when seen: its name, a product GUID (SDL's), an
//!   instance GUID made from SDL's and its instance number, force feedback when SDL opens a
//!   haptic device for it. A gamepad SDL knows (an Xbox pad and the like) as Windows'
//!   DirectInput shows an XInput pad, which is what the game was made for: x, y the left
//!   stick, z both triggers on one axis (the left one up, the right one down, at rest in the
//!   middle), rx, ry the right stick; ten buttons (A, B, X, Y, the shoulders, back, start,
//!   the sticks' presses), the pad's cross as a hat. Another joystick as SDL has it: its
//!   first six axes as x, y, z, rx, ry, rz, its buttons (up to 32) and hats (up to 4).
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

use crate::framework::log;
use crate::input::di_objects::{Backend, DeviceInfo, Effect, Kind, Reading};
use sdl3_sys::everything::*;
use std::collections::HashMap;

/// A joystick has appeared (SDL's event, also for those there at the start): open as a gamepad
/// if SDL knows it as one, so its buttons reach the menus (`input::menu_pad`) whether or not
/// the game has asked for its devices yet.
pub fn pad_added(id: SDL_JoystickID) {
    // SAFETY: SDL's calls on the id its event gave; the name it returns (null checked) lives
    // until the next call. A gamepad opened stays open for the program's life: SDL counts
    // the opens, so the game's own open of it is another.
    unsafe {
        let name = SDL_GetJoystickNameForID(id);
        let name = if name.is_null() {
            "a joystick".into()
        } else {
            std::ffi::CStr::from_ptr(name).to_string_lossy()
        };
        #[cfg(all(target_os = "linux", target_arch = "aarch64"))]
        if name == "ANBERNIC-keys" {
            let mut text = [0 as std::ffi::c_char; 33];
            SDL_GUIDToString(SDL_GetJoystickGUIDForID(id), text.as_mut_ptr(), 33);
            let guid = std::ffi::CStr::from_ptr(text.as_ptr()).to_string_lossy();
            if let Ok(mapping) = std::ffi::CString::new(crate::handheld::pad_mapping(&guid)) {
                SDL_AddGamepadMapping(mapping.as_ptr());
            }
        }
        let opened = SDL_IsGamepad(id) && !SDL_OpenGamepad(id).is_null();
        log(&format!(
            "platform: {name} appeared; a gamepad for the menus: {opened}"
        ));
    }
}

struct Device {
    sdl_id: SDL_JoystickID,
    joystick: *mut SDL_Joystick,
    /// Read as a gamepad (DirectInput's XInput layout), when SDL knows it as one.
    gamepad: *mut SDL_Gamepad,
    haptic: *mut SDL_Haptic,
    features: u32,
    info: DeviceInfo,
    /// The game's effects (their object numbers) as SDL's.
    effects: HashMap<u32, SDL_HapticEffectID>,
}

pub struct SdlBackend {
    devices: Vec<Device>,
    /// SDL's joystick count when last looked at (the list is looked over when it changes).
    seen: i32,
    warned: Vec<String>,
}

// SAFETY: SDL's joystick and haptic functions take SDL's own lock; the backend is used under
// the objects' lock, one thread at a time.
unsafe impl Send for SdlBackend {}

impl SdlBackend {
    /// SDL's joysticks and haptics started (on the main thread, before the game's threads).
    pub fn start() {
        // SAFETY: plain flags; SDL is started by then (`platform::start`).
        unsafe {
            // The handheld's dummy window never has the focus; its pad has no x and y axes, so
            // SDL does not take its event device for a joystick: named, by its joystick node.
            #[cfg(all(target_os = "linux", target_arch = "aarch64"))]
            {
                SDL_SetHint(SDL_HINT_JOYSTICK_ALLOW_BACKGROUND_EVENTS, c"1".as_ptr());
                SDL_SetHint(SDL_HINT_JOYSTICK_DEVICE, c"/dev/input/js0".as_ptr());
            }
            if !SDL_InitSubSystem(SDL_INIT_JOYSTICK | SDL_INIT_GAMEPAD | SDL_INIT_HAPTIC) {
                log("platform: SDL's joysticks do not start: no joysticks");
                return;
            }
        }
        crate::input::di_objects::set_backend(Box::new(SdlBackend {
            devices: Vec::new(),
            seen: -1,
            warned: Vec::new(),
        }));
    }

    fn warn(&mut self, what: String) {
        if !self.warned.contains(&what) {
            log(&format!("platform: {what}"));
            self.warned.push(what);
        }
    }

    /// New joysticks added to the list.
    fn look(&mut self) {
        // SAFETY: SDL's array of `count` ids (null checked); the devices opened from them
        // are kept in `devices` until closed.
        unsafe {
            let mut count = 0;
            let ids = SDL_GetJoysticks(&mut count);
            if ids.is_null() {
                return;
            }
            if count != self.seen {
                self.seen = count;
                for n in 0..count as usize {
                    let id = *ids.add(n);
                    if self.devices.iter().any(|d| d.sdl_id == id) {
                        continue;
                    }
                    self.add(id);
                }
            }
            SDL_free(ids as *mut _);
        }
    }

    fn add(&mut self, id: SDL_JoystickID) {
        // SAFETY: SDL's calls on a joystick id it listed; the handles it gives are kept in
        // the device, open while it is.
        unsafe {
            let gamepad = if SDL_IsGamepad(id) {
                SDL_OpenGamepad(id)
            } else {
                std::ptr::null_mut()
            };
            let joystick = if gamepad.is_null() {
                SDL_OpenJoystick(id)
            } else {
                SDL_GetGamepadJoystick(gamepad)
            };
            if joystick.is_null() {
                return;
            }
            let name = SDL_GetJoystickName(joystick);
            let name = if name.is_null() {
                "Joystick".to_string()
            } else {
                std::ffi::CStr::from_ptr(name)
                    .to_string_lossy()
                    .into_owned()
            };
            let product = SDL_GetJoystickGUID(joystick).data;
            let mut instance = product;
            for (n, b) in id.0.to_le_bytes().iter().enumerate() {
                instance[n] ^= b;
            }
            instance[15] ^= 0x5A;
            let axes = if gamepad.is_null() {
                SDL_GetNumJoystickAxes(joystick).clamp(0, 6) as u32
            } else {
                5
            };
            let haptic = if SDL_IsJoystickHaptic(joystick) {
                SDL_OpenHapticFromJoystick(joystick)
            } else {
                std::ptr::null_mut()
            };
            let features = if haptic.is_null() {
                0
            } else {
                SDL_GetHapticFeatures(haptic)
            };
            let info = DeviceInfo {
                name,
                instance,
                product,
                axes: (1 << axes) - 1,
                buttons: if gamepad.is_null() {
                    SDL_GetNumJoystickButtons(joystick).clamp(0, 32) as u32
                } else {
                    PAD_BUTTONS.len() as u32
                },
                hats: if gamepad.is_null() {
                    SDL_GetNumJoystickHats(joystick).clamp(0, 4) as u32
                } else {
                    1
                },
                force: !haptic.is_null(),
            };
            log(&format!(
                "platform: joystick {}: \"{}\"{}, {} axes, {} buttons, {} hats, force feedback {} (SDL's effects {features:#x})",
                self.devices.len(),
                info.name,
                if gamepad.is_null() {
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
                joystick,
                gamepad,
                haptic,
                features,
                info,
                effects: HashMap::new(),
            });
        }
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
fn pad_reading(device: &Device) -> Reading {
    let pad = device.gamepad;
    // SAFETY: the device's gamepad, opened by `add` and open while the device is listed.
    unsafe {
        let mut r = Reading::default();
        if !SDL_GamepadConnected(pad) {
            return r;
        }
        let axis = |a| SDL_GetGamepadAxis(pad, a) as i32;
        // the triggers 0 .. 32767 each: the left one up, the right one down
        let z = axis(SDL_GAMEPAD_AXIS_LEFT_TRIGGER) - axis(SDL_GAMEPAD_AXIS_RIGHT_TRIGGER);
        r.axes = [
            axis(SDL_GAMEPAD_AXIS_LEFTX),
            axis(SDL_GAMEPAD_AXIS_LEFTY),
            z.clamp(-32768, 32767),
            axis(SDL_GAMEPAD_AXIS_RIGHTX),
            axis(SDL_GAMEPAD_AXIS_RIGHTY),
            0,
        ];
        for (n, &b) in PAD_BUTTONS.iter().enumerate() {
            if SDL_GetGamepadButton(pad, b) {
                r.buttons |= 1 << n;
            }
        }
        let held = |b| SDL_GetGamepadButton(pad, b) as u8;
        let bits = held(SDL_GAMEPAD_BUTTON_DPAD_UP)
            | held(SDL_GAMEPAD_BUTTON_DPAD_RIGHT) << 1
            | held(SDL_GAMEPAD_BUTTON_DPAD_DOWN) << 2
            | held(SDL_GAMEPAD_BUTTON_DPAD_LEFT) << 3;
        r.hats[0] = hat_direction(bits);
        r
    }
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
    // SAFETY: a C union of plain integers: all zero is a valid value.
    let mut out: SDL_HapticEffect = unsafe { std::mem::zeroed() };
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
        let Some(d) = self.devices.get(device) else {
            return Reading::default();
        };
        let mut r = Reading::default();
        // SAFETY: the device's joystick, open while it is listed; the counts are SDL's own.
        unsafe {
            if !SDL_JoystickConnected(d.joystick) {
                return r;
            }
            if !d.gamepad.is_null() {
                return pad_reading(d);
            }
            for n in 0..d.info.axes.count_ones() as usize {
                r.axes[n] = SDL_GetJoystickAxis(d.joystick, n as i32) as i32;
            }
            for n in 0..d.info.buttons {
                if SDL_GetJoystickButton(d.joystick, n as i32) {
                    r.buttons |= 1 << n;
                }
            }
            for n in 0..d.info.hats as usize {
                r.hats[n] = hat_direction(SDL_GetJoystickHat(d.joystick, n as i32));
            }
        }
        r
    }

    fn effect(&mut self, device: usize, id: u32, effect: &Effect, start: bool, playing: bool) {
        let Some(d) = self.devices.get_mut(device) else {
            return;
        };
        if d.haptic.is_null() {
            return;
        }
        let sdl = match sdl_effect(effect, d.features) {
            Ok(e) => e,
            Err(why) => {
                self.warn(why);
                return;
            }
        };
        let (haptic, made) = (d.haptic, d.effects.get(&id).copied());
        // SAFETY: the device's haptic (not null, open while listed), an effect made on it,
        // and SDL's NUL-ended error text.
        unsafe {
            let sdl_id = match made {
                Some(sdl_id) => {
                    SDL_UpdateHapticEffect(haptic, sdl_id, &sdl);
                    sdl_id
                }
                None => {
                    let sdl_id = SDL_CreateHapticEffect(haptic, &sdl);
                    if sdl_id.0 < 0 {
                        let why = std::ffi::CStr::from_ptr(SDL_GetError())
                            .to_string_lossy()
                            .into_owned();
                        self.warn(format!("SDL makes no {:?} effect: {why}", effect.kind));
                        return;
                    }
                    d.effects.insert(id, sdl_id);
                    sdl_id
                }
            };
            if start {
                SDL_RunHapticEffect(haptic, sdl_id, 1);
            } else if !playing {
                SDL_StopHapticEffect(haptic, sdl_id);
            }
        }
    }

    fn effect_gone(&mut self, device: usize, id: u32) {
        let Some(d) = self.devices.get_mut(device) else {
            return;
        };
        if let Some(sdl_id) = d.effects.remove(&id) {
            // SAFETY: the device's haptic, open while it is listed (SDL refuses a null one).
            unsafe { SDL_DestroyHapticEffect(d.haptic, sdl_id) };
        }
    }

    fn gain(&mut self, device: usize, gain: u32) {
        if let Some(d) = self.devices.get(device)
            && d.features & SDL_GAIN != 0
        {
            // SAFETY: the device's haptic, open while it is listed (SDL refuses a null one).
            unsafe { SDL_SetHapticGain(d.haptic, (gain / 100) as i32) };
        }
    }

    fn centring(&mut self, device: usize, on: bool) {
        if let Some(d) = self.devices.get(device)
            && d.features & SDL_AUTOCENTER != 0
        {
            // SAFETY: the device's haptic, open while it is listed (SDL refuses a null one).
            unsafe { SDL_SetHapticAutocenter(d.haptic, if on { 100 } else { 0 }) };
        }
    }

    fn acquired(&mut self, device: usize, on: bool) {
        // Given back, a device's effects stop (as DirectInput's exclusive devices do).
        if !on
            && let Some(d) = self.devices.get(device)
            && !d.haptic.is_null()
        {
            // SAFETY: the device's haptic, open while it is listed (SDL refuses a null one).
            unsafe { SDL_StopHapticEffects(d.haptic) };
        }
    }
}
