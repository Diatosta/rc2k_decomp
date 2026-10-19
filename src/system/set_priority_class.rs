//! `system::set_priority_class` (0x410B68): the process's priority class set to eax
//! (`SetPriorityClass(GetCurrentProcess(), eax)`), unless `dword_518388` is not 0.
//!
//! In:  eax = the class (`start`: 0x80, high, at the start; 0x20, normal, at the end)
//! Out: every register preserved (eax, ecx, edx pushed and popped; the rest Win32 keeps)
//! Two callers, both in `start`. The Win32 calls are made from Rust (the same in both
//! programs). Shadow mode runs the original alone (it changes the process). Self-test: the
//! original and the Rust version with the switch 0 and 1, from the class there and to
//! another, the class after each read back; the class and the switch put back.

use crate::framework::{RawHook, Regs, SelfTest, game_read, game_write, hook_asm, rva, shadow};
use std::sync::atomic::AtomicUsize;
use windows_sys::Win32::System::Threading::{
    ABOVE_NORMAL_PRIORITY_CLASS, GetCurrentProcess, GetPriorityClass, SetPriorityClass,
};

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "system::set_priority_class",
    rva(0x410B68),
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

const KEEP_PRIORITY_RVA: usize = rva(0x518388); // dword_518388: not 0 = leave the class

// ---- Rust version ----

pub unsafe fn set_priority_class(class: u32) {
    unsafe {
        if game_read::<u32>(KEEP_PRIORITY_RVA) == 0 {
            SetPriorityClass(GetCurrentProcess(), class);
        }
    }
}

// ---- hook ----

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    unsafe { set_priority_class(r.eax) };
}

// ---- self-test ----

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    unsafe {
        let process = GetCurrentProcess();
        let class = GetPriorityClass(process);
        let switch = game_read::<u32>(KEEP_PRIORITY_RVA);
        let other = if class == ABOVE_NORMAL_PRIORITY_CLASS {
            0x20
        } else {
            ABOVE_NORMAL_PRIORITY_CLASS
        };
        for keep in [0u32, 1] {
            for to in [class, other] {
                game_write(KEEP_PRIORITY_RVA, keep);
                SetPriorityClass(process, class);
                let before = Regs {
                    eax: to,
                    ..Regs::SENTINEL
                };
                let mut ro = before;
                call_original(&mut ro);
                let theirs = GetPriorityClass(process);
                SetPriorityClass(process, class);
                set_priority_class(to);
                let mine = GetPriorityClass(process);
                let preserved = ro.preserved_except(&before, 0);
                t.check(preserved && theirs == mine, || {
                    format!(
                        "switch {keep}, class {class:#x} to {to:#x}: original preserved={preserved} {theirs:#x}; rust {mine:#x}"
                    )
                });
            }
        }
        SetPriorityClass(process, class);
        game_write(KEEP_PRIORITY_RVA, switch);
    }
    t.finish()
}
