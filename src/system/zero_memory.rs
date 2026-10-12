//! `system::zero_memory` (0x401829): fills a block of memory with zeros.
//!
//! In:  ebx = the block, ecx = its length in bytes
//! Out: every register preserved; the flags of `and`-ing the length with 3
//! 40 callers. Shadow mode runs the original alone and checks the block for zeros.
//! Refuses a null block of a length other than 0. The name is a reading.

use crate::errors::BadInput;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva, shadow};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("system::zero_memory", rva(0x401829), thunk, &ORIGINAL).self_test(self_test);

// The original's last instruction that sets flags is `and ebx, 3` on the length.
hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL,
    epilogue = "test ecx, 3"
);

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

// ---- hook ----

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    let (block, length) = (r.ebx as *mut u8, r.ecx);
    if !shadow() {
        if let Err(e) = unsafe { zero_memory(block, length) } {
            unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) };
        }
        return;
    }
    let mut o = *r;
    unsafe { call_original(&mut o) };
    let left = (length != 0)
        .then(|| unsafe { std::slice::from_raw_parts(block, length as usize) })
        .and_then(|bytes| bytes.iter().position(|&byte| byte != 0));
    if left.is_some() || !o.preserved_except(r, 0) {
        HOOK.mismatch(|| {
            format!(
                "block={:#x} length={length}: the original left byte {left:?} not zero",
                r.ebx
            )
        });
    }
}

// ---- self-test ----

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    for length in [0usize, 1, 2, 3, 4, 5, 7, 8, 9, 64, 67, 0x80, 0x301] {
        for offset in [0usize, 1, 3] {
            let mut theirs = vec![0xAAu8; length + 16];
            let mut ours = theirs.clone();
            let before = Regs {
                ebx: theirs[offset..].as_mut_ptr() as u32,
                ecx: length as u32,
                ..Regs::SENTINEL
            };
            let mut ro = before;
            unsafe { call_original(&mut ro) };
            let mine = unsafe { zero_memory(ours[offset..].as_mut_ptr(), length as u32) };
            let preserved = ro.preserved_except(&before, 0);
            t.check(mine.is_ok() && theirs == ours && preserved, || {
                format!(
                    "length={length} offset={offset}: preserved={preserved} rust {mine:?} same bytes={}",
                    theirs == ours
                )
            });
        }
    }
    t.finish()
}
