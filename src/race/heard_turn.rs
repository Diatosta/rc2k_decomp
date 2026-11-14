//! The turn the race's sounds are placed by: the first view's (`view_turn_matrix` as it leaves car
//! 0's view), kept after that view is drawn.
//!
//! Sounds (`physics::car_sounds`, `physics::place_car_sound`, `physics::car_sound_state`) used to
//! be turned by whatever view was drawn last, the mirror's when there was one, so with the mirror
//! on they were placed by a backwards camera while the listener faced forwards. Until the first
//! view of a stage is drawn, the current matrix is used.

use crate::race::matrices_store::SceneMatrices;
use crate::session::{GameThread, PlainFlag, PlainFloat};

/// The kept turn, written by `keep` after car 0's view and read by the physics that places the
/// sounds.
static TURN: GameThread<[PlainFloat; 16]> = GameThread::new([const { PlainFloat::new(0.0) }; 16]); // m6-root

/// `TURN` holds the first view's turn of this stage.
static KEPT: GameThread<PlainFlag> = GameThread::new(PlainFlag::new(false)); // m6-root

/// After car 0's view: keeps its turn (`view`, its own) for the sounds.
pub fn keep(view: &crate::scene::view_matrices::ViewMatrices) {
    for (kept, now) in TURN.get().iter().zip(&view.view_turn_matrix) {
        kept.set(now.get());
    }
    KEPT.get().set(true);
}

/// A stage loading: no view drawn yet.
pub fn forget() {
    KEPT.get().set(false);
}

/// The turn the race's sounds are placed by.
pub fn turn(matrices: &SceneMatrices) -> &[PlainFloat; 16] {
    if KEPT.get().is_set() {
        TURN.get()
    } else {
        &matrices.view_turn_matrix
    }
}
