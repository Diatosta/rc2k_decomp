//! `memory::allocate` (0x4028A7): takes a block of memory and puts it on a list.
//!
//! In:  eax = bytes, ebx = the list (`MemoryList`)
//! Out: eax = the block's memory, zeroed; carry clear; everything else preserved
//! The block is 0x40 bytes larger than asked for: a header in front, which holds the block
//! taken before on the same list and the bytes asked for. It comes from `VirtualAlloc`
//! (committed, read and write; none is fatal error 0x80108571 "Need more virtual memory!"),
//! or, with the setting `altmemenable`, from a heap of the game's own (`sub_402A05`, not
//! ported). The list's total and the three memory figures are counted up.
//! 56 callers. Shadow mode runs the original alone (memory cannot be taken twice) and
//! checks the block and the list it leaves.
//! Refuses the game's own heap, a null list and a size that with the header wraps to 0.
//!
//! Hook removed 2026-10-22 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::{BadInput, FatalError, Refused};
use crate::framework::game_read;
use crate::memory::{HEADER_LEN, MemoryList, OWN_HEAP_RVA, with_figures};
use rc2k_core::memory::block_size;
use windows_sys::Win32::System::Memory::{MEM_COMMIT, MEM_RESERVE, PAGE_READWRITE, VirtualAlloc};

// ---- Rust version ----

/// The block's memory.
pub unsafe fn allocate(list: *mut MemoryList, bytes: u32) -> Result<*mut u8, Refused> {
    unsafe {
        if game_read::<u32>(OWN_HEAP_RVA) == 1 {
            return Err(BadInput("the game's own heap is in use (altmemenable)").into());
        }
        // Converted: `rc2k_core::memory` (`block_size`, `List::add`, `Figures::taken`).
        let whole = block_size(bytes);
        let (false, Some(whole)) = (list.is_null(), whole) else {
            return Err(BadInput("no list, or a size that wraps").into());
        };
        let block = VirtualAlloc(
            std::ptr::null(),
            whole as usize,
            MEM_COMMIT | MEM_RESERVE,
            PAGE_READWRITE,
        ) as *mut u32;
        if block.is_null() {
            return Err(FatalError {
                code: 0x8010_8571,
                message: "Need more virtual memory!",
            }
            .into());
        }
        let header = (*list).add(block as u32, bytes);
        block.write(header[0]);
        block.add(1).write(header[1]);
        with_figures(|f| f.taken(bytes));
        Ok((block as *mut u8).add(HEADER_LEN as usize))
    }
}

// ---- self-test: blocks taken on lists of the test's, by both, and given back; the memory
// figures are put back ----
