use crate::framework::{MAX_LOGGED, RawHook, Regs, SHADOW_MODE, game_addr, hook_asm, log};
use std::sync::atomic::{AtomicUsize, Ordering};

static TRAMPOLINE: AtomicUsize = AtomicUsize::new(0);

const STATE_RVA: usize = 0x93E830 - 0x400000;
const STATE_LEN: usize = 0x80;

pub static HOOK: RawHook = RawHook::new(
    "text::reset_line_state",
    0x402563 - 0x400000,
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

fn state() -> *mut u8 {
    game_addr(STATE_RVA) as *mut u8
}

fn state_is_zero() -> bool {
    unsafe { std::slice::from_raw_parts(state(), STATE_LEN) }
        .iter()
        .all(|&b| b == 0)
}

/// Zeroes the 128-byte per-line state table.
unsafe fn reimplemented() {
    unsafe { std::ptr::write_bytes(state(), 0, STATE_LEN) };
}

// no inputs; eax = 0 on return (callers rely on a clean eax); ecx and edi preserved
extern "C" fn dispatch(r: &mut Regs) {
    HOOK.stats.calls.fetch_add(1, Ordering::Relaxed);

    if !SHADOW_MODE.load(Ordering::Relaxed) {
        unsafe { reimplemented() };
        r.eax = 0;
        return;
    }

    // The effect is fully determined (zero the table, eax = 0, nothing else changes),
    // so shadow mode runs the original and checks that it left exactly that behind.
    let mut o = *r;
    unsafe { call_original(&mut o) };
    let ok = state_is_zero()
        && o.eax == 0
        && o.ecx == r.ecx
        && o.edi == r.edi
        && o.ebx == r.ebx
        && o.esi == r.esi;
    if !ok {
        let n = HOOK.stats.mismatches.fetch_add(1, Ordering::Relaxed);
        if n < MAX_LOGGED {
            log(&format!(
                "MISMATCH reset_line_state: zeroed={} eax={:#x} ecx={:#x} edi={:#x}",
                state_is_zero(),
                o.eax,
                o.ecx,
                o.edi
            ));
        }
    }
    r.eax = o.eax;
}

pub fn self_test() -> (usize, usize) {
    if TRAMPOLINE.load(Ordering::Relaxed) == 0 {
        log("reset_line_state selftest SKIPPED: original not available");
        return (0, 0);
    }
    // This table is live renderer state, so save it and put it back afterward.
    let saved = unsafe { std::slice::from_raw_parts(state(), STATE_LEN) }.to_vec();
    let (mut cases, mut failures) = (0, 0);

    for fill in [0xAAu8, 0xFF, 0x01] {
        // original
        cases += 1;
        unsafe { std::ptr::write_bytes(state(), fill, STATE_LEN) };
        let mut ro = Regs {
            eax: 0xDEAD_BEEF,
            ecx: 0x1111_1111,
            edx: 0x3333_3333,
            ebx: 0x4444_4444,
            esi: 0x5555_5555,
            edi: 0x2222_2222,
            ebp: 0x6666_6666,
            ..Regs::default()
        };
        unsafe { call_original(&mut ro) };
        let ok = state_is_zero()
            && ro.eax == 0
            && ro.ecx == 0x1111_1111
            && ro.edx == 0x3333_3333
            && ro.ebx == 0x4444_4444
            && ro.esi == 0x5555_5555
            && ro.edi == 0x2222_2222
            && ro.ebp == 0x6666_6666;
        if !ok {
            failures += 1;
            log(&format!(
                "reset_line_state selftest FAILED (original, fill={fill:#04x}): zeroed={} eax={:#x} ecx={:#x} edi={:#x}",
                state_is_zero(),
                ro.eax,
                ro.ecx,
                ro.edi
            ));
        }

        // Rust
        cases += 1;
        unsafe { std::ptr::write_bytes(state(), fill, STATE_LEN) };
        unsafe { reimplemented() };
        if !state_is_zero() {
            failures += 1;
            log(&format!(
                "reset_line_state selftest FAILED (rust, fill={fill:#04x}): table not zeroed"
            ));
        }
    }

    unsafe { std::ptr::copy_nonoverlapping(saved.as_ptr(), state(), STATE_LEN) };
    (cases, failures)
}
