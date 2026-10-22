//! `system::read_windows_version` (0x410AC5): `VersionInformation` (0x518408, an
//! `OSVERSIONINFOA` of 0x94 bytes) filled by `GetVersionExA`, its size set first.
//!
//! In:  nothing
//! Out: every register preserved (`pusha` .. `popa`)
//! What `logfile::start_log` writes to the log ("Windows" major.minor.build). One caller,
//! `start` (0x401024); in the game this never gets a call (the hook DLL is loaded later).
//! The Win32 call is made from Rust (the same in both programs). Shadow mode: both (the
//! call only reads), compared. Self-test: the original and the Rust version from 0xAA, the
//! 0x94 bytes compared.

use crate::framework::{RawHook, Regs, SelfTest, game_addr, hook_asm, rva, shadow};
use crate::service::harness::{Region, capture, restore};
use std::sync::atomic::AtomicUsize;
use windows_sys::Win32::System::SystemInformation::{GetVersionExA, OSVERSIONINFOA};

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "system::read_windows_version",
    rva(0x410AC5),
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

// ---- game data ----

const VERSION_INFO_RVA: usize = rva(0x518408); // VersionInformation
const VERSION_INFO_LEN: usize = 0x94;

// ---- Rust version ----

pub unsafe fn read_windows_version() {
    let info = game_addr(VERSION_INFO_RVA) as *mut OSVERSIONINFOA;
    unsafe {
        (info as *mut u32).write_unaligned(VERSION_INFO_LEN as u32);
        GetVersionExA(info);
    }
}

// ---- hook ----

fn region() -> [Region; 1] {
    [(game_addr(VERSION_INFO_RVA) as u32, VERSION_INFO_LEN)]
}

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if !shadow() {
        unsafe { read_windows_version() };
        return;
    }
    let regions = region();
    let before = capture(&regions);
    unsafe { read_windows_version() };
    let mine = capture(&regions);
    restore(&regions, &before);
    unsafe { call_original(r) };
    let theirs = capture(&regions);
    if mine != theirs {
        HOOK.mismatch(|| format!("original {theirs:02x?}; rust {mine:02x?}"));
    }
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(_: &mut Regs) -> Result<(), crate::errors::Refused> {
    unsafe { read_windows_version() };
    Ok(())
}

// ---- self-test ----

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let regions = region();
    let saved = capture(&regions);
    restore(&regions, &[0xAA; VERSION_INFO_LEN]);
    let before = Regs::SENTINEL;
    let mut ro = before;
    unsafe { call_original(&mut ro) };
    let theirs = capture(&regions);
    restore(&regions, &[0xAA; VERSION_INFO_LEN]);
    unsafe { read_windows_version() };
    let mine = capture(&regions);
    let preserved = ro.preserved_except(&before, 0);
    t.check(preserved && theirs == mine, || {
        format!("original preserved={preserved} {theirs:02x?}; rust {mine:02x?}")
    });
    restore(&regions, &saved);
    t.finish()
}
