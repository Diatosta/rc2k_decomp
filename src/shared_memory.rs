//! Copies in and out of memory the game's threads share. The game keeps its state in memory
//! every thread reaches, so a plain or volatile copy that meets another thread's write is a
//! data race: undefined behaviour in Rust (found by ThreadSanitizer: the key table, the sound
//! options). These copy through relaxed atomics byte by byte (2026-11-05: wider moves over
//! the same bytes mixed sizes), so a dword copied here may be seen half written by another
//! thread; the cells that need a dword whole are `Dword`s, read at their own width.

#![allow(unsafe_code)]

#[cfg(test)]
use std::mem::MaybeUninit;
use std::sync::atomic::AtomicU8;
use std::sync::atomic::Ordering::Relaxed;

/// Calls `$step!(atomic, integer)` over `$len` bytes from `$at`, byte by byte: shared bytes
/// are reached at one width only, so no access mixes sizes with another (in Rust's model a
/// mixed-size race is undefined).
macro_rules! walk {
    ($at:expr, $len:expr, $n:ident, $step:ident) => {
        let _ = $at;
        while $n < $len {
            $step!(AtomicU8, u8);
        }
    };
}

/// `len` shared bytes at `from` copied to the private `to`.
///
/// # Safety
/// `from` is valid for reads and `to` for writes of `len` bytes, and they do not overlap; any
/// other thread reaching `from`'s bytes meanwhile does so by byte atomics too (as these); `to`
/// is reached by no other thread.
#[inline]
// `walk`'s last step leaves its count unread.
#[allow(unused_assignments)]
pub unsafe fn load(from: *const u8, to: *mut u8, len: usize) {
    let mut n = 0;
    macro_rules! step {
        ($atomic:ident, $int:ty) => {
            // SAFETY: byte `n` of the `len` the caller promises (`walk` stops at `len`); an
            // `AtomicU8` needs no alignment, and every thread reaches these bytes by byte
            // atomics (the caller's promise); `to` is private.
            unsafe {
                let v = $atomic::from_ptr(from.add(n).cast_mut().cast()).load(Relaxed);
                to.add(n).cast::<$int>().write_unaligned(v);
            }
            n += size_of::<$int>();
        };
    }
    walk!(from, len, n, step);
}

/// `len` private bytes at `from` copied to the shared `to`.
///
/// # Safety
/// As `load`'s, the roles swapped: `to` is the shared side, `from` the private.
#[inline]
// `walk`'s last step leaves its count unread.
#[allow(unused_assignments)]
pub unsafe fn store(to: *mut u8, from: *const u8, len: usize) {
    let mut n = 0;
    macro_rules! step {
        ($atomic:ident, $int:ty) => {
            // SAFETY: as `load`'s.
            unsafe {
                let v = from.add(n).cast::<$int>().read_unaligned();
                $atomic::from_ptr(to.add(n).cast()).store(v, Relaxed);
            }
            n += size_of::<$int>();
        };
    }
    walk!(to, len, n, step);
}

#[cfg(test)]
/// `len` shared bytes set to `value`.
///
/// # Safety
/// `to` is valid for writes of `len` bytes.
pub unsafe fn fill(to: *mut u8, value: u8, len: usize) {
    let chunk = [value; 512];
    let mut done = 0;
    while done < len {
        let now = (len - done).min(chunk.len());
        // SAFETY: inside the run (the caller's promise); `chunk` is private.
        unsafe { store(to.add(done), chunk.as_ptr(), now) };
        done += now;
    }
}

#[cfg(test)]
/// `len` shared bytes copied between shared places, overlapping as `memmove`.
///
/// # Safety
/// `from` is valid for reads and `to` for writes of `len` bytes.
pub unsafe fn copy(from: *const u8, to: *mut u8, len: usize) {
    let (a, b) = (from as usize, to as usize);
    if a.abs_diff(b) < len {
        let mut held = Vec::<u8>::with_capacity(len);
        // SAFETY: `held` has room for `len` bytes, all set by `load`; the rest as promised.
        unsafe {
            load(from, held.as_mut_ptr(), len);
            store(to, held.as_ptr(), len);
        }
        return;
    }
    let mut chunk = [MaybeUninit::<u8>::uninit(); 512];
    let mut done = 0;
    while done < len {
        let now = (len - done).min(chunk.len());
        // SAFETY: inside the runs (promised); `chunk` is private and `load` sets what `store`
        // reads.
        unsafe {
            load(from.add(done), chunk.as_mut_ptr().cast(), now);
            store(to.add(done), chunk.as_ptr().cast(), now);
        }
        done += now;
    }
}

