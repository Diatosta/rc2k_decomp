//! `memory::move_bytes` (0x46573F): eax bytes from esi to edi, `rep movsd` then `rep movsb`.
//!
//! In:  eax the count, esi from, edi to
//! Out: ecx 0, esi and edi past the bytes
//!
//! Callers: `sub_4BA388` (`marks::order_spline_points`, twice) and `sub_4BD1A1`, each inside
//! a `pusha`. Forwards, a dword at a time and then the last bytes: an edi a little past esi
//! repeats what was there, as the original. Self-test: the original and the Rust version on
//! buffers of the test's (0 .. 0x23 bytes, apart and overlapping).
//!
//! Hook removed 2026-10-22 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

/// The Rust version: `count` bytes from `from` to `to`, forwards (converted:
/// `rc2k_core::memory::move_forward`; ranges that overlap are one buffer, copied as that
/// does; apart, a plain copy).
///
/// # Safety
/// `from` readable and `to` writable for `count` bytes.
pub unsafe fn move_bytes(from: *const u8, to: *mut u8, count: u32) {
    let (f, t, n) = (from as usize, to as usize, count as usize);
    if n == 0 {
        return;
    }
    unsafe {
        if f < t + n && t < f + n {
            let low = f.min(t);
            let span = f.max(t) + n - low;
            let buf = std::slice::from_raw_parts_mut(low as *mut u8, span);
            rc2k_core::memory::move_forward(buf, f - low, t - low, n);
        } else {
            std::ptr::copy_nonoverlapping(from, to, n);
        }
    }
}
