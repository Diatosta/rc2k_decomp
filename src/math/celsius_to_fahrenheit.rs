//! `math::celsius_to_fahrenheit` (0x4019A5): converts whole degrees Celsius to Fahrenheit.
//!
//! In:  eax = Celsius
//! Out: eax = Fahrenheit; everything else preserved

use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva, shadow};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "math::celsius_to_fahrenheit",
    rva(0x4019A5),
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

/// F = trunc(C * 9 / 5) + 32. i64 division truncates toward zero, like `idiv`.
pub fn celsius_to_fahrenheit(celsius: i32) -> i32 {
    (((celsius as i64) * 9 / 5) as i32).wrapping_add(32)
}

// ---- hook ----

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    let celsius = r.eax as i32;
    let mine = celsius_to_fahrenheit(celsius);

    if !shadow() {
        r.eax = mine as u32;
        return;
    }

    let mut o = *r;
    unsafe { call_original(&mut o) };
    if o.eax as i32 != mine {
        HOOK.mismatch(|| format!("celsius={celsius}: original={} rust={mine}", o.eax as i32));
    }
    r.eax = o.eax;
}

// ---- self-test ----

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    // Stay well inside the range where the original's idiv can't trap.
    let inputs = (-5000..=5000).chain([-1_000_000_000, -1_000_000, 1_000_000, 1_000_000_000]);

    for c in inputs {
        let mut ro = Regs {
            eax: c as u32,
            ..Regs::default()
        };
        unsafe { call_original(&mut ro) };
        let (orig, mine) = (ro.eax as i32, celsius_to_fahrenheit(c));
        t.check(orig == mine, || {
            format!("celsius={c} original={orig} rust={mine}")
        });
    }
    t.finish()
}
