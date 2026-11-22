//! How far through the stage each racing player is, in percent: the racing machine tells the
//! others as its car passes checkpoints (and now and then again, in case a message was lost) and
//! the others note it by the sender's DirectPlay id; the lobby shows it in the stage time's place
//! for a player still racing. Cleared when the lobby comes up, so one race's figures never show
//! in the next. A copy that does not know the message drops it.

use crate::net::owner::Net;
use crate::race::owner::Race;
use rc2k_core::net::message::Message;
use rc2k_core::net::players::stage_share;
use std::sync::Mutex;
use std::sync::atomic::{AtomicU32, Ordering};

/// The shares heard of, by player id.
static SHARES: Mutex<Vec<(u32, u8)>> = Mutex::new(Vec::new());

/// What this machine last sent (`NONE`: nothing in this race).
static SENT: AtomicU32 = AtomicU32::new(NONE);
const NONE: u32 = u32::MAX;

/// The car's steps between two sends of an unchanged share.
const REPEAT_MASK: u32 = 0x7F;

/// Forgets every player's share.
pub fn clear() {
    SHARES
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .clear();
    SENT.store(NONE, Ordering::SeqCst);
}

/// Notes the share of the player with DirectPlay id `id` (held to 100).
pub fn note(id: u32, share: u8) {
    let mut shares = SHARES
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let share = share.min(100);
    match shares.iter_mut().find(|(known, _)| *known == id) {
        Some(entry) => entry.1 = share,
        None => shares.push((id, share)),
    }
}

/// The share heard of the player with DirectPlay id `id`.
pub(super) fn share_of(id: u32) -> Option<u8> {
    let shares = SHARES
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    shares.iter().find(|(known, _)| *known == id).map(|e| e.1)
}

/// The text a racing player's share is shown as, ended by no NUL.
pub fn text(share: u8) -> Vec<u8> {
    let mut text = crate::strings::format_u32::format_u32(u32::from(share), 0);
    text.push(b'%');
    text
}

/// The own car's step: its share is sent when it has changed since the last, and again every
/// 128th step.
pub fn send(race: &Race, net: &Net) {
    let car = race.shared.car();
    let share = stage_share(
        car.checkpoint,
        race.shared.vars.get().checkpoint_count.get(),
    );
    let changed = SENT.swap(u32::from(share), Ordering::SeqCst) != u32::from(share);
    if changed || car.step_clock & REPEAT_MASK == 3 {
        crate::net::dplay_send::dplay_send(
            &net.link,
            net.link.send_to(),
            &Message::Progress { share }.encode(),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_share_is_kept_by_id_and_held_to_100() {
        let _lock = crate::test_support::test_lock();
        clear();
        assert_eq!(share_of(7), None);
        note(7, 40);
        note(9, 250);
        note(7, 41);
        assert_eq!(share_of(7), Some(41));
        assert_eq!(share_of(9), Some(100));
        clear();
        assert_eq!(share_of(7), None);
    }

    #[test]
    fn a_share_shows_as_a_percentage() {
        assert_eq!(text(0), b"0%".to_vec());
        assert_eq!(text(57), b"57%".to_vec());
        assert_eq!(text(100), b"100%".to_vec());
    }
}
