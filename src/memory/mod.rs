//! The game's memory allocator: blocks on lists, and the figures it keeps about them. No
//! hooks left (conversion phase 4, 2026-10-22): Rust callers call these directly; the
//! game's own code (hooked DLL) runs the originals, on the same lists.

pub mod free_list;
pub mod free_own_heap;
pub mod open_own_heap;
pub mod owned;
pub mod pages;
pub mod take_from_list;
pub mod virtual_alloc;

/// A list of blocks: what `allocate` adds to and `free_list` gives back in one go. Every
/// user of the allocator has one of its own. (`rc2k_core::memory::List`, laid out as the
/// game's.)
pub type MemoryList = rc2k_core::memory::List;

/// Before each block's memory: +0 the block taken before it on the same list, +4 the bytes
/// asked for; the rest is not used.
pub const HEADER_LEN: u32 = rc2k_core::memory::HEADER_LEN;

/// The allocator's variables, out of the image since 2026-10-24 (phase 5; they were
/// `dword_4E0578` .. `dword_4E0590`, 7 dwords). Atomics: any thread allocates. `OWN_HEAP` and
/// `OWN_HEAP_KB` are `var\ral.cfg` settings (`altmemenable`, `altmemsize`, through
/// `crate::moved::place`). Each has its own entry in `moved::MOVED`.
pub mod state {
    use std::sync::atomic::AtomicU32;
    /// dword_4E0578: the most bytes held at once.
    pub static MOST_HELD: AtomicU32 = AtomicU32::new(0);
    /// dword_4E057C: the bytes held now.
    pub static HELD: AtomicU32 = AtomicU32::new(0);
    /// dword_4E0580: all bytes ever asked for ("MEM: memory turnover= .. maximum load= .."
    /// in the game's log).
    pub static TURNOVER: AtomicU32 = AtomicU32::new(0);
    /// dword_4E0584, the setting `altmemenable`: 1 = blocks come from a heap of the game's
    /// own (`sub_402A05`, `sub_402A92`, not ported) and not from `VirtualAlloc`.
    pub static OWN_HEAP: AtomicU32 = AtomicU32::new(0);
    /// dword_4E0588: the heap's Kb (the setting `altmemsize`), 0x10000 in the image.
    pub static OWN_HEAP_KB: AtomicU32 = AtomicU32::new(0x10000);
    /// dword_4E058C: the heap's memory.
    pub static HEAP_BASE: AtomicU32 = AtomicU32::new(0);
    /// dword_4E0590: the heap's memory (set with `HEAP_BASE`).
    pub static HEAP_CURSOR: AtomicU32 = AtomicU32::new(0);
}

/// `bytes` of Rust's memory counted on `list` as `allocate` counts a block it takes (the
/// list's total, the figures), for a user whose blocks are Rust's own (phase 12); the
/// list's `free_list` gives them back. The list holds no block of the game's for them.
pub fn count_owned(list: &crate::session::Shared<MemoryList>, bytes: u32) {
    list.update(|l| l.total = l.total.wrapping_add(bytes));
    with_figures(|f| f.taken(bytes));
}

/// The figures now (tests put them back).
#[cfg(test)]
pub fn figures() -> rc2k_core::memory::Figures {
    use std::sync::atomic::Ordering::Relaxed;
    rc2k_core::memory::Figures {
        most_held: state::MOST_HELD.load(Relaxed),
        held: state::HELD.load(Relaxed),
        turnover: state::TURNOVER.load(Relaxed),
    }
}

/// `altmemenable` now (`dword_4E0584`).
pub fn own_heap() -> u32 {
    state::OWN_HEAP.load(std::sync::atomic::Ordering::Relaxed)
}

/// The three figures loaded from the game, changed by `f`, stored back (only those that
/// changed are written, as the original writes only what it counts).
pub fn with_figures(f: impl FnOnce(&mut rc2k_core::memory::Figures)) {
    use state::{HELD, MOST_HELD, TURNOVER};
    use std::sync::atomic::Ordering::Relaxed;
    let before = rc2k_core::memory::Figures {
        most_held: MOST_HELD.load(Relaxed),
        held: HELD.load(Relaxed),
        turnover: TURNOVER.load(Relaxed),
    };
    let mut after = before;
    f(&mut after);
    for (at, old, new) in [
        (&TURNOVER, before.turnover, after.turnover),
        (&HELD, before.held, after.held),
        (&MOST_HELD, before.most_held, after.most_held),
    ] {
        if old != new {
            at.store(new, Relaxed);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::state::*;
    use crate::moved::image_bytes;
    use std::sync::atomic::Ordering::Relaxed;

    #[test]
    fn variables_start_as_the_games() {
        let _lock = crate::store::test_lock();
        let d = image_bytes(0x4E0578, 28);
        let word = |n: usize| u32::from_le_bytes(d[n * 4..n * 4 + 4].try_into().unwrap());
        for (n, v) in [
            &MOST_HELD,
            &HELD,
            &TURNOVER,
            &OWN_HEAP,
            &OWN_HEAP_KB,
            &HEAP_BASE,
            &HEAP_CURSOR,
        ]
        .iter()
        .enumerate()
        {
            assert_eq!(word(n), v.load(Relaxed), "dword {n}");
        }
    }
}
