//! The turn the race's sounds are placed by: the first view's (`view_turn_matrix` as it leaves car
//! 0's view), kept after that view is drawn, so the sounds are not turned by the mirror's backwards
//! camera. Until the first view of a stage is drawn, the current matrix is used.

use crate::memory::session::{PlainFlag, PlainFloat};
use crate::race::matrices_store::ViewKept;

/// The kept turn of the first view of this stage, held by `RaceShared`.
pub struct HeardTurn {
    /// The kept turn, written by `keep` after car 0's view and read by the physics that places
    /// the sounds.
    turn: [PlainFloat; 16],
    /// `turn` holds the first view's turn of this stage.
    kept: PlainFlag,
}

impl HeardTurn {
    pub const fn new() -> Self {
        HeardTurn {
            turn: [const { PlainFloat::new(0.0) }; 16],
            kept: PlainFlag::new(false),
        }
    }

    /// After car 0's view: keeps its turn (`view`, its own) for the sounds.
    pub fn keep(&self, view: &crate::scene::view_matrices::ViewMatrices) {
        for (kept, now) in self.turn.iter().zip(&view.view_turn_matrix) {
            kept.set(now.get());
        }
        self.kept.set(true);
    }

    /// A stage loading: no view drawn yet.
    pub fn forget(&self) {
        self.kept.set(false);
    }

    /// The turn the race's sounds are placed by.
    pub fn turn<'a>(&'a self, matrices: &'a ViewKept) -> &'a [PlainFloat; 16] {
        if self.kept.is_set() {
            &self.turn
        } else {
            &matrices.view_turn_matrix
        }
    }
}
