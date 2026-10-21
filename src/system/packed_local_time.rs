//! `system::packed_local_time` (0x410F99): the local date and time to the minute, packed
//! into one dword: year, 4 bits of month, 5 of day, 5 of hour, 6 of minute.
//!
//! In:  nothing
//! Out: eax = the stamp; everything else preserved
//! `GetLocalTime` fills the game's `SystemTime` (0x518988), which stays set. Six callers: the
//! save files carry this stamp.
//! The clock moves, so shadow mode and the self-test call the Rust version before and after
//! the original and accept either answer.

use crate::framework::{EAX, RawHook, Regs, SelfTest, game_ptr, hook_asm, rva, shadow};
use std::sync::atomic::AtomicUsize;
use windows_sys::Win32::Foundation::SYSTEMTIME;
use windows_sys::Win32::System::SystemInformation::GetLocalTime;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("system::packed_local_time", rva(0x410F99), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- game data ----

const SYSTEM_TIME_RVA: usize = rva(0x518988); // SystemTime

fn system_time() -> *mut SYSTEMTIME {
    game_ptr(SYSTEM_TIME_RVA)
}

// ---- Rust version ----

pub fn pack(t: &SYSTEMTIME) -> u32 {
    // Converted: `rc2k_core::system::time::pack`.
    rc2k_core::system::time::pack(t.wYear, t.wMonth, t.wDay, t.wHour, t.wMinute)
}

pub unsafe fn packed_local_time() -> u32 {
    unsafe {
        GetLocalTime(system_time());
        pack(&system_time().read_unaligned())
    }
}

// ---- hook ----

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    let first = unsafe { packed_local_time() };
    if !shadow() {
        r.eax = first;
        return;
    }
    let mut o = *r;
    unsafe { call_original(&mut o) };
    let second = pack(&unsafe { system_time().read_unaligned() });
    if (o.eax != first && o.eax != second) || !o.preserved_except(r, EAX) {
        HOOK.mismatch(|| format!("original {:#x} rust {first:#x} then {second:#x}", o.eax));
    }
    r.eax = o.eax;
}

// ---- self-test ----

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let saved = unsafe { system_time().read_unaligned() };
    for _ in 0..5 {
        let first = unsafe { packed_local_time() };
        let before = Regs::SENTINEL;
        let mut ro = before;
        unsafe { call_original(&mut ro) };
        let left = pack(&unsafe { system_time().read_unaligned() });
        let after = unsafe { packed_local_time() };
        let ok = (ro.eax == first || ro.eax == after)
            && left == ro.eax
            && ro.preserved_except(&before, EAX);
        t.check(ok, || {
            format!(
                "original {:#x} (its SystemTime packs to {left:#x}); rust {first:#x} before, {after:#x} after",
                ro.eax
            )
        });
    }
    // Known values through the packing alone.
    let fixed = SYSTEMTIME {
        wYear: 1999,
        wMonth: 12,
        wDayOfWeek: 0,
        wDay: 31,
        wHour: 23,
        wMinute: 59,
        wSecond: 0,
        wMilliseconds: 0,
    };
    let expected = ((((1999u32 << 4 | 12) << 5 | 31) << 5 | 23) << 6) | 59;
    t.check(pack(&fixed) == expected, || {
        format!(
            "1999-12-31 23:59 packs to {:#x}, expected {expected:#x}",
            pack(&fixed)
        )
    });
    unsafe { system_time().write_unaligned(saved) };
    t.finish()
}
