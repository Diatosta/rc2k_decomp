//! The best times' tables Rust owns as typed values (they were blocks of the
//! game's heap): taken by `saves::allocate_best_times` on the best
//! times' list, read from and written to `var\hst.dat` in the original's bytes
//! (`saves::load_best_times`, `saves::save_best_times`), given back with the list
//! (`saves::free_best_times`).

use crate::errors::{BadInput, Refused};
use crate::memory::owned::Owned;
use rc2k_core::saves::entries::{LevelScores, StageTimes};
use rc2k_core::saves::rankings::{ChampHistory, RankedTimes, Summary};
use rc2k_core::saves::scores::BestScores;
use rc2k_core::saves::tables::FileTable;

/// The best scores (table 1, `g_bestScores`).
pub static SCORES: Owned<BestScores> = Owned::new("no best-score table");
/// The arcade levels' best scores (table 3, `g_levelBestScores`).
pub static LEVEL_SCORES: Owned<LevelScores> = Owned::new("no level best scores table");

/// The championship's stages' best times (table 2, `g_stageBestTimes`).
pub static STAGE_TIMES: Owned<StageTimes> = Owned::new("no stage best times table");
/// The courses' best times (table 4, `g_courseBestTimes`: the arcade's, 36 stages).
pub static COURSE_TIMES: Owned<StageTimes> = Owned::new("no course best times table");
/// The other group's stage times (table 7, `g_stageBestTimesAlt`, 36 stages).
pub static STAGE_TIMES_ALT: Owned<StageTimes> = Owned::new("no other stage times table");
/// The time trial's best times (table 8, `ChampView.time_trial_best_times`).
pub static TIME_TRIAL_TIMES: Owned<StageTimes> = Owned::new("no time trial times table");

/// The summary the race takes its rivals from (table 0, `ChampView.summary`).
pub static SUMMARY: Owned<Summary> = Owned::new("no standings summary");
/// The ranked times (table 5, `g_rankedTimes`, standings table A).
pub static RANKED_TIMES: Owned<RankedTimes> = Owned::new("no standings table");
/// The championship's history (table 6, `g_champHistory`, standings table B).
pub static CHAMP_HISTORY: Owned<ChampHistory> = Owned::new("no championship history");

/// What allocating, loading, saving and freeing do with a typed table.
pub trait Typed: Sync {
    /// Taken new (zeros), `bytes` counted on `list`.
    fn take(&self, list: &dyn crate::pointers::Store, bytes: u32) -> Result<(), Refused>;
    fn give_back(&self);
    /// Its bytes as the file holds them.
    fn bytes(&self) -> Result<Vec<u8>, BadInput>;
    /// Made from the file's bytes (as long as `bytes()`).
    fn set_bytes(&self, b: &[u8]) -> Result<(), BadInput>;
}

impl<T: FileTable + Send> Typed for Owned<T> {
    fn take(&self, list: &dyn crate::pointers::Store, bytes: u32) -> Result<(), Refused> {
        let Some(t) = T::zeroed(bytes as usize) else {
            return Err(BadInput("a typed table asked for at a size it cannot be").into());
        };
        Owned::take(self, list, bytes, t)
    }

    fn give_back(&self) {
        Owned::give_back(self);
    }

    fn bytes(&self) -> Result<Vec<u8>, BadInput> {
        self.with(|t| t.write())
    }

    fn set_bytes(&self, b: &[u8]) -> Result<(), BadInput> {
        self.with(|t| *t = T::read(b))
    }
}

/// `allocate_best_times`' nine tables, in its order.
static TYPED: [&dyn Typed; rc2k_core::saves::tables::TABLES] = [
    &SUMMARY,
    &SCORES,
    &STAGE_TIMES,
    &LEVEL_SCORES,
    &COURSE_TIMES,
    &RANKED_TIMES,
    &CHAMP_HISTORY,
    &STAGE_TIMES_ALT,
    &TIME_TRIAL_TIMES,
];

/// Table `n` (below `TABLES`) of `allocate_best_times`' nine.
pub fn typed(n: usize) -> &'static dyn Typed {
    TYPED[n]
}

/// Every table given back.
pub fn give_back_all() {
    TYPED.iter().for_each(|t| t.give_back());
}
