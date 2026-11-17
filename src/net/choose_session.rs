//! The list screen of the sessions, like `net::choose_connection`'s, with the sessions listed
//! again once a second; the highlighted session is found again by its GUID and stays highlighted
//! (at the top when it is gone). A failed listing gives -2. Returns the session chosen. A game
//! started from the command line takes the first session listed without waiting for a key; an
//! accepted invitation takes the session it names as soon as it is listed (until then the list is
//! shown as usual).

use crate::errors::Refused;
use crate::net::choose_connection::list_move;
use crate::net::lobby::{list_key, other_key, run_frame};
use crate::net::tables::with_session;
use crate::race::viewer_screen_store::NetTexts;
use rc2k_core::net::screens::{self, ListKey};

/// The sessions listed again (not posted), the highlight kept on the same one; false for
/// a failed listing.
fn refresh(o: crate::owners::Owners<'static>, lines: &NetTexts) -> Result<bool, Refused> {
    let net = o.net;
    let list_state = &o.menus.list;
    let listed = net.link.sessions().get();
    if listed != 0 {
        let instance = with_session(&net.link, list_state.choice.get(), |s| s.instance)?;
        net.joined()
            .joined
            .write_at(0, &instance, "the joined session")?;
    }
    let posting = net.vars.receiver.posting.get();
    net.vars.receiver.posting.set(0);
    let found = crate::net::find_sessions::find_sessions(o, lines)?;
    net.vars.receiver.posting.set(posting);
    if !found {
        return Ok(false);
    }
    let (mut choice, mut top) = (0, 0);
    if listed != 0 && net.link.sessions().get() != 0 {
        let mut n = 0;
        loop {
            let joined = net.joined().joined.read();
            // `system::same_guid`: the four dwords alike.
            if with_session(&net.link, n, |s| s.instance)? == joined {
                // The logic is in `rc2k_core::net::screens` (`keeps_top`, `moved_top`).
                choice = n;
                top = if screens::keeps_top(net.link.sessions().get()) {
                    screens::moved_top(n, list_state.choice_before.get(), list_state.choice.get())
                } else {
                    0
                };
                break;
            }
            n = n.wrapping_add(1);
            if n == net.link.sessions().get() {
                break;
            }
        }
    }
    list_state.choice.set(choice);
    list_state.choice_before.set(top);
    Ok(true)
}
/// How the session is chosen.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pick {
    /// The player chooses from the list.
    List,
    /// The first session listed.
    First,
    /// The session with this instance.
    Instance([u8; 16]),
}

pub fn choose_session(
    o: crate::owners::Owners<'static>,
    lines: &NetTexts,
    pick: Pick,
) -> Result<u32, Refused> {
    let (net, system) = (o.net, o.system);
    let list_state = &o.menus.list;
    let now = system.session.key_clock.ticks.get();
    list_state.move_clock.set(now);
    list_state.key_clock.set(now);
    list_state.choice.set(0);
    list_state.choice_before.set(0);
    net.sessions()
        .listed_at
        .set(crate::system::tick_count::tick_count(
            &system.session.picture_clock,
        ));
    loop {
        run_frame(o, || crate::net::sessions_frame::sessions_frame(o))?;
        let tick = crate::system::tick_count::tick_count(&system.session.picture_clock);
        if (tick
            .wrapping_sub(net.sessions().listed_at.get())
            .wrapping_sub(1000) as i32)
            >= 0
        {
            net.sessions().listed_at.set(tick);
            if !refresh(o, lines)? {
                list_state.choice.set(0xFFFF_FFFE);
                return Ok(list_state.choice.get());
            }
        }
        list_state
            .frame_clock
            .set(system.session.key_clock.ticks.get());
        let key = list_key(list_state, o.input)?;
        let any = net.link.sessions().get() != 0;
        if any && pick == Pick::First {
            return Ok(list_state.choice.get());
        }
        if let Pick::Instance(wanted) = pick {
            for n in 0..net.link.sessions().get() {
                if with_session(&net.link, n, |s| s.instance)? == wanted {
                    list_state.choice.set(n);
                    return Ok(n);
                }
            }
        }
        if any && list_move(list_state, key, net.link.sessions().get()) {
            list_state.move_clock.set(list_state.frame_clock.get());
        } else if any && ListKey::of(key) == ListKey::Enter {
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
