//! Leaving a screen, and the yes/no question that may come before (converted 2026-10-21
//! from `menu::go_back` 0x471C82, `menu::ask_to_leave` 0x47E8BB, `menu::leave_with_carry`
//! 0x471CCE, `menu::handle_yes_no` 0x470D2E and `menu::confirm_leaving` 0x46FA90).

/// The screen loops' leaving switches: four dwords in a row from `dword_75555D`, and
/// `dword_755A0C`. What each means is a reading.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Leaving {
    /// `dword_75555D`: the event left too (0 when only the screen is).
    pub event_left: u32,
    /// `dword_755561`: the screen loop leaves.
    pub leave_screen: u32,
    /// `dword_755565`: set with the one before by `leave_with_carry`.
    pub left_with_carry: u32,
    /// `dword_755569`: the screen loop asks whether to leave.
    pub ask_leave: u32,
    /// `dword_755A0C` (`DrawWork::scratch_0c`, a scratch variable the screens share): -1 on going back; the screens after it use it for their own values.
    pub scratch_0c: u32,
}

impl Leaving {
    /// `menu::go_back`: this screen left, not the event.
    pub fn go_back(&mut self) {
        self.event_left = 0;
        self.leave_screen = 1;
        self.scratch_0c = u32::MAX;
    }

    /// `menu::ask_to_leave`.
    pub fn ask_to_leave(&mut self) {
        self.ask_leave = 1;
    }

    /// `menu::leave_with_carry` (its carry set is the caller's).
    pub fn leave_with_carry(&mut self) {
        self.leave_screen = 1;
        self.left_with_carry = 1;
    }
}

/// A yes/no question's state: 1 on a screen that asks (`dword_7554CC`), the answer
/// (`g_yesNoAnswer`, 1 yes, 0 no) and whether it was answered (`dword_758806`, which the
/// screen loops also read as "left").
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Question {
    pub asking: u32,
    pub answer: u32,
    pub answered: u32,
}

impl Question {
    /// Whether a key may answer (`menu::handle_yes_no` looks at the key only then).
    pub fn asks(&self) -> bool {
        self.asking == 1
    }

    /// `menu::handle_yes_no`: a key taken for yes (`Some(true)`) or no answers, and the
    /// screen goes back; true when it did.
    pub fn answer_key(&mut self, leaving: &mut Leaving, said: Option<bool>) -> bool {
        let Some(yes) = said else {
            return false;
        };
        self.answer = yes as u32;
        self.answered = 1;
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
        if self.answered == 1 {
            leaving.event_left = 0;
            leaving.leave_screen = 0;
            self.answered = 0;
            self.answer = 0;
        }
        if self.answer == 1 {
            leaving.event_left = 1;
            leaving.leave_screen = 1;
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
            event_left: 5,
            ..Leaving::default()
        };
        l.go_back();
        assert_eq!(
            (l.event_left, l.leave_screen, l.scratch_0c),
            (0, 1, u32::MAX)
        );
        l.leave_with_carry();
        l.ask_to_leave();
        assert_eq!((l.left_with_carry, l.ask_leave), (1, 1));
        let mut q = Question {
            asking: 1,
            ..Question::default()
        };
        let mut l = Leaving::default();
        assert!(!q.answer_key(&mut l, None) && l == Leaving::default());
        assert!(q.answer_key(&mut l, Some(true)));
        assert_eq!((q.answer, q.answered, l.leave_screen), (1, 1, 1));
        // Answered on the question's screen: cleared, not leaving.
        assert_eq!(q.confirm(&mut l), 0);
        assert_eq!((q.answered, l.event_left, l.leave_screen), (0, 0, 0));
        q.answer = 1;
        assert_eq!(q.confirm(&mut l), 1);
        assert_eq!((l.event_left, l.leave_screen), (1, 1));
    }
}
