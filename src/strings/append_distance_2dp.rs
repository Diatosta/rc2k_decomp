//! `strings::append_distance_2dp` (0x401B87): appends a distance given in hundredths of a mile
//! with two decimals, "Q.RR", converted to km when the game is set to metric units.
//!
//! In:  eax = hundredths of a mile, ebx = cursor
//! Out: ebx = new cursor; everything else preserved
//! In metric mode a value above `miles_to_km::MAX_INPUT` makes the original trap: left to the
//! original.

use crate::framework::{EBX, RawHook, Regs, SelfTest, hook_asm, rva, shadow};
use crate::math::miles_to_km::{MAX_INPUT, miles_to_km};
use crate::strings::format_u32::format_u32;
use crate::strings::units::is_metric;
use crate::strings::utils::bytes_str;
use std::ptr::copy_nonoverlapping;
use std::slice::from_raw_parts;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "strings::append_distance_2dp",
    rva(0x401B87),
    thunk,
    &ORIGINAL,
)
.self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- Rust version ----

/// With `metric`, `hundredths` must be at most `MAX_INPUT`.
pub fn append_distance_2dp(hundredths: u32, metric: bool) -> Vec<u8> {
    let v = if metric {
        miles_to_km(hundredths)
    } else {
        hundredths
    };
    let mut out = format_u32(v / 100, 0);
    out.push(b'.');
    out.extend(format_u32(v % 100, 0x12));
    out
}

// ---- hook ----

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    let (hundredths, cursor) = (r.eax, r.ebx as *mut u8);
    let metric = is_metric();

    if metric && hundredths > MAX_INPUT {
        unsafe { call_original(r) };
        return;
    }
    let mine = append_distance_2dp(hundredths, metric);

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
                "hundredths={hundredths} metric={metric}: original wrote {orig_len} bytes, rust {}",
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
    // Tests the unit system the game currently has selected: switch it in the game's options
    // and run selftest again to cover the other branch.
    let metric = is_metric();
    let mut values: Vec<u32> = (0..=5000).collect();
    values.extend([99_999, 1_000_000, 123_456_789, MAX_INPUT - 1, MAX_INPUT]);
    if !metric {
        values.extend([MAX_INPUT + 1, 4_294_967_295]); // metric: the original would trap
    }

    for &v in &values {
        let mut buf = [0xAAu8; 32];
        let before = Regs {
            eax: v,
            ebx: buf.as_mut_ptr() as u32,
            ..Regs::SENTINEL
        };
        let mut ro = before;
        unsafe { call_original(&mut ro) };
        let orig_len = (ro.ebx as usize).wrapping_sub(buf.as_ptr() as usize);
        let mine = append_distance_2dp(v, metric);
        let preserved = ro.preserved_except(&before, EBX);

        t.check(buf.get(..orig_len) == Some(&mine[..]) && preserved, || {
            format!(
                "hundredths={v} metric={metric} original {} rust {} preserved={preserved}",
                bytes_str(buf.get(..orig_len).unwrap_or(&[])),
                bytes_str(&mine)
            )
        });
    }
    t.finish()
}
