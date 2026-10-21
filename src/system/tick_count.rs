//! `system::tick_count` (0x411024): the game's millisecond clock: `GetTickCount`, unless the
//! override switch `dword_5C2CD4` is 1, in which case it is `dword_5C2CD0`.
//!
//! In:  nothing
//! Out: eax = milliseconds; everything else preserved
//! 110 callers. What sets the override has not been read.
//! The clock moves, so shadow mode and the self-test call the Rust version before and after
//! the original and accept anything in between.

use crate::framework::{
    EAX, RawHook, Regs, SelfTest, game_read, game_write, hook_asm, rva, shadow,
};
use std::sync::atomic::AtomicUsize;
use windows_sys::Win32::System::SystemInformation::GetTickCount;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("system::tick_count", rva(0x411024), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- game data ----

const OVERRIDE_ON_RVA: usize = rva(0x5C2CD4); // dword_5C2CD4
const OVERRIDE_RVA: usize = rva(0x5C2CD0); // dword_5C2CD0

// ---- Rust version ----

pub fn tick_count() -> u32 {
    unsafe {
        // The original asks Windows first in both cases.
        let now = GetTickCount();
        // Converted: `rc2k_core::system::time::clock`.
        rc2k_core::system::time::clock(now, game_read(OVERRIDE_ON_RVA), || game_read(OVERRIDE_RVA))
    }
}

// ---- hook ----

/// True if `value` is in the span from `first` to `last`, on a clock that wraps.
fn between(first: u32, value: u32, last: u32) -> bool {
    value.wrapping_sub(first) <= last.wrapping_sub(first)
}

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    let first = tick_count();
    if !shadow() {
        r.eax = first;
        return;
    }
    let mut o = *r;
    unsafe { call_original(&mut o) };
    let last = tick_count();
    if !between(first, o.eax, last) || !o.preserved_except(r, EAX) {
        HOOK.mismatch(|| format!("original {} rust {first} then {last}", o.eax));
    }
    r.eax = o.eax;
}

// ---- self-test: the real clock, and the override with both settings ----

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let saved: (u32, u32) = unsafe { (game_read(OVERRIDE_ON_RVA), game_read(OVERRIDE_RVA)) };
    for (on, value) in [
        (0u32, 0x5EED_0001u32),
        (1, 0x5EED_0002),
        (2, 0x5EED_0003),
        (1, 0),
    ] {
        unsafe {
            game_write(OVERRIDE_ON_RVA, on);
            game_write(OVERRIDE_RVA, value);
        }
        let first = tick_count();
        let before = Regs::SENTINEL;
        let mut ro = before;
        unsafe { call_original(&mut ro) };
        let last = tick_count();
        let ok = between(first, ro.eax, last)
            && (on != 1 || ro.eax == value)
            && ro.preserved_except(&before, EAX);
        t.check(ok, || {
            format!(
                "override on={on} value={value:#x}: original {:#x}; rust {first:#x} then {last:#x}",
                ro.eax
            )
        });
    }
    unsafe {
        game_write(OVERRIDE_ON_RVA, saved.0);
        game_write(OVERRIDE_RVA, saved.1);
    }
    t.finish()
}
