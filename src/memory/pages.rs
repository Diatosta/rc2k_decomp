//! Memory from the system in whole pages, zeroed (phase 11, `notes/conversion.md`: it was
//! `VirtualAlloc` with MEM_COMMIT | MEM_RESERVE and `VirtualFree` with MEM_RELEASE), on
//! `std::alloc`. A block is lent to the game (`pointers::lend_system_block`) and known by
//! its game address; giving it back forgets it before the memory goes.

#![allow(unsafe_code)]

use std::alloc::Layout;
use std::collections::HashMap;
use std::sync::Mutex;

const PAGE: usize = 0x1000;

/// Each block's layout, by its game address.
static BLOCKS: Mutex<Option<HashMap<u32, (usize, Layout)>>> = Mutex::new(None);

/// `bytes` of zeroed memory in whole pages, lent to the game: its game address; None when
/// the system has none (or for 0 bytes).
pub fn take(bytes: usize) -> Option<u32> {
    let layout = Layout::from_size_align(bytes.checked_next_multiple_of(PAGE)?, PAGE).ok()?;
    if layout.size() == 0 {
        return None;
    }
    // SAFETY: a layout of a non-zero size.
    let block = unsafe { std::alloc::alloc_zeroed(layout) };
    if block.is_null() {
        return None;
    }
    let at = crate::pointers::lend_system_block(block, bytes);
    BLOCKS
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .get_or_insert_default()
        .insert(at, (block.expose_provenance(), layout));
    Some(at)
}

/// The block at game address `at` given back (its region and game range forgotten first);
/// an address `take` did not hand out is refused.
pub fn give_back(at: u32) -> Result<(), crate::errors::BadInput> {
    let (host, layout) = BLOCKS
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .get_or_insert_default()
        .remove(&at)
        .ok_or(crate::errors::BadInput(
            "memory given back that was not taken",
        ))?;
    crate::pointers::forget(at);
    // SAFETY: the block `take` made with this layout, no longer lent.
    unsafe { std::alloc::dealloc(std::ptr::with_exposed_provenance_mut(host), layout) };
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::pointers::GamePtr;

    #[test]
    fn blocks_are_zeroed_lent_and_given_back() {
        let at = super::take(5000).unwrap();
        assert_eq!(at % 0x1000, 0, "whole pages");
        assert_eq!(GamePtr::new(at + 4999).read::<u8>("test").unwrap(), 0);
        GamePtr::new(at + 8).write(7u32, "test").unwrap();
        super::give_back(at).unwrap();
        assert!(super::give_back(at).is_err());
        assert!(super::take(0).is_none());
    }
}
