//! Memory from the system in whole pages, zeroed (phase 11, `notes/conversion.md`: it was
//! `VirtualAlloc` with MEM_COMMIT | MEM_RESERVE and `VirtualFree` with MEM_RELEASE), from
//! `pointers::pool`. A block is lent to the game (`pointers::lend_block`) and known by its
//! game address; giving it back forgets it, and the block returns to the pool.

use std::collections::HashSet;
use std::sync::Mutex;

/// Each block lent, by its game address.
static BLOCKS: Mutex<Option<HashSet<u32>>> = Mutex::new(None);

/// `bytes` of zeroed memory in whole pages (a block of `pointers::pool`), lent to the game:
/// its game address; None for 0 bytes.
pub fn take(bytes: usize) -> Option<u32> {
    if bytes == 0 {
        return None;
    }
    let block = crate::pointers::pool::take(bytes);
    let at = crate::pointers::lend_block(block, bytes, true);
    BLOCKS
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .get_or_insert_default()
        .insert(at);
    Some(at)
}

/// The block at game address `at` given back (its region and game range forgotten, the block
/// back in the pool); an address `take` did not hand out is refused.
pub fn give_back(at: u32) -> Result<(), crate::errors::BadInput> {
    BLOCKS
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .get_or_insert_default()
        .remove(&at)
        .then_some(())
        .ok_or(crate::errors::BadInput(
            "memory given back that was not taken",
        ))?;
    if let Some(block) = crate::pointers::forget_block(at) {
        crate::pointers::pool::give_back(block);
    }
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
