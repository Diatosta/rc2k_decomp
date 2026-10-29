//! Phase 5: a region of RAL.EXE's variables kept in one block of Rust-owned memory with the
//! image's layout (`effects::store` was the first; this is the same pattern for the audio,
//! sound and input modules).
//!
//! A block (`block!`) is a byte array whose first values are the image's bytes (generated
//! runs of the non-zero ones); its variables are typed overlays (phase 7) or named by IDA
//! address through `moved::place`. The functions here take a typed cell or, as the
//! framework's, an RVA of the image. (The checked `Slot` places went once the last user was
//! typed, 2026-10-25.)
//!
//! The block keeps gaps and unreferenced bytes, so a slip that reads past a variable reads
//! what the original read. 16-byte alignment holds relative to IDA's address (the block
//! starts `START % 16` bytes into an aligned array).
//!
//! Threads: whoever wrote under the original's discipline (the audio thread's turn, the
//! key thread) still does; the reads and writes are volatile, as the image's were.

use crate::framework;

/// A block's description: where its first byte now is.
pub trait Block: Copy {
    /// The address of the byte IDA calls `START`.
    fn base() -> usize;
}

/// What `game_read` and the others take: an RVA of the image, or a typed cell.
pub trait Where: Copy {
    fn address(self) -> usize;
}

impl Where for usize {
    fn address(self) -> usize {
        framework::game_addr(self) as usize
    }
}

/// A variable named by IDA's address (a plain `usize`: where it lives now, `moved::place`) or
/// a typed cell: what the helpers of modules whose variables are named by IDA's addresses
/// (`g(0x5165F0)`) take.
pub trait Ida: Copy {
    fn here(self) -> usize;
}

impl Ida for usize {
    fn here(self) -> usize {
        crate::moved::place(self)
    }
}

/// A variable named by IDA's address in the image (a plain `usize`, no lookup: for the hot
/// helpers of the physics, race and drive modules).
pub trait At: Copy {
    fn here(self) -> usize;
}

impl At for usize {
    fn here(self) -> usize {
        (framework::game_addr(framework::rva(self))) as usize
    }
}

/// A typed cell (phase 7) is its own place: the helpers that take a variable by `Where`,
/// `Ida` or `At` (`session.rs`) take the typed field as well, so a module's call sites move to fields one
/// by one.
macro_rules! typed_places {
    ($($t:ty),*) => {$(
        impl Where for &'static $t {
            fn address(self) -> usize {
                self.addr() as usize
            }
        }
        impl Ida for &'static $t {
            fn here(self) -> usize {
                self.addr() as usize
            }
        }
    )*};
}

typed_places!(crate::session::Dword, crate::session::Float);

/// Reads the variable named by `at` (an IDA address of the image).
pub unsafe fn at_read<T: Copy>(at: impl At) -> T {
    let p = crate::pointers::host_at(at.here()) as *const T;
    unsafe {
        if p.is_aligned() {
            p.read_volatile()
        } else {
            p.read_unaligned()
        }
    }
}

/// Writes the variable named by `at` (an IDA address of the image).
pub unsafe fn at_write<T: Copy>(at: impl At, value: T) {
    let p = crate::pointers::host_at(at.here()) as *mut T;
    unsafe {
        if p.is_aligned() {
            p.write_volatile(value)
        } else {
            p.write_unaligned(value)
        }
    }
}

/// Reads the variable named by `at` (an IDA address, or a typed cell).
pub unsafe fn ida_read<T: Copy>(at: impl Ida) -> T {
    let p = crate::pointers::host_at(at.here()) as *const T;
    unsafe {
        if p.is_aligned() {
            p.read_volatile()
        } else {
            p.read_unaligned()
        }
    }
}

/// The address of `place`.
pub fn game_addr(place: impl Where) -> u32 {
    place.address() as u32
}

/// Pointer to the game's memory at `place`.
pub fn game_ptr<T>(place: impl Where) -> *mut T {
    crate::pointers::host_at(place.address()) as *mut T
}

/// Reads a variable (`framework::game_read`, over a block or the image).
pub fn game_read<T: Copy>(place: impl Where) -> T {
    let p = game_ptr::<T>(place);
    crate::pointers::check_or_end(
        crate::pointers::game_address(p),
        size_of::<T>(),
        "a game variable",
    );
    // SAFETY: inside one live region (checked above).
    unsafe {
        if p.is_aligned() {
            p.read_volatile()
        } else {
            p.read_unaligned()
        }
    }
}

/// Writes a variable (`framework::game_write`, over a block or the image).
pub fn game_write<T: Copy>(place: impl Where, value: T) {
    let p = game_ptr::<T>(place);
    crate::pointers::check_or_end(
        crate::pointers::game_address(p),
        size_of::<T>(),
        "a game variable",
    );
    // SAFETY: inside one live region (checked above).
    unsafe {
        if p.is_aligned() {
            p.write_volatile(value)
        } else {
            p.write_unaligned(value)
        }
    }
}

/// Declares a block: `block!(pub mod name { start: IDA, end: IDA, runs: PATH });` makes
/// `name::{START, END, LEN, STORE, base(), from_ida(ida)}`; `PATH` is the generated
/// `&[(offset, &[u8])]` of the non-zero runs of the image's bytes.
#[macro_export]
macro_rules! block {
    ($vis:vis mod $name:ident { start: $start:expr, end: $end:expr, runs: $runs:expr $(,)? }) => {
        $vis mod $name {
            use $crate::moved::Place;

            /// IDA's address of the block's first byte.
            pub const START: usize = $start;
            /// IDA's address past the block's last byte.
            pub const END: usize = $end;
            pub const LEN: usize = END - START;
            const PAD: usize = START % 16;

            #[repr(C, align(16))]
            pub struct Store([u8; LEN + PAD]);

            const fn initial() -> Store {
                let runs: &[(usize, &[u8])] = $runs;
                let mut bytes = [0u8; LEN + PAD];
                let mut r = 0;
                while r < runs.len() {
                    let (offset, run) = runs[r];
                    let mut k = 0;
                    while k < run.len() {
                        bytes[PAD + offset + k] = run[k];
                        k += 1;
                    }
                    r += 1;
                }
                Store(bytes)
            }

            /// The block (its first values are the image's bytes).
            pub static STORE: Place<Store> = Place::new(initial());

            #[derive(Clone, Copy)]
            pub struct Zone;

            impl $crate::store::Block for Zone {
                fn base() -> usize {
                    (STORE.addr() + PAD as u32) as usize
                }
            }

            /// The address of IDA's first byte of the block.
            pub fn base() -> usize {
                <Zone as $crate::store::Block>::base()
            }

            #[cfg(test)]
            mod tests {
                use super::*;

                #[test]
                fn first_values_are_the_games() {
                    let _g = $crate::store::test_lock();
                    // Read in place: a copy of the block would not fit the test thread's stack.
                    let all = unsafe { &(&(*STORE.ptr()).0)[..] };
                    let image = $crate::moved::image_bytes(START, LEN);
                    assert!(
                        all[PAD..] == image[..],
                        "the block's bytes differ from the image's"
                    );
                }
            }
        }
    };
}

/// The lock every test holds that writes the game's statics (and restores them), or reads
/// their first values: tests run in parallel threads.
#[cfg(test)]
pub fn test_lock() -> std::sync::MutexGuard<'static, ()> {
    static LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());
    LOCK.lock().unwrap_or_else(|e| e.into_inner())
}
