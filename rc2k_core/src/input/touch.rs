//! The touch screen's controls in a race, shared by the phone and tablet builds (Android now, iOS
//! to come): no platform, only fingers in and what the game reads out. (The menus' fingers:
//! `menu_touch`.)
//!
//! A race has a layout for a phone held sideways: a steering area on the left (a finger put down
//! anywhere in it is the wheel's middle; moving it left and right steers, and past the reach the
//! middle follows the finger, so letting it come back is easy), pedals and the handbrake on the
//! right, small buttons for the gears, the camera (the lights under it) and the pause at the top, and the gears again under the pause. The game reads them as
//! a gamepad (`Output`: steering -1 .. 1 and the buttons held).
//!
//! Positions are in the window's pixels, times in milliseconds.

use super::menu_touch::{Phase, Rect, Screen};

/// The touch screen's virtual gamepad as the controller list names it.
pub const PAD_NAME: &str = "Touch screen";

/// The pad button the overlay's LIGHTS presses, in the profiles' numbering (Back).
pub const LIGHTS_BUTTON: u8 = 6;

/// A kept profile of the touch screen's pad from before the overlay had a LIGHTS button: the
/// lights bound to what that button presses, when the player has them on nothing.
pub fn give_lights(profile: &mut super::profile::Profile) {
    if profile.device.name == PAD_NAME && profile.bindings.lights.is_none() {
        profile.bindings.lights = Some(super::profile::Digital::Button {
            button: LIGHTS_BUTTON,
        });
    }
}

/// The buttons of the race's layout.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum Button {
    Throttle,
    Brake,
    Handbrake,
    GearUp,
    GearDown,
    Camera,
    Lights,
    Pause,
}

/// How many buttons there are.
pub const BUTTONS: usize = 8;

impl Button {
    pub const ALL: [Button; BUTTONS] = [
        Button::Throttle,
        Button::Brake,
        Button::Handbrake,
        Button::GearUp,
        Button::GearDown,
        Button::Camera,
        Button::Lights,
        Button::Pause,
    ];

    fn index(self) -> usize {
        self as usize
    }

    /// Whether a brief touch must still be seen: the game reads the controls at its own turns,
    /// and a tap lasts shorter than one of them. Pedals follow the finger exactly.
    fn latched(self) -> bool {
        !matches!(self, Button::Throttle | Button::Brake | Button::Handbrake)
    }

    /// What the button shows.
    pub fn label(self) -> &'static str {
        match self {
            Button::Throttle => "GO",
            Button::Brake => "BRAKE",
            Button::Handbrake => "HAND",
            Button::GearUp => "+",
            Button::GearDown => "-",
            Button::Camera => "CAM",
            Button::Lights => "LIGHTS",
            Button::Pause => "II",
        }
    }
}

/// How long a tap on a latched button is held down for the game to see it.
pub(crate) const LATCH_MS: u64 = 120;

/// Where everything is for one screen.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Layout {
    /// Where a finger put down steers.
    pub wheel: Rect,
    /// How far the finger goes for full lock.
    pub reach: f32,
    pub buttons: [Rect; BUTTONS],
    /// The gears' second pair, on the left under the pause (up, then down): the same buttons as
    /// the pair on the right, for whichever hand is free.
    pub left_gears: [Rect; 2],
}

