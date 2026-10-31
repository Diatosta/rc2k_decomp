//! Copies in and out of memory the game's threads share. The game keeps its state in memory
//! every thread reaches, so a plain or volatile copy that meets another thread's write is a
//! data race: undefined behaviour in Rust (found by ThreadSanitizer: the key table, the sound
//! options). These copy through relaxed atomics, the same moves on the processor, widest
//! first on aligned places, so an aligned dword stays whole as it did in the original.

use std::mem::MaybeUninit;
#[cfg(target_pointer_width = "64")]
use std::sync::atomic::AtomicU32;
use std::sync::atomic::Ordering::Relaxed;
use std::sync::atomic::{AtomicU8, AtomicU16, AtomicUsize};

/// Calls `$step!(atomic, integer)` over `$len` bytes from `$at`, each on an aligned place.
macro_rules! walk {
    ($at:expr, $len:expr, $n:ident, $step:ident) => {
        let aligned = |n: usize, width: usize| ($at as usize + n).is_multiple_of(width);
        if $len - $n >= 1 && !aligned($n, 2) {
            $step!(AtomicU8, u8);
        }
        if $len - $n >= 2 && !aligned($n, 4) {
            $step!(AtomicU16, u16);
        }
        #[cfg(target_pointer_width = "64")]
        if $len - $n >= 4 && !aligned($n, 8) {
            $step!(AtomicU32, u32);
        }
        while $len - $n >= size_of::<usize>() {
            $step!(AtomicUsize, usize);
        }
        #[cfg(target_pointer_width = "64")]
        if $len - $n >= 4 {
            $step!(AtomicU32, u32);
        }
        if $len - $n >= 2 {
            $step!(AtomicU16, u16);
        }
        if $len - $n >= 1 {
            $step!(AtomicU8, u8);
        }
    };
}

/// `len` shared bytes at `from` copied to the private `to`.
///
/// # Safety
/// `from` is valid for reads and `to` for writes of `len` bytes, and they do not overlap.
#[inline]
// `walk`'s last step leaves its count unread.
#[allow(unused_assignments)]
pub unsafe fn load(from: *const u8, to: *mut u8, len: usize) {
    let mut n = 0;
    macro_rules! step {
        ($atomic:ident, $int:ty) => {
            // SAFETY: inside the run and aligned (`walk`); only atomics touch shared memory.
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
/// `from` is valid for reads and `to` for writes of `len` bytes, and they do not overlap.
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

/// The `T` at `from` (any alignment).
///
/// # Safety
/// `from` is valid for reads of a `T`, and its bytes make a valid `T`.
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

    /// A dword written whole is never seen half written (aligned dwords in a copy).
    #[test]
    fn an_aligned_dword_stays_whole() {
        use std::sync::atomic::{AtomicBool, AtomicU32};
        let cell = AtomicU32::new(0);
        let stop = AtomicBool::new(false);
        std::thread::scope(|s| {
            s.spawn(|| {
                let mut v = 0u32;
                while !stop.load(Relaxed) {
                    v = !v;
                    // SAFETY: the cell is alive and aligned.
                    unsafe { write(cell.as_ptr(), v) };
                }
            });
            for _ in 0..if cfg!(miri) { 50 } else { 100_000 } {
                // SAFETY: as above.
                let seen = unsafe { read(cell.as_ptr()) };
                assert!(seen == 0 || seen == u32::MAX);
            }
            stop.store(true, Relaxed);
        });
    }
}
