//! The list screen of the sessions, like `net::choose_connection`'s, with the sessions listed
//! again once a second; the highlighted session is found again by its GUID and stays highlighted
//! (at the top when it is gone). A failed listing gives -2. Returns the session chosen.

use crate::errors::Refused;
use crate::net::choose_connection::list_move;
use crate::net::lobby::{list_key, other_key, run_frame};
use crate::net::tables::with_session;
use crate::race::viewer_screen_store::NetTexts;
use rc2k_core::net::screens::{self, ListKey};

/// The sessions listed again (not posted), the highlight kept on the same one; false for
/// a failed listing.
fn refresh(o: crate::owners::Owners<'static>, lines: &NetTexts) -> Result<bool, Refused> {
    let (install, net) = (o.install, o.net);
    let listed = net.vars.sessions.get();
    if listed != 0 {
        let instance = with_session(&net.vars, install.pages.choice.get(), |s| s.instance)?;
        net.joined()
            .joined
            .write_at(0, &instance, "the joined session")?;
    }
    let posting = net.vars.posting.get();
    net.vars.posting.set(0);
    let found = crate::net::find_sessions::find_sessions(o, lines)?;
    net.vars.posting.set(posting);
    if !found {
        return Ok(false);
    }
    let (mut choice, mut top) = (0, 0);
    if listed != 0 && net.vars.sessions.get() != 0 {
        let mut n = 0;
        loop {
            let joined = net.joined().joined.read();
            // `system::same_guid`: the four dwords alike.
            if with_session(&net.vars, n, |s| s.instance)? == joined {
                // The logic is in `rc2k_core::net::screens` (`keeps_top`, `moved_top`).
                choice = n;
                top = if screens::keeps_top(net.vars.sessions.get()) {
                    screens::moved_top(
                        n,
                        install.pages.choice_before.get(),
                        install.pages.choice.get(),
                    )
                } else {
                    0
                };
                break;
            }
            n = n.wrapping_add(1);
            if n == net.vars.sessions.get() {
                break;
            }
        }
    }
    install.pages.choice.set(choice);
    install.pages.choice_before.set(top);
    Ok(true)
}
pub fn choose_session(o: crate::owners::Owners<'static>, lines: &NetTexts) -> Result<u32, Refused> {
    let (install, net, system) = (o.install, o.net, o.system);
    let now = system.session.key_clock.ticks.get();
    install.pages.move_clock.set(now);
    install.pages.key_clock.set(now);
    install.pages.choice.set(0);
    install.pages.choice_before.set(0);
    net.edit()
        .clock_a
        .set(crate::system::tick_count::tick_count());
    loop {
        run_frame(o, || crate::net::sessions_frame::sessions_frame(o))?;
        let tick = crate::system::tick_count::tick_count();
        if (tick
            .wrapping_sub(net.edit().clock_a.get())
            .wrapping_sub(1000) as i32)
            >= 0
        {
            net.edit().clock_a.set(tick);
            if !refresh(o, lines)? {
                install.pages.choice.set(0xFFFF_FFFE);
                return Ok(install.pages.choice.get());
            }
        }
        install
            .pages
            .frame_clock
            .set(system.session.key_clock.ticks.get());
        let key = list_key(install, o.input)?;
        let any = net.vars.sessions.get() != 0;
        if any && list_move(install, key, net.vars.sessions.get()) {
            install
                .pages
                .move_clock
                .set(install.pages.frame_clock.get());
        } else if any && ListKey::of(key) == ListKey::Enter {
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
