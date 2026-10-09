//! `strings::hex_append` (0x40211D): appends `digits` uppercase hex digits of a value, MSB first.
//!
//! In:  eax = value, ecx = digits (0..=8), ebx = cursor
//! Out: ebx = new cursor
//! More than 8 digits is left to the original.

use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva, shadow};
use crate::strings::utils::written;
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

const HEX_DIGITS: &[u8; 16] = b"0123456789ABCDEF";

pub unsafe fn hex_append(value: u32, digits: usize, mut cursor: *mut u8) -> *mut u8 {
    for i in (0..digits).rev() {
        unsafe {
            *cursor = HEX_DIGITS[((value >> (i * 4)) & 0xF) as usize];
            cursor = cursor.add(1);
        }
    }
    cursor
}

// ---- hook ----

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    let (value, digits, cursor) = (r.eax, r.ecx, r.ebx as *mut u8);

    if digits > 8 {
        let mut o = *r;
        unsafe { call_original(&mut o) };
        r.ebx = o.ebx;
        return;
    }

    if !shadow() {
        r.ebx = unsafe { hex_append(value, digits as usize, cursor) } as u32;
        return;
    }

    // Shadow: Rust writes into scratch, the original does the real writing.
    let mut scratch = [0xAAu8; 8];
    let mine = unsafe { hex_append(value, digits as usize, scratch.as_mut_ptr()) };
    let mine_len = mine as usize - scratch.as_ptr() as usize;

    let mut o = *r;
    unsafe { call_original(&mut o) };
    let orig_len = (o.ebx as usize).wrapping_sub(cursor as usize);

    if orig_len != mine_len || unsafe { from_raw_parts(cursor, mine_len) } != &scratch[..mine_len] {
        HOOK.mismatch(|| {
            format!(
                "value={value:#x} digits={digits}: original wrote {orig_len} bytes, rust {mine_len}"
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
            let (mut buf_o, mut buf_r) = ([0xAAu8; 16], [0xAAu8; 16]);

            let mut ro = Regs {
                eax: v,
                ecx: digits,
                ebx: buf_o.as_mut_ptr() as u32,
                ..Regs::default()
            };
            unsafe { call_original(&mut ro) };
            let end_o = ro.ebx as *const u8;
            let end_r = unsafe { hex_append(v, digits as usize, buf_r.as_mut_ptr()) };

            let same_end = (end_o as usize).wrapping_sub(buf_o.as_ptr() as usize)
                == (end_r as usize).wrapping_sub(buf_r.as_ptr() as usize);
            t.check(same_end && buf_o == buf_r, || {
                format!(
                    "value={v:#x} digits={digits} original {} rust {}",
                    written(&buf_o, end_o),
                    written(&buf_r, end_r)
                )
            });
        }
    }
    t.finish()
}
