//! `threading::create_mutex` (0x41107F): a mutex without a name and without an owner,
//! `CreateMutexA(0, 0, 0)`.
//!
//! In:  nothing
//! Out: eax = the handle; `cmp eax, 1`: carry set (and zero clear) if there is none; ecx,
//!      edx pushed and popped, the rest kept by Win32
//! One caller, `net::set_up` (0x4184BF), which raises fatal error 0x80168502 on the carry.
//! The Win32 call is made from Rust (the same in both programs). Shadow mode runs the
//! original alone (a mutex cannot be made twice). Self-test: both make one, which another
//! thread can take at once (no owner), then closed.

use crate::framework::{EAX, RawHook, Regs, SelfTest, hook_asm, rva, shadow};
use crate::threading::create_event::handle_flags;
use std::sync::atomic::AtomicUsize;
use windows_sys::Win32::Foundation::CloseHandle;
use windows_sys::Win32::System::Threading::{CreateMutexA, ReleaseMutex, WaitForSingleObject};

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("threading::create_mutex", rva(0x41107F), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- Rust version ----

/// The handle, 0 for none.
pub fn create_mutex() -> u32 {
    unsafe { CreateMutexA(std::ptr::null(), 0, std::ptr::null()) as u32 }
}

// ---- hook ----

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    r.eax = create_mutex();
    let (carry, zero) = handle_flags(r.eax);
    r.set_carry(carry);
    r.set_zero(zero);
}

// ---- self-test ----

/// Whether another thread can take the mutex at once (it has no owner).
fn free(handle: u32) -> bool {
    std::thread::spawn(move || unsafe {
        let taken = WaitForSingleObject(handle as _, 0) == 0;
        if taken {
            ReleaseMutex(handle as _);
        }
        taken
    })
    .join()
    .unwrap_or(false)
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let before = Regs::SENTINEL;
    let mut ro = before;
    unsafe { call_original(&mut ro) };
    let mine = create_mutex();
    let preserved = ro.preserved_except(&before, EAX);
    let (theirs_free, mine_free) = (free(ro.eax), free(mine));
    t.check(
        ro.eax != 0 && mine != 0 && !ro.carry() && preserved && theirs_free && mine_free,
        || {
            format!(
                "original {:#x} carry={} preserved={preserved} free={theirs_free}; rust {mine:#x} free={mine_free}",
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
    t.finish()
}