impl Layout {
    pub fn new(screen: &Screen) -> Layout {
        let u = screen.usable();
        // One unit for the buttons' sizes: a quarter of the usable height, at least a finger.
        let s = (u.h * 0.27).max(1.0);
        let gap = s * 0.08;
        let throttle = Rect::new(
            u.right() - 1.1 * s,
            u.bottom() - 1.75 * s,
            1.1 * s,
            1.75 * s,
        );
        let brake = Rect::new(
            throttle.x - gap - 0.95 * s,
            u.bottom() - 1.25 * s,
            0.95 * s,
            1.25 * s,
        );
        let handbrake = Rect::new(brake.x, brake.y - gap - 0.7 * s, 0.95 * s, 0.7 * s);
        let gear_up = Rect::new(u.right() - 0.8 * s, u.y + 0.1 * s, 0.8 * s, 0.7 * s);
        let gear_down = Rect::new(gear_up.x, gear_up.bottom() + gap, 0.8 * s, 0.7 * s);
        let camera = Rect::new(gear_up.x - gap - 1.0 * s, u.y + 0.1 * s, 1.0 * s, 0.7 * s);
        let lights = Rect::new(camera.x, camera.bottom() + gap, 1.0 * s, 0.525 * s);
        let pause = Rect::new(u.x + 0.1 * s, u.y + 0.1 * s, 0.7 * s, 0.7 * s);
        let left_up = Rect::new(u.x + 0.1 * s, pause.bottom() + gap, 0.8 * s, 0.525 * s);
        let left_down = Rect::new(left_up.x, left_up.bottom() + gap, 0.8 * s, 0.525 * s);
        let mut buttons = [Rect::default(); BUTTONS];
        for (button, rect) in [
            (Button::Throttle, throttle),
            (Button::Brake, brake),
            (Button::Handbrake, handbrake),
            (Button::GearUp, gear_up),
            (Button::GearDown, gear_down),
            (Button::Camera, camera),
            (Button::Lights, lights),
            (Button::Pause, pause),
        ] {
            buttons[button.index()] = rect;
        }
        Layout {
            wheel: Rect::new(u.x, u.y + 0.3 * u.h, 0.42 * u.w, 0.7 * u.h),
            reach: (0.085 * u.w).max(1.0),
            buttons,
            left_gears: [left_up, left_down],
        }
    }

    pub fn rect(&self, button: Button) -> Rect {
        self.buttons[button.index()]
    }

    /// Every button's place, the left gears' too (a button can have two).
    pub fn places(&self) -> impl Iterator<Item = (Button, Rect)> + '_ {
        Button::ALL.into_iter().map(|b| (b, self.rect(b))).chain(
            [Button::GearUp, Button::GearDown]
                .into_iter()
                .zip(self.left_gears),
        )
    }

    /// The button under a point (a finger on a button is not on the wheel).
    pub(crate) fn button_at(&self, x: f32, y: f32) -> Option<Button> {
        self.places()
            .find(|(_, r)| r.contains(x, y))
            .map(|(b, _)| b)
    }
}

/// What the race is waiting for while the controls are off.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Asked {
    /// The plain pause: any key goes on, one button does.
    Resume,
    /// A question with two answers (retire?).
    YesNo,
}

/// The answers' buttons.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Answer {
    Yes,
    No,
}

/// The answers' buttons for one screen: big, side by side in the lower middle of the usable
/// area, clear of the race's own question text near the top.
pub fn answer_rects(screen: &Screen, asked: Asked) -> Vec<(Answer, Rect)> {
    let u = screen.usable();
    let h = (u.h * 0.3).max(1.0);
    let y = u.y + u.h * 0.52;
    let gap = h * 0.4;
    match asked {
        Asked::Resume => {
            let w = (u.w * 0.4).max(1.0);
            vec![(Answer::Yes, Rect::new(u.x + (u.w - w) / 2.0, y, w, h))]
        }
        Asked::YesNo => {
            let w = ((u.w - gap) * 0.36).max(1.0);
            let left = u.x + (u.w - 2.0 * w - gap) / 2.0;
            vec![
                (Answer::Yes, Rect::new(left, y, w, h)),
                (Answer::No, Rect::new(left + w + gap, y, w, h)),
            ]
        }
    }
}

/// The finger that went down on an answer's button: the answer is given when the same finger
/// lifts on the same button (a tap elsewhere answers nothing).
#[derive(Debug, Default)]
pub struct AnswerTaps {
    down: Option<(u64, Answer)>,
}

impl AnswerTaps {
    pub const fn new() -> Self {
        AnswerTaps { down: None }
    }

    /// The answer a finger's change gives, if it is one.
    pub fn finger(
        &mut self,
        screen: &Screen,
        asked: Asked,
        id: u64,
        phase: Phase,
        x: f32,
        y: f32,
    ) -> Option<Answer> {
        let at = answer_rects(screen, asked)
            .into_iter()
            .find(|(_, r)| r.contains(x, y))
            .map(|(a, _)| a);
        match phase {
            Phase::Down => {
                self.down = at.map(|a| (id, a));
                None
            }
            Phase::Move => None,
            Phase::Up => match self.down.take() {
                Some((down_id, a)) if down_id == id && at == Some(a) => Some(a),
                _ => None,
            },
        }
    }

    /// The button pressed now, to be drawn lit.
    pub fn pressed(&self) -> Option<Answer> {
        self.down.map(|(_, a)| a)
    }

    pub fn release_all(&mut self) {
        self.down = None;
    }
}

