//! A gamepad in the menus (the game's menus otherwise take only the keyboard): any pad SDL knows,
//! on every system. The menus read one pending key (`input::take_key`), so a button gives them the
//! key of what it means; the race's own controls are the game's devices (`platform::input`), and the
//! held-key table is left alone, so driving with the pad steers nothing through the keyboard's
//! controls.

use crate::input::owner::Input;
use std::sync::atomic::Ordering::Relaxed;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum MenuAction {
    Up,
    Down,
    Left,
    Right,
    Enter,
    Back,
    /// START in a race: Escape held long enough for a frame to see it (the race reads the
    /// held keys, at its frame rate), then let go.
    LeaveRace,
    LeaveRaceEnd,
    /// The third answer of a question.
    Third,
    /// A shoulder button: Tab, which the network lobby takes to show or hide the gaps.
    Tab,
}

// ---- answers to the game's yes / no questions ----

/// Not an answer yet.
pub(super) const NO_ANSWER: u32 = 0;

/// Notes the answer the last button gave (`input::yes_no_key`'s values), for a question to
/// take. The race's question (`drive::quit_question`) reads a typed character, the first of the
/// language's word, and shows nothing to choose from: the pad's buttons answer it directly, in
/// any language. The menus' question has items to choose from: there only Back answers
/// (`handle_yes_no`).
pub fn note_answer(input: &Input, answer: u32) {
    input.menu_answer.store(answer, Relaxed);
}

/// The noted answer, once.
pub fn take_answer(input: &Input) -> Option<u32> {
    match input.menu_answer.swap(NO_ANSWER, Relaxed) {
        NO_ANSWER => None,
        answer => Some(answer),
    }
}

/// How long START's Escape stays down in a race (ms): longer than a slow frame.
const LEAVE_HOLD: u64 = 400;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PadButton {
    DpadUp,
    DpadDown,
    DpadLeft,
    DpadRight,
    South,
    East,
    West,
    Start,
    LeftShoulder,
    RightShoulder,
}

/// The time a held direction waits before it repeats, and between repeats (ms).
const REPEAT_AFTER: u64 = 400;
const REPEAT_EVERY: u64 = 120;

/// How far the left stick must lean (of 32767) to count as a direction, and how far back it
/// must come to let go of it: the gap keeps a stick held near the line from chattering.
const STICK_PRESS: i32 = 16000;
const STICK_RELEASE: i32 = 10000;

/// One of the left stick's two axes.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum StickAxis {
    X,
    Y,
}

#[derive(Default)]
pub struct MenuPad {
    /// The button held (a direction, Enter or Tab) and when it next repeats.
    held: Option<(PadButton, u64)>,
    /// When START's Escape is let go.
    leave_until: Option<u64>,
    /// The direction each stick axis (x, y) is leaning past the press line, if any.
    stick: [Option<PadButton>; 2],
    /// A joystick's hat's bits as last seen (1 up, 2 right, 4 down, 8 left).
    hat: u8,
    /// Each hat's bits as last seen by the menu controls bound to hat directions.
    bound_hats: [u8; 4],
}

impl MenuPad {
    /// Hat `hat` moved to `bits`: the bits newly set and the bits newly let go (for the menu
    /// controls bound to the hat's directions).
    pub fn bound_hat_changes(&mut self, hat: u8, bits: u8) -> (u8, u8) {
        let Some(was) = self.bound_hats.get_mut(hat as usize) else {
            return (0, 0);
        };
        let old = std::mem::replace(was, bits);
        (bits & !old, old & !bits)
    }

    /// A button went down or up at `now` (ms); `in_race`: START is the race's pause (the
    /// keyboard's Escape), and in a menu it enters.
    pub fn button(
        &mut self,
        button: PadButton,
        down: bool,
        in_race: bool,
        now: u64,
    ) -> Option<MenuAction> {
        if !down {
            if self.held.is_some_and(|(b, _)| b == button) {
                self.held = None;
            }
            return None;
        }
        Some(match button {
            PadButton::South => {
                self.held = Some((button, now + REPEAT_AFTER));
                MenuAction::Enter
            }
            PadButton::East => MenuAction::Back,
            PadButton::West => MenuAction::Third,
            PadButton::LeftShoulder | PadButton::RightShoulder => {
                self.held = Some((button, now + REPEAT_AFTER));
                MenuAction::Tab
            }
            PadButton::Start if in_race => {
                self.leave_until = Some(now + LEAVE_HOLD);
                MenuAction::LeaveRace
            }
            PadButton::Start => {
                self.held = Some((button, now + REPEAT_AFTER));
                MenuAction::Enter
            }
            direction => {
                self.held = Some((direction, now + REPEAT_AFTER));
                direction_of(direction)?
            }
        })
    }

