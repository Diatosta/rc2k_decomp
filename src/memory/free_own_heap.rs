//! `memory::free_own_heap` (0x4029DB): at the game's end, with the game's own heap in use
//! (`altmemenable`, `dword_4E0584` 1), its memory (`dword_4E058C`) given back
//! (`memory::virtual_free`) and it and `dword_4E0590` made 0. The other half of
//! `sub_40298A` (agent A's: the heap's making).
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`)
//!
//! One caller, `start`'s way out, which calls it directly (2026-10-23). `VirtualFree`
//! with `MEM_RELEASE`, as `memory::virtual_free`; the registers above are the original's,
//! for reference.

use crate::memory::state::{HEAP_BASE, HEAP_CURSOR};

/// The Rust version (see the header).
pub fn free_own_heap() {
    // Converted: `rc2k_core::memory::own_heap_on`.
    if !rc2k_core::memory::own_heap_on(crate::memory::own_heap()) {
        return;
    }
    let heap = HEAP_BASE.get();
    // The heap is `virtual_alloc`'s (`open_own_heap`); its region and range go with it
    // (they were left behind before phase 11).
    if heap != 0 && crate::memory::pages::give_back(heap).is_err() {
        crate::fatal::raise::end(
            "memory::free_own_heap",
            crate::errors::BadInput("the own heap was not taken from the system"),
        );
    }
    HEAP_BASE.set(0);
    HEAP_CURSOR.set(0);
}
