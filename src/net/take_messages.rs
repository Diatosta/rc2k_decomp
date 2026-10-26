//! `net::take_messages` (0x4194CC): the receiving thread's turn: while there is a DirectPlay
//! object (`g_dplay`) and `net::receive_message` gives a message (ecx not 0), the messages
//! taken (`dword_52A91C` counted) and handled:
//! - sender 0: DirectPlay's system message (`net::system_message`);
//! - a sender not among the players (`g_netReceived` +0x20, `g_netPlayers`; the first
//!   player compared even with none): dropped;
//! - first byte 0x22: a chat line (`net::chat_line_locked`, eax the player, ebx the text
//!   after the byte; a NUL put at the end first, at the bytes + ecx, past the buffer for a
//!   message of 0x100 bytes: on `g_netSendBuffer`'s first byte, section 16);
//! - 0x40 / 0x50 (two bytes, the second the round `g_netSync`; on a machine not the host,
//!   `g_netOwnRecord` bit 0x10 clear): answered 0x41 / 0x51 with the round
//!   (`word_52A916`, `net::dplay_send` to the sender, flags 0) and `g_netAllRaced` /
//!   `g_netRaceOver` set;
//! - 0x41 / 0x51 (two bytes, the round; on the host): the player's byte of `g_netSynced` 1;
//! - bit 0x80 (a race message), 1 .. 0xFF bytes: the bytes after the first, with the time
//!   (`system::tick_count`), put into the player's block (`net::store_received`) under the
//!   lock (`net::lock_received` .. `net::unlock_received`);
//! - others dropped.
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`); the flags of the end's compare (zero set,
//!      carry and sign clear)
//! Caller: `net::receive_thread` (with a network session, when its event is signalled).
//! The registers go on from call to call as in the original. Its calls through `part` (in
//! the game the Rust versions). Refuses (before anything) more than 16 players (the
//! blocks' room). A loop target in its first five bytes: private copy (the whole function,
//! its nine calls relocated), manual patch. Shadow mode runs the original alone (messages
//! are taken). Self-test: a copy whose variables are the test's and whose calls go to
//! noting stand-ins (the receive one giving a script of messages), against the Rust
//! version: no object, no message, each kind of message, on the host and not.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::{BadInput, Refused};
use crate::framework::Regs;
use crate::net::lobby::set_sign;
use crate::net::message_vars::{MessageVars, get, get8, set, set8};
use crate::net::{
    CHAT_LINE, LOCK, Part, RECEIVE, RECEIVED_LEN, SEND, STORE, SYSTEM_MESSAGE, TICK, UNLOCK,
    net_entry,
};
use rc2k_core::net::messages::{self, Kind};

/// The players the blocks have room for.
pub const MOST_PLAYERS: u32 = 16;

/// The Rust version on the variables at `v`.
pub fn take_messages_with(r: &mut Regs, v: &MessageVars, part: Part) -> Result<(), Refused> {
    if get(v.players) > MOST_PLAYERS {
        return Err(BadInput("take_messages: more than 16 players").into());
    }
    let mut c = *r;
    while get(v.dplay) != 0 {
        part(RECEIVE, &mut c)?;
        if c.ecx == 0 {
            break;
        }
        set(v.taken, get(v.taken).wrapping_add(1));
        if c.eax == 0 {
            part(SYSTEM_MESSAGE, &mut c)?;
            continue;
        }
        c.edi = v.received;
        c.edx = 0;
        let found = loop {
            if c.eax == get(c.edi + 0x20) {
                break true;
            }
            c.edi = c.edi.wrapping_add(RECEIVED_LEN);
            c.edx += 1;
            if c.edx >= get(v.players) {
                break false;
            }
        };
        if !found {
            continue;
        }
        c.eax = get8(c.ebx) as u32;
        // Converted: `rc2k_core::net::messages` (`Kind`, `this_round`, `answers_call`,
        // `notes_answer`, `answer`, `race_fits`).
        let own_state = get8(v.own);
        let round = |c: &mut Regs| {
            messages::this_round(c.ecx, || {
                c.eax = get8(c.ebx + 1) as u32;
                (c.eax as u8, get8(v.sync))
            })
        };
        match Kind::of(c.eax as u8) {
            Kind::Chat => {
                c.eax = c.edx;
                set8(c.ecx.wrapping_add(c.ebx), 0);
                c.ebx = c.ebx.wrapping_add(1);
                part(CHAT_LINE, &mut c)?;
            }
            Kind::Call { reply, ready } => {
                let same = round(&mut c);
                if !messages::answers_call(same, own_state) {
                    continue;
                }
                c.eax = get(c.edi + 0x20);
                c.ebx = v.answer;
                c.ecx = (c.ecx & 0xFFFF_0000) | messages::answer(reply, get8(v.sync)) as u32;
                unsafe { (c.ebx as *mut u16).write_unaligned(c.ecx as u16) };
                c.ecx = 2;
                c.edx = 0;
                part(SEND, &mut c)?;
                set(if ready { v.all_raced } else { v.race_over }, 1);
            }
            Kind::Answer => {
                let same = round(&mut c);
                if !messages::notes_answer(same, own_state) {
                    continue;
                }
                set8(v.synced.wrapping_add(c.edx), 1);
            }
            Kind::Race => {
                c.ebx = c.ebx.wrapping_add(1);
                c.ecx = c.ecx.wrapping_sub(1);
                if !messages::race_fits(c.ecx) {
                    continue;
                }
                let first = c.eax;
                part(TICK, &mut c)?;
                c.edx = c.eax;
                c.eax = first;
                part(LOCK, &mut c)?;
                part(STORE, &mut c)?;
                part(UNLOCK, &mut c)?;
            }
            Kind::Other => {}
        }
    }
    // popa; the flags of `cmp g_dplay, 0` or `test ecx, ecx` on 0.
    r.set_carry(false);
    r.set_zero(true);
    set_sign(r, 0);
    Ok(())
}

/// The Rust version.
pub fn take_messages(r: &mut Regs, part: Part) -> Result<(), Refused> {
    take_messages_with(r, &MessageVars::game(), part)
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    net_entry(r, take_messages)
}
