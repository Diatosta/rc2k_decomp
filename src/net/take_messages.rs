//! The receiving thread's turn: while a DirectPlay object exists and a message comes, it is
//! counted and handled by sender and first byte:
//! - sender 0: a system message;
//! - a sender not among the players: dropped (the first player is compared even with none);
//! - 0x22: a chat line (the text after the byte): the host takes it and orders it, others drop it;
//! - 0x23: a chat line in the host's order (its number, the author's id, the text), taken by a
//!   machine that is not the host;
//! - 0x40 / 0x50 (a sync round, on a machine that is not the host): answered 0x41 / 0x51 and the
//!   race flags set; a machine that has just joined takes the round of the host's first call;
//! - 0x60: the sender's stage load's percentage, noted for the loading screen's list
//!   (`net::loading`);
//! - 0x41 / 0x51 (on the host): the player's synced byte is set;
//! - bit 0x80 (a race message, 1 to 0xFF bytes): the bytes after the first stored with the time
//!   in the player's inbox under the lock;
//! - others dropped.
//!
//! Refuses more than 16 players (the blocks' room).

use crate::errors::{BadInput, Refused};
use crate::net::dplay_send::dplay_send;
use crate::net::owner::Net;
use crate::net::receive_message::ROOM;
use crate::net::receive_message::receive_message;
use crate::net::store::inbox_of;
use crate::net::store_received::store_received;
use crate::net::system_message::system_message;
use rc2k_core::net::message::Message;
use rc2k_core::net::messages;

/// The players the blocks have room for.
pub const MOST_PLAYERS: u32 = rc2k_core::net::players::MOST;

/// A session joined (`net::start_link`): the round is taken from the host's first call.
pub fn joined(net: &Net) {
    net.vars.receiver.round_unknown.set(true);
}

pub fn take_messages(
    net: &Net,
    lines: &crate::race::viewer_screen_store::NetTexts,
    clock: &crate::system::tick_count::PictureClock,
) -> Result<(), Refused> {
    if net.race_slot.players() > MOST_PLAYERS {
        return Err(BadInput("take_messages: more than 8 players").into());
    }
    while net.link.has_dplay() {
        let m = receive_message(net);
        if m.size == 0 {
            break;
        }
        net.vars
            .receiver
            .messages_taken
            .set(net.vars.receiver.messages_taken.get().wrapping_add(1));
        if m.from == 0 {
            system_message(net, lines)?;
            continue;
        }
        // The sender among the players (the first compared even with none).
        let mut player = 0;
        let found = loop {
            if m.from == net.vars.player_block(player)?.id.get() {
                break true;
            }
            player += 1;
            if player >= net.race_slot.players() {
                break false;
            }
        };
        if !found {
            continue;
        }
        let block = net.vars.player_block(player)?;
        let bytes = net.vars.receiver.receive_data.read_at(
            0,
            (m.size as usize).min(ROOM as usize),
            "a received message",
        )?;
        let own_state = net.vars.own.state();
        // The rules of each kind are in `rc2k_core::net::messages` (`this_round`,
        // `answers_call`, `notes_answer`, `answer`).
        match Message::parse(&bytes) {
            // A joiner's line goes to the host, which orders it; a machine that is not the host
            // takes no such line.
            Message::Chat(_) if !own_state.is_host() => {}
            Message::Chat(mut text) => {
                // A message that fills the buffer gives up its last byte to the NUL (fixed: "A
                // full-length chat message writes past its buffer" in `wiki/Fixed-bugs.md`).
                net.vars
                    .receiver
                    .receive_data
                    .set((m.size as usize).min(ROOM as usize - 1), 0);
                text.truncate(ROOM as usize - 2);
                text.push(0);
                // Added to the chat by the game's thread (`net::drain_chat`), which owns the fonts.
                net.queue_chat(player, text, None);
            }
            // The host's order for a line: its author found by id. The host takes none.
            Message::ChatRelayed {
                sequence,
                author,
                mut text,
            } => {
                let mut author_player = None;
                for n in 0..net.race_slot.players() {
                    if net.vars.player_block(n)?.id.get() == author {
                        author_player = Some(n);
                        break;
                    }
                }
                if own_state.is_host() || author_player.is_none() {
                    continue;
                }
                text.truncate(ROOM as usize - 2);
                text.push(0);
                net.queue_chat(author_player.unwrap_or(0), text, Some(sequence));
            }
            Message::Loading { share } => crate::net::loading::note(player, share),
            Message::Call { ready, round } => {
                if !own_state.is_host()
                    && net
                        .vars
                        .receiver
                        .round_unknown
                        .atomic()
                        .swap(0, std::sync::atomic::Ordering::Relaxed)
                        != 0
                {
                    net.vars.receiver.sync.set(round);
                }
                let this_round = messages::this_round(2, || (round, net.vars.receiver.sync.get()));
                if !messages::answers_call(this_round, own_state) {
                    continue;
                }
                let reply = if ready {
                    messages::READY_ANSWER
                } else {
                    messages::DONE_ANSWER
                };
                let round = net.vars.receiver.sync.get();
                net.vars
                    .receiver
                    .sync_answer
                    .set(messages::answer(reply, round));
                dplay_send(
                    &net.link,
                    block.id.get(),
                    &Message::Answer { ready, round }.encode(),
                );
                (if ready {
                    &net.vars.all_raced
                } else {
                    &net.vars.race_over
                })
                .set(1);
            }
            Message::Answer { round, .. } => {
                let this_round = messages::this_round(2, || (round, net.vars.receiver.sync.get()));
                if !messages::notes_answer(this_round, own_state) {
                    continue;
                }
                net.vars.set_synced(player as usize, true)?;
            }
            Message::Race { sequence, payload } => {
                let time = crate::system::tick_count::tick_count(clock);
                let mut inboxes = net.lock_received();
                store_received(
                    sequence,
                    inbox_of(&mut inboxes, player)?,
                    time,
                    &payload.encode(),
                )?;
            }
            Message::Malformed(_) | Message::Unknown(_) => {}
        }
    }
    Ok(())
}
