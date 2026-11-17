//! A player's profile for one controller: which of its axes, buttons and hat directions mean
//! what, and how each axis is shaped. The race reads a controller through it (`readings`), the
//! controller pages learn into it (`learn`), the options file keeps it (each player has a list
//! of profiles, one for each controller the player has used).
//!
//! The controller's own numbers are SDL's: axes -32768 .. 32767 (`AXES` of them; a gamepad's
//! triggers each their own axis, from -32768 at rest to 32767 pressed, like a pedal), buttons by number from 0, hats by number with their direction.
//!
//! An axis's value goes through, in order: the invert, the dead zone, the range, the response
//! curve. The dead zone and the range are per mille of what the axis can move from its rest: a
//! centred axis (a wheel, a stick) moves its half of the travel on each side of the centre; a
//! pedal moves all of its travel from the end it rests at.
//! - dead zone: the first part of the travel that reads 0;
//! - range: the part of the travel that reads full: 1000 all of it, 300 full at 30 % (a wheel
//!   locked at 30 % of its rotation steers full);
//! - sensitivity -100 .. 100: 0 is linear; above 0 the curve is flatter near the rest and
//!   steeper at the end (`m + a (m m - m)`, a the sensitivity over 100); below 0 steeper near
//!   the rest (`m + a (2 m - m m - m)`, a the sensitivity's size over 100); both rise from 0 to
//!   1 and never fall;
//! - invert: the axis the other way round.
//!
//! Read as the race reads a controller: a key or a digital binding gives all of a reading (and
//! the digital flag of steering, brake or throttle), else the axis binding gives its value on
//! the game's scale: 0x100 is all of it.

use crate::input::device_id::DeviceId;
use crate::input::devices::{FULL, FULL_LEFT, HAT_CENTRED, Readings};
use serde::{Deserialize, Serialize};

/// How many axes a controller's profile numbers.
pub const AXES: usize = 8;
/// How many hats.
pub const HATS: usize = 4;
/// The most a sensitivity goes to either side.
pub const SENSITIVITY_LIMIT: i32 = 100;

/// A controller's reading as SDL gives it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RawInput {
    /// -32768 .. 32767 each.
    pub axes: [i32; AXES],
    /// A bit for each of up to 32 buttons.
    pub buttons: u32,
    /// Each hat as hundredths of degrees clockwise from up (0 up, 9000 right), or `u32::MAX`
    /// centred.
    pub hats: [u32; HATS],
}

impl Default for RawInput {
    fn default() -> Self {
        Self {
            axes: [0; AXES],
            buttons: 0,
            hats: [u32::MAX; HATS],
        }
    }
}

/// What a player does with the car, the order the race lists them in.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    Left,
    Right,
    Accelerate,
    Brake,
    GearUp,
    GearDown,
    Handbrake,
    Lights,
    Wipers,
    Camera,
    Pause,
    Reset,
}

/// How many actions.
pub const ACTIONS: usize = 12;

/// Which actions' keys are down.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Keys(pub [bool; ACTIONS]);

impl Keys {
    fn down(&self, action: Action) -> bool {
        self.0[action as usize]
    }
}

/// How an axis is used.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum AxisUse {
    /// -1 .. 1 around the centre (steering).
    Centred { axis: u8 },
    /// 0 .. 1 from the end it rests at (a pedal); `rest_high`: the end with the large values.
    Pedal { axis: u8, rest_high: bool },
    /// One side of a centred axis (a stick's up for the throttle); `positive`: the large values.
    Half { axis: u8, positive: bool },
}

impl AxisUse {
    pub fn axis(&self) -> u8 {
        match *self {
            AxisUse::Centred { axis }
            | AxisUse::Pedal { axis, .. }
            | AxisUse::Half { axis, .. } => axis,
        }
    }
}

/// A hat's direction.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HatDirection {
    Up,
    Right,
    Down,
    Left,
}

impl HatDirection {
    /// The direction nearest to a hat's angle (hundredths of degrees).
    pub fn nearest(angle: u32) -> HatDirection {
        match ((angle % 36000 + 4500) / 9000) % 4 {
            0 => HatDirection::Up,
            1 => HatDirection::Right,
            2 => HatDirection::Down,
            _ => HatDirection::Left,
        }
    }

    /// Whether a hat at `angle` presses this direction (a diagonal presses both of its sides).
    fn pressed_by(self, angle: u32) -> bool {
        if angle >= 36000 {
            return false;
        }
        let centre = match self {
            HatDirection::Up => 0,
            HatDirection::Right => 9000,
            HatDirection::Down => 18000,
            HatDirection::Left => 27000,
        };
        let apart = angle.abs_diff(centre);
        apart.min(36000 - apart) <= 4500
    }
}

/// A button or a hat direction.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Digital {
    Button { button: u8 },
    Hat { hat: u8, direction: HatDirection },
}

impl Digital {
    fn pressed(&self, raw: &RawInput) -> bool {
        match *self {
            Digital::Button { button } => button < 32 && raw.buttons >> button & 1 != 0,
            Digital::Hat { hat, direction } => raw
                .hats
                .get(hat as usize)
                .is_some_and(|&angle| direction.pressed_by(angle)),
        }
    }
}

/// How one axis is shaped (see the module header).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AxisSettings {
    /// Per mille of the travel from the rest that reads 0.
    pub dead_zone: u32,
    /// Per mille of the travel from the rest that gives all of it.
    pub range: u32,
    /// -100 .. 100, 0 linear.
    pub sensitivity: i32,
    pub invert: bool,
}

impl Default for AxisSettings {
    fn default() -> Self {
        Self {
            dead_zone: 0,
            range: 1000,
            sensitivity: 0,
            invert: false,
        }
    }
}

/// What a profile binds. The analogue bindings are for steering, throttle and brake; every
/// action has a digital binding too (a button or a hat direction), the analogue ones are what
/// an axis gives instead.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Bindings {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub steering_axis: Option<AxisUse>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub accelerate_axis: Option<AxisUse>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub brake_axis: Option<AxisUse>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub left: Option<Digital>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub right: Option<Digital>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub accelerate: Option<Digital>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub brake: Option<Digital>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gear_up: Option<Digital>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gear_down: Option<Digital>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub handbrake: Option<Digital>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub camera: Option<Digital>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pause: Option<Digital>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lights: Option<Digital>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reset: Option<Digital>,
    /// The gears of an H shifter (the controller page's switch): the input held while the
    /// shifter is in gear n, or in reverse. Absent: not bound.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gear_1: Option<Digital>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gear_2: Option<Digital>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gear_3: Option<Digital>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gear_4: Option<Digital>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gear_5: Option<Digital>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gear_6: Option<Digital>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gear_7: Option<Digital>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reverse: Option<Digital>,
}

