//! `math::hhmm_to_minutes` (0x4019BB): turns a time of day written as hours * 100 + minutes
//! (1300 for 13:00) into minutes. The last two decimal digits count once, everything above
//! them counts 60 times.
//!
//! In:  eax = value
//! Out: eax = (value / 100) * 60 + value % 100; everything else preserved
//! All 21 call sites are in the stage panel functions, on start, finish and total times; the
//! same arithmetic would turn minutes * 100 + seconds into seconds.

use crate::framework::{EAX, RawHook, Regs, SelfTest, hook_asm, rva, shadow};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("math::hhmm_to_minutes", rva(0x4019BB), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- Rust version ----

/// Converted: `rc2k_core::math::units`.
pub use rc2k_core::math::units::hhmm_to_minutes;

// ---- hook ----

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    let hhmm = r.eax;
    let mine = hhmm_to_minutes(hhmm);

    if !shadow() {
        r.eax = mine;
        return;
    }

    let mut o = *r;
    unsafe { call_original(&mut o) };
    if o.eax != mine {
        HOOK.mismatch(|| format!("hhmm={hhmm}: original={} rust={mine}", o.eax));
    }
    r.eax = o.eax;
}

// ---- self-test ----

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let inputs = (0..=20_000u32).chain([
        99_999,
        1_000_000,
        123_456_789,
        0x7FFF_FFFF,
        0x8000_0000,
        u32::MAX - 1,
        u32::MAX,
    ]);

    for hhmm in inputs {
        let before = Regs {
            eax: hhmm,
            ..Regs::SENTINEL
        };
        let mut ro = before;
        unsafe { call_original(&mut ro) };
        let mine = hhmm_to_minutes(hhmm);
        let preserved = ro.preserved_except(&before, EAX);

        t.check(ro.eax == mine && preserved, || {
            format!(
                "hhmm={hhmm} original={} rust={mine} preserved={preserved}",
                ro.eax
            )
        });
    }
    t.finish()
}
