//! Leaving a screen, and the yes/no question that may come before
//! (from `menu::go_back` 0x471C82, `menu::ask_to_leave` 0x47E8BB, `menu::leave_with_carry`
//! 0x471CCE, `menu::handle_yes_no` 0x470D2E and `menu::confirm_leaving` 0x46FA90).

/// The menus' leaving switches the event flows read and write; each is 0 or 1 in the game.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Switch {
    /// The event is left too, not only the screen.
    EventLeft,
    /// The screen loop leaves at once (cleared by the key poll and set-up).
    LeaveScreen,
    /// Set by `leave_with_carry` with `LeaveScreen`; the class pages set it
    /// while a page with the player is shown.
    LeftWithCarry,
    /// An action asks to leave the screen (and, on a screen that asks, the
    /// question is put).
    AskLeave,
    /// An event is over.
    EventOver,
    /// The screen was left with a key (Escape or a yes / no answer); the
    /// leave question is put when it is set.
    LeftByKey,
    /// Escape was pressed (`handle_keys`).
    EscapePressed,
    /// The rally's results reached.
    RallyDone,
    /// One was entered in the best times.
    Entered,
    /// `g_quitting`: the window was closed (read only).
    Quitting,
}

/// The screen loops' leaving switches: four flags in a row from `dword_75555D`, and
/// `dword_755A0C`. What each means is a reading.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Leaving {
    /// The event left too (0 when only the screen is).
    pub event_left: bool,
    /// The screen loop leaves.
    pub leave_screen: bool,
    /// Set with the one before by `leave_with_carry`.
    pub left_with_carry: bool,
    /// The screen loop asks whether to leave.
    pub ask_leave: bool,
    /// `dword_755A0C` (`DrawWork::scratch_0c`, a scratch variable the screens share): -1 on going
    /// back; the screens after it use it for their own values.
    pub scratch_0c: u32,
}

impl Leaving {
    /// `menu::go_back`: this screen left, not the event.
    pub fn go_back(&mut self) {
        self.event_left = false;
        self.leave_screen = true;
        self.scratch_0c = u32::MAX;
    }

    /// `menu::ask_to_leave`.
    pub fn ask_to_leave(&mut self) {
        self.ask_leave = true;
    }

    /// `menu::leave_with_carry` (its carry set is the caller's).
    pub fn leave_with_carry(&mut self) {
        self.leave_screen = true;
        self.left_with_carry = true;
    }
}

/// A yes/no question's state: 1 on a screen that asks, the answer
/// (`g_yesNoAnswer`, 1 yes, 0 no) and whether it was answered (`dword_758806`, which the
/// screen loops also read as "left").
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Question {
    pub asking: bool,
    pub answer: u32,
    pub answered: bool,
}

impl Question {
    /// Whether a key may answer (`menu::handle_yes_no` looks at the key only then).
    pub fn asks(&self) -> bool {
        self.asking
    }

    /// `menu::handle_yes_no`: a key taken for yes (`Some(true)`) or no answers, and the
    /// screen goes back; true when it did.
    pub fn answer_key(&mut self, leaving: &mut Leaving, said: Option<bool>) -> bool {
        let Some(yes) = said else {
            return false;
        };
        self.answer = yes as u32;
        self.answered = true;
        leaving.go_back();
        true
    }

    /// `menu::confirm_leaving` before the question's screen: no answer yet.
    pub fn begin(&mut self) {
        self.answer = 0;
    }

    /// `menu::confirm_leaving` after it: an answered question's switches and answer
    /// cleared; then yes leaves the screen and the event. The answer left.
    pub fn confirm(&mut self, leaving: &mut Leaving) -> u32 {
        if self.answered {
            leaving.event_left = false;
            leaving.leave_screen = false;
            self.answered = false;
            self.answer = 0;
        }
        if self.answer == 1 {
            leaving.event_left = true;
            leaving.leave_screen = true;
        }
        self.answer
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn leaving() {
        let mut l = Leaving {
            event_left: true,
            ..Leaving::default()
        };
        l.go_back();
        assert_eq!(
            (l.event_left, l.leave_screen, l.scratch_0c),
            (false, true, u32::MAX)
        );
        l.leave_with_carry();
        l.ask_to_leave();
        assert_eq!((l.left_with_carry, l.ask_leave), (true, true));
        let mut q = Question {
            asking: true,
            ..Question::default()
        };
        let mut l = Leaving::default();
        assert!(!q.answer_key(&mut l, None) && l == Leaving::default());
        assert!(q.answer_key(&mut l, Some(true)));
        assert_eq!((q.answer, q.answered, l.leave_screen), (1, true, true));
        // Answered on the question's screen: cleared, not leaving.
        assert_eq!(q.confirm(&mut l), 0);
        assert_eq!(
            (q.answered, l.event_left, l.leave_screen),
            (false, false, false)
        );
        q.answer = 1;
        assert_eq!(q.confirm(&mut l), 1);
        assert_eq!((l.event_left, l.leave_screen), (true, true));
    }
}
