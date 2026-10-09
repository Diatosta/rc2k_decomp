//! `strings::append_min_sec` (0x4019EB): appends a number of seconds as minutes and seconds,
//! "M:SS".
//!
//! In:  eax = seconds, ebx = cursor
//! Out: ebx = new cursor; everything else preserved

use crate::framework::{EBX, RawHook, Regs, SelfTest, hook_asm, rva, shadow};
use crate::strings::format_u32::format_u32;
use crate::strings::utils::bytes_str;
use std::ptr::copy_nonoverlapping;
use std::slice::from_raw_parts;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("strings::append_min_sec", rva(0x4019EB), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- Rust version ----

/// Minutes unpadded, seconds as two digits.
pub fn append_min_sec(seconds: u32) -> Vec<u8> {
    let mut out = format_u32(seconds / 60, 0);
    out.push(b':');
    out.extend(format_u32(seconds % 60, 0x12));
    out
}

// ---- hook ----

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    let (seconds, cursor) = (r.eax, r.ebx as *mut u8);
    let mine = append_min_sec(seconds);

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
                "seconds={seconds}: original wrote {orig_len} bytes, rust {}",
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
    let values = (0..=7300u32).chain([
        35_999,
        36_000,
        100_000,
        59_999_999,
        60_000_000,
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
        let mine = append_min_sec(v);
        let preserved = ro.preserved_except(&before, EBX);

        t.check(buf.get(..orig_len) == Some(&mine[..]) && preserved, || {
            format!(
                "seconds={v} original {} rust {} preserved={preserved}",
                bytes_str(buf.get(..orig_len).unwrap_or(&[])),
                bytes_str(&mine)
            )
        });
    }
    t.finish()
}
