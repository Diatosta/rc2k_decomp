/// `now` into the cell when it is not `was` (what was read from it).
pub fn set_changed(c: &crate::session::Dword, was: u32, now: u32) {
    if now != was {
        c.set_bits(now);
    }
}

/// The values of `cells`.
pub fn values<const N: usize>(cells: &[crate::session::Dword; N]) -> [u32; N] {
    std::array::from_fn(|k| cells[k].get())
}

/// What changed from `was` to `now` stored into `cells`.
pub fn set_values_changed(cells: &[crate::session::Dword], was: &[u32], now: &[u32]) {
    for ((c, &w), &n) in cells.iter().zip(was).zip(now) {
        set_changed(c, w, n);
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
