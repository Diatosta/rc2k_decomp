//! `system::keep_start_time` (0x410F29): keeps the date and time the game was started at.
//!
//! In:  nothing
//! Out: every register preserved
//! `GetLocalTime` into the game's `SystemTime` (0x518988), and from it the weekday to
//! `byte_51839F`, day, month and year (a word) to `dword_5183A0`, hour, minute, second
//! and hundredths to `dword_5183A4`: what `logfile::date_text` and `math::seed_random`
//! read. One caller, `start`, as its first call: the hook DLL is not loaded then, so in
//! the game this never gets a call; the standalone program needs it.
//! Shadow mode runs the original alone (the clock moves).
//! Self-test: the original between two runs of the Rust version; what it kept has to be
//! one of theirs, the seconds and hundredths aside.

use crate::framework::{RawHook, Regs, SelfTest, game_ptr, game_write, hook_asm, rva, shadow};
use crate::service::harness::{capture, restore};
use std::sync::atomic::AtomicUsize;
use windows_sys::Win32::Foundation::SYSTEMTIME;
use windows_sys::Win32::System::SystemInformation::GetLocalTime;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("system::keep_start_time", rva(0x410F29), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- game data ----

const SYSTEM_TIME_RVA: usize = rva(0x518988); // SystemTime
const KEPT_RVA: usize = rva(0x51839F); // byte_51839F, dword_5183A0, dword_5183A4
const KEPT_LEN: usize = 9;

// ---- Rust version ----

/// The nine bytes kept: weekday, day, month, year (two), hour, minute, second,
/// hundredths.
pub fn kept(t: &SYSTEMTIME) -> [u8; KEPT_LEN] {
    // Converted: `rc2k_core::system::time::start_time`.
    rc2k_core::system::time::start_time(&rc2k_core::system::time::LocalTime {
        year: t.wYear,
        month: t.wMonth,
        weekday: t.wDayOfWeek,
        day: t.wDay,
        hour: t.wHour,
        minute: t.wMinute,
        second: t.wSecond,
        milliseconds: t.wMilliseconds,
    })
}

pub unsafe fn keep_start_time() {
    unsafe {
        let time = game_ptr::<SYSTEMTIME>(SYSTEM_TIME_RVA);
        GetLocalTime(time);
        game_write(KEPT_RVA, kept(&time.read_unaligned()));
    }
}

// ---- hook ----

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    unsafe { keep_start_time() };
}

// ---- self-test ----

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let regions = [
        (game_ptr::<u8>(KEPT_RVA) as u32, KEPT_LEN),
        (game_ptr::<u8>(SYSTEM_TIME_RVA) as u32, 16),
    ];
    let saved = capture(&regions);
    for _ in 0..3 {
        let run = |rust: bool| unsafe {
            restore(&regions, &vec![0xAA; saved.len()]);
            let preserved = if rust {
                keep_start_time();
                true
            } else {
                let before = Regs::SENTINEL;
                let mut ro = before;
                call_original(&mut ro);
                ro.preserved_except(&before, 0)
            };
            (preserved, capture(&regions)[..KEPT_LEN].to_vec())
        };
        let (first, theirs, second) = (run(true), run(false), run(true));
        // Up to the minute: the seconds and hundredths move on between the runs.
        let same = |mine: &(bool, Vec<u8>)| mine.1[..7] == theirs.1[..7];
        t.check(theirs.0 && (same(&first) || same(&second)), || {
            format!(
                "original {theirs:x?}; rust before {:x?} and after {:x?}",
                first.1, second.1
            )
        });
    }
    restore(&regions, &saved);
    t.finish()
}
