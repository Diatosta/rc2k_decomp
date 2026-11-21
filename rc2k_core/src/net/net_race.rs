//! What the race code knows of a network game: that one is on, this machine's car and the number
//! of players. The network owner holds it (`NetRaceSlot`) and sets it when a link opens and
//! clears it when the link ends; the race code reads the slot it is handed (or the `NetRace` value).

use std::sync::atomic::{AtomicBool, AtomicU32, Ordering::Relaxed};

/// A network game that is on.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NetRace {
    /// This machine's car number (0 .. 7 once the players are listed).
    pub player: u32,
    /// The players' count (at most 8).
    pub players: u32,
    /// The computer drivers' cars after the players' (`net::bots`).
    pub bots: u32,
}

/// The network game's state shared between the game thread, the drawing threads and the
/// receiving thread: relaxed atomics, each field alone (no field depends on another).
pub struct NetRaceSlot {
    on: AtomicBool,
    player: AtomicU32,
    players: AtomicU32,
    bots: AtomicU32,
}

impl NetRaceSlot {
    pub const fn new() -> Self {
        NetRaceSlot {
            on: AtomicBool::new(false),
            player: AtomicU32::new(0),
            players: AtomicU32::new(0),
            bots: AtomicU32::new(0),
        }
    }

    /// The network game, when one is on.
    pub fn get(&self) -> Option<NetRace> {
        self.on.load(Relaxed).then(|| NetRace {
            player: self.player(),
            players: self.players(),
            bots: self.bots(),
        })
    }

    /// Whether a network game is on.
    pub fn is_on(&self) -> bool {
        self.on.load(Relaxed)
    }

    /// A link has opened.
    pub fn begin(&self) {
        self.on.store(true, Relaxed);
    }

    /// The link has ended. The counts stay: the lobby lists them again.
    pub fn end(&self) {
        self.on.store(false, Relaxed);
    }

    /// This machine's car number.
    pub fn player(&self) -> u32 {
        self.player.load(Relaxed)
    }

    pub fn set_player(&self, player: u32) {
        self.player.store(player, Relaxed);
    }

    /// The players' count.
    pub fn players(&self) -> u32 {
        self.players.load(Relaxed)
    }

    pub fn set_players(&self, players: u32) {
        self.players.store(players, Relaxed);
    }

    /// The computer drivers' count: their cars follow the players' (0 outside a race).
    pub fn bots(&self) -> u32 {
        self.bots.load(Relaxed)
    }

    pub fn set_bots(&self, bots: u32) {
        self.bots.store(bots, Relaxed);
    }

    /// The race's cars: the players' and the bots'.
    pub fn cars(&self) -> u32 {
        self.players() + self.bots()
    }

    /// Whether car `car` is a computer driver's.
    pub fn is_bot(&self, car: u32) -> bool {
        self.is_on() && crate::net::bots::is_bot(car, self.players(), self.bots())
    }
}

impl Default for NetRaceSlot {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_game_is_present_between_begin_and_end() {
        let slot = NetRaceSlot::new();
        assert_eq!(slot.get(), None);
        slot.set_players(3);
        slot.set_player(2);
        slot.set_bots(2);
        assert_eq!(slot.get(), None);
        assert!(!slot.is_bot(3));
        slot.begin();
        assert_eq!(
            slot.get(),
            Some(NetRace {
                player: 2,
                players: 3,
                bots: 2
            })
        );
        // The bots' cars follow the players'.
        assert_eq!(slot.cars(), 5);
        assert!(!slot.is_bot(2) && slot.is_bot(3) && slot.is_bot(4) && !slot.is_bot(5));
        slot.end();
        assert_eq!(slot.get(), None);
        assert_eq!((slot.player(), slot.players()), (2, 3));
    }
}
