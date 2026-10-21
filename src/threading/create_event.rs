//! `threading::create_event` (0x41103C): an event without a name,
//! `CreateEventA(0, eax, ebx, 0)`.
//!
//! In:  eax = manual reset (0 or 1), ebx = signalled at first (0 or 1)
//! Out: eax = the handle; `cmp eax, 1`: carry set (and zero clear) if there is none; ecx,
//!      edx pushed and popped, the rest kept by Win32
//! One caller, `net::set_up` (0x4184AF: eax 0, ebx 0), which raises fatal error 0x80168501
//! on the carry. The Win32 call is made from Rust (the same in both programs). Shadow mode
//! runs the original alone (an event cannot be made twice). Self-test: both make events of
//! each kind, the handles' state read (`WaitForSingleObject` with 0 ms), then closed.

use crate::framework::{EAX, RawHook, Regs, SelfTest, hook_asm, rva, shadow};
use std::sync::atomic::AtomicUsize;
use windows_sys::Win32::Foundation::CloseHandle;
use windows_sys::Win32::System::Threading::{CreateEventA, WaitForSingleObject};

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("threading::create_event", rva(0x41103C), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- Rust version ----

/// The handle, 0 for none.
pub fn create_event(manual_reset: u32, signalled: u32) -> u32 {
    unsafe {
        CreateEventA(
            std::ptr::null(),
            manual_reset as i32,
            signalled as i32,
            std::ptr::null(),
        ) as u32
    }
}

/// `cmp eax, 1` on a handle: (carry, zero).
// Converted: `rc2k_core::threading::handle_flags`.
pub use rc2k_core::threading::handle_flags;

// ---- hook ----

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    r.eax = create_event(r.eax, r.ebx);
    let (carry, zero) = handle_flags(r.eax);
    r.set_carry(carry);
    r.set_zero(zero);
}

// ---- self-test ----

/// An event's state: 0 signalled, 0x102 not.
fn state(handle: u32) -> u32 {
    unsafe { WaitForSingleObject(handle as _, 0) }
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    for manual in [0u32, 1] {
        for signalled in [0u32, 1] {
            let before = Regs {
                eax: manual,
                ebx: signalled,
                ..Regs::SENTINEL
            };
            let mut ro = before;
            unsafe { call_original(&mut ro) };
            let mine = create_event(manual, signalled);
            // Read twice: an auto-reset event is reset by the first wait.
            let (theirs, ours) = ((state(ro.eax), state(ro.eax)), (state(mine), state(mine)));
            let preserved = ro.preserved_except(&before, EAX);
            t.check(
                ro.eax != 0 && mine != 0 && !ro.carry() && preserved && theirs == ours,
                || {
                    format!(
                        "manual {manual}, signalled {signalled}: original {:#x} carry={} preserved={preserved} {theirs:x?}; rust {mine:#x} {ours:x?}",
                        ro.eax,
                        ro.carry()
                    )
                },
            );
            unsafe {
                for h in [ro.eax, mine] {
                    if h != 0 {
                        CloseHandle(h as _);
                    }
                }
            }
        }
    }
    t.finish()
}
