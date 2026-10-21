//! For the converted physics and drive adapters: dwords at addresses (the game's
//! variables, or a self-test's cells standing in for them) loaded into the core's
//! structures, and stored back where the core changed them.

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
