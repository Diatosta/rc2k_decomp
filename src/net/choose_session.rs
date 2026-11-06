//! `net::choose_session` (0x417726): the list screen of the sessions (`net::sessions_frame`
//! through `screens::run_frame`), as `net::choose_connection`'s, and once a second
//! (`system::tick_count` against `dword_52A4C4`) the sessions listed again
//! (`net::find_sessions_regs`, with `g_netPosting` 0 for it): the highlighted session's GUID
//! kept first (`unk_52AD00`) and found again after (`system::same_guid`), the highlight on
//! it (the top moved by as much, from 0 when sixteen or fewer, not below 0), or at the top
//! when it is gone; a failed listing gives -2. With no session listed only Escape, 'R' and
//! the lexicon's key work.
//!
//! In:  nothing (the registers it is given reach only its frame, `net::sessions_frame`, and
//!      `menu::list_take_key`, which read none)
//! Out: the session chosen (`g_listChoice`: -1 Escape, -2 the listing failed; negative the
//!      original's carry); the original keeps the rest
//! Caller: `net::start_link`.

use crate::errors::Refused;
use crate::net::choose_connection::list_move;
use crate::net::lobby::{list_key, other_key, run_frame};
use crate::net::owner::net;
use crate::net::tables::with_session;
use crate::pointers::GamePtr;
use crate::system::owner::system;
use rc2k_core::net::screens::{self, ListKey};

/// The sessions listed again (not posted), the highlight kept on the same one; false for
/// a failed listing.
fn refresh() -> Result<bool, Refused> {
    let net = net();
    let listed = net.vars.sessions.get();
    if listed != 0 {
        let instance = with_session(crate::install::owner::install().pages.choice.get(), |s| {
            s.instance
        })?;
        net.vars
            .joined
            .write_at(0, &instance, "the joined session")?;
    }
    let posting = net.vars.posting.get();
    net.vars.posting.set(0);
    let found = crate::net::find_sessions::find_sessions()?;
    net.vars.posting.set(posting);
    if !found {
        return Ok(false);
    }
    let (mut choice, mut top) = (0, 0);
    if listed != 0 && net.vars.sessions.get() != 0 {
        let mut n = 0;
        loop {
            let joined = GamePtr::to(&net.vars.joined).read::<[u8; 16]>("the joined session")?;
            // `system::same_guid`: the four dwords alike.
            if with_session(n, |s| s.instance)? == joined {
                // Converted: `rc2k_core::net::screens` (`keeps_top`, `moved_top`).
                choice = n;
                top = if screens::keeps_top(net.vars.sessions.get()) {
                    screens::moved_top(
                        n,
                        crate::install::owner::install().pages.choice_before.get(),
                        crate::install::owner::install().pages.choice.get(),
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
    crate::install::owner::install().pages.choice.set(choice);
    crate::install::owner::install()
        .pages
        .choice_before
        .set(top);
    Ok(true)
}

/// The Rust version.
pub fn choose_session() -> Result<u32, Refused> {
    let net = net();
    let system = system();
    let now = system.session.clocks.key_ticks.get();
    crate::install::owner::install().pages.move_clock.set(now);
    crate::install::owner::install().pages.key_clock.set(now);
    crate::install::owner::install().pages.choice.set(0);
    crate::install::owner::install().pages.choice_before.set(0);
    net.vars
        .clock_a
        .set(crate::system::tick_count::tick_count());
    loop {
        run_frame(crate::net::sessions_frame::sessions_frame)?;
        let tick = crate::system::tick_count::tick_count();
        if (tick.wrapping_sub(net.vars.clock_a.get()).wrapping_sub(1000) as i32) >= 0 {
            net.vars.clock_a.set(tick);
            if !refresh()? {
                crate::install::owner::install()
                    .pages
                    .choice
                    .set(0xFFFF_FFFE);
                return Ok(crate::install::owner::install().pages.choice.get());
            }
        }
        crate::install::owner::install()
            .pages
            .frame_clock
            .set(system.session.clocks.key_ticks.get());
        let key = list_key()?;
        let any = net.vars.sessions.get() != 0;
        if any && list_move(key, net.vars.sessions.get()) {
            crate::install::owner::install()
                .pages
                .move_clock
                .set(crate::install::owner::install().pages.frame_clock.get());
        } else if any && ListKey::of(key) == ListKey::Enter {
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
