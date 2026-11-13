//! The receiving thread's turn: while a DirectPlay object exists and a message comes, it is
//! counted and handled by sender and first byte:
//! - sender 0: a system message;
//! - a sender not among the players: dropped (the first player is compared even with none);
//! - 0x22: a chat line (the text after the byte);
//! - 0x40 / 0x50 (a sync round, on a machine that is not the host): answered 0x41 / 0x51 and the
//!   race flags set; a machine that has just joined takes the round of the host's first call;
//! - 0x41 / 0x51 (on the host): the player's synced byte is set;
//! - bit 0x80 (a race message, 1 to 0xFF bytes): the bytes after the first stored with the time
//!   in the player's block under the lock;
//! - others dropped.
//!
//! Refuses more than 16 players (the blocks' room).

use crate::errors::{BadInput, Refused};
use crate::net::chat_line_locked::chat_line_locked;
use crate::net::dplay_send::dplay_send;
use crate::net::owner::Net;
use crate::net::receive_message::ROOM;
use crate::net::receive_message::receive_message;
use crate::net::store::NetShared;
use crate::net::store_received::store_received;
use crate::net::system_message::system_message;
use crate::net::{lock_received::lock_received, unlock_received::unlock_received};
use rc2k_core::net::messages::{self, Kind};

/// The players the blocks have room for.
pub const MOST_PLAYERS: u32 = rc2k_core::net::players::MOST;

/// Whether this machine has joined a session and not yet heard the host's round. Every machine
/// counts the rounds up as it enters the lobby, so one that joins after races (the host lets
/// players in again between them, `net::enable_new_players`) is behind the others: it takes the
/// round of the host's first call. A machine there from the start has the host's round already
/// and takes the same.
static ROUND_UNKNOWN: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// A session joined (`net::start_link`): the round taken from the host's first call.
pub fn joined() {
    ROUND_UNKNOWN.store(true, std::sync::atomic::Ordering::Relaxed);
}

/// The received message's byte `at` (the buffer's `receive_data`).
fn get8(vars: &NetShared, at: usize) -> Result<u8, BadInput> {
    if at >= ROOM as usize {
        return Err(BadInput("a received message past its buffer"));
    }
    Ok(vars.receive_data.get(at))
}

/// The received message from its byte 1 on.
fn after_first(vars: &NetShared, length: usize) -> Result<Vec<u8>, BadInput> {
    vars.receive_data.read_at(1, length, "a received message")
}
pub fn take_messages(
    net: &Net,
    gfx: &'static crate::gfx::owner::Gfx,
    lines: &crate::race::viewer_screen_store::NetTexts,
) -> Result<(), Refused> {
    if net.vars.players.get() > MOST_PLAYERS {
        return Err(BadInput("take_messages: more than 8 players").into());
    }
    while net.vars.dplay.get() != 0 {
        let m = receive_message(net);
        if m.size == 0 {
            break;
        }
        net.vars
            .messages_taken
            .set(net.vars.messages_taken.get().wrapping_add(1));
        if m.from == 0 {
            system_message(&net.vars, lines)?;
            continue;
        }
        // The sender among the players (the first compared even with none).
        let mut player = 0;
        let found = loop {
            if m.from == net.vars.player_block(player)?.id.get() {
                break true;
            }
            player += 1;
            if player >= net.vars.players.get() {
                break false;
            }
        };
        if !found {
            continue;
        }
        let block = net.vars.player_block(player)?;
        let first = get8(&net.vars, 0)?;
        // The logic is in `rc2k_core::net::messages` (`Kind`, `this_round`, `answers_call`,
        // `notes_answer`, `answer`, `race_fits`).
        let own_state = net.vars.own.state();
        // The core asks for the second byte only when it needs it: a refusal is kept.
        let round = || -> Result<bool, BadInput> {
            let refused = std::cell::Cell::new(None);
            let answer = messages::this_round(m.size, || {
                let second = get8(&net.vars, 1).unwrap_or_else(|e| {
                    refused.set(Some(e));
                    0
                });
                (second, net.vars.sync.get())
            });
            refused.take().map_or(Ok(answer), Err)
        };
        match Kind::of(first) {
            Kind::Chat => {
                // A message that fills the buffer gives up its last byte to the NUL (fixed: "A
                // full-length chat message writes past its buffer" in `wiki/Fixed-bugs.md`).
                net.vars
                    .receive_data
                    .set((m.size as usize).min(ROOM as usize - 1), 0);
                chat_line_locked(
                    net,
                    gfx,
                    player,
                    &after_first(&net.vars, ROOM as usize - 1)?,
                )?;
            }
            Kind::Call { reply, ready } => {
                if m.size == 2
                    && !rc2k_core::net::record::is_host(own_state)
                    && ROUND_UNKNOWN.swap(false, std::sync::atomic::Ordering::Relaxed)
                {
                    net.vars.sync.set(get8(&net.vars, 1)?);
                }
                if !messages::answers_call(round()?, own_state) {
                    continue;
                }
                let answer = messages::answer(reply, net.vars.sync.get());
                net.vars.sync_answer.set(answer);
                dplay_send(
                    &net.vars,
                    block.id.get(),
                    &net.vars.sync_answer.get().to_le_bytes(),
                );
                (if ready {
                    &net.vars.all_raced
                } else {
                    &net.vars.race_over
                })
                .set(1);
            }
            Kind::Answer => {
                if !messages::notes_answer(round()?, own_state) {
                    continue;
                }
                net.vars.set_synced(player as usize, 1)?;
            }
            Kind::Race => {
                let length = m.size.wrapping_sub(1);
                if !messages::race_fits(length) {
                    continue;
                }
                let time = crate::system::tick_count::tick_count();
                lock_received(&net.vars);
                store_received(
                    first,
                    block,
                    time,
                    &after_first(&net.vars, length as usize)?,
                )?;
                unlock_received(&net.vars);
            }
            Kind::Other => {}
        }
    }
    Ok(())
}
