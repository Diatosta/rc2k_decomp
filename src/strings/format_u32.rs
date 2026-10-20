//! `strings::format_u32` (0x40204A): appends an unsigned decimal number.
//!
//! In:  eax = value, ecx = flags, ebx = cursor
//! Out: ebx = new cursor; everything else preserved
//! Flags: low nibble = width, 0x10 = zero-pad to width, 0x20 = space-pad to width,
//! neither = no padding. 0x40 and 0x80 are ignored.

use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva, shadow};
use crate::strings::utils::bytes_str;
use std::ptr::copy_nonoverlapping;
use std::slice::from_raw_parts;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("strings::format_u32", rva(0x40204A), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- Rust version ----

/// Converted: `rc2k_core::strings::format::decimal` (the flags byte as `Pad`).
pub fn format_u32(value: u32, flags: u8) -> Vec<u8> {
    rc2k_core::strings::format::decimal(value, rc2k_core::strings::format::Pad::from_flags(flags))
}

// ---- hook ----

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    let (value, flags, cursor) = (r.eax, r.ecx as u8, r.ebx as *mut u8);
    let mine = format_u32(value, flags);

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
                "value={value} flags={flags:#04x}: original wrote {orig_len} bytes, rust {}",
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
        9,
        10,
        99,
        100,
        1234,
        99_999,
        123_456,
        999_999_999,
        1_000_000_000,
        2_147_483_648,
        4_294_967_295,
    ];

    for &v in &values {
        for mode in [0x00u8, 0x10, 0x20, 0x30] {
            for width in 0..=15u8 {
                for extra in [0x00u8, 0xC0] {
                    let flags = mode | width | extra;
                    let mut buf = [0xAAu8; 32];
                    let mut ro = Regs {
                        eax: v,
                        ecx: flags as u32,
                        ebx: buf.as_mut_ptr() as u32,
                        ..Regs::default()
                    };
                    unsafe { call_original(&mut ro) };
                    let orig_len = (ro.ebx as usize).wrapping_sub(buf.as_ptr() as usize);
                    let mine = format_u32(v, flags);

                    t.check(buf.get(..orig_len) == Some(&mine[..]), || {
                        format!(
                            "value={v} flags={flags:#04x} original {} rust {}",
                            bytes_str(buf.get(..orig_len).unwrap_or(&[])),
                            bytes_str(&mine)
                        )
                    });
                }
            }
        }
    }
    t.finish()
}
