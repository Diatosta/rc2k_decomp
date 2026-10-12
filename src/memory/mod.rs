//! The game's memory allocator: blocks on lists, and the figures it keeps about them.

use crate::framework::{GameHook, rva};

pub mod allocate;
pub mod free_list;
pub mod take_from_list;

pub static HOOKS: &[&dyn GameHook] = &[&allocate::HOOK, &free_list::HOOK, &take_from_list::HOOK];

/// A list of blocks: what `allocate` adds to and `free_list` gives back in one go. Every
/// user of the allocator has one of its own.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct MemoryList {
    /// The block taken last (its header), or 0.
    pub head: u32,
    /// The bytes asked for, over all blocks.
    pub total: u32,
    /// Set by `sub_402941`: the newest block's memory and the total, for users that hand
    /// the memory out piece by piece. `free_list` leaves both 0.
    pub cursor: u32,
    pub remaining: u32,
}

/// Before each block's memory: +0 the block taken before it on the same list, +4 the bytes
/// asked for; the rest is not used.
pub const HEADER_LEN: u32 = 0x40;

/// dword_4E0584, the setting `altmemenable`: 1 = blocks come from a heap of the game's own
/// (`sub_402A05`, `sub_402A92`, not ported) and not from `VirtualAlloc`.
pub const OWN_HEAP_RVA: usize = rva(0x4E0584);
/// dword_4E0578, dword_4E057C, dword_4E0580: the most bytes held at once, the bytes held
/// now, and all bytes ever asked for ("MEM: memory turnover= .. maximum load= .." in the
/// game's log).
pub const MOST_HELD_RVA: usize = rva(0x4E0578);
pub const HELD_RVA: usize = rva(0x4E057C);
pub const TURNOVER_RVA: usize = rva(0x4E0580);
