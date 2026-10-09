//! `text::reset_line_state` (0x402563): zeroes the kerning row table at the start of a text line.
//!
//! In:  nothing
//! Out: eax = 0 (callers rely on it); everything else preserved

use crate::framework::{EAX, EBP, EDX, RawHook, Regs, SelfTest, game_ptr, hook_asm, rva, shadow};
use std::slice::from_raw_parts;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("text::reset_line_state", rva(0x402563), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- game data ----

const STATE_RVA: usize = rva(0x93E830); // g_textLineState: the kerning row table
const STATE_LEN: usize = 0x80;

fn state() -> *mut u8 {
    game_ptr(STATE_RVA)
}

fn state_is_zero() -> bool {
    unsafe { from_raw_parts(state(), STATE_LEN) }
        .iter()
        .all(|&b| b == 0)
}

// ---- Rust version ----

pub unsafe fn reset_line_state() {
    unsafe { std::ptr::write_bytes(state(), 0, STATE_LEN) };
}

// ---- hook ----

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();

    if !shadow() {
        unsafe { reset_line_state() };
        r.eax = 0;
        return;
    }

    // The effect is fully determined (zero the table, eax = 0), so shadow mode runs the
    // original and checks that it left exactly that behind.
    let mut o = *r;
    unsafe { call_original(&mut o) };
    if !(state_is_zero() && o.eax == 0 && o.preserved_except(r, EAX | EDX | EBP)) {
        HOOK.mismatch(|| {
            format!(
                "zeroed={} eax={:#x} ecx={:#x} edi={:#x}",
                state_is_zero(),
                o.eax,
                o.ecx,
                o.edi
            )
        });
    }
    r.eax = o.eax;
}

// ---- self-test ----

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    // This table is live renderer state, so save it and put it back afterwards.
    let saved = unsafe { from_raw_parts(state(), STATE_LEN) }.to_vec();

    for fill in [0xAAu8, 0xFF, 0x01] {
        // original
        unsafe { std::ptr::write_bytes(state(), fill, STATE_LEN) };
        let mut ro = Regs::SENTINEL;
        unsafe { call_original(&mut ro) };
        let ok = state_is_zero() && ro.eax == 0 && ro.preserved_except(&Regs::SENTINEL, EAX);
        t.check(ok, || {
            format!(
                "original, fill={fill:#04x}: zeroed={} eax={:#x} ecx={:#x} edi={:#x}",
                state_is_zero(),
                ro.eax,
                ro.ecx,
                ro.edi
            )
        });

        // Rust
        unsafe { std::ptr::write_bytes(state(), fill, STATE_LEN) };
        unsafe { reset_line_state() };
        t.check(state_is_zero(), || {
            format!("rust, fill={fill:#04x}: table not zeroed")
        });
    }

    unsafe { std::ptr::copy_nonoverlapping(saved.as_ptr(), state(), STATE_LEN) };
    t.finish()
}
