/// The dword at `a`.
///
/// # Safety
/// `a` must be readable.
pub unsafe fn get(a: u32) -> u32 {
    unsafe { (a as *const u32).read_unaligned() }
}

/// `v` into the dword at `a`.
///
/// # Safety
/// `a` must be writable.
pub unsafe fn put(a: u32, v: u32) {
    unsafe { (a as *mut u32).write_unaligned(v) }
}

/// `now` into the dword at `a` when it is not `was` (what was loaded from there).
///
/// # Safety
/// As `put`.
pub unsafe fn put_changed(a: u32, was: u32, now: u32) {
    if now != was {
        unsafe { put(a, now) }
    }
}

/// `N` dwords from `a` on.
///
/// # Safety
/// The `N` dwords must be readable.
pub unsafe fn get_list<const N: usize>(a: u32) -> [u32; N] {
    std::array::from_fn(|k| unsafe { get(a + 4 * k as u32) })
}

/// The dwords of `now` that are not `was`'s into their places from `a` on.
///
/// # Safety
/// As `put`, for each of them.
pub unsafe fn put_list_changed(a: u32, was: &[u32], now: &[u32]) {
    for (k, (&w, &n)) in was.iter().zip(now).enumerate() {
        unsafe { put_changed(a + 4 * k as u32, w, n) }
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

    fn keep<T: Default>(&self, r: Result<T, crate::errors::BadInput>) -> T {
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

    /// The dword at `a`.
    pub fn dword(&self, a: u32, what: &str) -> u32 {
        self.keep(crate::pointers::GamePtr::new(a).read::<u32>(what))
    }

    /// The byte at `a`.
    pub fn byte(&self, a: u32, what: &str) -> u8 {
        self.keep(crate::pointers::GamePtr::new(a).read::<u8>(what))
    }

    /// The word at `a`.
    pub fn word(&self, a: u32, what: &str) -> u16 {
        self.keep(crate::pointers::GamePtr::new(a).read::<u16>(what))
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
