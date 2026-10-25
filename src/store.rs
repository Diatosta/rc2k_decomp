//! Phase 5: a region of RAL.EXE's variables kept in one block of Rust-owned memory with the
//! image's layout (`marks::store` was the first; this is the same pattern for the audio,
//! sound and input modules).
//!
//! A block (`block!`) is a byte array whose first values are the image's bytes (generated
//! runs of the non-zero ones) and which a `Slot` addresses: a `Slot` is an IDA address
//! checked against its block when it is written, so a name the code keeps for a variable of
//! the block cannot reach the image's old, unchanging copy (it is not a `usize`). The
//! functions here take a `Slot` or, as the framework's, an RVA of the image.
//!
//! The block keeps gaps and unreferenced bytes, so a slip that reads past a variable reads
//! what the original read. 16-byte alignment holds relative to IDA's address (the block
//! starts `START % 16` bytes into an aligned array).
//!
//! Threads: whoever wrote under the original's discipline (the audio thread's turn, the
//! key thread) still does; the reads and writes are volatile, as the image's were.

use crate::framework;
use core::marker::PhantomData;
use core::ops::Add;

/// A block's description: IDA's address range and where its first byte now is.
pub trait Block: Copy {
    const START: usize;
    const END: usize;
    /// The address of the byte IDA calls `START`.
    fn base() -> usize;
}

/// A place in a block: the offset from its start.
pub struct Slot<B: Block>(usize, PhantomData<B>);

impl<B: Block> Clone for Slot<B> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<B: Block> Copy for Slot<B> {}

impl<B: Block> core::fmt::Debug for Slot<B> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Slot({:#x})", B::START.wrapping_add(self.0))
    }
}

impl<B: Block> PartialEq for Slot<B> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<B: Block> Slot<B> {
    /// The place of IDA's address `ida`; one outside the block does not compile (a
    /// constant) or stops the program.
    pub const fn at(ida: usize) -> Self {
        assert!(
            B::START <= ida && ida < B::END,
            "not a variable of the block"
        );
        Slot(ida - B::START, PhantomData)
    }

    /// The place's address.
    pub fn address(self) -> usize {
        B::base().wrapping_add(self.0)
    }
}

impl<B: Block> Add<usize> for Slot<B> {
    type Output = Slot<B>;
    fn add(self, n: usize) -> Slot<B> {
        Slot(self.0.wrapping_add(n), PhantomData)
    }
}

impl<B: Block> Add<u32> for Slot<B> {
    type Output = Slot<B>;
    fn add(self, n: u32) -> Slot<B> {
        Slot(self.0.wrapping_add(n as usize), PhantomData)
    }
}

impl<B: Block> Add<i32> for Slot<B> {
    type Output = Slot<B>;
    fn add(self, n: i32) -> Slot<B> {
        Slot(self.0.wrapping_add(n as usize), PhantomData)
    }
}

/// What `game_read` and the others take: a place in a block, or an RVA of the image.
pub trait Where: Copy {
    fn address(self) -> usize;
}

impl Where for usize {
    fn address(self) -> usize {
        framework::game_addr(self)
    }
}

impl<B: Block> Where for Slot<B> {
    fn address(self) -> usize {
        Slot::address(self)
    }
}

/// A variable named by IDA's address (a plain `usize`: where it lives now, `moved::place`) or
/// by a place of a block (checked): what the helpers of modules whose variables are named
/// by IDA's addresses (`g(0x5165F0)`) take, so that a converted one is a `Slot`.
pub trait Ida: Copy {
    fn here(self) -> usize;
}

impl Ida for usize {
    fn here(self) -> usize {
        crate::moved::place(self)
    }
}

impl<B: Block> Ida for Slot<B> {
    fn here(self) -> usize {
        Slot::address(self)
    }
}

/// A variable named by IDA's address in the image (a plain `usize`, no lookup: for the hot
/// helpers of the physics, race and drive modules) or by a place of a block (checked).
pub trait At: Copy {
    fn here(self) -> usize;
}

impl At for usize {
    fn here(self) -> usize {
        framework::game_addr(framework::rva(self))
    }
}

impl<B: Block> At for Slot<B> {
    fn here(self) -> usize {
        Slot::address(self)
    }
}

/// Reads the variable named by `at` (an IDA address of the image, or a `Slot`).
pub unsafe fn at_read<T: Copy>(at: impl At) -> T {
    let p = at.here() as *const T;
    unsafe {
        if p.is_aligned() {
            p.read_volatile()
        } else {
            p.read_unaligned()
        }
    }
}

/// Writes the variable named by `at` (an IDA address of the image, or a `Slot`).
pub unsafe fn at_write<T: Copy>(at: impl At, value: T) {
    let p = at.here() as *mut T;
    unsafe {
        if p.is_aligned() {
            p.write_volatile(value)
        } else {
            p.write_unaligned(value)
        }
    }
}

/// Reads the variable named by `at` (an IDA address or a `Slot`).
pub unsafe fn ida_read<T: Copy>(at: impl Ida) -> T {
    let p = at.here() as *const T;
    unsafe {
        if p.is_aligned() {
            p.read_volatile()
        } else {
            p.read_unaligned()
        }
    }
}

/// Writes the variable named by `at` (an IDA address or a `Slot`).
pub unsafe fn ida_write<T: Copy>(at: impl Ida, value: T) {
    let p = at.here() as *mut T;
    unsafe {
        if p.is_aligned() {
            p.write_volatile(value)
        } else {
            p.write_unaligned(value)
        }
    }
}

/// The address of `place`.
pub fn game_addr(place: impl Where) -> usize {
    place.address()
}

/// Pointer to the game's memory at `place`.
pub fn game_ptr<T>(place: impl Where) -> *mut T {
    place.address() as *mut T
}

/// Reads a variable (`framework::game_read`, over a block or the image).
pub unsafe fn game_read<T: Copy>(place: impl Where) -> T {
    let p = game_ptr::<T>(place);
    unsafe {
        if p.is_aligned() {
            p.read_volatile()
        } else {
            p.read_unaligned()
        }
    }
}

/// Writes a variable (`framework::game_write`, over a block or the image).
pub unsafe fn game_write<T: Copy>(place: impl Where, value: T) {
    let p = game_ptr::<T>(place);
    unsafe {
        if p.is_aligned() {
            p.write_volatile(value)
        } else {
            p.write_unaligned(value)
        }
    }
}

/// Declares a block: `block!(pub mod name { start: IDA, end: IDA, runs: PATH });` makes
/// `name::{START, END, LEN, STORE, Slot, base(), from_ida(ida)}`; `PATH` is the generated
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
                const START: usize = START;
                const END: usize = END;
                fn base() -> usize {
                    STORE.addr() + PAD
                }
            }

            /// A place of the block, checked when it is made.
            #[allow(dead_code)] // the code of some blocks still names them by IDA's address only
            pub type Slot = $crate::store::Slot<Zone>;

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