    /// The left stick's axis moved to `value` (an SDL axis, down and right positive) at
    /// `now`: leaning past the press line is that direction of the pad's cross pressed (with
    /// its repeat), coming back inside the release line lets go of it.
    pub fn stick(&mut self, axis: StickAxis, value: i16, now: u64) -> Option<MenuAction> {
        let (slot, negative, positive) = match axis {
            StickAxis::X => (0, PadButton::DpadLeft, PadButton::DpadRight),
            StickAxis::Y => (1, PadButton::DpadUp, PadButton::DpadDown),
        };
        let value = i32::from(value);
        let (press, release) = (STICK_PRESS, STICK_RELEASE);
        let leaning = if value <= -press {
            Some(negative)
        } else if value >= press {
            Some(positive)
        } else if value.unsigned_abs() > release.unsigned_abs() {
            // between the lines: as it was
            self.stick[slot]
        } else {
            None
        };
        if leaning == self.stick[slot] {
            return None;
        }
        if let Some(old) = self.stick[slot].take() {
            self.button(old, false, false, now);
        }
        self.stick[slot] = leaning;
        self.button(leaning?, true, false, now)
    }

    /// A joystick's hat moved to `bits` (1 up, 2 right, 4 down, 8 left) at `now`: each direction
    /// let go is let go, a direction newly pressed is that direction of the pad's cross pressed.
    pub fn hat(&mut self, bits: u8, now: u64) -> Option<MenuAction> {
        const DIRECTIONS: [(u8, PadButton); 4] = [
            (1, PadButton::DpadUp),
            (2, PadButton::DpadRight),
            (4, PadButton::DpadDown),
            (8, PadButton::DpadLeft),
        ];
        let was = std::mem::replace(&mut self.hat, bits);
        let mut action = None;
        for (bit, button) in DIRECTIONS {
            if was & bit != 0 && bits & bit == 0 {
                self.button(button, false, false, now);
            }
        }
        for (bit, button) in DIRECTIONS {
            if was & bit == 0 && bits & bit != 0 {
                action = action.or(self.button(button, true, false, now));
            }
        }
        action
    }

    /// The held direction's repeat, if it is due at `now`.
    pub fn tick(&mut self, now: u64) -> Option<MenuAction> {
        if self.leave_until.is_some_and(|until| now >= until) {
            self.leave_until = None;
            return Some(MenuAction::LeaveRaceEnd);
        }
        let (button, due) = self.held?;
        if now < due {
            return None;
        }
        self.held = Some((button, now + REPEAT_EVERY));
        repeat_of(button)
    }
}

/// What a held button repeats: the directions, Enter (the south button and Start in a menu,
/// as a held Enter key repeats) and Tab; Back and the third answer do not.
fn repeat_of(button: PadButton) -> Option<MenuAction> {
    match button {
        PadButton::South | PadButton::Start => Some(MenuAction::Enter),
        PadButton::LeftShoulder | PadButton::RightShoulder => Some(MenuAction::Tab),
        _ => direction_of(button),
    }
}

