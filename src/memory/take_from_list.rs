//! Hands out the next bytes of a list's newest block (a `MemoryList`, after
//! `memory::rewind_list`): their place, or none if the list has fewer bytes left. The list's
//! cursor moves on and what remains is counted down. The lexicon compiler and the sprite and
//! font builder cut their work memory this way.

use crate::errors::BadInput;
use crate::memory::HeldList;

/// The address of the bytes, or None if the list has not that many left.
pub fn take_from_list(list: &impl HeldList, bytes: u32) -> Result<Option<u32>, BadInput> {
    let mut l = list.get();
    let taken = l.take(bytes);
    list.set(l);
    Ok(taken)
}

pub fn rewind_list(list: &impl HeldList) -> Result<(), BadInput> {
    let mut l = list.get();
    l.rewind();
    list.set(l);
    Ok(())
}
