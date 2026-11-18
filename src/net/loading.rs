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

/// The list drawn on the loading picture (the surface locked), one line a player: the names in
/// a column and their percentages after it, right-aligned. It lies inside the picture's 4:3
/// middle (a wider screen shows only its sides' wash beside it), at its left, just above the
/// loading bar.
pub fn draw(o: crate::owners::Owners<'static>) -> Result<(), Refused> {
    let net = o.net;
    if !in_game(net) {
        return Ok(());
    }
    let players = listed(net)?;
    if players.is_empty() {
        return Ok(());
    }
    let font = crate::gfx::typefaces::fonts::LIST;
    let height = font.object(&o.gfx.fonts.typefaces).line_height.get() as i32;
    let screen = o.system.session.screen.get();
    let (width, tall) = (screen.width.get(), screen.height.get());
    let (left, mut y) =
        rc2k_core::race::loading::list_origin(width, tall, players.len() as u32, height as u32);
    let mut canvas = crate::text::draw_text::GameCanvas::new(o.gfx);
    let named: Vec<(Vec<u8>, Vec<u8>)> = players
        .iter()
        .map(|(name, share)| {
            let name = name.split(|&b| b == 0).next().unwrap_or_default().to_vec();
            (name, crate::net::progress::text(*share))
        })
        .collect();
    let widest = |texts: &mut dyn Iterator<Item = &Vec<u8>>, canvas: &mut _| {
        texts
            .map(|t| crate::gfx::button_hints::width_of(font, t, canvas))
            .max()
            .unwrap_or(0)
    };
    let names_end = left + widest(&mut named.iter().map(|(n, _)| n), &mut canvas);
    let shares_wide = widest(&mut named.iter().map(|(_, s)| s), &mut canvas);
    let shares_end = names_end + height + shares_wide;
    for (name, share) in &named {
        let mut name = name.clone();
        name.push(0);
        crate::text::draw_text::draw_text_at(font, &name, left, y, &mut canvas)?;
        let mut share = share.clone();
        let wide = crate::gfx::button_hints::width_of(font, &share, &mut canvas);
        share.push(0);
        crate::text::draw_text::draw_text_at(font, &share, shares_end - wide, y, &mut canvas)?;
        y += height;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

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
