//! `strings::hex_append` (0x40211D): appends `digits` uppercase hex digits of a value, MSB first.
//!
//! In:  eax = value, ecx = digits (0..=8), ebx = cursor
//! Out: ebx = new cursor
//! More than 8 digits is refused: the hook leaves that to the original.

use crate::errors::BadInput;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva, shadow};
use crate::strings::utils::bytes_str;
use std::ptr::copy_nonoverlapping;
use std::slice::from_raw_parts;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("strings::hex_append", rva(0x40211D), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- Rust version ----

/// The low `digits` hex digits of `value`, most significant first.
pub fn hex_append(value: u32, digits: u32) -> Result<Vec<u8>, BadInput> {
    rc2k_core::format::hex(value, digits).ok_or(BadInput(
        "more than 8 hex digits: not worked out in the original",
    ))
}

// ---- hook ----

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    let (value, digits, cursor) = (r.eax, r.ecx, r.ebx as *mut u8);
    let Ok(mine) = hex_append(value, digits) else {
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
            format!(
                "value={value:#x} digits={digits}: original wrote {orig_len} bytes, rust {}",
                mine.len()
            )
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
        0xF,
        0x10,
        0xFF,
        0x1234ABCD,
        0xDEADBEEF,
        0x8010_8601,
        0xFFFF_FFFF,
    ];

    for &v in &values {
        for digits in 0..=8u32 {
            let mut buf = [0xAAu8; 16];
            let mut ro = Regs {
                eax: v,
                ecx: digits,
                ebx: buf.as_mut_ptr() as u32,
                ..Regs::default()
            };
            unsafe { call_original(&mut ro) };
            let orig_len = (ro.ebx as usize).wrapping_sub(buf.as_ptr() as usize);
            let mine = hex_append(v, digits).unwrap_or_default();

            let mut want = [0xAAu8; 16];
            want[..mine.len()].copy_from_slice(&mine);
            t.check(orig_len == mine.len() && buf == want, || {
                format!(
                    "value={v:#x} digits={digits} original {} rust {}",
                    bytes_str(buf.get(..orig_len).unwrap_or(&[])),
                    bytes_str(&mine)
                )
            });
        }
    }
    t.finish()
}
