//! The audit build's record of the game state's reads and writes (cargo feature `write_watch`;
//! `race::draw_chunks` uses it to find what two runs of a list's entries both reach). Every
//! cell's load and store (`session`'s cells, `pointers`' cell runs, `shared_memory`) calls
//! `read` or `wrote`; without the feature they are empty and cost nothing. With it, while a
//! thread watches (`begin` .. `end`), each place reached is noted by its dword (a byte noted
//! as the dword that holds it: two neighbouring bytes count as one place, which can only add
//! to what is found).

/// `len` bytes at `at` read.
#[inline(always)]
pub fn read(_at: *const u8, _len: usize) {
    #[cfg(feature = "write_watch")]
    on::note(false, _at, _len);
}

/// `len` bytes at `at` written.
#[inline(always)]
pub fn wrote(_at: *const u8, _len: usize) {
    #[cfg(feature = "write_watch")]
    on::note(true, _at, _len);
}

#[cfg(feature = "write_watch")]
pub use on::{Seen, begin, end};

#[cfg(feature = "write_watch")]
mod on {
    use std::cell::{Cell, RefCell};
    use std::collections::HashSet;

    /// The dwords (host address / 4) read and written while watched.
    #[derive(Default)]
    pub struct Seen {
        pub read: HashSet<usize>,
        pub written: HashSet<usize>,
    }

    thread_local! {
        static ON: Cell<bool> = const { Cell::new(false) };
        static SEEN: RefCell<Seen> = RefCell::new(Seen::default());
    }

    pub fn note(write: bool, at: *const u8, len: usize) {
        if !ON.get() || len == 0 {
            return;
        }
        let first = at.addr() >> 2;
        let last = (at.addr() + len - 1) >> 2;
        SEEN.with_borrow_mut(|s| {
            let set = if write { &mut s.written } else { &mut s.read };
            set.extend(first..=last);
        });
    }

    /// The thread's watch begun (what it saw before forgotten).
    pub fn begin() {
        SEEN.with_borrow_mut(|s| *s = Seen::default());
        ON.set(true);
    }

    /// The thread's watch ended: what it saw.
    pub fn end() -> Seen {
        ON.set(false);
        SEEN.with_borrow_mut(std::mem::take)
    }
}
