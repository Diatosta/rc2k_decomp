//! An executable copy of an original function with some of its calls pointed elsewhere, for
//! self-tests of functions that cannot be run as they are: ones that run frames, wait on the
//! clock, or save the game. The calls that are not redirected go where they always went.

use crate::framework::game_addr;
use windows_sys::Win32::System::Memory::{
    MEM_COMMIT, MEM_RELEASE, MEM_RESERVE, PAGE_EXECUTE_READWRITE, VirtualAlloc, VirtualFree,
};

/// Freed when dropped.
pub struct RedirectedCopy(*mut u8);

impl RedirectedCopy {
    /// `original` is the function's address and `len` its size; it must be one contiguous
    /// piece with no jumps out of it. `entry_bytes` are its first bytes as they are in the
    /// image, which the hook's jump has overwritten in memory. `call_offsets` lists every
    /// `call rel32` (the E8 byte). `redirect` gets a call's target as an RVA and returns the
    /// address to call instead, or None to leave the call as it is.
    pub unsafe fn new(
        original: usize,
        len: usize,
        entry_bytes: &[u8],
        call_offsets: &[usize],
        redirect: &dyn Fn(usize) -> Option<usize>,
    ) -> Result<Self, &'static str> {
        unsafe {
            let copy = VirtualAlloc(
                std::ptr::null(),
                len,
                MEM_COMMIT | MEM_RESERVE,
                PAGE_EXECUTE_READWRITE,
            ) as *mut u8;
            if copy.is_null() {
                return Err("VirtualAlloc failed");
            }
            let this = Self(copy);
            std::ptr::copy_nonoverlapping(original as *const u8, copy, len);
            std::ptr::copy_nonoverlapping(entry_bytes.as_ptr(), copy, entry_bytes.len());
            for &offset in call_offsets {
                let at = copy.add(offset);
                if offset + 5 > len || at.read() != 0xE8 {
                    return Err("the function is not what the call list was made from");
                }
                let rel = (at.add(1) as *const i32).read_unaligned();
                let target = (original + offset + 5).wrapping_add(rel as usize);
                let new_target = redirect(target.wrapping_sub(game_addr(0))).unwrap_or(target);
                let new_rel = new_target.wrapping_sub(copy as usize + offset + 5) as i32;
                (at.add(1) as *mut i32).write_unaligned(new_rel);
            }
            Ok(this)
        }
    }

    pub fn addr(&self) -> usize {
        self.0 as usize
    }
}

impl Drop for RedirectedCopy {
    fn drop(&mut self) {
        unsafe { VirtualFree(self.0.cast(), 0, MEM_RELEASE) };
    }
}