/// An H shifter's position: a gear 1 ..= 7 or reverse.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Gear {
    Forward(u8),
    Reverse,
}

impl Gear {
    /// The seven forward gears, then reverse, in the order the page lists them.
    pub const ALL: [Gear; 8] = [
        Gear::Forward(1),
        Gear::Forward(2),
        Gear::Forward(3),
        Gear::Forward(4),
        Gear::Forward(5),
        Gear::Forward(6),
        Gear::Forward(7),
        Gear::Reverse,
    ];

    /// The bits of a reading that ask for this gear (the race reads bits 8 .. 11): the gear's
    /// number there, 0xF for reverse.
    pub fn code(self) -> u32 {
        match self {
            Gear::Forward(n) => u32::from(n) << 8,
            Gear::Reverse => 0xF00,
        }
    }
}

impl Bindings {
    /// The input bound to a gear of the H shifter.
    pub fn gear(&self, gear: Gear) -> Option<Digital> {
        match gear {
            Gear::Forward(1) => self.gear_1,
            Gear::Forward(2) => self.gear_2,
            Gear::Forward(3) => self.gear_3,
            Gear::Forward(4) => self.gear_4,
            Gear::Forward(5) => self.gear_5,
            Gear::Forward(6) => self.gear_6,
            Gear::Forward(7) => self.gear_7,
            Gear::Reverse => self.reverse,
            Gear::Forward(_) => None,
        }
    }

    fn gear_mut(&mut self, gear: Gear) -> Option<&mut Option<Digital>> {
        Some(match gear {
            Gear::Forward(1) => &mut self.gear_1,
            Gear::Forward(2) => &mut self.gear_2,
            Gear::Forward(3) => &mut self.gear_3,
            Gear::Forward(4) => &mut self.gear_4,
            Gear::Forward(5) => &mut self.gear_5,
            Gear::Forward(6) => &mut self.gear_6,
            Gear::Forward(7) => &mut self.gear_7,
            Gear::Reverse => &mut self.reverse,
            Gear::Forward(_) => return None,
        })
    }
}

impl Bindings {
    /// The digital binding of an action.
    pub fn digital(&self, action: Action) -> Option<Digital> {
        match action {
            Action::Left => self.left,
            Action::Right => self.right,
            Action::Accelerate => self.accelerate,
            Action::Brake => self.brake,
            Action::GearUp => self.gear_up,
            Action::GearDown => self.gear_down,
            Action::Handbrake => self.handbrake,
            Action::Lights => self.lights,
            // The race reads no wipers bit: a profile binds none (an old file's key is passed over).
            Action::Wipers => None,
            Action::Camera => self.camera,
            Action::Pause => self.pause,
            Action::Reset => self.reset,
        }
    }

    fn digital_mut(&mut self, action: Action) -> Option<&mut Option<Digital>> {
        Some(match action {
            Action::Left => &mut self.left,
            Action::Right => &mut self.right,
            Action::Accelerate => &mut self.accelerate,
            Action::Brake => &mut self.brake,
            Action::GearUp => &mut self.gear_up,
            Action::GearDown => &mut self.gear_down,
            Action::Handbrake => &mut self.handbrake,
            Action::Lights => &mut self.lights,
            Action::Wipers => return None,
            Action::Camera => &mut self.camera,
            Action::Pause => &mut self.pause,
            Action::Reset => &mut self.reset,
        })
    }
}

/// What kind of controller a profile is made for.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Kind {
    /// A gamepad SDL knows: sticks and triggers as `AXES` lists them.
    Pad,
    /// SDL's type says a wheel.
    Wheel,
    /// Anything else.
    #[default]
    Joystick,
}

/// A gamepad's axes in the profile: the left stick, the right stick, then the triggers.
pub mod pad_axes {
    pub const LEFT_X: u8 = 0;
    pub const LEFT_Y: u8 = 1;
    pub const RIGHT_X: u8 = 2;
    pub const RIGHT_Y: u8 = 3;
    pub const LEFT_TRIGGER: u8 = 4;
    pub const RIGHT_TRIGGER: u8 = 5;
}

/// One player's profile for one controller.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Profile {
    pub device: DeviceId,
    /// A steering wheel: the menus show its range in degrees.
    #[serde(default)]
    pub wheel: bool,
    /// How far the wheel turns lock to lock, in degrees.
    #[serde(default = "default_rotation")]
    pub full_rotation_degrees: u32,
    /// The controller's hat and buttons are an H shifter's (the controller page's switch): a
    /// property of the controller, so a pad used on its own does not inherit a wheel's switch.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub shifter: bool,
    /// By axis number (an axis past the list has the default settings).
    #[serde(default)]
    pub axes: Vec<AxisSettings>,
    #[serde(default)]
    pub bindings: Bindings,
}

fn default_rotation() -> u32 {
    900
}

/// Axis `n` of a controller of this kind, as its text names it: "x", "y", "z", "rx", "ry", "rz"
/// for a joystick's, "x", "y", "rx", "ry", "lt", "rt" for a gamepad's.
pub fn axis_label(kind: Kind, axis: u8) -> String {
    const JOYSTICK: [&str; 6] = ["x", "y", "z", "rx", "ry", "rz"];
    const PAD: [&str; 6] = ["x", "y", "rx", "ry", "lt", "rt"];
    let names = if kind == Kind::Pad { &PAD } else { &JOYSTICK };
    names
        .get(axis as usize)
        .map_or_else(|| format!("a{axis}"), |name| (*name).to_string())
}