/// What the game reads: the steering and the buttons held.
#[derive(Clone, Copy, PartialEq, Debug, Default)]
pub struct Output {
    /// -1 (left) .. 1 (right).
    pub steer: f32,
    pub held: [bool; BUTTONS],
}

impl Output {
    pub fn is_held(&self, button: Button) -> bool {
        self.held[button.index()]
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Role {
    Wheel { anchor: f32, steer: f32, y: f32 },
    Button(Button),
    Nothing,
}

#[derive(Clone, Copy, Debug)]
struct Finger {
    id: u64,
    role: Role,
}

/// The race's controls: the fingers down and what they are doing.
#[derive(Debug)]
pub struct Controls {
    layout: Layout,
    fingers: Vec<Finger>,
    /// Until when a latched button stays down.
    latch_until: [u64; BUTTONS],
}

/// The steering's dead zone, as a part of the reach.
const DEAD_ZONE: f32 = 0.06;

impl Controls {
    pub fn new(screen: &Screen) -> Self {
        Controls {
            layout: Layout::new(screen),
            fingers: Vec::new(),
            latch_until: [0; BUTTONS],
        }
    }

    pub fn layout(&self) -> &Layout {
        &self.layout
    }

    /// The screen's size or safe area changed: fingers on the old layout are let go.
    pub fn resize(&mut self, screen: &Screen) {
        let layout = Layout::new(screen);
        if layout != self.layout {
            self.layout = layout;
            self.fingers.clear();
        }
    }

    /// Every finger lifted (the app went to the background, a menu came up).
    pub fn release_all(&mut self) {
        self.fingers.clear();
    }

    /// A finger's change at (`x`, `y`).
    pub fn finger(&mut self, id: u64, phase: Phase, x: f32, y: f32, now: u64) {
        let at = self.fingers.iter().position(|f| f.id == id);
        match (phase, at) {
            (Phase::Up, Some(n)) => {
                self.fingers.remove(n);
            }
            (Phase::Up, None) => {}
            (Phase::Down, _) | (Phase::Move, None) => {
                if let Some(n) = at {
                    self.fingers.remove(n);
                }
                let role = self.role_for(x, y, now);
                self.fingers.push(Finger { id, role });
            }
            (Phase::Move, Some(n)) => {
                let reach = self.layout.reach;
                match self.fingers[n].role {
                    Role::Wheel { anchor, .. } => {
                        // Past the reach the middle follows, so the finger is always at the lock
                        // or inside it.
                        let anchor = anchor.clamp(x - reach, x + reach);
                        let steer = steering((x - anchor) / reach);
                        self.fingers[n].role = Role::Wheel { anchor, steer, y };
                    }
                    Role::Button(held) => {
                        // A finger that slides from one button to the next presses the next; a
                        // pedal stays held while the finger drifts off it onto nothing.
                        let role = match self.role_for(x, y, now) {
                            Role::Nothing if !held.latched() => Role::Button(held),
                            role => role,
                        };
                        self.fingers[n].role = role;
                    }
                    Role::Nothing => {
                        let role = self.role_for(x, y, now);
                        self.fingers[n].role = role;
                    }
                }
            }
        }
    }

    fn role_for(&mut self, x: f32, y: f32, now: u64) -> Role {
        if let Some(button) = self.layout.button_at(x, y) {
            let held_by_a_finger = self.fingers.iter().any(|f| f.role == Role::Button(button));
            if button.latched() && !held_by_a_finger {
                self.latch_until[button.index()] = now + LATCH_MS;
            }
            return Role::Button(button);
        }
        let wheel_taken = self
            .fingers
            .iter()
            .any(|f| matches!(f.role, Role::Wheel { .. }));
        if self.layout.wheel.contains(x, y) && !wheel_taken {
            return Role::Wheel {
                anchor: x,
                steer: 0.0,
                y,
            };
        }
        Role::Nothing
    }

    /// What the game reads now.
    pub fn output(&self, now: u64) -> Output {
        let mut out = Output::default();
        for finger in &self.fingers {
            match finger.role {
                Role::Wheel { steer, .. } => out.steer = steer,
                Role::Button(button) => out.held[button.index()] = true,
                Role::Nothing => {}
            }
        }
        for button in Button::ALL {
            if button.latched() && now < self.latch_until[button.index()] {
                out.held[button.index()] = true;
            }
        }
        out
    }

    /// Where the wheel's middle is (x, y) and the steering, for drawing (`None` while no finger
    /// steers).
    pub fn wheel_position(&self) -> Option<(f32, f32, f32)> {
        self.fingers.iter().find_map(|f| match f.role {
            Role::Wheel { anchor, steer, y } => Some((anchor, y, steer)),
            _ => None,
        })
    }
}

/// A steering from the finger's offset in reaches: a dead zone, then a straight rise.
fn steering(offset: f32) -> f32 {
    let magnitude = offset.abs();
    if magnitude <= DEAD_ZONE {
        return 0.0;
    }
    if magnitude >= 0.999 {
        return offset.signum();
    }
    (magnitude - DEAD_ZONE) / (1.0 - DEAD_ZONE) * offset.signum()
}

/// Which controls player 1 of a phone or tablet drives with.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum Source {
    /// Nothing has been used yet: the touch screen's pad stands in for the keyboard.
    #[default]
    Auto,
    /// A finger touched the screen in a race: the touch screen's pad, whatever else is chosen.
    Touch,
    /// A real controller was used (a button, a stick pushed): it drives, with its own profile.
    Pad,
}

/// The controller number (the game's own: 0 the keyboard, n the n-th of the controllers
/// listed) player 1 drives with, given `current` (what the player has) and the numbers of the
/// touch screen's pad and of the real controller to use (`None` when not listed).
///
/// `Auto`: only the keyboard (0) is replaced, by the touch pad. `Touch`: the touch pad. `Pad`:
/// the keyboard or the touch pad is replaced by the real controller; another real controller
/// the player chose is left.
pub fn player_one_controller(
    source: Source,
    current: u32,
    touch: Option<u32>,
    pad: Option<u32>,
) -> u32 {
    match source {
        Source::Auto if current == 0 => touch.unwrap_or(0),
        Source::Auto => current,
        Source::Touch => touch.unwrap_or(current),
        Source::Pad if current == 0 || Some(current) == touch => pad.or(touch).unwrap_or(current),
        Source::Pad => current,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn phone() -> Screen {
        Screen {
            width: 2400.0,
            height: 1080.0,
            left: 90.0,
            top: 0.0,
            right: 90.0,
            bottom: 0.0,
        }
    }

    fn centre(r: Rect) -> (f32, f32) {
        (r.x + r.w / 2.0, r.y + r.h / 2.0)
    }

    #[test]
    fn the_layout_stays_inside_the_safe_area_and_apart() {
        for screen in [
            phone(),
            Screen::new(1280.0, 720.0),
            Screen::new(2000.0, 1200.0),
        ] {
            let layout = Layout::new(&screen);
            let u = screen.usable();
            let places: Vec<_> = layout.places().collect();
            for &(b, r) in &places {
                assert!(
                    r.x >= u.x - 0.01 && r.right() <= u.right() + 0.01,
                    "{b:?} {r:?}"
                );
                assert!(
                    r.y >= u.y - 0.01 && r.bottom() <= u.bottom() + 0.01,
                    "{b:?} {r:?}"
                );
                assert!(r.w > 40.0 && r.h > 40.0, "{b:?} too small: {r:?}");
            }
            for (i, &(a, ra)) in places.iter().enumerate() {
                for (j, &(b, rb)) in places.iter().enumerate() {
                    if i == j {
                        continue;
                    }
                    let apart = ra.right() <= rb.x
                        || rb.right() <= ra.x
                        || ra.bottom() <= rb.y
                        || rb.bottom() <= ra.y;
                    assert!(apart, "{a:?} overlaps {b:?}");
                }
            }
            // The wheel's area is clear of every button but those on the left, which keep
            // to its top corner and take a finger first.
            for (n, &(b, r)) in places.iter().enumerate() {
                let left = b == Button::Pause || n >= BUTTONS;
                assert!(
                    layout.wheel.right() <= r.x
                        || r.right() <= layout.wheel.x
                        || r.bottom() <= layout.wheel.y
                        || layout.wheel.bottom() <= r.y
                        || left,
                    "{b:?} in the wheel's area"
                );
                if left {
                    assert!(
                        r.bottom() <= u.y + 0.7 * u.h,
                        "{b:?} low in the wheel: {r:?}"
                    );
                    assert_eq!(layout.button_at(centre(r).0, centre(r).1), Some(b));
                }
            }
        }
    }

    #[test]
    fn a_kept_touch_profile_gets_the_lights_button_and_keeps_a_choice() {
        use crate::input::device_id::DeviceId;
        use crate::input::profile::{Digital, Kind, Profile};
        let id = |name: &str| DeviceId {
            name: name.into(),
            serial: None,
            nth: 1,
        };
        let mut old = Profile::new(id(PAD_NAME), Kind::Pad);
        old.bindings.lights = None;
        give_lights(&mut old);
        assert_eq!(
            old.bindings.lights,
            Some(Digital::Button {
                button: LIGHTS_BUTTON
            })
        );
        let chosen = Some(Digital::Button { button: 9 });
        old.bindings.lights = chosen;
        give_lights(&mut old);
        assert_eq!(old.bindings.lights, chosen);
        let mut other = Profile::new(id("Xbox Controller"), Kind::Pad);
        other.bindings.lights = None;
        give_lights(&mut other);
        assert_eq!(other.bindings.lights, None);
    }

    #[test]
    fn a_finger_in_the_wheel_area_steers_both_ways_and_comes_back() {
        let mut c = Controls::new(&phone());
        let (x, y) = (400.0, 700.0);
        c.finger(1, Phase::Down, x, y, 0);
        assert_eq!(c.output(0).steer, 0.0);
        c.finger(1, Phase::Move, x + 100.0, y, 10);
        assert!(c.output(10).steer > 0.2);
        c.finger(1, Phase::Move, x + 2000.0, y, 20);
        assert_eq!(c.output(20).steer, 1.0);
        // The middle followed: coming back by the reach is the middle again.
        c.finger(1, Phase::Move, x + 2000.0 - c.layout().reach, y, 30);
        assert!(c.output(30).steer.abs() < 0.01);
        c.finger(1, Phase::Move, x + 1500.0, y, 40);
        assert_eq!(c.output(40).steer, -1.0);
        c.finger(1, Phase::Up, 0.0, 0.0, 50);
        assert_eq!(c.output(50), Output::default());
    }

    #[test]
    fn pedals_follow_the_fingers_and_can_be_held_together() {
        let mut c = Controls::new(&phone());
        let throttle = centre(c.layout().rect(Button::Throttle));
        let brake = centre(c.layout().rect(Button::Brake));
        c.finger(1, Phase::Down, throttle.0, throttle.1, 0);
        c.finger(2, Phase::Down, 400.0, 700.0, 0);
        assert!(c.output(1).is_held(Button::Throttle));
        assert!(!c.output(1).is_held(Button::Brake));
        // Sliding onto the brake: the throttle is let go.
        c.finger(1, Phase::Move, brake.0, brake.1, 5);
        assert!(!c.output(6).is_held(Button::Throttle));
        assert!(c.output(6).is_held(Button::Brake));
        c.finger(1, Phase::Up, 0.0, 0.0, 9);
        assert!(!c.output(10).is_held(Button::Brake));
    }

    #[test]
    fn a_quick_tap_on_the_gear_button_is_still_seen() {
        let mut c = Controls::new(&phone());
        let up = centre(c.layout().rect(Button::GearUp));
        c.finger(1, Phase::Down, up.0, up.1, 1000);
        c.finger(1, Phase::Up, up.0, up.1, 1010);
        assert!(c.output(1050).is_held(Button::GearUp));
        assert!(!c.output(1000 + LATCH_MS).is_held(Button::GearUp));
        // A pedal tap is not extended.
        let throttle = centre(c.layout().rect(Button::Throttle));
        c.finger(2, Phase::Down, throttle.0, throttle.1, 2000);
        c.finger(2, Phase::Up, throttle.0, throttle.1, 2010);
        assert!(!c.output(2020).is_held(Button::Throttle));
    }

    #[test]
    fn the_left_gears_change_gear_and_do_not_steer() {
        let mut c = Controls::new(&phone());
        let [up, down] = c.layout().left_gears.map(centre);
        c.finger(1, Phase::Down, down.0, down.1, 1000);
        let out = c.output(1010);
        assert!(out.is_held(Button::GearDown) && !out.is_held(Button::GearUp));
        assert_eq!(out.steer, 0.0);
        c.finger(1, Phase::Up, down.0, down.1, 1020);
        c.finger(2, Phase::Down, up.0, up.1, 2000);
        assert!(c.output(2010).is_held(Button::GearUp));
    }

    #[test]
    fn a_resize_lets_the_fingers_go() {
        let mut c = Controls::new(&phone());
        c.finger(1, Phase::Down, 400.0, 700.0, 0);
        c.resize(&phone());
        assert!(c.wheel_position().is_some());
        c.resize(&Screen::new(1280.0, 720.0));
        assert!(c.wheel_position().is_none());
    }

    #[test]
    fn a_pedal_stays_held_while_the_finger_drifts_off_it() {
        let screen = phone();
        let layout = Layout::new(&screen);
        let mut controls = Controls::new(&screen);
        let go = centre(layout.rect(Button::Throttle));
        controls.finger(1, Phase::Down, go.0, go.1, 0);
        assert!(controls.output(10).is_held(Button::Throttle));
        // Off the pedal onto nothing: still held.
        let off = (
            layout.rect(Button::Throttle).x - 1.0,
            layout.rect(Button::Throttle).y - 5.0,
        );
        controls.finger(1, Phase::Move, off.0, off.1, 20);
        assert!(controls.output(30).is_held(Button::Throttle));
        // Onto another button: that one.
        let brake = centre(layout.rect(Button::Brake));
        controls.finger(1, Phase::Move, brake.0, brake.1, 40);
        let out = controls.output(50);
        assert!(out.is_held(Button::Brake) && !out.is_held(Button::Throttle));
        controls.finger(1, Phase::Up, brake.0, brake.1, 60);
        assert!(!controls.output(70).is_held(Button::Brake));
    }

    #[test]
    fn the_answer_buttons_answer_on_a_tap_and_stay_apart() {
        let screen = phone();
        let [(_, yes), (_, no)] = answer_rects(&screen, Asked::YesNo)[..] else {
            panic!("two buttons");
        };
        let u = screen.usable();
        for r in [yes, no] {
            assert!(r.x >= u.x && r.right() <= u.right() && r.y >= u.y && r.bottom() <= u.bottom());
            assert!(r.h >= 0.25 * u.h);
        }
        assert!(yes.right() < no.x);
        let (yc, nc) = (centre(yes), centre(no));
        let mut taps = AnswerTaps::new();
        let asked = Asked::YesNo;
        assert_eq!(
            taps.finger(&screen, asked, 1, Phase::Down, yc.0, yc.1),
            None
        );
        assert_eq!(taps.pressed(), Some(Answer::Yes));
        assert_eq!(
            taps.finger(&screen, asked, 1, Phase::Up, yc.0, yc.1),
            Some(Answer::Yes)
        );
        taps.finger(&screen, asked, 2, Phase::Down, nc.0, nc.1);
        assert_eq!(
            taps.finger(&screen, asked, 2, Phase::Up, nc.0, nc.1),
            Some(Answer::No)
        );
        // down on one, up on the other or outside: nothing
        taps.finger(&screen, asked, 3, Phase::Down, yc.0, yc.1);
        assert_eq!(taps.finger(&screen, asked, 3, Phase::Up, nc.0, nc.1), None);
        taps.finger(&screen, asked, 4, Phase::Down, 1.0, 1.0);
        assert_eq!(taps.finger(&screen, asked, 4, Phase::Up, yc.0, yc.1), None);
        // the plain pause has the one button, the yes
        let one = centre(answer_rects(&screen, Asked::Resume)[0].1);
        taps.finger(&screen, Asked::Resume, 5, Phase::Down, one.0, one.1);
        assert_eq!(
            taps.finger(&screen, Asked::Resume, 5, Phase::Up, one.0, one.1),
            Some(Answer::Yes)
        );
    }

    #[test]
    fn player_one_follows_what_was_used() {
        use Source::*;
        // The keyboard becomes the touch pad until a controller is used.
        assert_eq!(player_one_controller(Auto, 0, Some(3), Some(1)), 3);
        assert_eq!(player_one_controller(Auto, 2, Some(3), Some(1)), 2);
        // A controller used: it replaces the touch pad and the keyboard, not another chosen.
        assert_eq!(player_one_controller(Pad, 3, Some(3), Some(1)), 1);
        assert_eq!(player_one_controller(Pad, 0, Some(3), Some(1)), 1);
        assert_eq!(player_one_controller(Pad, 2, Some(3), Some(1)), 2);
        // It went away: the touch pad again.
        assert_eq!(player_one_controller(Pad, 1, Some(3), None), 1);
        assert_eq!(player_one_controller(Pad, 0, Some(3), None), 3);
        // A finger: the touch pad whatever the player had.
        assert_eq!(player_one_controller(Touch, 1, Some(3), Some(1)), 3);
        assert_eq!(player_one_controller(Touch, 1, None, Some(1)), 1);
    }
}
