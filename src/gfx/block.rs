//! Blocks of RAL.EXE's data that have left the image for memory Rust owns (phase 5,
//! `notes/conversion.md`): one contiguous IDA range per block, kept with the image's layout
//! (gaps and unreferenced bytes included, so a slip that reads past a variable reads what
//! the original read), its first values the image's bytes.
//!
//! A [`Slot`] is an IDA address in a block (`Slot::<Hud>::at(0x75C060)`), checked against the
//! block when it is made (at compile time for a constant). `game_read` / `game_write` /
//! `game_addr` / `game_ptr` here take a `Slot` or, as the framework's, an RVA of the image
//! (the other variables the same code reads): a name kept for a variable of a block cannot
//! reach the image's old, unchanging copy, because it is not a `usize`. Inline assembly gets
//! `base::<R>()` and the offset `IDA address - R::START`.
//!
//! Threads: whoever used the image's bytes keeps to its thread (a block is no more
//! synchronised than the image was).

use core::marker::PhantomData;
use core::ops::Add;

/// The bytes of a block, 16-aligned as the image's sections were.
#[repr(C, align(16))]
pub struct Bytes<const LEN: usize>(pub [u8; LEN]);

/// A block's first values: `(offset from its start, bytes)` runs in order, apart; the rest
/// is zero.
pub type Runs = &'static [(usize, &'static [u8])];

pub const fn initial<const LEN: usize>(runs: Runs) -> Bytes<LEN> {
    let mut bytes = [0u8; LEN];
    let mut r = 0;
    while r < runs.len() {
        let (offset, run) = runs[r];
        let mut k = 0;
        while k < run.len() {
            bytes[offset + k] = run[k];
            k += 1;
        }
        r += 1;
    }
    Bytes(bytes)
}

/// A moved block: the marker type of one `region!`.
pub trait Region: Copy {
    /// IDA's address of the block's first byte.
    const START: usize;
    /// IDA's address past the block's last byte.
    const END: usize;
    /// The block's address now.
    fn base() -> usize;
}

/// Defines a block: the marker type `$name`, the static `$store`, the first values `$runs`.
#[macro_export]
macro_rules! region {
    ($(#[$meta:meta])* $name:ident, $store:ident, $start:expr, $end:expr, $runs:expr) => {
        $(#[$meta])*
        #[derive(Clone, Copy, Debug)]
        pub struct $name;

        pub const LEN: usize = $end - $start;

        /// The block (its first values are the image's bytes).
        pub static $store: $crate::moved::Place<$crate::gfx::block::Bytes<LEN>> =
            $crate::moved::Place::new($crate::gfx::block::initial::<LEN>($runs));

        impl $crate::gfx::block::Region for $name {
            const START: usize = $start;
            const END: usize = $end;
            fn base() -> usize {
                $store.addr()
            }
        }
    };
}

/// A place in the block of `R`: the offset from its start.
pub struct Slot<R: Region>(usize, PhantomData<R>);

impl<R: Region> Clone for Slot<R> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<R: Region> Copy for Slot<R> {}

impl<R: Region> core::fmt::Debug for Slot<R> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Slot({:#x})", R::START + self.0)
    }
}

impl<R: Region> PartialEq for Slot<R> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<R: Region> Slot<R> {
    /// The place of IDA's address `ida`; one outside the block does not compile (a
    /// constant) or stops the program.
    pub const fn at(ida: usize) -> Self {
        assert!(
            R::START <= ida && ida < R::END,
            "not a variable of the block"
        );
        Slot(ida - R::START, PhantomData)
    }

    /// The place's address.
    pub fn address(self) -> usize {
        R::base() + self.0
    }
}

impl<R: Region> Add<usize> for Slot<R> {
    type Output = Slot<R>;
    fn add(self, n: usize) -> Slot<R> {
        Slot(self.0.wrapping_add(n), PhantomData)
    }
}

impl<R: Region> Add<u32> for Slot<R> {
    type Output = Slot<R>;
    fn add(self, n: u32) -> Slot<R> {
        Slot(self.0.wrapping_add(n as usize), PhantomData)
    }
}

impl<R: Region> Add<i32> for Slot<R> {
    type Output = Slot<R>;
    fn add(self, n: i32) -> Slot<R> {
        Slot(self.0.wrapping_add(n as usize), PhantomData)
    }
}

/// What `game_read` and the others take: a place in a block, or an RVA of the image (one
/// trait for every block kind: `crate::store::Where`).
pub use crate::store::Where;

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

/// The block's first values against the image (a test helper).
#[cfg(test)]
pub fn first_values_are_the_games<R: Region>(store: &crate::moved::Place<impl Sized>) {
    let _g = crate::store::test_lock();
    let len = R::END - R::START;
    // Read in place: a copy of a big block would not fit the test thread's stack.
    let first = unsafe { core::slice::from_raw_parts(store.ptr() as *const u8, len) };
    let image = crate::moved::image_bytes(R::START, len);
    assert!(
        first == &image[..],
        "the block's bytes differ from the image's"
    );
}
