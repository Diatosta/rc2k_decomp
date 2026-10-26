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
use crate::install::vars::INSTALLER_PAGES;
use crate::net::lobby::{list_key, other_key, run_frame};
use crate::net::store::NET;
use crate::session::CLOCKS;
use crate::session::FLAGS;
use rc2k_core::net::screens::{self, ListKey};

/// A list's up (0x66) or down (0x6B) on a list of `count` items: the highlight and the top
/// moved; false for another key.
pub fn list_move(key: u8, count: u32) -> bool {
    // Converted: `rc2k_core::net::screens` (`ListKey`, `up`, `down`).
    let (choice, top) = match ListKey::of(key) {
        ListKey::Up => screens::up(
            INSTALLER_PAGES.choice.get(),
            INSTALLER_PAGES.choice_before.get(),
        ),
        ListKey::Down => screens::down(
            INSTALLER_PAGES.choice.get(),
            INSTALLER_PAGES.choice_before.get(),
            count,
        ),
        _ => return false,
    };
    INSTALLER_PAGES.choice.set(choice);
    INSTALLER_PAGES.choice_before.set(top);
    true
}

/// The Rust version.
pub fn choose_connection() -> Result<u32, Refused> {
    let now = CLOCKS.key_ticks.get();
    INSTALLER_PAGES.move_clock.set(now);
    INSTALLER_PAGES.key_clock.set(now);
    INSTALLER_PAGES.choice_before.set(0);
    INSTALLER_PAGES.choice.set(0);
    loop {
        run_frame(crate::net::connections_frame::connections_frame)?;
        INSTALLER_PAGES.frame_clock.set(CLOCKS.key_ticks.get());
        let key = list_key()?;
        if list_move(key, NET.connections.get()) {
            INSTALLER_PAGES
                .move_clock
                .set(INSTALLER_PAGES.frame_clock.get());
        } else if ListKey::of(key) == ListKey::Enter {
            return Ok(INSTALLER_PAGES.choice.get());
        } else if ListKey::of(key) == ListKey::Escape {
            break;
        } else {
            other_key(key)?;
        }
        if FLAGS.quitting.get() == 1 {
            break;
        }
    }
    INSTALLER_PAGES.choice.set(u32::MAX);
    Ok(INSTALLER_PAGES.choice.get())
}