fn direction_of(button: PadButton) -> Option<MenuAction> {
    match button {
        PadButton::DpadUp => Some(MenuAction::Up),
        PadButton::DpadDown => Some(MenuAction::Down),
        PadButton::DpadLeft => Some(MenuAction::Left),
        PadButton::DpadRight => Some(MenuAction::Right),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn buttons_mean_what_they_should() {
        let mut p = MenuPad::default();
        assert_eq!(
            p.button(PadButton::South, true, false, 0),
            Some(MenuAction::Enter)
        );
        assert_eq!(
            p.button(PadButton::East, true, false, 0),
            Some(MenuAction::Back)
        );
        assert_eq!(
            p.button(PadButton::Start, true, false, 0),
            Some(MenuAction::Enter)
        );
        assert_eq!(p.button(PadButton::South, false, false, 0), None);
    }

    #[test]
    fn a_held_enter_repeats_as_the_key_does() {
        let mut p = MenuPad::default();
        assert_eq!(
            p.button(PadButton::South, true, false, 0),
            Some(MenuAction::Enter)
        );
        assert_eq!(p.tick(399), None);
        assert_eq!(p.tick(400), Some(MenuAction::Enter));
        assert_eq!(p.tick(520), Some(MenuAction::Enter));
        assert_eq!(p.button(PadButton::South, false, false, 530), None);
        assert_eq!(p.tick(5000), None);
        // Back does not repeat.
        p.button(PadButton::East, true, false, 6000);
        assert_eq!(p.tick(7000), None);
    }

    #[test]
    fn a_shoulder_button_is_tab() {
        let mut p = MenuPad::default();
        assert_eq!(
            p.button(PadButton::LeftShoulder, true, false, 0),
            Some(MenuAction::Tab)
        );
        assert_eq!(
            p.button(PadButton::RightShoulder, true, false, 0),
            Some(MenuAction::Tab)
        );
        assert_eq!(p.button(PadButton::RightShoulder, false, false, 0), None);
        assert_eq!(p.tick(5000), None);
    }

    #[test]
    fn start_in_a_race_holds_escape_for_a_while() {
        let mut p = MenuPad::default();
        assert_eq!(
            p.button(PadButton::Start, true, true, 100),
            Some(MenuAction::LeaveRace)
        );
        // Let go at once: Escape stays until a slow frame has seen it.
        assert_eq!(p.button(PadButton::Start, false, true, 110), None);
        assert_eq!(p.tick(499), None);
        assert_eq!(p.tick(500), Some(MenuAction::LeaveRaceEnd));
        assert_eq!(p.tick(501), None);
    }

    #[test]
    fn an_answer_is_taken_once() {
        let input = Input::new();
        note_answer(&input, 7);
        assert_eq!(take_answer(&input), Some(7));
        assert_eq!(take_answer(&input), None);
    }

    #[test]
    fn the_left_stick_is_the_cross() {
        let mut p = MenuPad::default();
        assert_eq!(p.stick(StickAxis::Y, -5000, 0), None);
        assert_eq!(p.stick(StickAxis::Y, -20000, 10), Some(MenuAction::Up));
        // held: no new press while it stays leaning, the repeat comes from `tick`
        assert_eq!(p.stick(StickAxis::Y, -30000, 20), None);
        assert_eq!(p.tick(410), Some(MenuAction::Up));
        // back between the lines: still held
        assert_eq!(p.stick(StickAxis::Y, -12000, 420), None);
        assert_eq!(p.tick(530), Some(MenuAction::Up));
        // back to the middle: let go
        assert_eq!(p.stick(StickAxis::Y, -2000, 540), None);
        assert_eq!(p.tick(5000), None);
        // straight across to the other side
        assert_eq!(p.stick(StickAxis::X, 20000, 6000), Some(MenuAction::Right));
        assert_eq!(p.stick(StickAxis::X, -20000, 6010), Some(MenuAction::Left));
        assert_eq!(p.tick(6410), Some(MenuAction::Left));
        assert_eq!(p.stick(StickAxis::X, 0, 6420), None);
        assert_eq!(p.tick(9000), None);
    }

    #[test]
    fn a_bound_hat_reports_its_changes() {
        let mut p = MenuPad::default();
        assert_eq!(p.bound_hat_changes(1, 4), (4, 0));
        assert_eq!(p.bound_hat_changes(1, 4), (0, 0));
        assert_eq!(p.bound_hat_changes(1, 2), (2, 4));
        assert_eq!(p.bound_hat_changes(0, 1), (1, 0));
        assert_eq!(p.bound_hat_changes(9, 1), (0, 0));
    }

    #[test]
    fn a_hat_is_the_cross() {
        let mut p = MenuPad::default();
        assert_eq!(p.hat(4, 0), Some(MenuAction::Down));
        assert_eq!(p.tick(400), Some(MenuAction::Down));
        assert_eq!(p.hat(0, 410), None);
        assert_eq!(p.tick(5000), None);
        assert_eq!(p.hat(1, 6000), Some(MenuAction::Up));
    }

    #[test]
    fn a_held_direction_repeats_until_let_go() {
        let mut p = MenuPad::default();
        assert_eq!(
            p.button(PadButton::DpadUp, true, false, 1000),
            Some(MenuAction::Up)
        );
        assert_eq!(p.tick(1399), None);
        assert_eq!(p.tick(1400), Some(MenuAction::Up));
        assert_eq!(p.tick(1500), None);
        assert_eq!(p.tick(1520), Some(MenuAction::Up));
        // Another button going up changes nothing.
        p.button(PadButton::South, false, false, 1530);
        assert_eq!(p.tick(1640), Some(MenuAction::Up));
        p.button(PadButton::DpadUp, false, false, 1650);
        assert_eq!(p.tick(5000), None);
    }
}
