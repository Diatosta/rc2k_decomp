//! `gfx::clear_picture` (0x4060D1): fills the picture buffer with zeros.
//!
//! In:  nothing
//! Out: every register preserved
//! 0x75300 dwords (800 x 600) at the buffer `dword_4F49DC` points at, whatever size the
//! picture in it has. Three callers, the picture loader `sub_411863` among them.
//! Refuses a null buffer pointer. The name is a reading.

use crate::errors::BadInput;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva, shadow};
use crate::gfx::picture::{PICTURE_DWORDS, picture, picture_region};
use crate::service::harness::{capture, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("gfx::clear_picture", rva(0x4060D1), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- Rust version ----

pub unsafe fn clear_picture() -> Result<(), BadInput> {
    let buffer = picture();
    if buffer.is_null() {
        return Err(BadInput("no picture buffer"));
    }
    unsafe { std::ptr::write_bytes(buffer, 0, PICTURE_DWORDS * 4) };
    Ok(())
}

// ---- hook ----

/// The number of the first byte of the buffer that is not zero.
fn first_not_zero() -> Option<usize> {
    unsafe { std::slice::from_raw_parts(picture(), PICTURE_DWORDS * 4) }
        .iter()
        .position(|&byte| byte != 0)
}

// Written out: the buffer is nearly two megabytes, and what the function leaves is known
// without running the Rust version beside it. In shadow mode the original runs and the
// buffer is checked for what the Rust version writes, zeros.
extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if !shadow() {
        if let Err(e) = unsafe { clear_picture() } {
            unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) };
        }
        return;
    }
    let mut o = *r;
    unsafe { call_original(&mut o) };
    let left = first_not_zero();
    if left.is_some() || !o.preserved_except(r, 0) {
        HOOK.mismatch(|| format!("the original left byte {left:?} of the buffer not zero"));
    }
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(_: &mut Regs) -> Result<(), crate::errors::Refused> {
    if let Err(e) = unsafe { clear_picture() } {
        return Err(e.into());
    }
    Ok(())
}

// ---- self-test: the game's own buffer, filled and put back ----

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    if picture().is_null() {
        return t.skip("the picture buffer is not allocated");
    }
    let regions = [picture_region(PICTURE_DWORDS * 4)];
    let saved = capture(&regions);
    for fill in [0xAAu8, 0x01] {
        let before = Regs::SENTINEL;
        unsafe { std::ptr::write_bytes(picture(), fill, PICTURE_DWORDS * 4) };
        let mut ro = before;
        unsafe { call_original(&mut ro) };
        let theirs = first_not_zero();
        unsafe { std::ptr::write_bytes(picture(), fill, PICTURE_DWORDS * 4) };
        let mine = unsafe { clear_picture() };
        let left = first_not_zero();
        let preserved = ro.preserved_except(&before, 0);
        t.check(
            mine.is_ok() && theirs.is_none() && left.is_none() && preserved,
            || {
                format!(
                    "a buffer full of {fill:#x}: original left byte {theirs:?} preserved={preserved}; rust {mine:?} left byte {left:?}"
                )
            },
        );
    }
    restore(&regions, &saved);
    t.finish()
}
