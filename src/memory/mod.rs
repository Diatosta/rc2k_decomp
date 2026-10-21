//! The game's memory allocator: blocks on lists, and the figures it keeps about them.

use crate::framework::{GameHook, rva};

pub mod allocate;
pub mod free_list;
pub mod free_own_heap;
pub mod move_bytes;
pub mod open_own_heap;
pub mod take_from_list;
pub mod virtual_alloc;

pub static HOOKS: &[&dyn GameHook] = &[
    &allocate::HOOK,
    &free_list::HOOK,
    &move_bytes::HOOK,
    &take_from_list::HOOK,
    &virtual_alloc::HOOK,
    &open_own_heap::HOOK,
    &free_own_heap::HOOK,
];

/// A list of blocks: what `allocate` adds to and `free_list` gives back in one go. Every
/// user of the allocator has one of its own. (`rc2k_core::memory::List`, laid out as the
/// game's.)
pub type MemoryList = rc2k_core::memory::List;

/// Before each block's memory: +0 the block taken before it on the same list, +4 the bytes
/// asked for; the rest is not used.
pub const HEADER_LEN: u32 = rc2k_core::memory::HEADER_LEN;

/// dword_4E0584, the setting `altmemenable`: 1 = blocks come from a heap of the game's own
/// (`sub_402A05`, `sub_402A92`, not ported) and not from `VirtualAlloc`.
pub const OWN_HEAP_RVA: usize = rva(0x4E0584);
/// dword_4E0578, dword_4E057C, dword_4E0580: the most bytes held at once, the bytes held
/// now, and all bytes ever asked for ("MEM: memory turnover= .. maximum load= .." in the
/// game's log).
pub const MOST_HELD_RVA: usize = rva(0x4E0578);
pub const HELD_RVA: usize = rva(0x4E057C);
pub const TURNOVER_RVA: usize = rva(0x4E0580);

/// The three figures loaded from the game, changed by `f`, stored back (only those that
/// changed are written, as the original writes only what it counts).
pub unsafe fn with_figures(f: impl FnOnce(&mut rc2k_core::memory::Figures)) {
    use crate::framework::{game_read, game_write};
    unsafe {
        let before = rc2k_core::memory::Figures {
            most_held: game_read(MOST_HELD_RVA),
            held: game_read(HELD_RVA),
            turnover: game_read(TURNOVER_RVA),
        };
        let mut after = before;
        f(&mut after);
        for (at, old, new) in [
            (TURNOVER_RVA, before.turnover, after.turnover),
            (HELD_RVA, before.held, after.held),
            (MOST_HELD_RVA, before.most_held, after.most_held),
        ] {
            if old != new {
                game_write(at, new);
            }
        }
    }
}
