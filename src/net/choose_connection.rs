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
use crate::net::lobby::{list_key, other_key, run_frame};
use crate::net::owner::net;
use crate::system::owner::system;
use rc2k_core::net::screens::{self, ListKey};

/// A list's up (0x66) or down (0x6B) on a list of `count` items: the highlight and the top
/// moved; false for another key.
pub fn list_move(key: u8, count: u32) -> bool {
    // Converted: `rc2k_core::net::screens` (`ListKey`, `up`, `down`).
    let (choice, top) = match ListKey::of(key) {
        ListKey::Up => screens::up(
            crate::install::owner::install().pages.choice.get(),
            crate::install::owner::install().pages.choice_before.get(),
        ),
        ListKey::Down => screens::down(
            crate::install::owner::install().pages.choice.get(),
            crate::install::owner::install().pages.choice_before.get(),
            count,
        ),
        _ => return false,
    };
    crate::install::owner::install().pages.choice.set(choice);
    crate::install::owner::install()
        .pages
        .choice_before
        .set(top);
    true
}

/// The Rust version.
pub fn choose_connection() -> Result<u32, Refused> {
    let system = system();
    let now = system.session.clocks.key_ticks.get();
    crate::install::owner::install().pages.move_clock.set(now);
    crate::install::owner::install().pages.key_clock.set(now);
    crate::install::owner::install().pages.choice_before.set(0);
    crate::install::owner::install().pages.choice.set(0);
    loop {
        run_frame(crate::net::connections_frame::connections_frame)?;
        crate::install::owner::install()
            .pages
            .frame_clock
            .set(system.session.clocks.key_ticks.get());
        let key = list_key()?;
        if list_move(key, net().vars.connections.get()) {
            crate::install::owner::install()
                .pages
                .move_clock
                .set(crate::install::owner::install().pages.frame_clock.get());
        } else if ListKey::of(key) == ListKey::Enter {
            return Ok(crate::install::owner::install().pages.choice.get());
        } else if ListKey::of(key) == ListKey::Escape {
            break;
        } else {
            other_key(key)?;
        }
        if system.session.flags.quitting.get() == 1 {
            break;
        }
    }
    crate::install::owner::install().pages.choice.set(u32::MAX);
    Ok(crate::install::owner::install().pages.choice.get())
}
