//! A region of RAL.EXE's variables kept in one block of Rust-owned memory with the image's
//! layout (`effects::store` was the first; the audio, sound and input modules follow the
//! same pattern).
//!
//! A block (`block!`) is a byte array whose first values are the image's bytes (generated
//! runs of the non-zero ones); its variables are typed overlays or named by IDA address
//! through `moved::place`. The functions here take a typed cell or an RVA of the image.
//!
//! The block keeps gaps and unreferenced bytes, so a slip that reads past a variable reads
//! what the original read. 16-byte alignment holds relative to IDA's address (the block
//! starts `START % 16` bytes into an aligned array).
//!
//! Threads: whoever wrote under the original's discipline (the audio thread's turn, the
//! key thread) still does; the reads and writes are volatile, as the image's were.

#![allow(unsafe_code)]

/// A block's description: where its first byte now is.
pub trait Block: Copy {
    /// The address of the byte IDA calls `START`.
    fn base() -> usize;
}

/// A range of a block's plain shared bytes as a store (`pointers::Store`): `len` bytes from
/// `offset` in the block, whose first IDA byte `first` names. Read and written through
/// `shared_memory`, the discipline of every `Bytes`.
pub struct BlockRange {
    first: fn() -> *mut u8,
    offset: usize,
    len: usize,
}

// SAFETY: raw access only, under the original's discipline (as `FixedBlock`).
unsafe impl Sync for BlockRange {}

impl BlockRange {
    pub const fn new(first: fn() -> *mut u8, offset: usize, len: usize) -> Self {
        Self { first, offset, len }
    }

    fn at(&self, offset: usize, len: usize) -> Result<*mut u8, crate::errors::BadInput> {
        offset
            .checked_add(len)
            .filter(|&end| end <= self.len)
            .ok_or(crate::errors::BadInput("past the end of a block range"))?;
        Ok((self.first)().wrapping_add(self.offset + offset))
    }
}

// SAFETY: `first` and `size` name a range of the block's plain shared bytes.
unsafe impl crate::pointers::Store for BlockRange {
    fn size(&self) -> usize {
        self.len
    }

    fn first(&self) -> *const u8 {
        (self.first)().wrapping_add(self.offset).cast_const()
    }

    fn load(&self, offset: usize, to: &mut [u8]) -> Result<(), crate::errors::BadInput> {
        let from = self.at(offset, to.len())?;
        // SAFETY: the run lies inside the block's bytes (`at`, the range lies in the block);
        // shared bytes read through `shared_memory`; `to` is the caller's.
        unsafe { crate::shared_memory::load(from, to.as_mut_ptr(), to.len()) };
        Ok(())
    }

    fn store(&self, offset: usize, from: &[u8]) -> Result<(), crate::errors::BadInput> {
        let to = self.at(offset, from.len())?;
        // SAFETY: as `load`'s.
        unsafe { crate::shared_memory::store(to, from.as_ptr(), from.len()) };
        Ok(())
    }
}

/// Declares a block: `block!(pub mod name { start: IDA, end: IDA, runs: PATH });` makes
/// `name::{START, END, LEN, STORE, base(), from_ida(ida)}`; `PATH` is the generated
/// `&[(offset, &[u8])]` of the non-zero runs of the image's bytes.
#[macro_export]
macro_rules! block {
    ($vis:vis mod $name:ident { start: $start:expr, end: $end:expr, runs: $runs:expr $(,)? }) => {
        $vis mod $name {
            use $crate::moved::FixedBlock;

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
            pub static STORE: FixedBlock<Store> = FixedBlock::new(initial());

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
    LOCK.lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}
