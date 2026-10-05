use crate::framework::{MAX_LOGGED, RawHook, Regs, SHADOW_MODE, hook_asm, log};
use std::sync::atomic::{AtomicUsize, Ordering};

static TRAMPOLINE: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "math::celsius_to_fahrenheit",
    0x4019A5 - 0x400000,
    thunk,
    &TRAMPOLINE,
)
.self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = TRAMPOLINE
);

/// F = trunc(C * 9 / 5) + 32. i64 division truncates toward zero, like `idiv`.
fn reimplemented(celsius: i32) -> i32 {
    (((celsius as i64) * 9 / 5) as i32).wrapping_add(32)
}

// eax = Celsius -> eax = Fahrenheit; all other registers preserved
extern "C" fn dispatch(r: &mut Regs) {
    HOOK.stats.calls.fetch_add(1, Ordering::Relaxed);
    let celsius = r.eax as i32;
    let mine = reimplemented(celsius);

    if !SHADOW_MODE.load(Ordering::Relaxed) {
        r.eax = mine as u32;
        return;
    }

    let mut o = *r;
    unsafe { call_original(&mut o) };
    if o.eax as i32 != mine {
        let n = HOOK.stats.mismatches.fetch_add(1, Ordering::Relaxed);
        if n < MAX_LOGGED {
            log(&format!(
                "MISMATCH celsius_to_fahrenheit({celsius}): original={} rust={mine}",
                o.eax as i32
            ));
        }
    }
    r.eax = o.eax;
}

pub fn self_test() -> (usize, usize) {
    if TRAMPOLINE.load(Ordering::Relaxed) == 0 {
        log("celsius_to_fahrenheit selftest SKIPPED: original not available");
        return (0, 0);
    }
    // Stay well inside the range where the original's idiv can't trap.
    let inputs = (-5000..=5000).chain([-1_000_000_000, -1_000_000, 1_000_000, 1_000_000_000]);
    let (mut cases, mut failures) = (0, 0);

    for c in inputs {
        cases += 1;
        let mut ro = Regs {
            eax: c as u32,
            ..Regs::default()
        };
        unsafe { call_original(&mut ro) };
        let (orig, mine) = (ro.eax as i32, reimplemented(c));
        if orig != mine {
            failures += 1;
            log(&format!(
                "celsius_to_fahrenheit selftest FAILED: celsius={c} original={orig} rust={mine}"
            ));
        }
    }
    (cases, failures)
}
