//! The best times' tables Rust owns as typed values (phase 12; they were blocks of the
//! game's heap, then `memory::blocks`): taken by `saves::allocate_best_times` on the best
//! times' list, read from and written to `var\hst.dat` in the original's bytes
//! (`saves::load_best_times`, `saves::save_best_times`), given back with the list
//! (`saves::free_best_times`).

use crate::memory::owned::Owned;
use rc2k_core::saves::scores::BestScores;

/// The best scores (table 1, `g_bestScores`).
pub static SCORES: Owned<BestScores> = Owned::new("no best-score table");
