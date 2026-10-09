//! `strings::append_temperature` (0x401AD9): appends a temperature with its unit label.
//!
//! In:  eax = temperature in Celsius, ebx = cursor
//! Out: ebx = new cursor; eax = what `append_string_by_id` leaves for the unit label

use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva, shadow};
use crate::math::celsius_to_fahrenheit::celsius_to_fahrenheit;
use crate::strings::append_string_by_id::append_string_by_id;
use crate::strings::format_u32::format_u32;
use crate::strings::units::{temperature_label_id, units};
use crate::strings::utils::bytes_str;
use std::ptr::copy_nonoverlapping;
use std::slice::from_raw_parts;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "strings::append_temperature",
    rva(0x401AD9),
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

/// Returns (bytes to append, the eax the original leaves behind).
pub fn append_temperature(celsius: i32) -> (Vec<u8>, u32) {
    let unit = units();
    let mut v = celsius;
    if unit == 0 {
        v = celsius_to_fahrenheit(v);
    }

    let mut out = Vec::new();
    if v < 0 {
        out.push(b'-');
        v = v.wrapping_neg(); // the `neg eax` before the call to append_minus
    }
    out.extend(format_u32(v as u32, 0));

    let label_id = temperature_label_id(unit);
    let eax = unsafe {
        append_string_by_id(label_id, &mut |p, n| {
            out.extend_from_slice(from_raw_parts(p, n))
        })
    };
    (out, eax)
}

// ---- hook ----

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    let (celsius, cursor) = (r.eax as i32, r.ebx as *mut u8);
    let (mine, mine_eax) = append_temperature(celsius);

    if !shadow() {
        unsafe { copy_nonoverlapping(mine.as_ptr(), cursor, mine.len()) };
        r.ebx = (cursor as usize + mine.len()) as u32;
        r.eax = mine_eax;
        return;
    }

    let mut o = *r;
    unsafe { call_original(&mut o) };
    let orig_len = (o.ebx as usize).wrapping_sub(cursor as usize);

    if orig_len != mine.len()
        || unsafe { from_raw_parts(cursor, orig_len) } != &mine[..]
        || o.eax != mine_eax
    {
        HOOK.mismatch(|| {
            format!(
                "celsius={celsius}: original wrote {orig_len} bytes (eax={:#x}), rust {} (eax={mine_eax:#x})",
                o.eax,
                mine.len()
            )
        });
    }
    r.ebx = o.ebx;
    r.eax = o.eax;
}

// ---- self-test ----

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    // Tests the unit the game currently has selected: switch it in the game's options
    // and run selftest again to cover the other branch.
    let unit = units();
    let mut inputs: Vec<i32> = (-5000..=5000).collect();
    inputs.extend([-1_000_000_000, -1_000_000, 1_000_000, 1_000_000_000]);
    if unit != 0 {
        // Only without the C -> F conversion: with it, the original's idiv traps beyond ~±1.19e9.
        inputs.extend([i32::MIN, i32::MAX]);
    }

    for &c in &inputs {
        let (mine, mine_eax) = append_temperature(c);
        let mut buf = vec![0xAAu8; mine.len() + 4096];
        let mut ro = Regs {
            eax: c as u32,
            ebx: buf.as_mut_ptr() as u32,
            ..Regs::default()
        };
        unsafe { call_original(&mut ro) };
        let orig_len = (ro.ebx as usize).wrapping_sub(buf.as_ptr() as usize);

        t.check(
            buf.get(..orig_len) == Some(&mine[..]) && ro.eax == mine_eax,
            || {
                format!(
                    "celsius={c} units={unit} original {} rust {}",
                    bytes_str(buf.get(..orig_len).unwrap_or(&[])),
                    bytes_str(&mine)
                )
            },
        );
    }
    t.finish()
}
