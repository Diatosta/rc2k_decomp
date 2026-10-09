//! `strings::append_speed` (0x401B10): appends a speed given in tenths of a mile per hour with
//! one decimal, "Q.R", converted to km/h when the game is set to metric units, optionally
//! followed by the unit label ("km/h" on a metric game).
//!
//! In:  eax = tenths of a mile per hour, ecx = non-zero to append the unit label, ebx = cursor
//! Out: ebx = new cursor; everything else preserved
//! In metric mode a value above `miles_to_km::MAX_INPUT` makes the original trap: refused, and
//! the hook leaves it to the original.
//!
//! Screen: confirmed on the stage selection screen ("Escolha Etapa"), where
//! `draw_stage_records` uses it for the top speed, shown as "187.3km/h".

use crate::errors::BadInput;
use crate::framework::{EBX, RawHook, Regs, SelfTest, hook_asm, rva, shadow};
use crate::math::miles_to_km::{MAX_INPUT, miles_to_km};
use crate::strings::append_string_by_id::append_string_by_id;
use crate::strings::format_u32::format_u32;
use crate::strings::units::{speed_label_id, units};
use crate::strings::utils::bytes_str;
use std::ptr::copy_nonoverlapping;
use std::slice::from_raw_parts;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("strings::append_speed", rva(0x401B10), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- Rust version ----

/// `units` is the game's unit setting (0 = imperial). Refuses what `miles_to_km` refuses.
pub fn append_speed(tenths: u32, with_label: bool, units: u32) -> Result<Vec<u8>, BadInput> {
    let v = if units != 0 {
        miles_to_km(tenths)?
    } else {
        tenths
    };
    let mut out = format_u32(v / 10, 0);
    out.push(b'.');
    out.extend(format_u32(v % 10, 0));
    if with_label {
        unsafe {
            append_string_by_id(speed_label_id(units), &mut |p, n| {
                out.extend_from_slice(from_raw_parts(p, n))
            });
        }
    }
    Ok(out)
}

// ---- hook ----

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    let (tenths, with_label, cursor) = (r.eax, r.ecx != 0, r.ebx as *mut u8);
    let units = units();

    let Ok(mine) = append_speed(tenths, with_label, units) else {
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
                "tenths={tenths} with_label={with_label} units={units}: original wrote {orig_len} bytes, rust {}",
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
    let units = units();
    let mut values: Vec<u32> = (0..=2500).collect();
    values.extend([99_999, 1_000_000, 123_456_789, MAX_INPUT - 1, MAX_INPUT]);
    if units == 0 {
        values.extend([MAX_INPUT + 1, 4_294_967_295]); // metric: the original would trap
    }

    for &v in &values {
        // ecx is 0 or 1 and the other registers are 0: the label path ends in a `rep movsb`.
        for with_label in [0u32, 1] {
            let mine = append_speed(v, with_label != 0, units).unwrap_or_default();
            let mut buf = vec![0xAAu8; mine.len() + 4096];
            let before = Regs {
                eax: v,
                ecx: with_label,
                ebx: buf.as_mut_ptr() as u32,
                ..Regs::default()
            };
            let mut ro = before;
            unsafe { call_original(&mut ro) };
            let orig_len = (ro.ebx as usize).wrapping_sub(buf.as_ptr() as usize);
            let preserved = ro.preserved_except(&before, EBX);

            t.check(
                buf.get(..orig_len) == Some(&mine[..]) && preserved,
                || {
                    format!(
                        "tenths={v} with_label={with_label} units={units} original {} rust {} preserved={preserved}",
                        bytes_str(buf.get(..orig_len).unwrap_or(&[])),
                        bytes_str(&mine)
                    )
                },
            );
        }
    }
    t.finish()
}
