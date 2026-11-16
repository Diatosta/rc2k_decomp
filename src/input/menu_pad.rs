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
}

// ---- answers to the game's yes / no questions ----

/// Not an answer yet.
pub const NO_ANSWER: u32 = 0;

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
}

/// The time a held direction waits before it repeats, and between repeats (ms).
const REPEAT_AFTER: u64 = 400;
const REPEAT_EVERY: u64 = 120;

#[derive(Default)]
pub struct MenuPad {
    /// The direction held and when it next repeats.
    held: Option<(PadButton, u64)>,
    /// When START's Escape is let go.
    leave_until: Option<u64>,
}

impl MenuPad {
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
            PadButton::South => MenuAction::Enter,
            PadButton::East => MenuAction::Back,
            PadButton::West => MenuAction::Third,
            PadButton::Start if in_race => {
                self.leave_until = Some(now + LEAVE_HOLD);
                MenuAction::LeaveRace
            }
            PadButton::Start => MenuAction::Enter,
            direction => {
                self.held = Some((direction, now + REPEAT_AFTER));
                direction_of(direction)?
            }
        })
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
        direction_of(button)
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
