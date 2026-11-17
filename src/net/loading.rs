//! How far each player's stage has loaded: this machine tells the others as its load goes
//! (`Message::Loading`, a percentage) and notes theirs as they arrive; the loading screen lists
//! them. Cleared when the lobby comes up, so a race's figures never show in the next one.

use crate::errors::Refused;
use crate::net::owner::Net;
use rc2k_core::net::message::Message;
use std::sync::atomic::{AtomicU8, Ordering};

/// A player whose load has not been heard of.
const UNKNOWN: u8 = u8::MAX;

static SHARES: [AtomicU8; rc2k_core::net::players::MOST as usize] =
    [const { AtomicU8::new(UNKNOWN) }; rc2k_core::net::players::MOST as usize];

/// Forgets every player's share.
pub fn clear() {
    for share in &SHARES {
        share.store(UNKNOWN, Ordering::SeqCst);
    }
}

/// Notes player `player`'s share (held to 100; a player past the table is not noted).
pub fn note(player: u32, share: u8) {
    if let Some(slot) = SHARES.get(player as usize) {
        slot.store(share.min(100), Ordering::SeqCst);
    }
}

/// Whether a game with other players is going: what the loading screen is shown for.
pub fn in_game(net: &Net) -> bool {
    net.link.has_dplay() && net.race_slot.players() > 1
}

/// This machine's load is `share` percent done: noted for its own player and sent to the others.
pub fn report(net: &Net, share: u32) {
    if !in_game(net) {
        return;
    }
    let share = share.min(100) as u8;
    note(net.race_slot.player(), share);
    crate::net::dplay_send::dplay_send(
        &net.link,
        net.link.send_to(),
        &Message::Loading { share }.encode(),
    );
}

/// The players that have a share, in the players' order: their names and shares.
pub fn listed(net: &Net) -> Result<Vec<(Vec<u8>, u8)>, Refused> {
    let mut list = Vec::new();
    for n in 0..net.race_slot.players().min(SHARES.len() as u32) {
        let share = SHARES[n as usize].load(Ordering::SeqCst);
        if share != UNKNOWN {
            list.push((net.vars.player_block(n)?.name.text().to_vec(), share));
        }
    }
    Ok(list)
}

/// A line of the loading screen's list: the name, a gap and the percentage, ended by a NUL.
pub fn line(name: &[u8], share: u8) -> Vec<u8> {
    let name = name.split(|&b| b == 0).next().unwrap_or_default();
    let mut text = name.to_vec();
    text.extend_from_slice(b"  ");
    text.extend(crate::strings::format_u32::format_u32(u32::from(share), 0));
    text.extend_from_slice(b"%\0");
    text
}

/// The list drawn in a corner of the screen (the surface locked), one line a player.
pub fn draw(o: crate::owners::Owners<'static>) -> Result<(), Refused> {
    let net = o.net;
    if !in_game(net) {
        return Ok(());
    }
    let font = crate::net::lobby::FONT;
    let height = font.object(&o.gfx.fonts.typefaces).line_height.get();
    let mut y = 0x20;
    for (name, share) in listed(net)? {
        crate::text::draw_text::draw_text_at(
            font,
            &line(&name, share),
            0x20,
            y,
            &mut crate::text::draw_text::GameCanvas::new(o.gfx),
        )?;
        y += height as i32;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_line_is_a_name_and_its_percentage() {
        assert_eq!(line(b"Ana\0junk", 7), b"Ana  7%\0".to_vec());
        assert_eq!(line(b"Rui", 100), b"Rui  100%\0".to_vec());
    }

    #[test]
    fn shares_are_held_to_100_and_a_player_past_the_table_is_passed_over() {
        let _lock = crate::test_support::test_lock();
        clear();
        note(1, 250);
        note(99, 5);
        assert_eq!(SHARES[1].load(Ordering::SeqCst), 100);
        assert_eq!(SHARES[0].load(Ordering::SeqCst), UNKNOWN);
        clear();
    }
}
