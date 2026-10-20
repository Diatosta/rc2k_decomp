//! `strings::bin_append` (0x40201D): appends the low `digits` bits of a value as '0'/'1',
//! MSB first, without a NUL.
//!
//! In:  eax = value, ecx = digits (0..=32), ebx = cursor
//! Out: ebx = new cursor; everything else preserved
//! More than 32 digits makes the original read below its stack buffer: refused, and the hook
//! leaves that to the original.
//! Its only caller is the fatal-error reporter, so the self-test is the only real coverage.

use crate::errors::BadInput;
use crate::framework::{EBX, RawHook, Regs, SelfTest, hook_asm, rva, shadow};
use crate::strings::utils::bytes_str;
use std::ptr::copy_nonoverlapping;
use std::slice::from_raw_parts;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("strings::bin_append", rva(0x40201D), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- Rust version ----

/// The low `digits` bits of `value` as '0' and '1', most significant first.
pub fn bin_append(value: u32, digits: u32) -> Result<Vec<u8>, BadInput> {
    rc2k_core::format::binary(value, digits).ok_or(BadInput(
        "more than 32 binary digits: the original reads below its stack buffer",
    ))
}

// ---- hook ----

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    let (value, digits, cursor) = (r.eax, r.ecx, r.ebx as *mut u8);
    let Ok(mine) = bin_append(value, digits) else {
        unsafe { call_original(r) };
        return;
    };

    if !shadow() {
        unsafe { copy_nonoverlapping(mine.as_ptr(), cursor, mine.len()) };
        r.ebx = (cursor as usize + mine.len()) as u32;
        return;
    }

    let mut o = *r;
    unsafe { call_original(&mut o) };
    let orig_len = (o.ebx as usize).wrapping_sub(cursor as usize);

    if orig_len != mine.len() || unsafe { from_raw_parts(cursor, orig_len) } != &mine[..] {
        HOOK.mismatch(|| {
            format!("value={value:#x} digits={digits}: original wrote {orig_len} bytes")
        });
    }
    r.ebx = o.ebx;
}

// ---- self-test ----

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let values = [
        0u32,
        1,
        0x8000_0000,
        0xFFFF_FFFF,
        0xA5A5_A5A5,
        0x1234_5678,
        0xDEAD_BEEF,
        0xFFFF,
    ];

    for &v in &values {
        for digits in 0..=32u32 {
            let mut buf = [0xAAu8; 64];
            let cursor = unsafe { buf.as_mut_ptr().add(8) };
            let before = Regs {
                eax: v,
                ecx: digits,
                ebx: cursor as u32,
                ..Regs::SENTINEL
            };
            let mut ro = before;
            unsafe { call_original(&mut ro) };

            let n = digits as usize;
            let mut want = [0xAAu8; 64];
            want[8..8 + n].copy_from_slice(&bin_append(v, digits).unwrap_or_default());
            let preserved = ro.preserved_except(&before, EBX);

            t.check(
                buf == want && ro.ebx == cursor as u32 + digits && preserved,
                || {
                    format!(
                        "value={v:#x} digits={digits}: original {} want {} ebx={:#x} preserved={preserved}",
                        bytes_str(&buf),
                        bytes_str(&want),
                        ro.ebx
                    )
                },
            );
        }
    }
    t.finish()
}
