use crate::framework::{MAX_LOGGED, RawHook, Regs, SHADOW_MODE, hook_asm, log};
use std::sync::atomic::{AtomicUsize, Ordering};

static TRAMPOLINE: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "strings::digit_step",
    0x402112 - 0x400000,
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

/// Returns (remainder, new byte). Only the low byte of the quotient is added, wrapping.
fn reimplemented(value: u32, divisor: u32, byte: u8) -> (u32, u8) {
    (value % divisor, byte.wrapping_add((value / divisor) as u8))
}

// eax = value, ebp = digit pointer, edi = divisor -> eax = edx = remainder, ebp += 1
extern "C" fn dispatch(r: &mut Regs) {
    HOOK.stats.calls.fetch_add(1, Ordering::Relaxed);
    let (value, digit, divisor) = (r.eax, r.ebp as *mut u8, r.edi);

    // The original traps on divide-by-zero. Let it, instead of panicking in Rust.
    if divisor == 0 {
        let mut o = *r;
        unsafe { call_original(&mut o) };
        return;
    }

    let (rem, new_byte) = reimplemented(value, divisor, unsafe { *digit });

    if !SHADOW_MODE.load(Ordering::Relaxed) {
        unsafe { *digit = new_byte };
        r.eax = rem;
        r.edx = rem;
        r.ebp += 1;
        return;
    }

    // Shadow: the original does the real write, then compare with what Rust computed.
    let mut o = *r;
    unsafe { call_original(&mut o) };
    let orig_byte = unsafe { *digit };
    if o.eax != rem || o.edx != rem || o.ebp != r.ebp + 1 || orig_byte != new_byte {
        let n = HOOK.stats.mismatches.fetch_add(1, Ordering::Relaxed);
        if n < MAX_LOGGED {
            log(&format!(
                "MISMATCH digit_step(value={value}, divisor={divisor}): original rem={} byte={orig_byte:#04x}, rust rem={rem} byte={new_byte:#04x}",
                o.eax
            ));
        }
    }
    r.eax = o.eax;
    r.edx = o.edx;
    r.ebp = o.ebp;
}

pub fn self_test() -> (usize, usize) {
    if TRAMPOLINE.load(Ordering::Relaxed) == 0 {
        log("digit_step selftest SKIPPED: original not available");
        return (0, 0);
    }
    let values = [
        0u32,
        1,
        9,
        10,
        99,
        100,
        255,
        256,
        1234,
        65_535,
        999_999_999,
        1_000_000_000,
        2_147_483_648,
        u32::MAX,
    ];
    let divisors = [
        1u32,
        2,
        7,
        10,
        100,
        1_000,
        10_000,
        100_000,
        1_000_000,
        10_000_000,
        100_000_000,
        1_000_000_000,
        u32::MAX,
    ];
    // '0' is the real use; 0x00 and 0xFF exercise the wrapping add.
    let bytes = [b'0', 0x00, 0xFF];
    let (mut cases, mut failures) = (0, 0);

    for &v in &values {
        for &d in &divisors {
            for &b in &bytes {
                cases += 1;
                let mut cell = [b, 0xAAu8];
                let ptr = cell.as_mut_ptr() as u32;
                let mut ro = Regs {
                    eax: v,
                    ebp: ptr,
                    edi: d,
                    ..Regs::default()
                };
                unsafe { call_original(&mut ro) };

                let (rem, nb) = reimplemented(v, d, b);
                let ok = ro.eax == rem && ro.edx == rem && ro.ebp == ptr + 1 && cell == [nb, 0xAA];
                if !ok {
                    failures += 1;
                    log(&format!(
                        "digit_step selftest FAILED: value={v} divisor={d} byte={b:#04x} original rem={} byte={:#04x} rust rem={rem} byte={nb:#04x}",
                        ro.eax, cell[0]
                    ));
                }
            }
        }
    }
    (cases, failures)
}
