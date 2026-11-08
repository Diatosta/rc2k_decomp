//! A table of bytes every thread reaches (the course's faces, records and info, the scene
//! points, the objects' points and faces): held in the game's own thread's storage until the
//! drawing threads (`workers`) needed to read it too.
//!
//! The bytes are atomic words (relaxed, which compile to plain loads and stores), so threads
//! drawing at once can read and write them with no lock; what they write is the same as one
//! thread's in the list's order wherever `race::draw_chunks` found the order does not matter.
//! Each table's words are allocated once, as many as its most bytes (zeroed memory: pages the
//! system gives only when touched), and kept for the program's life; a step is a check of the
//! length (`len`) and the words' loads or stores, as cheap as the thread's own storage was.
//!
//! Reads and writes copy (`read`, `bytes`, `array`, `write`): no thread is lent the bytes.

use crate::errors::BadInput;
use std::sync::OnceLock;
use std::sync::atomic::{AtomicU32, AtomicUsize, Ordering};

/// `len` while none is held.
const NONE: usize = usize::MAX;

pub struct SharedTable {
    /// Four bytes to a word, little-endian.
    words: OnceLock<Box<[AtomicU32]>>,
    /// Its length in bytes (`NONE` while not held).
    len: AtomicUsize,
    /// The most bytes it holds.
    room: usize,
    /// What it is, for the refusals.
    what: &'static str,
}

impl SharedTable {
    pub const fn new(room: usize, what: &'static str) -> Self {
        Self {
            words: OnceLock::new(),
            len: AtomicUsize::new(NONE),
            room,
            what,
        }
    }

    fn words(&self) -> &[AtomicU32] {
        self.words.get_or_init(|| {
            let words: Box<[AtomicU32]> = (0..self.room.div_ceil(4))
                .map(|_| AtomicU32::new(0))
                .collect();
            #[cfg(feature = "write_watch")]
            WATCHED_TABLES
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .push((words.as_ptr().addr(), words.len() * 4, self.what));
            words
        })
    }

    /// The table set to `bytes`; refused past its room.
    pub fn set(&self, bytes: &[u8]) -> Result<(), BadInput> {
        if bytes.len() > self.room {
            return Err(BadInput("a table larger than its room"));
        }
        self.len.store(NONE, Ordering::Relaxed);
        let words = self.words();
        for (w, c) in words.iter().zip(bytes.chunks(4)) {
            let mut b = [0; 4];
            b[..c.len()].copy_from_slice(c);
            w.store(u32::from_le_bytes(b), Ordering::Relaxed);
        }
        self.len.store(bytes.len(), Ordering::Relaxed);
        Ok(())
    }

    /// Given back: none held.
    pub fn give_back(&self) {
        self.len.store(NONE, Ordering::Relaxed);
    }

    pub fn is_taken(&self) -> bool {
        self.len.load(Ordering::Relaxed) != NONE
    }

    /// Its length in bytes. (Relaxed: a table is set while no other thread reaches it, the
    /// drawing threads being started after it, which orders their reads after the set.)
    #[inline(always)]
    pub fn len(&self) -> Result<usize, BadInput> {
        match self.len.load(Ordering::Relaxed) {
            NONE => Err(BadInput(self.what)),
            len => Ok(len),
        }
    }

    /// The words, if `len` bytes from `at` are inside the table.
    #[inline]
    fn holding(&self, at: usize, len: usize) -> Result<Option<&[AtomicU32]>, BadInput> {
        let all = self.len()?;
        let inside = at.checked_add(len).is_some_and(|end| end <= all);
        Ok(inside.then(|| self.words()))
    }

    /// `into`'s length of bytes from byte `at`; `None` past the end.
    #[inline(always)]
    pub fn read(&self, at: usize, into: &mut [u8]) -> Result<Option<()>, BadInput> {
        let Some(words) = self.holding(at, into.len())? else {
            return Ok(None);
        };
        crate::write_watch::read(words[at / 4..].as_ptr().cast(), into.len() + at % 4);
        let word = |n: usize| words[n].load(Ordering::Relaxed).to_le_bytes();
        if at.is_multiple_of(4) {
            // The common case: whole words from a word's start.
            let (whole, rest) = into.as_chunks_mut::<4>();
            for (to, w) in whole.iter_mut().zip(&words[at / 4..]) {
                *to = w.load(Ordering::Relaxed).to_le_bytes();
            }
            if !rest.is_empty() {
                let n = rest.len();
                rest.copy_from_slice(&word(at / 4 + whole.len())[..n]);
            }
            return Ok(Some(()));
        }
        let mut k = 0;
        while k < into.len() {
            let byte = at + k;
            let from = byte % 4;
            let n = (4 - from).min(into.len() - k);
            into[k..k + n].copy_from_slice(&word(byte / 4)[from..from + n]);
            k += n;
        }
        Ok(Some(()))
    }

