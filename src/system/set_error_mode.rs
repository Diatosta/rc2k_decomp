//! `system::set_error_mode` (0x402B7C): `SetErrorMode(SEM_FAILCRITICALERRORS)` (no
//! "insert a disc" boxes from Windows), the mode before it kept in `uMode` (0x4E05A0), and
//! `dword_4E059C` 1.
//!
//! In:  nothing
//! Out: every register preserved (`pusha` .. `popa`)
//! One caller, `start` (0x401035); in the game this never gets a call (the hook DLL is
//! loaded later). The Win32 call is made from Rust (the same in both programs). Manual
//! patch with a private copy (0x19 bytes, the call relocated): the call starts at +3.
//! Shadow mode runs the original alone (it changes the process's mode). Self-test: the
//! original and the Rust version from the same mode, both variables compared; the mode and
//! the variables put back.

use crate::framework::{RawHook, Regs, SelfTest, game_addr, hook_asm, rva, shadow};
use crate::service::harness::{Region, capture, restore};
use std::sync::atomic::AtomicUsize;
use windows_sys::Win32::System::Diagnostics::Debug::{SEM_FAILCRITICALERRORS, SetErrorMode};

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new("system::set_error_mode", rva(0x402B7C), thunk, &ORIGINAL)
    .private_copy(0x19)
    .relocate_calls(&[0x03])
    .manual_patch()
    .self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- game data ----

const KEPT_MODE_RVA: usize = rva(0x4E05A0); // uMode
const MODE_SET_RVA: usize = rva(0x4E059C); // dword_4E059C

// ---- Rust version ----

pub unsafe fn set_error_mode() {
    unsafe {
        let was = SetErrorMode(SEM_FAILCRITICALERRORS);
        crate::framework::game_write(KEPT_MODE_RVA, was);
        crate::framework::game_write(MODE_SET_RVA, 1u32);
    }
}

// ---- hook ----

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    unsafe { set_error_mode() };
}

// ---- self-test ----

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let regions: [Region; 2] = [
        (game_addr(KEPT_MODE_RVA) as u32, 4),
        (game_addr(MODE_SET_RVA) as u32, 4),
    ];
    let saved = capture(&regions);
    unsafe {
        let mode = SetErrorMode(0);
        SetErrorMode(mode);
        // From the mode as it is, and from 0 (so that the kept one differs from 1).
        for from in [mode, 0] {
            SetErrorMode(from);
            restore(&regions, &[0xAA; 8]);
            let before = Regs::SENTINEL;
            let mut ro = before;
            call_original(&mut ro);
            let theirs = (capture(&regions), SetErrorMode(from));
            restore(&regions, &[0xAA; 8]);
            set_error_mode();
            let mine = (capture(&regions), SetErrorMode(from));
            let preserved = ro.preserved_except(&before, 0);
            t.check(preserved && theirs == mine, || {
                format!(
                    "from {from:#x}: original preserved={preserved} {theirs:x?}; rust {mine:x?}"
                )
            });
        }
        SetErrorMode(mode);
    }
    restore(&regions, &saved);
    t.finish()
}
