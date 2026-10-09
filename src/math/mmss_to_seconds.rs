//! `math::mmss_to_seconds` (0x4019BB): turns a decimal "minutes and seconds" number into
//! seconds: the last two decimal digits count once, everything above them counts 60 times
//! (130 -> 90). The same arithmetic would serve hours and minutes.
//!
//! In:  eax = value
//! Out: eax = (value / 100) * 60 + value % 100; everything else preserved

use crate::framework::{EAX, RawHook, Regs, SelfTest, hook_asm, rva, shadow};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("math::mmss_to_seconds", rva(0x4019BB), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- Rust version ----

/// Unsigned, and cannot overflow: u32::MAX / 100 * 60 + 99 fits in 32 bits.
pub fn mmss_to_seconds(mmss: u32) -> u32 {
    (mmss / 100) * 60 + mmss % 100
}

// ---- hook ----

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    let mmss = r.eax;
    let mine = mmss_to_seconds(mmss);

    if !shadow() {
        r.eax = mine;
        return;
    }

    let mut o = *r;
    unsafe { call_original(&mut o) };
    if o.eax != mine {
        HOOK.mismatch(|| format!("mmss={mmss}: original={} rust={mine}", o.eax));
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

    for mmss in inputs {
        let before = Regs {
            eax: mmss,
            ..Regs::SENTINEL
        };
        let mut ro = before;
        unsafe { call_original(&mut ro) };
        let mine = mmss_to_seconds(mmss);
        let preserved = ro.preserved_except(&before, EAX);

        t.check(ro.eax == mine && preserved, || {
            format!(
                "mmss={mmss} original={} rust={mine} preserved={preserved}",
                ro.eax
            )
        });
    }
    t.finish()
}
