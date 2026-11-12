//! `memory::take_from_list` (0x402956): hands out the next bytes of a list's newest block.
//!
//! Takes: bytes, the list (`MemoryList`, after `memory::rewind_list`)
//! Returns their address, or none (the original's carry flag) if the list has fewer bytes
//! left.
//! The list's cursor moves on and what remains is counted down. 12 call sites: the lexicon
//! compiler and the sprite and font builder cut their work memory this way.
//! The name is a reading.

use crate::cells::{load_value, store_value};
use crate::errors::BadInput;
use crate::memory::MemoryList;

/// The address of the bytes, or None if the list has not that many left.
pub fn take_from_list(list: &dyn crate::cells::Store, bytes: u32) -> Result<Option<u32>, BadInput> {
    let mut l = load_value::<MemoryList>(list, 0)?;
    let taken = l.take(bytes);
    store_value(list, 0, l)?;
    Ok(taken)
}

pub fn rewind_list(list: &dyn crate::cells::Store) -> Result<(), BadInput> {
    let mut l = load_value::<MemoryList>(list, 0)?;
    l.rewind();
    store_value(list, 0, l)
}
