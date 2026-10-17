//! `race::record_replay_header` (0x460246): the race's settings (the arcade's seed, start
//! place and count, the stage, the mode, the players, the cars, the first four kinds, the
//! two players' absences) into the replay's car header (`dword_719B14` ..), each logged
//! ("[replayheader.x]" and the value).
//!
//! In:  nothing
//! Out: every register preserved
//!
//! See `race/race_end.rs`. Callers: `sub_443F0B`, `sub_443F96` (`race::open_replay`
//! follows; no flags read). Shadow mode runs the original alone (it logs). Self-test: a
//! copy whose 26 log calls note themselves, against the Rust version on the same, the
//! header's fields filled two ways.

use crate::framework::{RawHook, Regs, SelfTest, game_addr, hook_asm, rva, shadow};
use crate::race::race_end::*;
use crate::service::harness::{capture, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "race::record_replay_header",
    rva(0x460246),
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

/// The log entries' places.
const LOGS: &[(usize, u32)] = &[(0x401427, LOG_WRITE), (0x401476, LOG_INT_NL)];

/// `log_write_l1` / `log_int_l1_nl` in the game.
fn log_part(place: u32, r: &mut Regs) -> Result<(), crate::errors::Refused> {
    unsafe {
        if place == LOG_WRITE {
            crate::logfile::log_write::log_write(r.eax as *const u8, false, 1);
        } else {
            crate::logfile::log_int::log_int(r.eax as i32, true, 1);
        }
    }
    Ok(())
}

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    if let Err(e) = unsafe { record_replay_header(r, &mut log_part) } {
        unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) };
    }
}

fn self_test() -> (usize, usize) {
    use crate::race::mud::{copy, run_both, test_part};
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let calls: Vec<usize> = (0..26)
        .map(|k| [0xA3usize, 0xAF][k % 2] + 0x19 * (k / 2))
        .collect();
    let copy = match copy(
        0x460246,
        0x1E2,
        &[0xFF, 0x35, 0x9C, 0x51, 0x60, 0x00],
        &calls,
        LOGS,
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    let regions = replay_header_regions();
    let saved = capture(&regions);
    for fill in [0u8, 0xA5] {
        run_both(
            &mut t,
            &format!("filled {fill:#x}"),
            &copy,
            &regions,
            &|| {
                for &(a, len) in &regions {
                    unsafe { std::ptr::write_bytes(a as *mut u8, fill, len) };
                }
            },
            &|r| unsafe { record_replay_header(r, &mut test_part) },
        );
    }
    let _ = game_addr(0);
    restore(&regions, &saved);
    t.finish()
}
