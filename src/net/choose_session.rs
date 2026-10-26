//! `net::choose_session` (0x417726): the list screen of the sessions (`net::sessions_frame`
//! through `screens::run_frame`), as `net::choose_connection`'s, and once a second
//! (`system::tick_count` against `dword_52A4C4`) the sessions listed again
//! (`net::find_sessions_regs`, with `g_netPosting` 0 for it): the highlighted session's GUID
//! kept first (`unk_52AD00`) and found again after (`system::same_guid`), the highlight on
//! it (the top moved by as much, from 0 when sixteen or fewer, not below 0), or at the top
//! when it is gone; a failed listing gives -2. With no session listed only Escape, 'R' and
//! the lexicon's key work.
//!
//! In:  nothing
//! Out: eax = the session chosen (`g_listChoice`: -1 Escape, -2 the listing failed), carry
//!      set if negative (else the flags of `test eax, eax`); the rest kept
//! Caller: `net::start_link`. Its calls through `part` (the frames through the game). The
//! whole screen runs inside it: ungated. Shadow mode runs the original alone. Self-test: a
//! copy whose calls go to stand-ins (the keys and the clock scripted, the listings giving
//! records of a test table), against the Rust version.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::field;
use crate::framework::{Regs, ZERO_FLAG};
use crate::net::choose_connection::{chosen, list_move};
use crate::net::lobby::{
    CHOICE, CHOICE_BEFORE, CLOCK, CLOCK_A, FRAME_CLOCK, JOINED, KEY_CLOCK, LIST_TAKE_KEY,
    MAYBE_LEXICON, MOVE_CLOCK, POSTING, Part, QUITTING, RUN_FRAME, SAME_GUID, SCREENSHOT,
    SCREENSHOTS_ON, SESSIONS, TICK, a, g, game, lobby_entry, movsb, p,
};
use crate::net::types::NetSession;
use rc2k_core::net::screens::{self, ListKey};

const FRAME: u32 = 0x41790E; // net::sessions_frame
const SESSION_ENTRY: u32 = 0x418553; // net::session_entry
const FIND_SESSIONS: u32 = 0x41771E; // net::find_sessions_regs

/// The sessions listed again, the highlight kept on the same one. Err(()) for a failed
/// listing.
fn refresh(r: &mut Regs, part: Part) -> Result<bool, Refused> {
    r.ebx = g(SESSIONS);
    if r.ebx != 0 {
        r.eax = g(CHOICE);
        part(SESSION_ENTRY, r)?;
        r.esi = r.eax + field!(NetSession, instance);
        r.edi = a(JOINED);
        movsb(r.edi, r.esi, 0x10);
        r.esi += 0x10;
        r.edi += 0x10;
        r.ecx = 0;
    }
    let posting = g(POSTING);
    p(POSTING, 0);
    part(FIND_SESSIONS, r)?;
    p(POSTING, posting);
    if r.carry() {
        return Ok(false);
    }
    if r.ebx == 0 || g(SESSIONS) == 0 {
        // loc_4177F1: xor ecx, ecx; xor edx, edx
        r.ecx = 0;
        r.edx = 0;
    } else {
        r.ebx = a(JOINED);
        r.ecx = 0;
        loop {
            r.eax = r.ecx;
            part(SESSION_ENTRY, r)?;
            r.eax = r.eax.wrapping_add(0x40);
            part(SAME_GUID, r)?;
            if r.eflags & ZERO_FLAG != 0 {
                // Converted: `rc2k_core::net::screens` (`keeps_top`, `moved_top`).
                r.edx = if screens::keeps_top(g(SESSIONS)) {
                    screens::moved_top(r.ecx, g(CHOICE_BEFORE), g(CHOICE))
                } else {
                    0
                };
                break;
            }
            r.ecx = r.ecx.wrapping_add(1);
            if r.ecx == g(SESSIONS) {
                r.ecx = 0;
                r.edx = 0;
                break;
            }
        }
    }
    p(CHOICE, r.ecx);
    p(CHOICE_BEFORE, r.edx);
    Ok(true)
}

/// The Rust version: the registers it leaves.
pub fn choose_session(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut r = *entry;
    r.eax = g(CLOCK);
    p(MOVE_CLOCK, r.eax);
    p(KEY_CLOCK, r.eax);
    p(CHOICE, 0);
    p(CHOICE_BEFORE, 0);
    part(TICK, &mut r)?;
    p(CLOCK_A, r.eax);
    loop {
        r.eax = game(FRAME);
        part(RUN_FRAME, &mut r)?;
        part(TICK, &mut r)?;
        r.ebx = r.eax;
        r.eax = r.eax.wrapping_sub(g(CLOCK_A));
        if (r.eax.wrapping_sub(1000) as i32) >= 0 {
            p(CLOCK_A, r.ebx);
            if !refresh(&mut r, part)? {
                p(CHOICE, 0xFFFF_FFFE);
                return Ok(chosen(entry, &r));
            }
        }
        r.eax = g(CLOCK);
        p(FRAME_CLOCK, r.eax);
        part(LIST_TAKE_KEY, &mut r)?;
        let key = r.eax as u8;
        let any = g(SESSIONS) != 0;
        if any && list_move(&mut r, key, g(SESSIONS)) {
            r.ebx = g(FRAME_CLOCK);
            p(MOVE_CLOCK, r.ebx);
        } else if any && ListKey::of(key) == ListKey::Enter {
            r.eax = g(CHOICE);
            return Ok(chosen(entry, &r));
        } else if ListKey::of(key) == ListKey::Escape {
            break;
        } else {
            if g(SCREENSHOTS_ON) != 0 && key == 0x52 {
                part(SCREENSHOT, &mut r)?;
            }
            part(MAYBE_LEXICON, &mut r)?;
        }
        if g(QUITTING) == 1 {
            break;
        }
    }
    p(CHOICE, u32::MAX);
    Ok(chosen(entry, &r))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    lobby_entry(r, choose_session)
}
