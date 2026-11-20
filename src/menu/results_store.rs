//! The players' results: the rounds of a multi-player event, the order of the places, and each
//! player's place, points and totals. One static, owned by the menus.

use crate::errors::BadInput;
use crate::memory::session::PlainDword;

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

/// One player's figures in their cells.
#[derive(rc2k_derive::Cells)]
pub struct PlayerFigures {
    pub position: PlainDword,
    pub time_bonus: PlainDword,
    pub points: PlainDword,
    pub extra: PlainDword,
    pub sum: PlainDword,
    pub before: PlainDword,
    pub total: PlainDword,
}

impl PlayerFigures {
    const fn new() -> Self {
        PlayerFigures {
            position: PlainDword::new(0),
            time_bonus: PlainDword::new(0),
            points: PlainDword::new(0),
            extra: PlainDword::new(0),
            sum: PlainDword::new(0),
            before: PlainDword::new(0),
            total: PlainDword::new(0),
        }
    }

    /// The figures as values.
    pub fn get(&self) -> Figures {
        Figures {
            position: self.position.get(),
            time_bonus: self.time_bonus.get(),
            points: self.points.get(),
            extra: self.extra.get(),
            sum: self.sum.get(),
            before: self.before.get(),
            total: self.total.get(),
        }
    }

    /// Every figure 0 (the start of an event).
    pub fn clear(&self) {
        self.set(Figures::default());
    }

    /// Every figure from `figures`.
    pub fn set(&self, figures: Figures) {
        self.position.set(figures.position);
        self.time_bonus.set(figures.time_bonus);
        self.points.set(figures.points);
        self.extra.set(figures.extra);
        self.sum.set(figures.sum);
        self.before.set(figures.before);
        self.total.set(figures.total);
    }
}

/// The players' results.
#[derive(rc2k_derive::Cells)]
pub struct Results {
    /// The rounds of a multi-player event.
    pub round: PlainDword,
    /// The players in the order of their places, as the byte offsets of their records in the
    /// player table (0, 4, 8, 12); the first two are the duel's order.
    pub order: [PlainDword; 4],
    /// The four players' figures.
    pub players: [PlayerFigures; 4],
}

impl Results {
    pub const fn new() -> Self {
        Results {
            round: PlainDword::new(0),
            order: [const { PlainDword::new(0) }; 4],
            players: [const { PlayerFigures::new() }; 4],
        }
    }

    /// Player `k`'s figures, refused past the four.
    pub fn player(&self, k: usize) -> Result<&PlayerFigures, BadInput> {
        self.players
            .get(k)
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
        self.round.set(0);
        for player in &self.players {
            player.clear();
        }
    }
}
