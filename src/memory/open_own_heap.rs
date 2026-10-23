//! `memory::open_own_heap` (0x40298A): with the setting `altmemenable` (`g_ownHeap`
//! 0x4E0584 = 1), the memory of the game's own heap taken at the start.
//!
//! In:  nothing
//! Out: every register preserved (`pusha` .. `popa`)
//! The heap's size in Kb is `dword_4E0588`, at least 0x2800 (signed). "MEM: Using
//! alternative scheme, heap=" and the size go to the log (level 1); the size * 0x400 (an
//! `imul`, wrapping) from `memory::virtual_alloc` goes to `dword_4E058C` and
//! `dword_4E0590`, and the memory's first dwords are 0 and the bytes. One caller, `start`
//! (0x401030); in the game this never gets a call (the hook DLL is loaded later). Refuses
//! (before anything) a size whose bytes wrap to 0: the original writes at address 0 there.
//! `start` calls it directly (2026-10-23); the registers above are the original's, for
//! reference.

use crate::errors::{BadInput, Refused};
use crate::framework::{game_read, game_write, rva};
use crate::logfile::{log_int::log_int, write_line::log_text};

const SIZE_RVA: usize = rva(0x4E0588); // dword_4E0588: the heap's Kb (`altmemsize`)
const HEAP_RVA: usize = rva(0x4E058C); // dword_4E058C, dword_4E0590: the heap's memory

/// The Rust version (see the header).
pub fn open_own_heap() -> Result<(), Refused> {
    // Converted: `rc2k_core::memory::{own_heap_on, own_heap_size, own_heap_head}`.
    use rc2k_core::memory::{own_heap_head, own_heap_on, own_heap_size};
    unsafe {
        if !own_heap_on(game_read(crate::memory::OWN_HEAP_RVA)) {
            return Ok(());
        }
        let Some((kb, bytes)) = own_heap_size(game_read(SIZE_RVA)) else {
            return Err(BadInput("a heap size whose bytes wrap to 0").into());
        };
        log_text(b"MEM: Using alternative scheme, heap=", false, 1);
        log_int(kb as i32, true, 1);
        let memory = crate::memory::virtual_alloc::virtual_alloc(bytes)?;
        game_write(HEAP_RVA, memory);
        game_write(HEAP_RVA + 4, memory);
        let [first, second] = own_heap_head(bytes);
        (memory as *mut u32).write_unaligned(first);
        ((memory + 4) as *mut u32).write_unaligned(second);
    }
    Ok(())
}
