//! `strings::append_u32` (0x4019E1): appends an unsigned decimal number without padding.
//! A wrapper around `format_u32` with flags 0.
//!
//! In:  eax = value, ebx = cursor
//! Out: ebx = new cursor; everything else preserved

use crate::framework::{EBX, RawHook, Regs, SelfTest, hook_asm, rva, shadow};
use crate::strings::utils::bytes_str;
use std::ptr::copy_nonoverlapping;
use std::slice::from_raw_parts;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

// The first 5 bytes end inside a `call`, so the jump is written by hand and the original runs
// from a private copy (its call at +3 relocated).
pub static HOOK: RawHook = RawHook::new("strings::append_u32", rva(0x4019E1), thunk, &ORIGINAL)
    .private_copy(0x0A)
    .relocate_calls(&[0x03])
    .manual_patch()
    .self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- Rust version ----

pub fn append_u32(value: u32) -> Vec<u8> {
    rc2k_core::format::decimal(value, rc2k_core::format::Pad::None)
}

// ---- hook ----

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    let (value, cursor) = (r.eax, r.ebx as *mut u8);
    let mine = append_u32(value);

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
                "value={value}: original wrote {orig_len} bytes, rust {}",
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
    let values = (0..=1100u32).chain([
        9_999,
        10_000,
        99_999,
        123_456,
        999_999_999,
        1_000_000_000,
        2_147_483_648,
        4_294_967_295,
    ]);

    for v in values {
        let mut buf = [0xAAu8; 32];
        let before = Regs {
            eax: v,
            ebx: buf.as_mut_ptr() as u32,
            ..Regs::SENTINEL
        };
        let mut ro = before;
        unsafe { call_original(&mut ro) };
        let orig_len = (ro.ebx as usize).wrapping_sub(buf.as_ptr() as usize);
        let mine = append_u32(v);
        let preserved = ro.preserved_except(&before, EBX);

        t.check(buf.get(..orig_len) == Some(&mine[..]) && preserved, || {
            format!(
                "value={v} original {} rust {} preserved={preserved}",
                bytes_str(buf.get(..orig_len).unwrap_or(&[])),
                bytes_str(&mine)
            )
        });
    }
    t.finish()
}
