//! The list screen of the DirectPlay connections. Up and down move the highlight (sixteen rows
//! visible), Enter chooses, Escape or the game closing gives -1. Returns the connection chosen.

use crate::errors::Refused;
use crate::net::lobby::{list_key, other_key, run_frame};
use rc2k_core::net::screens::{self, ListKey};

/// A list's up (0x66) or down (0x6B) on a list of `count` items: the highlight and the top
/// moved; false for another key.
pub(super) fn list_move(
    list_state: &crate::menu::list_state::ListState,
    key: u8,
    count: u32,
) -> bool {
    // The logic is in `rc2k_core::net::screens` (`ListKey`, `up`, `down`).
    let (choice, top) = match ListKey::of(key) {
        ListKey::Up => screens::up(list_state.choice.get(), list_state.choice_before.get()),
        ListKey::Down => screens::down(
            list_state.choice.get(),
            list_state.choice_before.get(),
            count,
        ),
        _ => return false,
    };
    list_state.choice.set(choice);
    list_state.choice_before.set(top);
    true
}
pub(super) fn choose_connection(o: crate::owners::Owners<'static>) -> Result<u32, Refused> {
    let (net, system) = (o.net, o.system);
    let list_state = &o.menus.list;
    let now = system.session.key_clock.ticks.get();
    list_state.move_clock.set(now);
    list_state.key_clock.set(now);
    list_state.choice_before.set(0);
    list_state.choice.set(0);
    loop {
        run_frame(o, || crate::net::connections_frame::connections_frame(o))?;
        list_state
            .frame_clock
            .set(system.session.key_clock.ticks.get());
        let key = list_key(list_state, o.input)?;
        if list_move(list_state, key, net.link.connections().get()) {
            list_state.move_clock.set(list_state.frame_clock.get());
        } else if ListKey::of(key) == ListKey::Enter {
            return Ok(list_state.choice.get());
        } else if ListKey::of(key) == ListKey::Escape {
            break;
        } else {
            other_key(o, key)?;
        }
        if system.session.flags.quitting.is_set() {
            break;
        }
    }
    list_state.choice.set(u32::MAX);
    Ok(list_state.choice.get())
}
