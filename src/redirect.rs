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
        unsafe { Self::with_bytes(original, len, &[(0, entry_bytes)], call_offsets, redirect) }
    }

    /// `new` with the original bytes put back at several places: the entry, and the first
    /// bytes of a function inside the copy that is hooked itself (its patch is in memory;
    /// 2026-10-20: `run_particles`, whose 0x221 starts a hooked function). `call_offsets`
    /// may name `E8` calls and `E9` tail jumps.
    pub unsafe fn with_bytes(
        original: usize,
        len: usize,
        bytes: &[(usize, &[u8])],
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
            for &(offset, b) in bytes {
                if offset + b.len() > len {
                    return Err("bytes to put back past the copy");
                }
                std::ptr::copy_nonoverlapping(b.as_ptr(), copy.add(offset), b.len());
            }
            if let Some(offset) = unlisted_call(
                std::slice::from_raw_parts(copy, len),
                original,
                call_offsets,
            ) {
                // A call out of the copy that is not moved runs into whatever lies at the
                // same distance from the copy (2026-10-18: `draw_cell_sprite`'s test).
                return Err(Box::leak(
                    format!("a call out of the copy at +{offset:#x} that is not in its list")
                        .into_boxed_str(),
                ));
            }
            for &offset in call_offsets {
                let at = copy.add(offset);
                if offset + 5 > len || !matches!(at.read(), 0xE8 | 0xE9) {
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

    /// Points a `rel32` operand of the copy (a near jump out of the function, say) at
    /// `new_target`. `rel_offset` is the offset of the four operand bytes.
    pub unsafe fn retarget(&self, rel_offset: usize, new_target: usize) {
        unsafe {
            let at = self.0.add(rel_offset);
            let rel = new_target.wrapping_sub(at as usize + 4) as i32;
            (at as *mut i32).write_unaligned(rel);
        }
    }

    /// Replaces four bytes of the copy: the address operand of an instruction that reads or
    /// writes a game variable, to point it at a test variable instead.
    pub unsafe fn patch_u32(&self, offset: usize, value: u32) {
        unsafe { (self.0.add(offset) as *mut u32).write_unaligned(value) };
    }

    pub fn addr(&self) -> usize {
        self.0 as usize
    }
}

/// The first `call rel32` of `code` (the bytes of `original`, `len` long) whose target is
/// outside it and whose offset `listed` lacks.
fn unlisted_call(code: &[u8], original: usize, listed: &[usize]) -> Option<usize> {
    use iced_x86::{Decoder, DecoderOptions, FlowControl, OpKind};
    let end = original + code.len();
    let mut decoder = Decoder::with_ip(32, code, original as u64, DecoderOptions::NONE);
    while decoder.can_decode() {
        let insn = decoder.decode();
        let offset = insn.ip() as usize - original;
        if insn.flow_control() == FlowControl::Call
            && insn.op0_kind() == OpKind::NearBranch32
            && code.get(offset) == Some(&0xE8)
        {
            let target = insn.near_branch_target() as usize;
            if !(original..end).contains(&target) && !listed.contains(&offset) {
                return Some(offset);
            }
        }
    }
    None
}

impl Drop for RedirectedCopy {
    fn drop(&mut self) {
        unsafe { VirtualFree(self.0.cast(), 0, MEM_RELEASE) };
    }
}
