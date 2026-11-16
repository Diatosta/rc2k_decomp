//! Typed text: the characters the text fields take.
//!
//! While a field is being edited SDL's text input is on, and the characters it reports (UTF-8,
//! turned into Windows-1252; one the code page lacks is dropped) wait in a queue that the field
//! takes from. Each field asks for its characters every time it runs (`take`); text input stays
//! on while some field asked in the last moments, and ends with the editing. With text input on,
//! the printable keys are not read as scan codes (no letter twice); Escape, Enter, Backspace and
//! the others still come as keys.
//!
//! A field entered from a controller shows an on-screen keyboard (`rc2k_core::input::osk`),
//! unless the system has a screen keyboard of its own: the keyboard types into the same queue
//! and gives Backspace and Enter as keys. Typing on a physical keyboard closes it.

use rc2k_core::input::osk::Osk;
use std::collections::VecDeque;
use std::sync::Mutex;
use std::time::{Duration, Instant};

/// How long after a field's last ask text input stays on.
const FIELD_GRACE: Duration = Duration::from_millis(500);

/// The most characters waiting.
const QUEUE_MOST: usize = 64;

#[derive(Default)]
struct State {
    asked_at: Option<Instant>,
    active: bool,
    queue: VecDeque<u8>,
    /// The last menu confirmation came from a controller.
    confirmed_by_pad: bool,
    osk: Option<Osk>,
}

fn state() -> std::sync::MutexGuard<'static, State> {
    static STATE: Mutex<State> = Mutex::new(State {
        asked_at: None,
        active: false,
        queue: VecDeque::new(),
        confirmed_by_pad: false,
        osk: None,
    });
    STATE
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

/// What a field gets when it asks.
pub struct Typed {
    /// Text input is on: the printable keys come as `chars`, not as scan codes.
    pub text_mode: bool,
    /// The characters typed since the last ask, in order (Windows-1252).
    pub chars: Vec<u8>,
}

/// A field's ask, once each time it runs.
pub fn take() -> Typed {
    let mut st = state();
    st.asked_at = Some(Instant::now());
    Typed {
        text_mode: st.active,
        chars: st.queue.drain(..).collect(),
    }
}

fn push(st: &mut State, text: &str) {
    for c in text.chars() {
        if let Some(byte) = rc2k_core::files::ansi::typed_byte(c)
            && st.queue.len() < QUEUE_MOST
        {
            st.queue.push_back(byte);
        }
    }
}

/// Text SDL reported from the keyboard (or a screen keyboard): queued while a field is being
/// edited; our own keyboard closes.
pub fn typed_by_keyboard(text: &str) {
    let mut st = state();
    if st.active {
        st.osk = None;
        push(&mut st, text);
    }
}

/// Who last confirmed in the menus: a controller (true) or the keyboard. A field entered next
/// shows the on-screen keyboard when it was a controller.
pub fn note_confirm(from_pad: bool) {
    state().confirmed_by_pad = from_pad;
}

/// Text input started or ended as the fields ask. Returns what the window's text input should
/// now be, when it changed.
pub fn sync(screen_keyboard: impl FnOnce() -> bool) -> Option<bool> {
    let mut st = state();
    let wanted = st.asked_at.is_some_and(|at| at.elapsed() < FIELD_GRACE);
    if wanted == st.active {
        return None;
    }
    st.active = wanted;
    st.queue.clear();
    st.osk = (wanted && st.confirmed_by_pad && !screen_keyboard()).then(Osk::default);
    Some(wanted)
}

/// Text input is on (a field is being edited and asked recently).
pub fn text_mode() -> bool {
    state().active
}

/// The on-screen keyboard, if shown.
pub fn osk() -> Option<Osk> {
    state().osk
}

/// What the on-screen keyboard asks of the game after a move or press.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Asks {
    Nothing,
    /// Backspace as the game's key code.
    Backspace,
    /// Enter: the field is done.
    Done,
    /// Escape: the field's cancel.
    Cancel,
}

/// The cursor moved, if the keyboard is shown; whether it was.
pub fn osk_step(dx: i32, dy: i32) -> bool {
    let mut st = state();
    match st.osk.as_mut() {
        Some(osk) => {
            osk.step(dx, dy);
            true
        }
        None => false,
    }
}

/// The key under the cursor pressed, if the keyboard is shown: a character goes in the queue.
pub fn osk_press() -> Option<Asks> {
    let mut st = state();
    let press = st.osk.as_mut()?.press();
    use rc2k_core::input::osk::Press;
    Some(match press {
        Press::Type(c) => {
            if st.queue.len() < QUEUE_MOST {
                st.queue.push_back(c);
            }
            Asks::Nothing
        }
        Press::Backspace => Asks::Backspace,
        Press::Done => Asks::Done,
        Press::Nothing => Asks::Nothing,
    })
}

/// The keyboard closed (and the field's done or cancelled by the caller).
pub fn osk_close() {
    state().osk = None;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn queue_and_keyboard() {
        // One test: the state is the process's.
        let t = take();
        assert!(!t.text_mode && t.chars.is_empty());
        note_confirm(true);
        assert_eq!(sync(|| false), Some(true));
        assert!(osk().is_some());
        assert!(osk_step(1, 0));
        assert_eq!(osk_press(), Some(Asks::Nothing));
        typed_by_keyboard("a\u{3B1}\u{e7}");
        assert!(osk().is_none());
        let t = take();
        assert!(t.text_mode);
        assert_eq!(t.chars, [b'2', b'a', 0xE7]);
        assert!(take().chars.is_empty());
        assert_eq!(osk_press(), None);
    }
}
