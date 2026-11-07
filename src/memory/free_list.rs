//! `memory::free_list` (0x4028F5): gives back every block of a list.
//!
//! In:  ebx = the list (`MemoryList`), or 0
//! Out: every register preserved; CF clear (read by `sub_4D06F3`'s `jb`)
//! Nothing happens without a list. The bytes held now are counted down by the list's total
//! (also when it has no block). Then, if it has one: the list is emptied (head, total, and
//! through `sub_402941` the cursor and what remains, all 0) and each block is given back
//! with `VirtualFree`, or, with the setting `altmemenable`, to the game's own heap
//! (`sub_402A92`, not ported).
//! 38 callers. Shadow mode runs the original alone and checks that it left the list empty.
//! Manual patch with a private copy (0x4C bytes, three calls relocated): a conditional
//! jump sits in the first bytes.
//! Refuses the game's own heap when the list has a block.
//!
//! Hook removed 2026-10-22 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::BadInput;
use crate::memory::{MemoryList, own_heap, with_figures};
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
    let block = l.head;
    if block != 0 && own_heap() == 1 {
        return Err(BadInput("the game's own heap is in use (altmemenable)"));
    }
    // Converted: `rc2k_core::memory` (`Figures::given_back`, `List::emptied`).
    with_figures(|f| f.given_back(l.total));
    if block == 0 {
        // Only Rust's memory counted on it (`count_owned`): its total emptied too.
        if l.total != 0 {
            l.emptied();
            write(l)?;
        }
        return Ok(());
    }
    let mut block = l.emptied();
    write(l)?;
    while block != 0 {
        // The block is the system's, a block id; its first dword the next one's id.
        let before = crate::memory::pages::block(block)?.read::<u32>("a block's header")?;
        crate::memory::pages::give_back(block)?;
        block = before;
    }
    Ok(())
}
