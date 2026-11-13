//! The turn the race's sounds are placed by: the first view's (`g_viewTurnMatrix` as it
//! leaves car 0's view), kept after that view is drawn.
//!
//! Not the original's: its sounds (`physics::car_sounds`, `physics::place_car_sound`,
//! `physics::car_sound_state`) were turned by whatever view the drawing left last, the
//! mirror's when there was one, so with the mirror on the race's sounds were placed by a
//! backwards camera while the listener faced forwards (`notes/bugs.md`). Until the first view
//! of a stage is drawn, the matrix as it is (what the original used then too).

use crate::race::matrices_store::SceneMatrices;
use crate::session::{GameThread, PlainFlag, PlainFloat};

/// The kept turn: the game's thread keeps it (after car 0's view, `keep`) and the physics
/// that places the sounds reads it.
static TURN: GameThread<[PlainFloat; 16]> = GameThread::new([const { PlainFloat::new(0.0) }; 16]); // m6-root

/// `TURN` holds the first view's turn of this stage.
static KEPT: GameThread<PlainFlag> = GameThread::new(PlainFlag::new(false)); // m6-root

/// After car 0's view: its turn (`view`, its own) kept for the sounds.
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
