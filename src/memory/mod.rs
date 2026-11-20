//! The game's memory: the allocator's blocks on lists and the figures it keeps about them, and the
//! typed cells the game's data lives in (`cells`, `session`, `place`, `part_cell`,
//! `list_handle`, `shared_memory`).

pub(crate) mod cells;
pub(crate) mod draw_order;
pub(crate) mod free_list;
pub(crate) mod host_hooks;
pub(crate) mod list_handle;
pub(crate) mod owned;
pub(crate) mod part_cell;
pub(crate) mod place;
pub(crate) mod session;
pub(crate) mod shared_memory;
pub(crate) mod shared_table;
pub(crate) mod take_from_list;

/// A list of blocks: what `allocate` adds to and `free_list` gives back in one go. Every
/// user of the allocator has one of its own. (`rc2k_core::memory::List`, laid out as the
/// game's.)
pub type MemoryList = rc2k_core::memory::List;

/// A `MemoryList` kept as four cells (a store's field): the list read and written whole as
/// `Shared<MemoryList>` was, but through its dwords, so that the record is plain cells.
#[cfg_attr(test, repr(C))]
#[derive(rc2k_derive::Cells, rc2k_derive::ZeroCells)]
pub struct ListCells {
    pub head: crate::memory::session::PlainDword,
    pub total: crate::memory::session::PlainDword,
    cursor: crate::memory::session::PlainDword,
    remaining: crate::memory::session::PlainDword,
}

impl ListCells {
    pub const fn new(list: MemoryList) -> Self {
        use crate::memory::session::PlainDword;
        Self {
            head: PlainDword::new(list.head),
            total: PlainDword::new(list.total),
            cursor: PlainDword::new(list.cursor),
            remaining: PlainDword::new(list.remaining),
        }
    }

    /// A copy of the list (its fields may be read half changed).
    pub fn get(&self) -> MemoryList {
        MemoryList {
            head: self.head.get(),
            total: self.total.get(),
            cursor: self.cursor.get(),
            remaining: self.remaining.get(),
        }
    }

    /// The list put in whole.
    pub fn set(&self, list: MemoryList) {
        self.head.set(list.head);
        self.total.set(list.total);
        self.cursor.set(list.cursor);
        self.remaining.set(list.remaining);
    }

    /// An empty list.
    pub const fn empty() -> Self {
        Self::new(MemoryList {
            head: 0,
            total: 0,
            cursor: 0,
            remaining: 0,
        })
    }
}

/// A `MemoryList` held in cells: what the allocator's functions take.
pub trait HeldList {
    /// A copy of the list.
    fn get(&self) -> MemoryList;
    /// The list put in whole.
    fn set(&self, list: MemoryList);
}

impl HeldList for ListCells {
    fn get(&self) -> MemoryList {
        ListCells::get(self)
    }
    fn set(&self, list: MemoryList) {
        ListCells::set(self, list);
    }
}

/// A list kept in a thread-shared record's four atomic dwords (head, total, cursor, remaining).
impl HeldList for [crate::memory::session::Dword; 4] {
    fn get(&self) -> MemoryList {
        MemoryList {
            head: self[0].get(),
            total: self[1].get(),
            cursor: self[2].get(),
            remaining: self[3].get(),
        }
    }
    fn set(&self, list: MemoryList) {
        self[0].set(list.head);
        self[1].set(list.total);
        self[2].set(list.cursor);
        self[3].set(list.remaining);
    }
}

/// Before each block's memory: +0 the block taken before it on the same list, +4 the bytes
/// asked for; the rest is not used.
pub const HEADER_LEN: u32 = rc2k_core::memory::HEADER_LEN;

/// The allocator's counters. Atomics: any thread allocates.
pub mod state {

    /// The most bytes held at once.
    pub static MOST_HELD: crate::memory::session::Dword = crate::memory::session::Dword::new(0);
    /// The bytes held now.
    pub static HELD: crate::memory::session::Dword = crate::memory::session::Dword::new(0);
    /// All bytes ever asked for ("MEM: memory turnover= .. maximum load= .."
    /// in the game's log).
    pub static TURNOVER: crate::memory::session::Dword = crate::memory::session::Dword::new(0);
}

/// `bytes` of Rust's memory counted on `list` as `allocate` counts a block it takes (the
/// list's total, the figures), for a user whose blocks are Rust's own; the
/// list's `free_list` gives them back. The list holds no block of the game's for them.
pub fn count_owned(list: &ListCells, bytes: u32) {
    list.total.set(list.total.get().wrapping_add(bytes));
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
/// changed are written).
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
