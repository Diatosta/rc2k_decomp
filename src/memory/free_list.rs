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
use windows_sys::Win32::System::Memory::{MEM_RELEASE, VirtualFree};

// ---- Rust version ----

pub unsafe fn free_list(list: *mut MemoryList) -> Result<(), BadInput> {
    unsafe {
        if list.is_null() {
            return Ok(());
        }
        let mut block = (*list).head;
        if block != 0 && own_heap() == 1 {
            return Err(BadInput("the game's own heap is in use (altmemenable)"));
        }
        // Converted: `rc2k_core::memory` (`Figures::given_back`, `List::emptied`).
        with_figures(|f| f.given_back((*list).total));
        if block == 0 {
            return Ok(());
        }
        block = (*list).emptied();
        while block != 0 {
            let before = (block as *const u32).read();
            crate::pointers::unregister(block);
            VirtualFree(block as _, 0, MEM_RELEASE);
            block = before;
        }
        Ok(())
    }
}

// ---- self-test: lists of blocks taken for the purpose, given back by both; the memory
// figures are put back ----