impl Profile {
    /// The profile a controller gets when a player first picks it.
    /// - gamepad: the left stick's x steers, the right trigger accelerates, the left one
    ///   brakes, the cross left and right steer too, buttons 0 .. 3 (A, B, X, Y) are gear up,
    ///   gear down, handbrake and camera;
    /// - wheel: x steers, the same buttons, no pedals (they are learnt);
    /// - other: x steers, y up accelerates and y down brakes, the same buttons.
    pub fn new(device: DeviceId, kind: Kind) -> Profile {
        use pad_axes::*;
        let button = |button| Some(Digital::Button { button });
        let mut profile = Profile {
            device,
            wheel: kind == Kind::Wheel,
            full_rotation_degrees: default_rotation(),
            shifter: false,
            axes: Vec::new(),
            bindings: Bindings {
                steering_axis: Some(AxisUse::Centred { axis: 0 }),
                gear_up: button(0),
                gear_down: button(1),
                handbrake: button(2),
                camera: button(3),
                ..Bindings::default()
            },
        };
        match kind {
            Kind::Pad => {
                let b = &mut profile.bindings;
                b.accelerate_axis = Some(AxisUse::Pedal {
                    axis: RIGHT_TRIGGER,
                    rest_high: false,
                });
                b.brake_axis = Some(AxisUse::Pedal {
                    axis: LEFT_TRIGGER,
                    rest_high: false,
                });
                b.left = Some(Digital::Hat {
                    hat: 0,
                    direction: HatDirection::Left,
                });
                b.right = Some(Digital::Hat {
                    hat: 0,
                    direction: HatDirection::Right,
                });
                profile.settings_mut(LEFT_X).dead_zone = 100;
            }
            Kind::Wheel => {}
            Kind::Joystick => {
                let b = &mut profile.bindings;
                b.accelerate_axis = Some(AxisUse::Half {
                    axis: 1,
                    positive: false,
                });
                b.brake_axis = Some(AxisUse::Half {
                    axis: 1,
                    positive: true,
                });
                profile.settings_mut(0).dead_zone = 100;
                profile.settings_mut(1).dead_zone = 100;
            }
        }
        profile
    }

    /// The settings of axis `axis` (the default for one the profile has not set).
    pub fn settings(&self, axis: u8) -> AxisSettings {
        self.axes.get(axis as usize).copied().unwrap_or_default()
    }

    /// The settings of axis `axis`, made if the list stops short of it (an axis past `AXES`
    /// shares the last one's).
    pub fn settings_mut(&mut self, axis: u8) -> &mut AxisSettings {
        let at = (axis as usize).min(AXES - 1);
        if self.axes.len() <= at {
            self.axes.resize(at + 1, AxisSettings::default());
        }
        &mut self.axes[at]
    }

    /// The range of an axis in degrees for a wheel: the range's share of the full rotation.
    pub fn range_degrees(&self, axis: u8) -> u32 {
        self.settings(axis).range * self.full_rotation_degrees / 1000
    }
}

/// A raw axis value as -1 .. 1.
fn unit(raw: i32) -> f64 {
    (raw as f64 / 32767.0).clamp(-1.0, 1.0)
}

/// How far an axis has moved from its rest, 0 .. 1, through the dead zone, the range and the
/// curve of `settings` (`travel` the fraction moved, 0 .. 1).
pub fn shaped(settings: &AxisSettings, travel: f64) -> f64 {
    let dead = settings.dead_zone.min(1000) as f64 / 1000.0;
    let range = settings.range.clamp(1, 1000) as f64 / 1000.0;
    let m = if travel <= dead {
        0.0
    } else if range <= dead {
        1.0
    } else {
        ((travel - dead) / (range - dead)).min(1.0)
    };
    let a = settings
        .sensitivity
        .clamp(-SENSITIVITY_LIMIT, SENSITIVITY_LIMIT) as f64
        / SENSITIVITY_LIMIT as f64;
    if a >= 0.0 {
        m + a * (m * m - m)
    } else {
        m - a * (2.0 * m - m * m - m)
    }
}

/// The game's scale of a value 0 .. 1.
fn scaled(v: f64) -> i32 {
    (v * FULL as f64).round() as i32
}

/// An axis binding's value on the game's scale: -0x100 .. 0x100 for a centred axis, 0 .. 0x100
/// for the others.
pub fn axis_value(profile: &Profile, using: &AxisUse, raw: &RawInput) -> i32 {
    let axis = using.axis();
    let settings = profile.settings(axis);
    let mut u = raw.axes.get(axis as usize).map_or(0.0, |&v| unit(v));
    if settings.invert {
        u = -u;
    }
    match *using {
        AxisUse::Centred { .. } => {
            let size = scaled(shaped(&settings, u.abs()));
            if u < 0.0 { -size } else { size }
        }
        AxisUse::Pedal { rest_high, .. } => {
            let travel = if rest_high {
                (1.0 - u) / 2.0
            } else {
                (1.0 + u) / 2.0
            };
            scaled(shaped(&settings, travel))
        }
        AxisUse::Half { positive, .. } => {
            let travel = if positive { u } else { -u };
            scaled(shaped(&settings, travel.max(0.0)))
        }
    }
}

/// The hat's bits of a reading, as the race has always had them: with no shifter the hat's
/// angle in 256ths in the top byte and "no direction" (0xE00); with an H shifter the gear it is
/// in, as `shifter_bits` says.
fn hat_bits(raw: &RawInput, bindings: &Bindings, shifter: bool) -> u32 {
    if shifter {
        return shifter_bits(raw, bindings);
    }
    let hat = raw.hats[0];
    let angle = if hat & 0xFFFF == 0xFFFF {
        0
    } else {
        (hat << 8) / 0x8CA0
    };
    (angle << 24) | HAT_CENTRED
}

/// The gear an H shifter is in, as the bits the race reads (`Gear::code`; 0, neutral, when no
/// bound input is held): the first of gear 1 .. 7 and reverse whose bound input is held. A profile
/// with no gear bound at all (made before the gears could be bound) reads the shifter the way
/// the first one supported was wired: buttons 6, 7, 1 and 2 are gears 1 .. 4, the hat up,
/// left and down gears 5 .. 7 and the hat right reverse.
fn shifter_bits(raw: &RawInput, bindings: &Bindings) -> u32 {
    if Gear::ALL.iter().all(|&g| bindings.gear(g).is_none()) {
        return fixed_shifter_bits(raw);
    }
    Gear::ALL
        .iter()
        .find(|&&gear| bindings.gear(gear).is_some_and(|d| d.pressed(raw)))
        .map_or(0, |gear| gear.code())
}

/// The shifter's gear with its inputs as they were before they could be bound.
fn fixed_shifter_bits(raw: &RawInput) -> u32 {
    for (button, gear) in [(5, 1), (6, 2), (0, 3), (1, 4)] {
        if raw.buttons >> button & 1 != 0 {
            return Gear::Forward(gear).code();
        }
    }
    match raw.hats[0] {
        0 => Gear::Forward(5).code(),
        27000 => Gear::Forward(6).code(),
        18000 => Gear::Forward(7).code(),
        9000 => Gear::Reverse.code(),
        _ => 0,
    }
}