/// The bytes from `offset` of an object of `size` bytes at `base`, which is 4-aligned, copied
/// into `to`: each whole dword of the object (`size` rounded down to 4) read as one
/// `AtomicU32` (whole, as the processor read the original's dwords), the bytes past them as
/// `AtomicU8`. Every access to the object takes these widths, whatever its offset and length,
/// so none mixes sizes.
///
/// # Safety
/// `base` is 4-aligned and starts `size` live bytes that are reached only through
/// `load_words` / `store_words`, and `offset + to.len() <= size`.
pub unsafe fn load_words(base: *const u8, size: usize, offset: usize, to: &mut [u8]) {
    use std::sync::atomic::AtomicU32;
    let words = size & !3;
    let end = offset + to.len();
    let mut at = offset;
    while at < end {
        if at < words {
            let word = at & !3;
            // SAFETY: `word + 4 <= words <= size`: a whole dword of the object, 4-aligned (`base`
            // is, the caller's promise), reached by every access as this one `AtomicU32`.
            let bytes = unsafe { AtomicU32::from_ptr(base.add(word).cast_mut().cast()) }
                .load(Relaxed)
                .to_ne_bytes();
            let upto = end.min(word + 4);
            to[at - offset..upto - offset].copy_from_slice(&bytes[at - word..upto - word]);
            at = upto;
        } else {
            // SAFETY: `words <= at < size` (`end <= size`, promised): a byte of the object past its
            // whole dwords, reached by every access as this `AtomicU8`.
            to[at - offset] = unsafe { AtomicU8::from_ptr(base.add(at).cast_mut()) }.load(Relaxed);
            at += 1;
        }
    }
}

/// `from` written at `offset` of the object `load_words` reads: a whole dword stored, a part
/// of one merged into it by one compare-and-swap (the other bytes kept as another thread left
/// them), the bytes past the whole dwords one by one.
///
/// # Safety
/// As `load_words`'.
pub unsafe fn store_words(base: *mut u8, size: usize, offset: usize, from: &[u8]) {
    use std::sync::atomic::AtomicU32;
    let words = size & !3;
    let end = offset + from.len();
    let mut at = offset;
    while at < end {
        if at < words {
            let word = at & !3;
            let upto = end.min(word + 4);
            let new = &from[at - offset..upto - offset];
            // SAFETY: as `load_words`'.
            let cell = unsafe { AtomicU32::from_ptr(base.add(word).cast()) };
            if new.len() == 4 {
                cell.store(
                    u32::from_ne_bytes([new[0], new[1], new[2], new[3]]),
                    Relaxed,
                );
            } else {
                let _ = cell.try_update(Relaxed, Relaxed, |old| {
                    let mut bytes = old.to_ne_bytes();
                    bytes[at - word..upto - word].copy_from_slice(new);
                    Some(u32::from_ne_bytes(bytes))
                });
            }
            at = upto;
        } else {
            // SAFETY: as `load_words`'.
            unsafe { AtomicU8::from_ptr(base.add(at)) }.store(from[at - offset], Relaxed);
            at += 1;
        }
    }
}

/// The `T` at `from` (any alignment).
///
/// # Safety
/// `from` is valid for reads of a `T`, and its bytes make a valid `T`.
#[cfg(test)]
#[inline]
pub unsafe fn read<T: Copy>(from: *const T) -> T {
    let mut value = MaybeUninit::<T>::uninit();
    // SAFETY: as promised; `load` sets every byte of `value`.
    unsafe {
        load(from.cast(), value.as_mut_ptr().cast(), size_of::<T>());
        value.assume_init()
    }
}

