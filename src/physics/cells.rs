use crate::memory::session::DwordLike;

/// A word the notice and event steps write only when it changed: a dword cell (by reference) or a
/// plain word of a record (by mutable reference).
pub trait DwordPlace {
    fn put(&mut self, v: u32);
}

impl<T: DwordLike + ?Sized> DwordPlace for &T {
    fn put(&mut self, v: u32) {
        self.set(v);
    }
}

impl DwordPlace for &mut u32 {
    fn put(&mut self, v: u32) {
        **self = v;
    }
}

/// `now` into the word when it is not `was` (what was read from it).
pub(super) fn set_changed(mut c: impl DwordPlace, was: u32, now: u32) {
    if now != was {
        c.put(now);
    }
}

/// A row of words: cells by reference, or the plain words of a record.
pub trait DwordRow<const N: usize> {
    fn row(&self) -> [u32; N];
}

impl<C: DwordLike, const N: usize> DwordRow<N> for [C; N] {
    fn row(&self) -> [u32; N] {
        std::array::from_fn(|k| self[k].get())
    }
}

impl<const N: usize> DwordRow<N> for [u32; N] {
    fn row(&self) -> [u32; N] {
        *self
    }
}

/// The values of `cells`.
pub fn values<const N: usize>(cells: &impl DwordRow<N>) -> [u32; N] {
    cells.row()
}

/// A run of words written one by one: cells by reference, or the plain words of a record.
pub trait DwordPlaces {
    fn put_at(&mut self, k: usize, v: u32);
    fn len(&self) -> usize;
}

impl<C: DwordLike> DwordPlaces for &[C] {
    fn put_at(&mut self, k: usize, v: u32) {
        self[k].set(v);
    }
    fn len(&self) -> usize {
        <[C]>::len(self)
    }
}

impl<C: DwordLike, const N: usize> DwordPlaces for &[C; N] {
    fn put_at(&mut self, k: usize, v: u32) {
        self[k].set(v);
    }
    fn len(&self) -> usize {
        N
    }
}

impl DwordPlaces for &mut [u32] {
    fn put_at(&mut self, k: usize, v: u32) {
        self[k] = v;
    }
    fn len(&self) -> usize {
        <[u32]>::len(self)
    }
}

impl<const N: usize> DwordPlaces for &mut [u32; N] {
    fn put_at(&mut self, k: usize, v: u32) {
        self[k] = v;
    }
    fn len(&self) -> usize {
        N
    }
}

/// What changed from `was` to `now` stored into `cells`.
pub(super) fn set_values_changed(mut cells: impl DwordPlaces, was: &[u32], now: &[u32]) {
    for (k, (&w, &n)) in was.iter().zip(now).enumerate().take(cells.len()) {
        if n != w {
            cells.put_at(k, n);
        }
    }
}

/// Reads of the game's memory made inside a closure the core calls (which can answer a
/// number only): the first refusal is kept, the read answers 0, and `finish` hands the
/// refusal back after the core returns.
#[derive(Default)]
pub struct Trap(std::cell::Cell<Option<crate::errors::Refused>>);

impl Trap {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn keep<T: Default>(&self, r: Result<T, crate::errors::BadInput>) -> T {
        r.unwrap_or_else(|e| {
            self.refuse(e);
            T::default()
        })
    }

    /// Keep a refusal (the first one stays).
    pub fn refuse(&self, e: impl Into<crate::errors::Refused>) {
        if self.0.get().is_none() {
            self.0.set(Some(e.into()));
        }
    }

    /// Whether a refusal has been kept.
    pub fn failed(&self) -> bool {
        self.0.get().is_some()
    }

    /// The first refusal, if any.
    pub fn finish(&self) -> Result<(), crate::errors::Refused> {
        self.0.get().map_or(Ok(()), Err)
    }
}
