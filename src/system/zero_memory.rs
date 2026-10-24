//! `system::zero_memory` (0x401829): fills a block of memory with zeros.
//!
//! In:  ebx = the block, ecx = its length in bytes
//! Out: every register preserved; the flags of `and`-ing the length with 3
//! 40 callers. Shadow mode runs the original alone and checks the block for zeros.
//! Refuses a null block of a length other than 0. The name is a reading.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::BadInput;

// The original's last instruction that sets flags is `and ebx, 3` on the length.
// ---- Rust version ----

pub unsafe fn zero_memory(block: *mut u8, length: u32) -> Result<(), BadInput> {
    if length == 0 {
        return Ok(());
    }
    if block.is_null() {
        return Err(BadInput("no block to zero"));
    }
    unsafe { std::ptr::write_bytes(block, 0, length as usize) };
    Ok(())
}
