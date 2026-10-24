//! `net::choose_connection` (0x4174C5): the list screen of the DirectPlay connections
//! (`net::connections_frame` through `screens::run_frame`): the clocks from `dword_518998`,
//! the top and the highlight 0; each frame a key (`menu::list_take_key`): up (0x66) and down
//! (0x6B) move the highlight (within 0 .. the count - 1, signed) and the top with it
//! (sixteen rows), Enter (0x1C) chooses, Escape (1) or the game closing gives -1; 'R' takes a
//! screenshot (with screenshots on), any other key may compile the lexicon again.
//!
//! In:  nothing (the registers it is given reach only its frame, `net::connections_frame`,
//!      which reads none, and `menu::list_take_key`, which reads none)
//! Out: the connection chosen (`g_listChoice`; negative: none, the original's carry); the
//!      original keeps the rest (`pusha` .. `popa`)
//! Callers: `net::settings_link`, `net::start_link`.

use crate::errors::Refused;
use crate::net::lobby::{
    CHOICE, CHOICE_BEFORE, CLOCK, CONNECTIONS, FRAME_CLOCK, KEY_CLOCK, MOVE_CLOCK, QUITTING, g,
    list_key, other_key, p, run_frame,
};
use rc2k_core::net::screens::{self, ListKey};

/// A list's up (0x66) or down (0x6B) on a list of `count` items: the highlight and the top
/// moved; false for another key.
pub fn list_move(key: u8, count: u32) -> bool {
    // Converted: `rc2k_core::net::screens` (`ListKey`, `up`, `down`).
    let (choice, top) = match ListKey::of(key) {
        ListKey::Up => screens::up(g(CHOICE), g(CHOICE_BEFORE)),
        ListKey::Down => screens::down(g(CHOICE), g(CHOICE_BEFORE), count),
        _ => return false,
    };
    p(CHOICE, choice);
    p(CHOICE_BEFORE, top);
    true
}

/// The Rust version.
pub fn choose_connection() -> Result<u32, Refused> {
    let now = g(CLOCK);
    p(MOVE_CLOCK, now);
    p(KEY_CLOCK, now);
    p(CHOICE_BEFORE, 0);
    p(CHOICE, 0);
    loop {
        run_frame(crate::net::connections_frame::connections_frame)?;
        p(FRAME_CLOCK, g(CLOCK));
        let key = list_key()?;
        if list_move(key, g(CONNECTIONS)) {
            p(MOVE_CLOCK, g(FRAME_CLOCK));
        } else if ListKey::of(key) == ListKey::Enter {
            return Ok(g(CHOICE));
        } else if ListKey::of(key) == ListKey::Escape {
            break;
        } else {
            other_key(key)?;
        }
        if g(QUITTING) == 1 {
            break;
        }
    }
    p(CHOICE, u32::MAX);
    Ok(g(CHOICE))
}
