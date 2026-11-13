//! `net::choose_connection` (0x4174C5): the list screen of the DirectPlay connections
//! (`net::connections_frame` through `screens::run_frame`): the clocks from `clocks.key_ticks`,
//! the top and the highlight 0; each frame a key (`menu::list_take_key`): up (0x66) and down
//! (0x6B) move the highlight (within 0 .. the count - 1, signed) and the top with it
//! (sixteen rows), Enter (0x1C) chooses, Escape (1) or the game closing gives -1; 'R' takes a
//! screenshot (with screenshots on), any other key may compile the lexicon again.
//!
//! Returns the connection chosen.
//! Callers: `net::settings_link`, `net::start_link`.

use crate::errors::Refused;
use crate::net::lobby::{list_key, other_key, run_frame};
use rc2k_core::net::screens::{self, ListKey};

/// A list's up (0x66) or down (0x6B) on a list of `count` items: the highlight and the top
/// moved; false for another key.
pub fn list_move(install: &crate::install::owner::Install, key: u8, count: u32) -> bool {
    // Converted: `rc2k_core::net::screens` (`ListKey`, `up`, `down`).
    let (choice, top) = match ListKey::of(key) {
        ListKey::Up => screens::up(
            install.pages.choice.get(),
            install.pages.choice_before.get(),
        ),
        ListKey::Down => screens::down(
            install.pages.choice.get(),
            install.pages.choice_before.get(),
            count,
        ),
        _ => return false,
    };
    install.pages.choice.set(choice);
    install.pages.choice_before.set(top);
    true
}
pub fn choose_connection(o: crate::owners::Owners<'static>) -> Result<u32, Refused> {
    let (install, net, system) = (o.install, o.net, o.system);
    let vars = &net.vars;
    let now = system.session.key_clock.ticks.get();
    install.pages.move_clock.set(now);
    install.pages.key_clock.set(now);
    install.pages.choice_before.set(0);
    install.pages.choice.set(0);
    loop {
        run_frame(o, || crate::net::connections_frame::connections_frame(o))?;
        install
            .pages
            .frame_clock
            .set(system.session.key_clock.ticks.get());
        let key = list_key(install, o.input)?;
        if list_move(install, key, vars.connections.get()) {
            install
                .pages
                .move_clock
                .set(install.pages.frame_clock.get());
        } else if ListKey::of(key) == ListKey::Enter {
            return Ok(install.pages.choice.get());
        } else if ListKey::of(key) == ListKey::Escape {
            break;
        } else {
            other_key(o, key)?;
        }
        if system.session.flags.quitting.is_set() {
            break;
        }
    }
    install.pages.choice.set(u32::MAX);
    Ok(install.pages.choice.get())
}
