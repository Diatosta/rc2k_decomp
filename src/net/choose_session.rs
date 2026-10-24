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
use crate::field;
use crate::net::choose_connection::list_move;
use crate::net::lobby::{
    CHOICE, CHOICE_BEFORE, CLOCK_A, FRAME_CLOCK, JOINED, KEY_CLOCK, MOVE_CLOCK, POSTING, SESSIONS,
    a, g, list_key, movsb, other_key, p, run_frame, session,
};
use crate::net::types::NetSession;
use crate::session::CLOCKS;
use crate::session::FLAGS;
use crate::system::same_guid::same_guid;
use rc2k_core::net::screens::{self, ListKey};

/// The sessions listed again (not posted), the highlight kept on the same one; false for
/// a failed listing.
fn refresh() -> Result<bool, Refused> {
    let listed = g(SESSIONS);
    if listed != 0 {
        movsb(
            a(JOINED),
            session(g(CHOICE))? + field!(NetSession, instance),
            0x10,
        );
    }
    let posting = g(POSTING);
    p(POSTING, 0);
    let found = crate::net::find_sessions::find_sessions()?;
    p(POSTING, posting);
    if !found {
        return Ok(false);
    }
    let (mut choice, mut top) = (0, 0);
    if listed != 0 && g(SESSIONS) != 0 {
        let mut n = 0;
        loop {
            if same_guid(session(n)?.wrapping_add(0x40), a(JOINED)) {
                // Converted: `rc2k_core::net::screens` (`keeps_top`, `moved_top`).
                choice = n;
                top = if screens::keeps_top(g(SESSIONS)) {
                    screens::moved_top(n, g(CHOICE_BEFORE), g(CHOICE))
                } else {
                    0
                };
                break;
            }
            n = n.wrapping_add(1);
            if n == g(SESSIONS) {
                break;
            }
        }
    }
    p(CHOICE, choice);
    p(CHOICE_BEFORE, top);
    Ok(true)
}

/// The Rust version.
pub fn choose_session() -> Result<u32, Refused> {
    let now = CLOCKS.key_ticks.get();
    p(MOVE_CLOCK, now);
    p(KEY_CLOCK, now);
    p(CHOICE, 0);
    p(CHOICE_BEFORE, 0);
    p(CLOCK_A, crate::system::tick_count::tick_count());
    loop {
        run_frame(crate::net::sessions_frame::sessions_frame)?;
        let tick = crate::system::tick_count::tick_count();
        if (tick.wrapping_sub(g(CLOCK_A)).wrapping_sub(1000) as i32) >= 0 {
            p(CLOCK_A, tick);
            if !refresh()? {
                p(CHOICE, 0xFFFF_FFFE);
                return Ok(g(CHOICE));
            }
        }
        p(FRAME_CLOCK, CLOCKS.key_ticks.get());
        let key = list_key()?;
        let any = g(SESSIONS) != 0;
        if any && list_move(key, g(SESSIONS)) {
            p(MOVE_CLOCK, g(FRAME_CLOCK));
        } else if any && ListKey::of(key) == ListKey::Enter {
            return Ok(g(CHOICE));
        } else if ListKey::of(key) == ListKey::Escape {
            break;
        } else {
            other_key(key)?;
        }
        if FLAGS.quitting.get() == 1 {
            break;
        }
    }
    p(CHOICE, u32::MAX);
    Ok(g(CHOICE))
}