    /// The `N` bytes from byte `at`; `None` past the end.
    #[inline]
    pub fn array<const N: usize>(&self, at: usize) -> Result<Option<[u8; N]>, BadInput> {
        let mut to = [0; N];
        Ok(self.read(at, &mut to)?.map(|()| to))
    }

    /// `len` bytes from byte `at`; `None` past the end.
    pub fn bytes(&self, at: usize, len: usize) -> Result<Option<Vec<u8>>, BadInput> {
        let mut to = vec![0; len];
        Ok(self.read(at, &mut to)?.map(|()| to))
    }

    /// Every byte.
    pub fn all(&self) -> Result<Vec<u8>, BadInput> {
        let mut to = vec![0; self.len()?];
        self.read(0, &mut to)?;
        Ok(to)
    }

    /// `data` written from byte `at`; `None` (nothing written) past the end.
    #[inline]
    pub fn write(&self, at: usize, data: &[u8]) -> Result<Option<()>, BadInput> {
        self.write_noting(at, data, |_| ())
    }

    /// `data` written from byte `at`, `changed` told each of `data`'s offsets it changes;
    /// `None` (nothing written) past the end. A whole word is one store, a part of one a
    /// compare-and-swap that keeps the other bytes as another thread left them.
    pub fn write_noting(
        &self,
        at: usize,
        data: &[u8],
        mut changed: impl FnMut(usize),
    ) -> Result<Option<()>, BadInput> {
        let Some(words) = self.holding(at, data.len())? else {
            return Ok(None);
        };
        crate::write_watch::wrote(words[at / 4..].as_ptr().cast(), data.len() + at % 4);
        let mut k = 0;
        while k < data.len() {
            let byte = at + k;
            let from = byte % 4;
            let n = (4 - from).min(data.len() - k);
            let new = &data[k..k + n];
            let cell = &words[byte / 4];
            let old = if n == 4 {
                cell.swap(
                    u32::from_le_bytes([new[0], new[1], new[2], new[3]]),
                    Ordering::Relaxed,
                )
            } else {
                let merge = |old: u32| {
                    let mut w = old.to_le_bytes();
                    w[from..from + n].copy_from_slice(new);
                    Some(u32::from_le_bytes(w))
                };
                match cell.try_update(Ordering::Relaxed, Ordering::Relaxed, merge) {
                    Ok(old) | Err(old) => old,
                }
            }
            .to_le_bytes();
            for (j, (&o, &v)) in old[from..from + n].iter().zip(new).enumerate() {
                if o != v {
                    changed(k + j);
                }
            }
            k += n;
        }
        Ok(Some(()))
    }
}

/// With the feature `write_watch`: each table's words (host address, bytes) and what it is.
#[cfg(feature = "write_watch")]
static WATCHED_TABLES: std::sync::Mutex<Vec<(usize, usize, &'static str)>> =
    std::sync::Mutex::new(Vec::new());

/// With the feature `write_watch`: the table and byte a host address lies in.
#[cfg(feature = "write_watch")]
pub fn table_of(host: usize) -> Option<(&'static str, usize)> {
    WATCHED_TABLES
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .iter()
        .find(|&&(start, len, _)| (start..start + len).contains(&host))
        .map(|&(start, _, what)| (what, host - start))
}

#[cfg(test)]
mod tests {
    static TABLE: super::SharedTable = super::SharedTable::new(8, "no test table");

    #[test]
    fn read_written_and_seen_from_other_threads() {
        assert!(TABLE.read(0, &mut [0]).is_err());
        TABLE.set(&[1, 2, 3, 4]).unwrap();
        assert_eq!(TABLE.array::<2>(1), Ok(Some([2, 3])));
        assert_eq!(TABLE.array::<2>(3), Ok(None));
        let mut seen = Vec::new();
        TABLE.write_noting(0, &[1, 9, 3], |k| seen.push(k)).unwrap();
        assert_eq!(seen, [1]);
        std::thread::scope(|s| {
            s.spawn(|| TABLE.write(3, &[7]).unwrap());
        });
        assert_eq!(TABLE.all(), Ok(vec![1, 9, 3, 7]));
        TABLE.set(&[5]).unwrap();
        assert!(TABLE.set(&[0; 9]).is_err());
        assert_eq!(TABLE.all(), Ok(vec![5]));
        TABLE.give_back();
        assert!(!TABLE.is_taken());
    }
}