/// A controller's reading as the race wants it: steering (`FULL_LEFT` .. `FULL`), brake and
/// throttle (0 .. `FULL`) and the bits (the eight buttons as bits 1 .. 0x80, 0x1000 / 0x2000 /
/// 0x4000 for a digital steering / brake / throttle, the hat's). A key or a digital binding gives
/// all of a reading; pressing both left and right gives none. `raw` none: the controller is not
/// there, only the keys count. `shifter`: the controller's hat and buttons are a gear shifter's.
pub fn readings(
    profile: Option<&Profile>,
    raw: Option<&RawInput>,
    keys: &Keys,
    shifter: bool,
) -> Readings {
    // A player with no controller, or a controller that is not there, reads as its keys alone.
    let none = Bindings::default();
    let still = RawInput::default();
    let (profile, b, raw) = match (profile, raw) {
        (Some(profile), Some(raw)) => (profile, &profile.bindings, raw),
        (Some(profile), None) => (profile, &none, &still),
        (None, _) => return keys_only(keys),
    };
    let pressed =
        |action: Action| keys.down(action) || b.digital(action).is_some_and(|d| d.pressed(raw));
    let mut bits = hat_bits(raw, b, shifter);
    let (left, right) = (pressed(Action::Left), pressed(Action::Right));
    let steering = if left || right {
        bits |= 0x1000;
        match (left, right) {
            (true, false) => FULL_LEFT,
            (false, true) => FULL,
            _ => 0,
        }
    } else {
        b.steering_axis
            .map_or(0, |using| axis_value(profile, &using, raw)) as u32
    };
    let mut pedal = |action: Action, using: Option<AxisUse>, flag: u32| {
        if pressed(action) {
            bits |= flag;
            FULL
        } else {
            using.map_or(0, |using| axis_value(profile, &using, raw).max(0) as u32)
        }
    };
    let throttle = pedal(Action::Accelerate, b.accelerate_axis, 0x4000);
    let brake = pedal(Action::Brake, b.brake_axis, 0x2000);
    for (action, bit) in BUTTON_BITS {
        if pressed(action) {
            bits |= bit;
        }
    }
    Readings {
        steering,
        brake,
        throttle,
        bits,
    }
}

/// The reading of a player with no controller: the keys alone.
fn keys_only(keys: &Keys) -> Readings {
    let key = |action: Action| keys.down(action);
    let mut bits = HAT_CENTRED;
    let steering = match (key(Action::Left), key(Action::Right)) {
        (false, false) => 0,
        (true, false) => FULL_LEFT,
        (false, true) => FULL,
        (true, true) => 0,
    };
    if key(Action::Left) || key(Action::Right) {
        bits |= 0x1000;
    }
    let mut pedal = |action: Action, flag: u32| {
        if key(action) {
            bits |= flag;
            FULL
        } else {
            0
        }
    };
    let throttle = pedal(Action::Accelerate, 0x4000);
    let brake = pedal(Action::Brake, 0x2000);
    for (action, bit) in BUTTON_BITS {
        if key(action) {
            bits |= bit;
        }
    }
    Readings {
        steering,
        brake,
        throttle,
        bits,
    }
}

/// The actions the eight buttons' bits stand for.
const BUTTON_BITS: [(Action, u32); 8] = [
    (Action::GearUp, 0x01),
    (Action::GearDown, 0x02),
    (Action::Handbrake, 0x04),
    (Action::Lights, 0x08),
    (Action::Wipers, 0x10),
    (Action::Camera, 0x20),
    (Action::Pause, 0x40),
    (Action::Reset, 0x80),
];

// ---- learning ----

/// A row of the controller buttons page, from 1 as the page numbers its controls: the digital
/// rows take a button or a hat direction, the three axis rows an axis.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Row {
    /// Accelerate with a button.
    Accelerate,
    /// Brake with a button.
    Brake,
    /// Steer left with a button.
    Left,
    /// Steer right with a button.
    Right,
    GearUp,
    GearDown,
    Handbrake,
    Camera,
    Pause,
    Lights,
    Reset,
    /// The axis that steers.
    SteeringAxis,
    /// The axis that accelerates.
    AccelerateAxis,
    /// The axis that brakes.
    BrakeAxis,
    /// An H shifter's gear (the page lists these only while the shifter switch is on).
    Shifter(Gear),
}

impl Row {
    /// The rows in the order the page lists them, without the H shifter's.
    pub const LISTED: [Row; 14] = [
        Row::SteeringAxis,
        Row::AccelerateAxis,
        Row::BrakeAxis,
        Row::Left,
        Row::Right,
        Row::Accelerate,
        Row::Brake,
        Row::GearUp,
        Row::GearDown,
        Row::Handbrake,
        Row::Camera,
        Row::Pause,
        Row::Lights,
        Row::Reset,
    ];

    /// Every row, the H shifter's last.
    pub const ALL: [Row; 22] = [
        Row::SteeringAxis,
        Row::AccelerateAxis,
        Row::BrakeAxis,
        Row::Left,
        Row::Right,
        Row::Accelerate,
        Row::Brake,
        Row::GearUp,
        Row::GearDown,
        Row::Handbrake,
        Row::Camera,
        Row::Pause,
        Row::Lights,
        Row::Reset,
        Row::Shifter(Gear::Forward(1)),
        Row::Shifter(Gear::Forward(2)),
        Row::Shifter(Gear::Forward(3)),
        Row::Shifter(Gear::Forward(4)),
        Row::Shifter(Gear::Forward(5)),
        Row::Shifter(Gear::Forward(6)),
        Row::Shifter(Gear::Forward(7)),
        Row::Shifter(Gear::Reverse),
    ];

