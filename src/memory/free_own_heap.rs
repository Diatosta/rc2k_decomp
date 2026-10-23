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

use crate::framework::{game_read, game_write, rva};

const OWN_HEAP_RVA: usize = rva(0x4E0584); // g_ownHeap (altmemenable)
const HEAP_RVA: usize = rva(0x4E058C); // dword_4E058C: the heap's memory
const HEAP_MORE_RVA: usize = rva(0x4E0590); // dword_4E0590

/// The Rust version (see the header).
pub fn free_own_heap() {
    use windows_sys::Win32::System::Memory::{MEM_RELEASE, VirtualFree};
    unsafe {
        // Converted: `rc2k_core::memory::own_heap_on`.
        if !rc2k_core::memory::own_heap_on(game_read(OWN_HEAP_RVA)) {
            return;
        }
        let heap: u32 = game_read(HEAP_RVA);
        if heap != 0 {
            VirtualFree(heap as _, 0, MEM_RELEASE);
        }
        game_write(HEAP_RVA, 0u32);
        game_write(HEAP_MORE_RVA, 0u32);
    }
}
