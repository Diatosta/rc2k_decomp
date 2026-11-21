//! The players' results: the rounds of a multi-player event, the order of the places, and each
//! player's place, points and totals. Plain fields behind one `RefCell` (the game's thread alone
//! reads and writes them, each borrow lasting one statement); owned by the menus.

use crate::errors::BadInput;
use crate::memory::session::GameThread;
use std::cell::{Ref, RefCell, RefMut};

/// The title string ids of the two stage groups.
const GROUP_TITLES: [u32; 2] = [0x340, 0x341];

/// One player's figures, as plain values.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Figures {
    /// The place, 1 .. 4.
    pub position: u32,
    /// The stage record's time less the player's, 0 when greater.
    pub time_bonus: u32,
    /// The points of the place.
    pub points: u32,
    /// The arcade's extra points (the player's top speed).
    pub extra: u32,
    /// The sum of points, bonus and extra.
    pub sum: u32,
    /// The total before the stage.
    pub before: u32,
    /// The total.
    pub total: u32,
}

impl Figures {
    /// The six numbers of a results column: points, bonus, extra, sum, total before, total.
    pub fn column(&self) -> [u32; 6] {
        [
            self.points,
            self.time_bonus,
            self.extra,
            self.sum,
            self.before,
            self.total,
        ]
    }
}

/// The results' fields.
pub struct ResultsData {
    /// The rounds of a multi-player event.
    pub round: u32,
    /// The players in the order of their places, as the byte offsets of their records in the
    /// player table (0, 4, 8, 12); the first two are the duel's order.
    pub order: [u32; 4],
    /// The four players' figures.
    pub players: [Figures; 4],
}

/// The players' results.
pub struct Results(GameThread<RefCell<ResultsData>>);

impl Results {
    pub const fn new() -> Self {
        Results(GameThread::new(RefCell::new(ResultsData {
            round: 0,
            order: [0; 4],
            players: [Figures {
                position: 0,
                time_bonus: 0,
                points: 0,
                extra: 0,
                sum: 0,
                before: 0,
                total: 0,
            }; 4],
        })))
    }

    /// The fields, for reading (one statement).
    pub fn read(&self) -> Ref<'_, ResultsData> {
        self.0.get().borrow()
    }

    /// The fields, for writing (one statement).
    pub fn write(&self) -> RefMut<'_, ResultsData> {
        self.0.get().borrow_mut()
    }

    /// Player `k`'s figures, refused past the four.
    pub fn player(&self, k: usize) -> Result<Figures, BadInput> {
        self.read()
            .players
            .get(k)
            .copied()
            .ok_or(BadInput("a results figure past the four players"))
    }

    /// The title string id of stage group `group`.
    pub fn group_title(group: u32) -> Result<u32, BadInput> {
        GROUP_TITLES
            .get(group as usize)
            .copied()
            .ok_or(BadInput("a stage group past its title"))
    }

    /// The figures an event's start clears: every player's, and the round (the order is kept).
    pub(super) fn clear_for_event(&self) {
        let mut results = self.write();
        results.round = 0;
        results.players = [Figures::default(); 4];
    }
}
