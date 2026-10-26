//! `memory::free_own_heap` (0x4029DB): at the game's end, with the game's own heap in use
//! (`altmemenable`, `dword_4E0584` 1), its memory (`dword_4E058C`) given back
//! (`memory::virtual_free`) and it and `dword_4E0590` made 0. The other half of
//! `sub_40298A` (agent A's: the heap's making).
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`)
//!
//! One caller, `start`'s way out. Its call through `part` (in the game `VirtualFree` with
//! `MEM_RELEASE`, as `virtual_free`).
//! Hook removed 2026-10-22 (direct calls): Rust callers run its `entry` (`calls::call`,
//! `routes.rs`); the registers above are the original's, for reference.

use crate::errors::Refused;
use crate::framework::{Regs, game_addr, rva};
use crate::system::api_stand_ins::Api;
use crate::system::exit_parts::{Part, VIRTUAL_FREE, exit_entry};

const OWN_HEAP: usize = 0x4E0584; // g_ownHeap (altmemenable)
const HEAP: usize = 0x4E058C; // dword_4E058C: the heap's memory
const HEAP_MORE: usize = 0x4E0590; // dword_4E0590

/// The Rust version on the switch at `on` and the heap's two dwords at `heap`, `more`.
pub fn free_own_heap_with(
    r: &mut Regs,
    on: u32,
    heap: u32,
    more: u32,
    part: Part,
) -> Result<(), Refused> {
    let rd = |at: u32| unsafe { (at as *const u32).read_volatile() };
    // Converted: `rc2k_core::memory::own_heap_on`.
    if !rc2k_core::memory::own_heap_on(rd(on)) {
        return Ok(());
    }
    let mut g = Regs {
        eax: rd(heap),
        ..*r
    };
    part(VIRTUAL_FREE, &mut g)?;
    unsafe {
        (heap as *mut u32).write_volatile(0);
        (more as *mut u32).write_volatile(0);
    }
    Ok(())
}

fn in_game(r: &mut Regs, part: Part, _api: Api) -> Result<(), Refused> {
    let a = |x: usize| game_addr(rva(x)) as u32;
    free_own_heap_with(r, a(OWN_HEAP), a(HEAP), a(HEAP_MORE), part)
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    exit_entry(r, in_game)
}
