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
//! Out: nothing (every register kept, `pusha` .. `popa`; the flags of the end's compare are
//!      read by no caller: the receiving thread goes on to its own stores and compares)
//! Caller: `net::receive_thread` (with a network session, when its event is signalled).
//! Refuses (before anything) more than 16 players (the blocks' room).

use crate::errors::{BadInput, Refused};
use crate::net::chat_line_locked::chat_line_locked;
use crate::net::dplay_send::dplay_send;
use crate::net::receive_message::receive_message;
use crate::net::store::NET;
use crate::net::store_received::store_received;
use crate::net::system_message::system_message;
use crate::net::{lock_received::lock_received, unlock_received::unlock_received};
use crate::pointers::GamePtr;
use rc2k_core::net::messages::{self, Kind};

/// The players the blocks have room for.
pub const MOST_PLAYERS: u32 = 16;

fn get8(at: u32) -> Result<u8, BadInput> {
    GamePtr::new(at).read::<u8>("a received message")
}

/// The Rust version.
pub fn take_messages() -> Result<(), Refused> {
    if NET.players.get() > MOST_PLAYERS {
        return Err(BadInput("take_messages: more than 16 players").into());
    }
    while NET.dplay.get() != 0 {
        let m = receive_message();
        if m.size == 0 {
            break;
        }
        NET.messages_taken
            .set(NET.messages_taken.get().wrapping_add(1));
        if m.from == 0 {
            system_message(m.bytes)?;
            continue;
        }
        // The sender among the players (the first compared even with none).
        let mut player = 0;
        let found = loop {
            if m.from == NET.player_block(player).id.get() {
                break true;
            }
            player += 1;
            if player >= NET.players.get() {
                break false;
            }
        };
        if !found {
            continue;
        }
        let block = NET.player_block(player);
        let first = get8(m.bytes)?;
        // Converted: `rc2k_core::net::messages` (`Kind`, `this_round`, `answers_call`,
        // `notes_answer`, `answer`, `race_fits`).
        let own_state = NET.own.state();
        // The core asks for the second byte only when it needs it: a refusal is kept.
        let round = || -> Result<bool, BadInput> {
            let refused = std::cell::Cell::new(None);
            let answer = messages::this_round(m.size, || {
                let second = get8(m.bytes + 1).unwrap_or_else(|e| {
                    refused.set(Some(e));
                    0
                });
                (second, NET.sync.get())
            });
            refused.take().map_or(Ok(answer), Err)
        };
        match Kind::of(first) {
            Kind::Chat => {
                GamePtr::new(m.size.wrapping_add(m.bytes)).write(0u8, "a received message")?;
                chat_line_locked(
                    player,
                    crate::pointers::host(m.bytes.wrapping_add(1)) as *const u8,
                )?;
            }
            Kind::Call { reply, ready } => {
                if !messages::answers_call(round()?, own_state) {
                    continue;
                }
                let answer = messages::answer(reply, NET.sync.get());
                NET.sync_answer.set(answer);
                dplay_send(block.id.get(), 0, NET.sync_answer.addr(), 2);
                (if ready {
                    &NET.all_raced
                } else {
                    &NET.race_over
                })
                .set(1);
            }
            Kind::Answer => {
                if !messages::notes_answer(round()?, own_state) {
                    continue;
                }
                NET.set_synced(player as usize, 1);
            }
            Kind::Race => {
                let length = m.size.wrapping_sub(1);
                if !messages::race_fits(length) {
                    continue;
                }
                let time = crate::system::tick_count::tick_count();
                lock_received();
                store_received(
                    first,
                    block,
                    time,
                    crate::pointers::host(m.bytes.wrapping_add(1)) as *const u8,
                    length,
                )?;
                unlock_received();
            }
            Kind::Other => {}
        }
    }
    Ok(())
}
