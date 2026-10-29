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
use crate::install::vars::INSTALLER_PAGES;
use crate::net::choose_connection::list_move;
use crate::net::lobby::{list_key, other_key, run_frame};
use crate::net::store::NET;
use crate::net::tables::with_session;
use crate::pointers::GamePtr;
use crate::session::CLOCKS;
use crate::session::FLAGS;
use rc2k_core::net::screens::{self, ListKey};

/// The sessions listed again (not posted), the highlight kept on the same one; false for
/// a failed listing.
fn refresh() -> Result<bool, Refused> {
    let listed = NET.sessions.get();
    if listed != 0 {
        let instance = with_session(INSTALLER_PAGES.choice.get(), |s| s.instance)?;
        NET.joined.write_at(0, &instance, "the joined session")?;
    }
    let posting = NET.posting.get();
    NET.posting.set(0);
    let found = crate::net::find_sessions::find_sessions()?;
    NET.posting.set(posting);
    if !found {
        return Ok(false);
    }
    let (mut choice, mut top) = (0, 0);
    if listed != 0 && NET.sessions.get() != 0 {
        let mut n = 0;
        loop {
            let joined = GamePtr::new(NET.joined.addr()).read::<[u8; 16]>("the joined session")?;
            // `system::same_guid`: the four dwords alike.
            if with_session(n, |s| s.instance)? == joined {
                // Converted: `rc2k_core::net::screens` (`keeps_top`, `moved_top`).
                choice = n;
                top = if screens::keeps_top(NET.sessions.get()) {
                    screens::moved_top(
                        n,
                        INSTALLER_PAGES.choice_before.get(),
                        INSTALLER_PAGES.choice.get(),
                    )
                } else {
                    0
                };
                break;
            }
            n = n.wrapping_add(1);
            if n == NET.sessions.get() {
                break;
            }
        }
    }
    INSTALLER_PAGES.choice.set(choice);
    INSTALLER_PAGES.choice_before.set(top);
    Ok(true)
}

/// The Rust version.
pub fn choose_session() -> Result<u32, Refused> {
    let now = CLOCKS.key_ticks.get();
    INSTALLER_PAGES.move_clock.set(now);
    INSTALLER_PAGES.key_clock.set(now);
    INSTALLER_PAGES.choice.set(0);
    INSTALLER_PAGES.choice_before.set(0);
    NET.clock_a.set(crate::system::tick_count::tick_count());
    loop {
        run_frame(crate::net::sessions_frame::sessions_frame)?;
        let tick = crate::system::tick_count::tick_count();
        if (tick.wrapping_sub(NET.clock_a.get()).wrapping_sub(1000) as i32) >= 0 {
            NET.clock_a.set(tick);
            if !refresh()? {
                INSTALLER_PAGES.choice.set(0xFFFF_FFFE);
                return Ok(INSTALLER_PAGES.choice.get());
            }
        }
        INSTALLER_PAGES.frame_clock.set(CLOCKS.key_ticks.get());
        let key = list_key()?;
        let any = NET.sessions.get() != 0;
        if any && list_move(key, NET.sessions.get()) {
            INSTALLER_PAGES
                .move_clock
                .set(INSTALLER_PAGES.frame_clock.get());
        } else if any && ListKey::of(key) == ListKey::Enter {
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
