//! The best times' tables Rust owns as typed values: taken by `saves::allocate_best_times` on
//! the best times' list, read from and written to `var\hst.dat` in the file's bytes
//! (`saves::load_best_times`, `saves::save_best_times`), given back with the list
//! (`saves::free_best_times`).

use crate::errors::{BadInput, Refused};
use crate::memory::owned::Owned;
use rc2k_core::saves::entries::{LevelScores, StageTimes};
use rc2k_core::saves::rankings::{ChampHistory, RankedTimes, Summary};
use rc2k_core::saves::scores::BestScores;
use rc2k_core::saves::tables::FileTable;

/// The nine typed tables; `typed` numbers them in the file's order.
pub struct BestTables {
    /// The summary the race takes its rivals from (table 0, `ChampView.summary`).
    pub summary: Owned<Summary>,
    /// The best scores (table 1).
    pub scores: Owned<BestScores>,
    /// The championship's stages' best times (table 2).
    pub stage_times: Owned<StageTimes>,
    /// The arcade levels' best scores (table 3).
    pub level_scores: Owned<LevelScores>,
    /// The courses' best times (table 4: the arcade's, 36 stages).
    pub course_times: Owned<StageTimes>,
    /// The ranked times (table 5, standings table A).
    pub ranked_times: Owned<RankedTimes>,
    /// The championship's history (table 6, standings table B).
    pub champ_history: Owned<ChampHistory>,
    /// The other group's stage times (table 7, 36 stages).
    pub stage_times_alt: Owned<StageTimes>,
    /// The time trial's best times (table 8, `ChampView.time_trial_best_times`).
    pub time_trial_times: Owned<StageTimes>,
}

impl BestTables {
    pub const fn new() -> Self {
        Self {
            summary: Owned::new("no standings summary"),
            scores: Owned::new("no best-score table"),
            stage_times: Owned::new("no stage best times table"),
            level_scores: Owned::new("no level best scores table"),
            course_times: Owned::new("no course best times table"),
            ranked_times: Owned::new("no standings table"),
            champ_history: Owned::new("no championship history"),
            stage_times_alt: Owned::new("no other stage times table"),
            time_trial_times: Owned::new("no time trial times table"),
        }
    }

    /// Table `n` (below `TABLES`), in the order the file holds them.
    pub fn typed(&self, n: usize) -> &dyn Typed {
        let tables: [&dyn Typed; rc2k_core::saves::tables::TABLES] = [
            &self.summary,
            &self.scores,
            &self.stage_times,
            &self.level_scores,
            &self.course_times,
            &self.ranked_times,
            &self.champ_history,
            &self.stage_times_alt,
            &self.time_trial_times,
        ];
        tables[n]
    }

    /// Every table given back.
    pub fn give_back_all(&self) {
        for n in 0..rc2k_core::saves::tables::TABLES {
            self.typed(n).give_back();
        }
    }
}

/// What allocating, loading, saving and freeing do with a typed table.
pub trait Typed: Sync {
    /// Taken new (zeros), `bytes` counted on `list`.
    fn take(&self, list: &crate::memory::ListCells, bytes: u32) -> Result<(), Refused>;
    fn give_back(&self);
    /// Its bytes as the file holds them.
    fn bytes(&self) -> Result<Vec<u8>, BadInput>;
    /// Made from the file's bytes (as long as `bytes()`).
    fn set_bytes(&self, b: &[u8]) -> Result<(), BadInput>;
}

impl<T: FileTable + Send> Typed for Owned<T> {
    fn take(&self, list: &crate::memory::ListCells, bytes: u32) -> Result<(), Refused> {
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
