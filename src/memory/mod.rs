//! The game's memory allocator: blocks on lists, and the figures it keeps about them.

pub mod free_list;
pub mod owned;
pub mod shared_table;
pub mod take_from_list;

/// A list of blocks: what `allocate` adds to and `free_list` gives back in one go. Every
/// user of the allocator has one of its own. (`rc2k_core::memory::List`, laid out as the
/// game's.)
pub type MemoryList = rc2k_core::memory::List;

/// A `MemoryList` kept as four cells (a store's field): the list read and written whole as
/// `Shared<MemoryList>` was, but through its dwords, so that the record is plain cells.
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct ListCells {
    head: crate::session::Dword,
    total: crate::session::Dword,
    cursor: crate::session::Dword,
    remaining: crate::session::Dword,
}

impl ListCells {
    pub const fn new(list: MemoryList) -> Self {
        use crate::session::Dword;
        Self {
            head: Dword::new(list.head),
            total: Dword::new(list.total),
            cursor: Dword::new(list.cursor),
            remaining: Dword::new(list.remaining),
        }
    }

    /// A copy of the list (its fields may be read half changed, as the original's).
    pub fn get(&self) -> MemoryList {
        MemoryList {
            head: self.head.get(),
            total: self.total.get(),
            cursor: self.cursor.get(),
            remaining: self.remaining.get(),
        }
    }

    /// An empty list (tests).
    #[cfg(test)]
    pub const fn empty() -> Self {
        Self::new(MemoryList {
            head: 0,
            total: 0,
            cursor: 0,
            remaining: 0,
        })
    }

    #[cfg(test)]
    pub fn addr(&self) -> u32 {
        crate::pointers::game_address(std::ptr::from_ref(self))
    }
}

/// Before each block's memory: +0 the block taken before it on the same list, +4 the bytes
/// asked for; the rest is not used.
pub const HEADER_LEN: u32 = rc2k_core::memory::HEADER_LEN;

/// The allocator's variables, out of the image since 2026-10-24 (phase 5; they were
/// `dword_4E0578` .. `dword_4E0588`, 5 dwords). Atomics: any thread allocates. `OWN_HEAP` and
/// `OWN_HEAP_KB` are `var\ral.cfg` settings (`altmemenable`, `altmemsize`, through
/// `crate::moved::place`). Each has its own entry in `moved::MOVED`.
pub mod state {

    /// dword_4E0578: the most bytes held at once.
    pub static MOST_HELD: crate::session::Dword = crate::session::Dword::new(0);
    /// dword_4E057C: the bytes held now.
    pub static HELD: crate::session::Dword = crate::session::Dword::new(0);
    /// dword_4E0580: all bytes ever asked for ("MEM: memory turnover= .. maximum load= .."
    /// in the game's log).
    pub static TURNOVER: crate::session::Dword = crate::session::Dword::new(0);
    /// dword_4E0584, the setting `altmemenable`: in the original, 1 = blocks come from a heap
    /// of the game's own and not from `VirtualAlloc`. Read and not used: the own heap was
    /// removed (2026-11-07; every block is Rust's memory, nothing the player sees changes).
    pub static OWN_HEAP: crate::session::Dword = crate::session::Dword::new(0);
    /// dword_4E0588: the heap's Kb (the setting `altmemsize`), 0x10000 in the image. Read and
    /// not used (see `OWN_HEAP`). (`dword_4E058C` / `dword_4E0590`, the heap's memory, went
    /// with it.)
    pub static OWN_HEAP_KB: crate::session::Dword = crate::session::Dword::new(0x10000);
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
    rc2k_core::memory::Figures {
        most_held: state::MOST_HELD.get(),
        held: state::HELD.get(),
        turnover: state::TURNOVER.get(),
    }
}

/// The three figures loaded from the game, changed by `f`, stored back (only those that
/// changed are written, as the original writes only what it counts).
pub fn with_figures(f: impl FnOnce(&mut rc2k_core::memory::Figures)) {
    use state::{HELD, MOST_HELD, TURNOVER};

    let before = rc2k_core::memory::Figures {
        most_held: MOST_HELD.get(),
        held: HELD.get(),
        turnover: TURNOVER.get(),
    };
    let mut after = before;
    f(&mut after);
    for (at, old, new) in [
        (&TURNOVER, before.turnover, after.turnover),
        (&HELD, before.held, after.held),
        (&MOST_HELD, before.most_held, after.most_held),
    ] {
        if old != new {
            at.set(new);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::state::*;
    use crate::moved::image_bytes;

    #[test]
    fn variables_start_as_the_games() {
        let _lock = crate::store::test_lock();
        let d = image_bytes(0x4E0578, 20);
        let word = |n: usize| u32::from_le_bytes(d[n * 4..n * 4 + 4].try_into().unwrap());
        for (n, v) in [&MOST_HELD, &HELD, &TURNOVER, &OWN_HEAP, &OWN_HEAP_KB]
            .iter()
            .enumerate()
        {
            assert_eq!(word(n), v.get(), "dword {n}");
        }
    }
}
