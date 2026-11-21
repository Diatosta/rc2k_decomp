//! The turn the race's sounds are placed by: the first view's (`view_turn_matrix` as it leaves car
//! 0's view), kept after that view is drawn, so the sounds are not turned by the mirror's backwards
//! camera. Until the first view of a stage is drawn, the current matrix is used.

use crate::memory::session::PlainFlag;
use crate::scene::matrix::Matrix;

/// The kept turn of the first view of this stage, held by `RaceShared`.
pub struct HeardTurn {
    /// The kept turn, written by `keep` after car 0's view and read by the physics that places
    /// the sounds.
    turn: std::cell::Cell<Matrix>,
    /// `turn` holds the first view's turn of this stage.
    kept: PlainFlag,
}

impl HeardTurn {
    pub const fn new() -> Self {
        HeardTurn {
            turn: std::cell::Cell::new([0.0; 16]),
            kept: PlainFlag::new(false),
        }
    }

    /// After car 0's view: keeps its turn (`view`, its own) for the sounds.
    pub fn keep(&self, view: &crate::scene::view_matrices::ViewMatrices) {
        self.turn.set(view.borrow().view_turn_matrix);
        self.kept.set(true);
    }

    /// A stage loading: no view drawn yet.
    pub fn forget(&self) {
        self.kept.set(false);
    }

    /// The turn the race's sounds are placed by.
    pub fn turn(&self, current: &Matrix) -> Matrix {
        if self.kept.is_set() {
            self.turn.get()
        } else {
            *current
        }
    }
}
