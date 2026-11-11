//! The turn the race's sounds are placed by: the first view's (`g_viewTurnMatrix` as it
//! leaves car 0's view), kept after that view is drawn.
//!
//! Not the original's: its sounds (`physics::car_sounds`, `physics::place_car_sound`,
//! `physics::car_sound_state`) were turned by whatever view the drawing left last, the
//! mirror's when there was one, so with the mirror on the race's sounds were placed by a
//! backwards camera while the listener faced forwards (`notes/bugs.md`). Until the first view
//! of a stage is drawn, the matrix as it is (what the original used then too).

use crate::race::matrices_store::SceneMatrices;
use crate::session::Float;
use std::sync::atomic::{AtomicBool, Ordering};

static TURN: [Float; 16] = [const { Float::new(0.0) }; 16];

/// `TURN` holds the first view's turn of this stage.
static KEPT: AtomicBool = AtomicBool::new(false);

/// After car 0's view: its turn (`view`, its own) kept for the sounds.
pub fn keep(view: &crate::scene::view_matrices::ViewMatrices) {
    for (kept, now) in TURN.iter().zip(&view.view_turn_matrix) {
        kept.set_bits(now.bits());
    }
    KEPT.store(true, Ordering::Relaxed);
}

/// A stage loading: no view drawn yet.
pub fn forget() {
    KEPT.store(false, Ordering::Relaxed);
}

/// The turn the race's sounds are placed by.
pub fn turn(matrices: &SceneMatrices) -> &[Float; 16] {
    if KEPT.load(Ordering::Relaxed) {
        &TURN
    } else {
        &matrices.view_turn_matrix
    }
}
