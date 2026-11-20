//! The best lists of ten: the tests of a new figure, the new entry's parts, and the records' best
//! holders. The lists themselves (entering, ranking, clearing the marks) work on the typed
//! entries, `saves::entries`.

use super::standings::NAME_LEN;

/// The 0x40 bytes a best time keeps (its checkpoints; the player's `best_line`).
pub const LINE_LEN: usize = 0x40;

/// Whether a time makes a list whose tenth is `tenth`: at most it (signed).
pub fn makes_time(time: u32, tenth: u32) -> bool {
    time as i32 <= tenth as i32
}

/// Whether a score makes a list whose tenth is `tenth`: at least it (signed).
pub fn makes_score(score: u32, tenth: u32) -> bool {
    score as i32 >= tenth as i32
}

/// The seal a best time keeps: the cheats and the sealed time.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Seal {
    pub cheats: u32,
    pub sealed: [u32; 2],
}

/// A new entry.
pub struct NewBest {
    /// The time or score.
    pub key: u32,
    /// The player: 1, or the player's index + 1 for the records.
    pub player: u32,
    /// The player's nationality, name, car number and group (the class).
    pub nationality: u32,
    pub name_id: crate::saves::name_id::NameId,
    pub car: u32,
    pub group: u32,
    /// The name's first bytes.
    pub name: [u8; NAME_LEN],
    /// The records' taking part (the player's `entry`).
    pub entered: Option<u32>,
    /// The best times' seal.
    pub seal: Option<Seal>,
    /// The best times' line.
    pub line: Option<[u8; LINE_LEN]>,
}

/// Whether a figure takes a record whose own is `kept`: above it (signed).
pub fn takes_record(figure: u32, kept: u32) -> bool {
    (kept as i32) < figure as i32
}

/// The record entries' pass (`menu::record_entries_pass`) on the game's variables.
pub trait RecordEntries {
    type Error;
    /// `menu::clear_record_entries`.
    fn clear(&mut self) -> Result<(), Self::Error>;
    /// One was entered.
    fn set_entered(&mut self, on: bool);
    /// The current player.
    fn set_current(&mut self, n: u32);
    /// `menu::beat_target_time` for player n: `EventState.trial.qualifies` 1 after it.
    fn beats(&mut self, n: u32) -> Result<bool, Self::Error>;
    /// `menu::enter_record_entry` for player n.
    fn enter(&mut self, n: u32) -> Result<(), Self::Error>;
}

/// The current event's entries cleared, then each of the `players` counted players that
/// beat the target time entered, `EventState.trial.any_qualified` 1 once one is (0 first).
pub fn record_entries<R: RecordEntries>(players: u32, e: &mut R) -> Result<(), R::Error> {
    e.clear()?;
    e.set_entered(false);
    for n in 0..players {
        e.set_current(n);
        if e.beats(n)? {
            e.set_entered(true);
            e.enter(n)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tests() {
        assert!(makes_time(5, 5) && !makes_time(6, 5) && makes_time(u32::MAX, 0));
        assert!(makes_score(5, 5) && !makes_score(4, 5) && !makes_score(u32::MAX, 0));
        assert!(takes_record(6, 5) && !takes_record(5, 5) && takes_record(0, u32::MAX));
    }

    #[test]
    fn record_entries_entered() {
        struct Log(Vec<String>);
        impl RecordEntries for Log {
            type Error = ();
            fn clear(&mut self) -> Result<(), ()> {
                self.0.push("clear".into());
                Ok(())
            }
            fn set_entered(&mut self, on: bool) {
                self.0.push(format!("entered {}", u32::from(on)));
            }
            fn set_current(&mut self, n: u32) {
                self.0.push(format!("current {n}"));
            }
            fn beats(&mut self, n: u32) -> Result<bool, ()> {
                Ok(n == 1)
            }
            fn enter(&mut self, n: u32) -> Result<(), ()> {
                self.0.push(format!("enter {n}"));
                Ok(())
            }
        }
        let mut log = Log(Vec::new());
        record_entries(3, &mut log).unwrap();
        assert_eq!(
            log.0,
            [
                "clear",
                "entered 0",
                "current 0",
                "current 1",
                "entered 1",
                "enter 1",
                "current 2"
            ]
        );
    }
}