    /// The rows the page lists: `LISTED`, and with the H shifter switched on the gears' too.
    pub fn listed(shifter: bool) -> &'static [Row] {
        if shifter { &Row::ALL } else { &Row::LISTED }
    }

    /// The row of control `control` (1 ..= 23).
    pub fn of_control(control: u32) -> Option<Row> {
        Some(match control {
            1 => Row::Accelerate,
            2 => Row::Brake,
            3 => Row::Left,
            4 => Row::Right,
            5 => Row::GearUp,
            6 => Row::GearDown,
            7 => Row::Handbrake,
            8 => Row::Camera,
            9 => Row::Pause,
            10 => Row::Lights,
            11 => Row::Reset,
            13 => Row::SteeringAxis,
            14 => Row::AccelerateAxis,
            15 => Row::BrakeAxis,
            16..=22 => Row::Shifter(Gear::Forward(control as u8 - 15)),
            23 => Row::Shifter(Gear::Reverse),
            _ => return None,
        })
    }

    /// The row's place in `ALL`.
    pub fn place(self) -> usize {
        Row::ALL.iter().position(|&r| r == self).unwrap_or(0)
    }

    /// The control number of the row (the inverse of `of_control`).
    pub fn control(self) -> u32 {
        LISTED_CONTROLS[self.place()]
    }

    fn action(self) -> Option<Action> {
        Some(match self {
            Row::Accelerate => Action::Accelerate,
            Row::Brake => Action::Brake,
            Row::Left => Action::Left,
            Row::Right => Action::Right,
            Row::GearUp => Action::GearUp,
            Row::GearDown => Action::GearDown,
            Row::Handbrake => Action::Handbrake,
            Row::Camera => Action::Camera,
            Row::Pause => Action::Pause,
            Row::Lights => Action::Lights,
            Row::Reset => Action::Reset,
            Row::SteeringAxis | Row::AccelerateAxis | Row::BrakeAxis | Row::Shifter(_) => {
                return None;
            }
        })
    }

    /// Whether the row takes an axis (the other rows take a button or a hat direction).
    pub fn takes_axis(self) -> bool {
        matches!(
            self,
            Row::SteeringAxis | Row::AccelerateAxis | Row::BrakeAxis
        )
    }
}

/// The control number of each of `Row::ALL`.
const LISTED_CONTROLS: [u32; 22] = [
    13, 14, 15, 3, 4, 1, 2, 5, 6, 7, 8, 9, 10, 11, 16, 17, 18, 19, 20, 21, 22, 23,
];

/// How far an axis must move from where it rested (of -32768 .. 32767) to count.
pub const MOVED: i32 = 16384;

/// What a controller's movement shows the player meant.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Learned {
    /// An axis moved from `rest` to `now`.
    Axis {
        axis: u8,
        rest: i32,
        now: i32,
    },
    Digital(Digital),
}

/// What moved from `rest` to `now`. For an axis row (`axes`) the axis that moved furthest past
/// `MOVED` among the first `axis_count`; for the others a button newly down (alone) or a hat
/// newly off its centre.
pub fn detect(now: &RawInput, rest: &RawInput, axis_count: usize, axes: bool) -> Option<Learned> {
    if axes {
        return (0..axis_count.min(AXES))
            .map(|n| (n, (now.axes[n] - rest.axes[n]).abs()))
            .filter(|&(_, moved)| moved > MOVED)
            .max_by_key(|&(_, moved)| moved)
            .map(|(n, _)| Learned::Axis {
                axis: n as u8,
                rest: rest.axes[n],
                now: now.axes[n],
            });
    }
    let fresh = now.buttons & !rest.buttons;
    if fresh.count_ones() == 1 {
        return Some(Learned::Digital(Digital::Button {
            button: fresh.trailing_zeros() as u8,
        }));
    }
    for (hat, (&angle, &was)) in now.hats.iter().zip(&rest.hats).enumerate() {
        if angle < 36000 && was >= 36000 {
            return Some(Learned::Digital(Digital::Hat {
                hat: hat as u8,
                direction: HatDirection::nearest(angle),
            }));
        }
    }
    None
}

/// What the profile does for a learnt thing.
/// - an axis for the accelerate and brake axis rows: a pedal when it rested at an end of its
///   travel (past three quarters), else one side of it, the side it moved to;
/// - an axis for the steering row: steering, taking the side it moved to as right (moved the
///   other way, the axis is inverted);
/// - a button or a hat direction: the digital row's binding.
///
/// A row never changes the bindings of the others: a button for accelerate leaves the axis
/// that accelerates as it was.
pub fn learn(profile: &mut Profile, row: Row, learned: Learned) {
    match learned {
        Learned::Digital(digital) => {
            if let Some(action) = row.action() {
                if let Some(slot) = profile.bindings.digital_mut(action) {
                    *slot = Some(digital);
                }
            } else if let Row::Shifter(gear) = row
                && let Some(slot) = profile.bindings.gear_mut(gear)
            {
                *slot = Some(digital);
            }
        }
        Learned::Axis { axis, rest, now } => {
            let up = now > rest;
            let using = if rest.abs() > 3 * 32768 / 4 {
                AxisUse::Pedal {
                    axis,
                    rest_high: rest > 0,
                }
            } else {
                AxisUse::Half { axis, positive: up }
            };
            match row {
                Row::AccelerateAxis => profile.bindings.accelerate_axis = Some(using),
                Row::BrakeAxis => profile.bindings.brake_axis = Some(using),
                Row::SteeringAxis => {
                    profile.bindings.steering_axis = Some(AxisUse::Centred { axis });
                    profile.settings_mut(axis).invert = !up;
                }
                _ => {}
            }
        }
    }
}

/// What a row shows on the controller buttons page.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Shown {
    Nothing,
    /// A button, numbered from 1.
    Button(u32),
    Hat(HatDirection),
    /// An axis; the `bool`: the side of it, none for a centred one.
    Axis(u8, Option<bool>),
}

