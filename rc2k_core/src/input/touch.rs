//! The touch screen's controls in a race, shared by the phone and tablet builds (Android now, iOS
//! to come): no platform, only fingers in and what the game reads out. (The menus' fingers:
//! `menu_touch`.)
//!
//! A race has a layout for a phone held sideways: a steering area on the left (a finger put down
//! anywhere in it is the wheel's middle; moving it left and right steers, and past the reach the
//! middle follows the finger, so letting it come back is easy), pedals and the handbrake on the
//! right, small buttons for the gears, the camera and the pause at the top. The game reads them as
//! a gamepad (`Output`: steering -1 .. 1 and the buttons held).
//!
//! Positions are in the window's pixels, times in milliseconds.

use super::menu_touch::{Phase, Rect, Screen};

/// The buttons of the race's layout.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum Button {
    Throttle,
    Brake,
    Handbrake,
    GearUp,
    GearDown,
    Camera,
    Pause,
}

/// How many buttons there are.
pub const BUTTONS: usize = 7;

impl Button {
    pub const ALL: [Button; BUTTONS] = [
        Button::Throttle,
        Button::Brake,
        Button::Handbrake,
        Button::GearUp,
        Button::GearDown,
        Button::Camera,
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
            Button::Pause => "II",
        }
    }
}

/// How long a tap on a latched button is held down for the game to see it.
pub const LATCH_MS: u64 = 120;

/// Where everything is for one screen.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Layout {
    /// Where a finger put down steers.
    pub wheel: Rect,
    /// How far the finger goes for full lock.
    pub reach: f32,
    pub buttons: [Rect; BUTTONS],
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
        let pause = Rect::new(u.x + 0.1 * s, u.y + 0.1 * s, 0.7 * s, 0.7 * s);
        let mut buttons = [Rect::default(); BUTTONS];
        for (button, rect) in [
            (Button::Throttle, throttle),
            (Button::Brake, brake),
            (Button::Handbrake, handbrake),
            (Button::GearUp, gear_up),
            (Button::GearDown, gear_down),
            (Button::Camera, camera),
            (Button::Pause, pause),
        ] {
            buttons[button.index()] = rect;
        }
        Layout {
            wheel: Rect::new(u.x, u.y + 0.3 * u.h, 0.42 * u.w, 0.7 * u.h),
            reach: (0.085 * u.w).max(1.0),
            buttons,
        }
    }

    pub fn rect(&self, button: Button) -> Rect {
        self.buttons[button.index()]
    }

    /// The button under a point.
    pub fn button_at(&self, x: f32, y: f32) -> Option<Button> {
        Button::ALL
            .into_iter()
            .find(|b| self.rect(*b).contains(x, y))
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
                    Role::Button(_) | Role::Nothing => {
                        // A finger that slides from one button to the next presses the next.
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
            for b in Button::ALL {
                let r = layout.rect(b);
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
            for a in Button::ALL {
                for b in Button::ALL {
                    if a == b {
                        continue;
                    }
                    let (ra, rb) = (layout.rect(a), layout.rect(b));
                    let apart = ra.right() <= rb.x
                        || rb.right() <= ra.x
                        || ra.bottom() <= rb.y
                        || rb.bottom() <= ra.y;
                    assert!(apart, "{a:?} overlaps {b:?}");
                }
            }
            // The wheel's area is clear of every button.
            for b in Button::ALL {
                let r = layout.rect(b);
                assert!(
                    layout.wheel.right() <= r.x
                        || r.right() <= layout.wheel.x
                        || r.bottom() <= layout.wheel.y
                        || layout.wheel.bottom() <= r.y
                        || b == Button::Pause,
                    "{b:?} in the wheel's area"
                );
            }
        }
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
    fn a_resize_lets_the_fingers_go() {
        let mut c = Controls::new(&phone());
        c.finger(1, Phase::Down, 400.0, 700.0, 0);
        c.resize(&phone());
        assert!(c.wheel_position().is_some());
        c.resize(&Screen::new(1280.0, 720.0));
        assert!(c.wheel_position().is_none());
    }
}
