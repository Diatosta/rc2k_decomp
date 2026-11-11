//! `net::choose_session` (0x417726): the list screen of the sessions (`net::sessions_frame`
//! through `screens::run_frame`), as `net::choose_connection`'s, and once a second
//! (`system::tick_count` against `clock_a`) the sessions listed again
//! (`net::find_sessions_regs`, with `g_netPosting` 0 for it): the highlighted session's GUID
//! kept first (`joined`) and found again after (`system::same_guid`), the highlight on
//! it (the top moved by as much, from 0 when sixteen or fewer, not below 0), or at the top
//! when it is gone; a failed listing gives -2. With no session listed only Escape, 'R' and
//! the lexicon's key work.
//!
//! Returns the session chosen.
//! Caller: `net::start_link`.

use crate::errors::Refused;
use crate::net::choose_connection::list_move;
use crate::net::lobby::{list_key, other_key, run_frame};
use crate::net::owner::Net;
use crate::net::tables::with_session;
use crate::race::viewer_screen_store::ViewerPanelBlock;
use crate::system::owner::System;
use rc2k_core::net::screens::{self, ListKey};

/// The sessions listed again (not posted), the highlight kept on the same one; false for
/// a failed listing.
fn refresh(
    install: &crate::install::owner::Install,
    net: &Net,
    panel: &ViewerPanelBlock,
) -> Result<bool, Refused> {
    let listed = net.vars.sessions.get();
    if listed != 0 {
        let instance = with_session(&net.vars, install.pages.choice.get(), |s| s.instance)?;
        net.joined()
            .joined
            .write_at(0, &instance, "the joined session")?;
    }
    let posting = net.vars.posting.get();
    net.vars.posting.set(0);
    let found = crate::net::find_sessions::find_sessions(net, panel)?;
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
                // Converted: `rc2k_core::net::screens` (`keeps_top`, `moved_top`).
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
pub fn choose_session(
    install: &'static crate::install::owner::Install,
    net: &Net,
    system: &System,
    panel: &ViewerPanelBlock,
) -> Result<u32, Refused> {
    let now = system.session.clocks.key_ticks.get();
    install.pages.move_clock.set(now);
    install.pages.key_clock.set(now);
    install.pages.choice.set(0);
    install.pages.choice_before.set(0);
    net.edit()
        .clock_a
        .set(crate::system::tick_count::tick_count());
    loop {
        run_frame(|| crate::net::sessions_frame::sessions_frame(net, install))?;
        let tick = crate::system::tick_count::tick_count();
        if (tick
            .wrapping_sub(net.edit().clock_a.get())
            .wrapping_sub(1000) as i32)
            >= 0
        {
            net.edit().clock_a.set(tick);
            if !refresh(install, net, panel)? {
                install.pages.choice.set(0xFFFF_FFFE);
                return Ok(install.pages.choice.get());
            }
        }
        install
            .pages
            .frame_clock
            .set(system.session.clocks.key_ticks.get());
        let key = list_key()?;
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
            other_key(key)?;
        }
        if system.session.flags.quitting.is_set() {
            break;
        }
    }
    install.pages.choice.set(u32::MAX);
    Ok(install.pages.choice.get())
}