/// What the row is bound to: an axis row its axis, a digital row its button or hat direction.
pub fn shown(profile: &Profile, row: Row) -> Shown {
    let b = &profile.bindings;
    let digital = match row {
        Row::Shifter(gear) => Some(b.gear(gear)),
        _ => row.action().map(|action| b.digital(action)),
    };
    if let Some(digital) = digital {
        return match digital {
            Some(Digital::Button { button }) => Shown::Button(button as u32 + 1),
            Some(Digital::Hat { direction, .. }) => Shown::Hat(direction),
            None => Shown::Nothing,
        };
    }
    let using = match row {
        Row::AccelerateAxis => b.accelerate_axis,
        Row::BrakeAxis => b.brake_axis,
        _ => b.steering_axis,
    };
    match using {
        None => Shown::Nothing,
        Some(AxisUse::Centred { axis }) => Shown::Axis(axis, None),
        Some(AxisUse::Pedal { axis, rest_high }) => Shown::Axis(axis, Some(!rest_high)),
        Some(AxisUse::Half { axis, positive }) => Shown::Axis(axis, Some(positive)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn id() -> DeviceId {
        DeviceId {
            name: "Pad".into(),
            serial: None,
            nth: 1,
        }
    }

    fn raw(axis: usize, v: i32) -> RawInput {
        let mut r = RawInput::default();
        r.axes[axis] = v;
        r
    }

    fn only(using: AxisUse, settings: AxisSettings) -> Profile {
        let mut p = Profile::new(id(), Kind::Joystick);
        p.bindings = Bindings::default();
        p.axes = vec![AxisSettings::default(); AXES];
        p.axes[using.axis() as usize] = settings;
        p
    }

    #[test]
    fn a_pedal_resting_low_and_one_resting_high() {
        let low = AxisUse::Pedal {
            axis: 4,
            rest_high: false,
        };
        let high = AxisUse::Pedal {
            axis: 4,
            rest_high: true,
        };
        let p = only(low, AxisSettings::default());
        assert_eq!(axis_value(&p, &low, &raw(4, -32768)), 0);
        assert_eq!(axis_value(&p, &low, &raw(4, 32767)), 0x100);
        assert_eq!(axis_value(&p, &low, &raw(4, 0)), 0x80);
        assert_eq!(axis_value(&p, &high, &raw(4, 32767)), 0);
        assert_eq!(axis_value(&p, &high, &raw(4, -32768)), 0x100);
        // inverted: the other end rests
        let mut q = p.clone();
        q.axes[4].invert = true;
        assert_eq!(axis_value(&q, &low, &raw(4, 32767)), 0);
        assert_eq!(axis_value(&q, &low, &raw(4, -32768)), 0x100);
    }

    #[test]
    fn a_wheel_with_a_short_range_locks_early() {
        let wheel = AxisUse::Centred { axis: 0 };
        let p = only(
            wheel,
            AxisSettings {
                range: 300,
                ..AxisSettings::default()
            },
        );
        // 30 % of the half travel either side of the centre is full lock
        assert_eq!(axis_value(&p, &wheel, &raw(0, 0)), 0);
        assert_eq!(axis_value(&p, &wheel, &raw(0, 9830)), 0x100);
        assert_eq!(axis_value(&p, &wheel, &raw(0, -9830)), -0x100);
        assert_eq!(axis_value(&p, &wheel, &raw(0, 32767)), 0x100);
        let half = axis_value(&p, &wheel, &raw(0, 4915));
        assert!((0x7F..=0x81).contains(&half), "{half}");
        // 900 degrees lock to lock: a range of 300 is 270 degrees
        assert_eq!(p.range_degrees(0), 270);
    }

    #[test]
    fn dead_zones() {
        let wheel = AxisUse::Centred { axis: 0 };
        let p = only(
            wheel,
            AxisSettings {
                dead_zone: 200,
                ..AxisSettings::default()
            },
        );
        assert_eq!(axis_value(&p, &wheel, &raw(0, 6000)), 0);
        assert_eq!(axis_value(&p, &wheel, &raw(0, -6000)), 0);
        assert_eq!(axis_value(&p, &wheel, &raw(0, 32767)), 0x100);
        assert!(axis_value(&p, &wheel, &raw(0, 8000)) > 0);
        // a half: only its side
        let up = AxisUse::Half {
            axis: 1,
            positive: false,
        };
        let p = only(up, AxisSettings::default());
        assert_eq!(axis_value(&p, &up, &raw(1, -32768)), 0x100);
        assert_eq!(axis_value(&p, &up, &raw(1, 32767)), 0);
        assert_eq!(axis_value(&p, &up, &raw(1, 0)), 0);
        // a dead zone as wide as the range: all or nothing
        let p = only(
            up,
            AxisSettings {
                dead_zone: 500,
                range: 400,
                ..AxisSettings::default()
            },
        );
        assert_eq!(axis_value(&p, &up, &raw(1, -20000)), 0x100);
        assert_eq!(axis_value(&p, &up, &raw(1, -10000)), 0);
    }

    #[test]
    fn curves_rise_and_keep_their_ends() {
        for sensitivity in (-SENSITIVITY_LIMIT..=SENSITIVITY_LIMIT).step_by(10) {
            let s = AxisSettings {
                sensitivity,
                ..AxisSettings::default()
            };
            assert_eq!(shaped(&s, 0.0), 0.0);
            assert!((shaped(&s, 1.0) - 1.0).abs() < 1e-12);
            let mut last = 0.0;
            for n in 0..=1000 {
                let v = shaped(&s, n as f64 / 1000.0);
                assert!(v >= last - 1e-12, "{sensitivity} falls at {n}");
                last = v;
            }
        }
        let soft = AxisSettings {
            sensitivity: 100,
            ..AxisSettings::default()
        };
        let sharp = AxisSettings {
            sensitivity: -100,
            ..AxisSettings::default()
        };
        assert!(shaped(&soft, 0.5) < 0.5 && shaped(&sharp, 0.5) > 0.5);
        // out of range values are held
        let wild = AxisSettings {
            sensitivity: 9000,
            ..AxisSettings::default()
        };
        assert!((shaped(&wild, 0.5) - 0.25).abs() < 1e-12);
    }

    #[test]
    fn keys_win_and_set_the_flags() {
        let p = Profile::new(id(), Kind::Pad);
        let mut keys = Keys::default();
        // a gamepad at rest: its triggers at the low end of their travel
        let mut still = RawInput::default();
        still.axes[4] = -32768;
        still.axes[5] = -32768;
        let r = readings(Some(&p), Some(&still), &keys, false);
        assert_eq!((r.steering, r.throttle, r.brake), (0, 0, 0));
        assert_eq!(r.bits, HAT_CENTRED);
        // the right trigger as an analogue throttle, no flag
        let mut pressed = still;
        pressed.axes[5] = 32767;
        let r = readings(Some(&p), Some(&pressed), &keys, false);
        assert_eq!((r.throttle, r.bits & 0x7000), (0x100, 0));
        // a key: all of it, with its flag, over the axis
        keys.0[Action::Accelerate as usize] = true;
        keys.0[Action::Left as usize] = true;
        keys.0[Action::Wipers as usize] = true;
        keys.0[Action::Brake as usize] = true;
        let r = readings(Some(&p), Some(&raw(0, 20000)), &keys, false);
        assert_eq!(r.steering, FULL_LEFT);
        assert_eq!((r.throttle, r.brake), (FULL, FULL));
        assert_eq!(r.bits & 0x7000, 0x7000);
        assert_eq!(r.bits & 0xFF, 0x10);
        // a controller that is not there: the keys alone
        let r = readings(Some(&p), None, &keys, false);
        assert_eq!((r.throttle, r.brake, r.steering), (FULL, FULL, FULL_LEFT));
        assert_eq!(readings(None, Some(&still), &keys, false), r);
        keys.0[Action::Left as usize] = false;
        keys.0[Action::Accelerate as usize] = false;
        let r = readings(Some(&p), None, &keys, false);
        assert_eq!((r.throttle, r.steering), (0, 0));
        keys.0[Action::Left as usize] = true;
        keys.0[Action::Accelerate as usize] = true;
        // both sides: nothing, still digital
        keys.0[Action::Right as usize] = true;
        let r = readings(Some(&p), Some(&still), &keys, false);
        assert_eq!(r.steering, 0);
        assert_eq!(r.bits & 0x1000, 0x1000);
    }

    #[test]
    fn digital_bindings_and_the_bits() {
        let mut p = Profile::new(id(), Kind::Joystick);
        p.bindings.pause = Some(Digital::Button { button: 9 });
        p.bindings.reset = Some(Digital::Hat {
            hat: 1,
            direction: HatDirection::Down,
        });
        p.bindings.lights = Some(Digital::Button { button: 40 });
        let mut r = RawInput {
            buttons: 1 | 1 << 9,
            ..RawInput::default()
        };
        r.hats[1] = 18000;
        let got = readings(Some(&p), Some(&r), &Keys::default(), false);
        assert_eq!(got.bits & 0xFF, 0x01 | 0x40 | 0x80);
        // a hat's diagonal presses both of its sides, the other directions not
        r.hats[1] = 13500;
        let got = readings(Some(&p), Some(&r), &Keys::default(), false);
        assert_eq!(got.bits & 0x80, 0x80);
        r.hats[1] = 9000;
        assert_eq!(
            readings(Some(&p), Some(&r), &Keys::default(), false).bits & 0x80,
            0
        );
        // the steering stick and a held left button
        let mut r = raw(0, -32767);
        let still = readings(Some(&p), Some(&r), &Keys::default(), false);
        assert_eq!((still.steering, still.bits & 0x1000), (FULL_LEFT, 0));
        p.bindings.left = Some(Digital::Button { button: 2 });
        r.buttons = 4;
        let held = readings(Some(&p), Some(&r), &Keys::default(), false);
        assert_eq!((held.steering, held.bits & 0x1000), (FULL_LEFT, 0x1000));
    }

    #[test]
    fn the_hat_bits_as_before() {
        let none = Bindings::default();
        let mut r = RawInput::default();
        assert_eq!(hat_bits(&r, &none, false), 0xE00);
        r.hats[0] = 9000;
        assert_eq!(hat_bits(&r, &none, false), (64 << 24) | 0xE00);
        // a shifter with no gear bound: the first wiring
        assert_eq!(hat_bits(&r, &none, true), 0xF00);
        r.hats[0] = u32::MAX;
        r.buttons = 1 << 6;
        assert_eq!(hat_bits(&r, &none, true), 0x200);
        assert_eq!(hat_bits(&RawInput::default(), &none, true), 0);
        let mut r = RawInput::default();
        for (hat, gear) in [(0, 0x500), (27000, 0x600), (18000, 0x700)] {
            r.hats[0] = hat;
            assert_eq!(hat_bits(&r, &none, true), gear);
        }
    }

    fn shifter_profile() -> Profile {
        let mut p = Profile::new(id(), Kind::Wheel);
        let button = |button| Some(Digital::Button { button });
        let b = &mut p.bindings;
        b.gear_1 = button(10);
        b.gear_2 = button(11);
        b.gear_3 = Some(Digital::Hat {
            hat: 1,
            direction: HatDirection::Up,
        });
        b.gear_7 = button(12);
        b.reverse = button(13);
        p
    }

    #[test]
    fn a_bound_shifter_gives_the_gear_held() {
        let p = shifter_profile();
        let keys = Keys::default();
        let gear = |raw: &RawInput| readings(Some(&p), Some(raw), &keys, true).bits;
        // neutral: nothing held, and the old fixed inputs mean nothing now
        assert_eq!(gear(&RawInput::default()), 0);
        let old = RawInput {
            buttons: 1 << 5,
            ..RawInput::default()
        };
        assert_eq!(gear(&old), 0);
        for (button, code) in [(10, 0x100), (11, 0x200), (12, 0x700), (13, 0xF00)] {
            let held = RawInput {
                buttons: 1 << button,
                ..RawInput::default()
            };
            assert_eq!(gear(&held), code, "button {button}");
        }
        let mut hat = RawInput::default();
        hat.hats[1] = 0;
        assert_eq!(gear(&hat), 0x300);
        // two held: the lower gear
        hat.buttons = 1 << 11 | 1 << 13;
        assert_eq!(gear(&hat), 0x200);
        // the switch off: the hat's angle as for any controller
        let off = readings(Some(&p), Some(&hat), &keys, false).bits;
        assert_eq!(off & 0xFFF, 0xE00);
    }

    #[test]
    fn gears_are_learnt_shown_and_saved() {
        let mut p = Profile::new(id(), Kind::Wheel);
        let found = Learned::Digital(Digital::Button { button: 4 });
        for (n, row) in Row::ALL[14..].iter().enumerate() {
            assert!(!row.takes_axis());
            assert_eq!(shown(&p, *row), Shown::Nothing);
            learn(&mut p, *row, found);
            assert_eq!(shown(&p, *row), Shown::Button(5), "row {n}");
            assert_eq!(Row::of_control(row.control()), Some(*row));
        }
        assert_eq!(p.bindings.gear_5, Some(Digital::Button { button: 4 }));
        assert_eq!(p.bindings.reverse, Some(Digital::Button { button: 4 }));
        let text = toml::to_string(&p).unwrap();
        for word in [
            "[bindings.gear_1]",
            "[bindings.gear_7]",
            "[bindings.reverse]",
        ] {
            assert!(text.contains(word), "{word} not in\n{text}");
        }
        assert_eq!(toml::from_str::<Profile>(&text).unwrap(), p);
        // the shifter switch is saved only when on, and an old file without it reads off
        assert!(!text.contains("shifter"));
        p.shifter = true;
        let on = toml::to_string(&p).unwrap();
        assert!(toml::from_str::<Profile>(&on).unwrap().shifter);
        // unbound gears are not written
        let bare = toml::to_string(&Profile::new(id(), Kind::Wheel)).unwrap();
        assert!(!bare.contains("gear_1") && !bare.contains("reverse"));
        // the page lists the gears only with the switch on
        assert_eq!(Row::listed(false).len(), 14);
        assert_eq!(Row::listed(true).len(), 22);
    }

    #[test]
    fn learning_a_pedal_and_a_wheel() {
        let mut p = Profile::new(id(), Kind::Wheel);
        assert_eq!(p.bindings.accelerate_axis, None);
        // the clutch rests at the top, the accelerator too: only the one pressed counts
        let mut rest = RawInput::default();
        rest.axes[1] = 32767;
        rest.axes[2] = 32767;
        let mut now = rest;
        now.axes[2] = -20000;
        let found = detect(&now, &rest, 4, true).unwrap();
        learn(&mut p, Row::AccelerateAxis, found);
        assert_eq!(
            p.bindings.accelerate_axis,
            Some(AxisUse::Pedal {
                axis: 2,
                rest_high: true
            })
        );
        // a centred axis moved down is half of it
        let mut now = RawInput::default();
        now.axes[1] = -25000;
        let found = detect(&now, &RawInput::default(), 4, true).unwrap();
        learn(&mut p, Row::BrakeAxis, found);
        assert_eq!(
            p.bindings.brake_axis,
            Some(AxisUse::Half {
                axis: 1,
                positive: false
            })
        );
        // steering learnt with the wheel to the large values: not inverted; the other way: inverted
        let found = detect(&raw(0, 30000), &RawInput::default(), 4, true).unwrap();
        learn(&mut p, Row::SteeringAxis, found);
        assert_eq!(p.bindings.steering_axis, Some(AxisUse::Centred { axis: 0 }));
        assert!(!p.settings(0).invert);
        let found = detect(&raw(0, -30000), &RawInput::default(), 4, true).unwrap();
        learn(&mut p, Row::SteeringAxis, found);
        assert!(p.settings(0).invert);
        // nothing moved, an axis past the count, an axis for a digital row
        assert_eq!(detect(&rest, &rest, 4, true), None);
        assert_eq!(detect(&raw(6, 30000), &RawInput::default(), 4, true), None);
        assert_eq!(detect(&raw(0, 30000), &RawInput::default(), 4, false), None);
        // an axis row does not take a button
        let pressed = RawInput {
            buttons: 1,
            ..RawInput::default()
        };
        assert_eq!(detect(&pressed, &RawInput::default(), 4, true), None);
    }

    #[test]
    fn learning_buttons_and_hats() {
        let mut p = Profile::new(id(), Kind::Pad);
        let mut now = RawInput {
            buttons: 1 << 7,
            ..RawInput::default()
        };
        let rest = RawInput {
            buttons: 1 << 2,
            ..RawInput::default()
        };
        let found = detect(&now, &rest, 6, false).unwrap();
        learn(&mut p, Row::Pause, found);
        assert_eq!(p.bindings.pause, Some(Digital::Button { button: 7 }));
        assert_eq!(shown(&p, Row::Pause), Shown::Button(8));
        // two new buttons: not one
        now.buttons |= 1 << 3;
        assert_eq!(detect(&now, &rest, 6, false), None);
        let mut hat = RawInput::default();
        hat.hats[0] = 27000;
        let found = detect(&hat, &RawInput::default(), 6, false).unwrap();
        learn(&mut p, Row::Lights, found);
        assert_eq!(shown(&p, Row::Lights), Shown::Hat(HatDirection::Left));
        // a button for accelerate leaves the axis that accelerates
        let found = Learned::Digital(Digital::Button { button: 1 });
        learn(&mut p, Row::Accelerate, found);
        assert!(p.bindings.accelerate_axis.is_some());
        assert_eq!(shown(&p, Row::Accelerate), Shown::Button(2));
        assert_eq!(shown(&p, Row::BrakeAxis), Shown::Axis(4, Some(true)));
        assert_eq!(shown(&p, Row::Left), Shown::Hat(HatDirection::Left));
        // the wipers have no row and no binding: the race reads none
        assert_eq!(p.bindings.digital(Action::Wipers), None);
        // every listed row has its control number back
        for row in Row::ALL {
            assert_eq!(Row::of_control(row.control()), Some(row));
        }
    }

    #[test]
    fn the_defaults() {
        let p = Profile::new(id(), Kind::Pad);
        assert_eq!(shown(&p, Row::GearUp), Shown::Button(1));
        assert_eq!(shown(&p, Row::Camera), Shown::Button(4));
        assert_eq!(shown(&p, Row::Pause), Shown::Nothing);
        assert!(!p.wheel);
        let w = Profile::new(id(), Kind::Wheel);
        assert!(w.wheel && w.full_rotation_degrees == 900);
        assert_eq!(
            (w.bindings.accelerate_axis, w.bindings.brake_axis),
            (None, None)
        );
        let j = Profile::new(id(), Kind::Joystick);
        assert_eq!(shown(&j, Row::AccelerateAxis), Shown::Axis(1, Some(false)));
        assert_eq!(axis_label(Kind::Pad, 5), "rt");
        assert_eq!(axis_label(Kind::Joystick, 3), "rx");
        assert_eq!(axis_label(Kind::Joystick, 7), "a7");
    }

    #[test]
    fn the_file_text_names_everything() {
        let mut p = Profile::new(id(), Kind::Pad);
        p.bindings.pause = Some(Digital::Button { button: 7 });
        let text = toml::to_string(&p).unwrap();
        for word in [
            "steering_axis",
            "kind = \"centred\"",
            "kind = \"pedal\"",
            "rest_high = false",
            "[bindings.gear_up]",
            "button = 0",
            "direction = \"left\"",
            "dead_zone = 100",
        ] {
            assert!(text.contains(word), "{word} not in\n{text}");
        }
        let back: Profile = toml::from_str(&text).unwrap();
        assert_eq!(back, p);
        // a profile with only its device is the empty one
        let bare: Profile = toml::from_str("[device]\nname = \"X\"\nnth = 1\n").unwrap();
        assert_eq!(bare.full_rotation_degrees, 900);
        assert_eq!(bare.bindings, Bindings::default());
    }
}
