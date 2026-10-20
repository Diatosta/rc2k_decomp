//! `math::miles_to_km` (0x40191E): multiplies by 1.6093, rounding to nearest. The distance
//! formatters call it when the game is set to metric units.
//!
//! In:  eax = value in miles (any fixed-point scale)
//! Out: eax = value in km; everything else preserved
//! Above `MAX_INPUT` the result no longer fits 32 bits and the original's `div` traps: such
//! values are refused, and the hook leaves them to the original.

use crate::errors::BadInput;
use crate::framework::{EAX, RawHook, Regs, SelfTest, hook_asm, rva, shadow};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("math::miles_to_km", rva(0x40191E), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- Rust version ----

/// Largest input whose result fits in 32 bits (converted: `rc2k_core::units`).
pub const MAX_INPUT: u32 = rc2k_core::units::MAX_MILES;

/// (miles * 16093 + 5000) / 10000 (converted: `rc2k_core::units::miles_to_km`).
pub fn miles_to_km(miles: u32) -> Result<u32, BadInput> {
    rc2k_core::units::miles_to_km(miles).ok_or(BadInput(
        "result does not fit 32 bits: the original's div traps",
    ))
}

// ---- hook ----

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    let miles = r.eax;
    let Ok(mine) = miles_to_km(miles) else {
        unsafe { call_original(r) };
        return;
    };

    if !shadow() {
        r.eax = mine;
        return;
    }

    let mut o = *r;
    unsafe { call_original(&mut o) };
    if o.eax != mine {
        HOOK.mismatch(|| format!("miles={miles}: original={} rust={mine}", o.eax));
    }
    r.eax = o.eax;
}

// ---- self-test ----

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    // Never above MAX_INPUT: the original would trap.
    let inputs = (0..=20_000u32).chain([
        99_999,
        1_000_000,
        123_456_789,
        0x7FFF_FFFF,
        0x8000_0000,
        MAX_INPUT - 1,
        MAX_INPUT,
    ]);

    for miles in inputs {
        let before = Regs {
            eax: miles,
            ..Regs::SENTINEL
        };
        let mut ro = before;
        unsafe { call_original(&mut ro) };
        let mine = miles_to_km(miles).unwrap_or(0);
        let preserved = ro.preserved_except(&before, EAX);

        t.check(ro.eax == mine && preserved, || {
            format!(
                "miles={miles} original={} rust={mine} preserved={preserved}",
                ro.eax
            )
        });
    }
    t.finish()
}