/// `value` written at `to` (any alignment).
///
/// # Safety
/// `to` is valid for writes of a `T`.
#[cfg(test)]
#[inline]
pub unsafe fn write<T: Copy>(to: *mut T, value: T) {
    // SAFETY: as promised; `value` is private.
    unsafe { store(to.cast(), (&raw const value).cast(), size_of::<T>()) }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every start and length against a plain copy, in both directions.
    #[test]
    fn copies_as_plain_memory() {
        let source: Vec<u8> = (0..64u8)
            .map(|b| b.wrapping_mul(37).wrapping_add(1))
            .collect();
        for start in 0..16 {
            for len in 0..40 {
                let mut out = vec![0xAAu8; 64];
                // SAFETY: both runs are inside their vectors.
                unsafe { load(source.as_ptr().add(start), out.as_mut_ptr().add(3), len) };
                assert_eq!(out[3..3 + len], source[start..start + len]);
                assert!(out[..3].iter().all(|&b| b == 0xAA));
                assert!(out[3 + len..].iter().all(|&b| b == 0xAA));
                let mut shared = vec![0xAAu8; 64];
                // SAFETY: as above.
                unsafe { store(shared.as_mut_ptr().add(start), source.as_ptr().add(3), len) };
                assert_eq!(shared[start..start + len], source[3..3 + len]);
                assert!(shared[..start].iter().all(|&b| b == 0xAA));
                assert!(shared[start + len..].iter().all(|&b| b == 0xAA));
            }
        }
    }

    #[test]
    fn overlapping_copies_as_memmove() {
        for (from, to) in [(0, 5), (5, 0), (2, 3), (3, 2), (0, 700)] {
            for len in [1usize, 7, 64, 600] {
                let mut mine: Vec<u8> = (0..1400u32).map(|n| (n % 251) as u8).collect();
                let mut plain = mine.clone();
                plain.copy_within(from..from + len, to);
                let at = mine.as_mut_ptr();
                // SAFETY: both runs are inside the vector.
                unsafe { copy(at.add(from), at.add(to), len) };
                assert_eq!(mine, plain);
            }
        }
    }

    #[test]
    fn fills_and_typed_reads() {
        let mut mine = vec![1u8; 1100];
        // SAFETY: inside the vector.
        unsafe { fill(mine.as_mut_ptr().add(3), 9, 1050) };
        assert!(mine[3..1053].iter().all(|&b| b == 9));
        assert_eq!((mine[2], mine[1053]), (1, 1));
        let at = mine.as_mut_ptr();
        for offset in 0..9 {
            // SAFETY: inside the vector; any bits make a `u32`.
            unsafe {
                write(at.add(offset).cast::<u32>(), 0x1234_5678);
                assert_eq!(read(at.add(offset).cast::<u32>()), 0x1234_5678);
            }
        }
    }

    /// A dword written whole is never seen half written (the word copies of a 4-aligned
    /// object); the writer stops even when the check fails.
    #[test]
    fn an_aligned_dword_stays_whole() {
        use std::sync::atomic::{AtomicBool, AtomicU32};
        let cell = [AtomicU32::new(0), AtomicU32::new(0)];
        let base = cell.as_ptr().cast::<u8>().cast_mut();
        let stop = AtomicBool::new(false);
        struct Stop<'a>(&'a AtomicBool);
        impl Drop for Stop<'_> {
            fn drop(&mut self) {
                self.0.store(true, Relaxed);
            }
        }
        std::thread::scope(|s| {
            let _stop = Stop(&stop);
            let base = base as usize;
            let stop = &stop;
            s.spawn(move || {
                let mut v = 0u32;
                while !stop.load(Relaxed) {
                    v = !v;
                    // SAFETY: the cells are alive, aligned, and reached only by word copies.
                    unsafe {
                        store_words(
                            base as *mut u8,
                            8,
                            0,
                            &[v.to_ne_bytes(), v.to_ne_bytes()].concat(),
                        );
                    }
                }
            });
            for _ in 0..if cfg!(miri) { 50 } else { 100_000 } {
                let mut seen = [0u8; 6];
                // SAFETY: as above.
                unsafe { load_words(base as *const u8, 8, 1, &mut seen) };
                let first = u32::from_ne_bytes([0, seen[0], seen[1], seen[2]]) >> 8;
                assert!(first == 0 || first == 0x00FF_FFFF, "{seen:?}");
            }
        });
    }

    #[test]
    fn word_copies_keep_the_other_bytes() {
        let mut object = [0u32; 3];
        let base = object.as_mut_ptr().cast::<u8>();
        // SAFETY: the array's own bytes, reached only by word copies here.
        unsafe {
            store_words(base, 11, 3, &[1, 2, 3, 4, 5, 6, 7, 8]);
            let mut back = [0u8; 11];
            load_words(base, 11, 0, &mut back);
            assert_eq!(back, [0, 0, 0, 1, 2, 3, 4, 5, 6, 7, 8]);
        }
    }
}
