//! `threading::wait_for_object` (0x411063): `WaitForSingleObject(eax, ebx)`.
//!
//! In:  eax = the handle, ebx = milliseconds
//! Out: every register preserved (eax, ecx, edx pushed and popped). Failed (`WAIT_FAILED`):
//!      carry and zero set. Else carry clear and zero set if the time ran out
//!      (`WAIT_TIMEOUT`, 0x102), clear if the object was signalled.
//! One caller, `net::receive_thread` (0x4183F4: the network's event, 1000 ms), which reads
//! only the zero flag (a failure counts as the time running out). The Win32 call is made
//! from Rust (the same in both programs). The thunk is not gated (the call waits). Shadow
//! mode runs the original alone (a wait takes what it waits for). Self-test: events of the
//! test's, signalled and not, and no handle, 0 ms each.

use crate::framework::{RawHook, Regs, SelfTest, ZERO_FLAG, hook_asm, rva, shadow};
use std::sync::atomic::AtomicUsize;
use windows_sys::Win32::Foundation::CloseHandle;
use windows_sys::Win32::System::Threading::WaitForSingleObject;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "threading::wait_for_object",
    rva(0x411063),
    thunk,
    &ORIGINAL,
)
.self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL,
    ungated
);

// ---- Rust version ----

pub use rc2k_core::threading::Waited;

pub fn wait_for_object(handle: u32, milliseconds: u32) -> Waited {
    // Converted: `rc2k_core::threading::Waited`.
    Waited::of(unsafe { WaitForSingleObject(handle as _, milliseconds) })
}

// ---- hook ----

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    let (carry, zero) = wait_for_object(r.eax, r.ebx).flags();
    r.set_carry(carry);
    r.set_zero(zero);
}

// ---- self-test ----

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    // Manual reset: a wait does not change them.
    let signalled = crate::threading::create_event::create_event(1, 1);
    let not = crate::threading::create_event::create_event(1, 0);
    for handle in [signalled, not, 0] {
        let before = Regs {
            eax: handle,
            ebx: 0,
            ..Regs::SENTINEL
        };
        let mut ro = before;
        unsafe { call_original(&mut ro) };
        let mine = wait_for_object(handle, 0).flags();
        let theirs = (ro.carry(), ro.eflags & ZERO_FLAG != 0);
        let preserved = ro.preserved_except(&before, 0);
        t.check(preserved && theirs == mine, || {
            format!("handle {handle:#x}: original {theirs:?} preserved={preserved}; rust {mine:?}")
        });
    }
    unsafe {
        CloseHandle(signalled as _);
        CloseHandle(not as _);
    }
    t.finish()
}
