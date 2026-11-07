//! `memory::free_list` (0x4028F5): gives back every block of a list.
//!
//! Takes: the list (`MemoryList`), or 0
//! Nothing happens without a list. The bytes held now are counted down by the list's total
//! (also when it has no block). Then, if it has one: the list is emptied (head, total, and
//! through `sub_402941` the cursor and what remains, all 0) and each block is given back
//! with `VirtualFree`, or, with the setting `altmemenable`, to the game's own heap
//! (`sub_402A92`). No block is put on a list any more (the own heap and the system's
//! blocks removed 2026-11-07): a list holding one is refused.
//! 38 callers.

use crate::errors::BadInput;
use crate::memory::{MemoryList, with_figures};
use crate::pointers::{load_value, store_value};

// ---- Rust version ----

/// The list's blocks given back, the list a store (or a field of one).
pub fn free_list(list: &dyn crate::pointers::Store) -> Result<(), BadInput> {
    free_with(
        crate::pointers::game_address(list.first()),
        || load_value::<MemoryList>(list, 0),
        |l| store_value(list, 0, l),
    )
}

/// `free_list` for a list that is a computed place (a record by offset); none given: nothing
/// happens.
pub fn free_list_at(list: crate::pointers::GamePtr) -> Result<(), BadInput> {
    if list.is_null() {
        return Ok(());
    }
    // Stores type a list by its first dword: the record runs on past that field.
    let at = list.unbounded();
    free_with(
        at.addr(),
        || at.read::<MemoryList>("a memory list"),
        |l| at.write(l, "a memory list"),
    )
}

/// The work of `free_list`; `key` names the list for what `gfx::glyph_store` holds on it.
fn free_with(
    key: u32,
    read: impl Fn() -> Result<MemoryList, BadInput>,
    write: impl Fn(MemoryList) -> Result<(), BadInput>,
) -> Result<(), BadInput> {
    // Rust's own tables counted on it (`memory::owned`) are given back by their owners
    // (`saves::free_best_times`); its total counts them like the others.
    // The palettes and glyph tables carved off it (`gfx::glyph_store`) go with it.
    crate::gfx::glyph_store::give_back(key);
    let mut l = read()?;
    // No block of the system's is put on a list any more (every user's memory is Rust's,
    // phase 12; the original gave each back with `VirtualFree`, or to its own heap): a list
    // holding one is refused.
    if l.head != 0 {
        return Err(BadInput("a memory list holding a block"));
    }
    // Converted: `rc2k_core::memory` (`Figures::given_back`, `List::emptied`).
    with_figures(|f| f.given_back(l.total));
    // Only Rust's memory counted on it (`count_owned`): its total emptied too.
    if l.total != 0 {
        l.emptied();
        write(l)?;
    }
    Ok(())
}
