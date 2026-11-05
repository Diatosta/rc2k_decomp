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
use crate::logfile::{log_int::log_int, write_line::log_text};
use crate::memory::state::{HEAP_BASE, HEAP_CURSOR, OWN_HEAP_KB};

/// The Rust version (see the header).
pub fn open_own_heap() -> Result<(), Refused> {
    // Converted: `rc2k_core::memory::{own_heap_on, own_heap_size, own_heap_head}`.
    use rc2k_core::memory::{own_heap_head, own_heap_on, own_heap_size};
    if !own_heap_on(crate::memory::own_heap()) {
        return Ok(());
    }
    let Some((kb, bytes)) = own_heap_size(OWN_HEAP_KB.get()) else {
        return Err(BadInput("a heap size whose bytes wrap to 0").into());
    };
    log_text(b"MEM: Using alternative scheme, heap=", false, 1);
    log_int(kb as i32, true, 1);
    let memory = crate::memory::virtual_alloc::virtual_alloc(bytes)?;
    HEAP_BASE.set(memory);
    HEAP_CURSOR.set(memory);
    let [first, second] = own_heap_head(bytes);
    let head = crate::memory::pages::block(memory)?;
    head.write(first, "the own heap's head")?;
    head.offset(4).write(second, "the own heap's head")?;
    Ok(())
}
