//! Memory from the system in whole pages, zeroed (phase 11, `notes/conversion.md`: it was
//! `VirtualAlloc` with MEM_COMMIT | MEM_RESERVE and `VirtualFree` with MEM_RELEASE), from
//! `pointers::pool`. A block is lent to the game (`pointers::lend_block`) and known by a
//! block id (a handle); giving it back forgets it, and the block returns to the pool.

use std::collections::HashMap;
use std::sync::Mutex;

/// The tag of a block id (top byte); the game's addresses are all below 0xC000_0000.
pub const BLOCK_TAG: u32 = 0xC800_0000;

/// What the table holds: the ids handed out so far, and each block lent: its game address
/// (the one place a block's number is an address) and its bytes.
struct Table {
    last: u32,
    lent: HashMap<u32, (u32, usize)>,
}

static BLOCKS: Mutex<Option<Table>> = Mutex::new(None);

fn with_table<R>(f: impl FnOnce(&mut Table) -> R) -> R {
    let mut guard = BLOCKS
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    f(guard.get_or_insert_with(|| Table {
        last: 0,
        lent: HashMap::new(),
    }))
}

/// `bytes` of zeroed memory in whole pages (a block of `pointers::pool`), lent to the game:
/// its block id (`BLOCK_TAG` + a count from 1, never an address); None for 0 bytes.
pub fn take(bytes: usize) -> Option<u32> {
    if bytes == 0 {
        return None;
    }
    let block = crate::pointers::pool::take(bytes);
    let at = crate::pointers::lend_block(block, bytes, true);
    Some(with_table(|t| {
        t.last += 1;
        let id = BLOCK_TAG | t.last;
        t.lent.insert(id, (at, bytes));
        id
    }))
}

/// The block `id` given back (its region and game range forgotten, the block back in the
/// pool); an id `take` did not hand out is refused.
pub fn give_back(id: u32) -> Result<(), crate::errors::BadInput> {
    let (at, _) = with_table(|t| t.lent.remove(&id)).ok_or(crate::errors::BadInput(
        "memory given back that was not taken",
    ))?;
    if let Some(block) = crate::pointers::forget_block(at) {
        crate::pointers::pool::give_back(block);
    }
    Ok(())
}

/// The block `take` handed out as `id`, as a place bounded to its bytes (its header, a
/// list's link): the one resolver of a block id to a game address. An id `take` did not
/// hand out is refused.
pub fn block(id: u32) -> Result<crate::pointers::GamePtr, crate::errors::BadInput> {
    let (at, bytes) = with_table(|t| t.lent.get(&id).copied())
        .ok_or(crate::errors::BadInput("a block that was not taken"))?;
    Ok(crate::pointers::GamePtr::new(at).limited(bytes as u32))
}

#[cfg(test)]
mod tests {
    #[test]
    fn blocks_are_zeroed_lent_and_given_back() {
        let at = super::take(5000).unwrap();
        assert_eq!(at & 0xFF00_0000, super::BLOCK_TAG, "an id, not an address");
        let block = super::block(at).unwrap();
        assert_eq!(block.addr() % 0x1000, 0, "whole pages");
        assert_eq!(block.offset(4999).read::<u8>("test").unwrap(), 0);
        assert!(
            block.offset(5000).read::<u8>("test").is_err(),
            "past its bytes"
        );
        block.offset(8).write(7u32, "test").unwrap();
        assert!(super::block(at + 8).is_err(), "not a block's start");
        super::give_back(at).unwrap();
        assert!(super::give_back(at).is_err());
        assert!(super::block(at).is_err(), "given back");
        assert!(super::take(0).is_none());
    }
}
