//! The best times' tables Rust owns as typed values (phase 12; they were blocks of the
//! game's heap, then `memory::blocks`): taken by `saves::allocate_best_times` on the best
//! times' list, read from and written to `var\hst.dat` in the original's bytes
//! (`saves::load_best_times`, `saves::save_best_times`), given back with the list
//! (`saves::free_best_times`).

use crate::errors::{BadInput, Refused};
use crate::memory::MemoryList;
use crate::memory::owned::Owned;
use rc2k_core::saves::entries::LevelScores;
use rc2k_core::saves::scores::BestScores;
use rc2k_core::saves::tables::FileTable;

/// The best scores (table 1, `g_bestScores`).
pub static SCORES: Owned<BestScores> = Owned::new("no best-score table");
/// The arcade levels' best scores (table 3, `g_levelBestScores`).
pub static LEVEL_SCORES: Owned<LevelScores> = Owned::new("no level best scores table");

/// What allocating, loading, saving and freeing do with a typed table.
pub trait Typed: Sync {
    /// Taken new (zeros), `bytes` counted on `list`.
    fn take(&self, list: *mut MemoryList, bytes: u32) -> Result<(), Refused>;
    fn give_back(&self);
    /// Its bytes as the file holds them.
    fn bytes(&self) -> Result<Vec<u8>, BadInput>;
    /// Made from the file's bytes (as long as `bytes()`).
    fn set_bytes(&self, b: &[u8]) -> Result<(), BadInput>;
}

impl<T: FileTable + Send> Typed for Owned<T> {
    fn take(&self, list: *mut MemoryList, bytes: u32) -> Result<(), Refused> {
        if bytes as usize != T::LEN {
            return Err(BadInput("a typed table asked for at another size").into());
        }
        Owned::take(self, list, bytes, T::default())
    }

    fn give_back(&self) {
        Owned::give_back(self)
    }

    fn bytes(&self) -> Result<Vec<u8>, BadInput> {
        self.with(|t| t.write())
    }

    fn set_bytes(&self, b: &[u8]) -> Result<(), BadInput> {
        self.with(|t| *t = T::read(b))
    }
}

/// Table `n` of `allocate_best_times`' nine when it is typed.
pub fn typed(n: usize) -> Option<&'static dyn Typed> {
    match n {
        1 => Some(&SCORES),
        3 => Some(&LEVEL_SCORES),
        _ => None,
    }
}

/// Every typed table given back.
pub fn give_back_all() {
    for n in 0..rc2k_core::saves::tables::TABLES {
        if let Some(t) = typed(n) {
            t.give_back();
        }
    }
}
