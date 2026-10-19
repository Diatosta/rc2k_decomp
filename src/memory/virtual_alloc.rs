//! `memory::virtual_alloc` (0x402B1A): eax bytes of committed read-write memory from
//! `VirtualAlloc`, or none for 0 bytes.
//!
//! In:  eax = bytes
//! Out: eax = the memory (0 for 0 bytes); carry clear; ecx, edx pushed and popped, the rest
//!      kept by Win32. None to be had: fatal error 0x80108571 "Need more virtual memory!"
//! Two callers: `memory::allocate` (whose Rust version makes the call itself) and
//! `memory::open_own_heap`. Manual patch with a private copy (0x21 bytes; the call and the
//! `jb` to the fatal stub relocated): the `jz` at +4 is in the first bytes. Shadow mode runs
//! the original alone (memory cannot be taken twice). Self-test: 0 bytes, and some sizes
//! from each, the memory checked (page-aligned, zeroed, writable) and given back.

use crate::errors::{FatalError, Refused};
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva, shadow};
use std::sync::atomic::AtomicUsize;
use windows_sys::Win32::System::Memory::{
    MEM_COMMIT, MEM_RELEASE, MEM_RESERVE, PAGE_READWRITE, VirtualAlloc, VirtualFree,
};

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new("memory::virtual_alloc", rva(0x402B1A), thunk, &ORIGINAL)
    .private_copy(0x21)
    .relocate_calls(&[0x10, 0x18])
    .manual_patch()
    .self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

pub const NO_MEMORY: FatalError = FatalError {
    code: 0x8010_8571,
    message: "Need more virtual memory!",
};

// ---- Rust version ----

/// The memory, 0 for 0 bytes.
pub unsafe fn virtual_alloc(bytes: u32) -> Result<u32, Refused> {
    if bytes == 0 {
        return Ok(0);
    }
    let at = unsafe {
        VirtualAlloc(
            std::ptr::null(),
            bytes as usize,
            MEM_COMMIT | MEM_RESERVE,
            PAGE_READWRITE,
        )
    } as u32;
    if at == 0 {
        return Err(NO_MEMORY.into());
    }
    Ok(at)
}

// ---- hook ----

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    match unsafe { virtual_alloc(r.eax) } {
        Ok(at) => {
            // 0 bytes: `test eax, eax` (zero set); else `cmp eax, 1` (zero clear, as a
            // block is never at 1).
            r.set_zero(at == 0);
            r.set_carry(false);
            r.eax = at;
        }
        Err(e) => unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) },
    }
}

// ---- self-test ----

/// Whether `at` is a usable block of `bytes`: page-aligned, zeroed, writable.
unsafe fn usable(at: u32, bytes: u32) -> bool {
    if at == 0 || at & 0xFFF != 0 {
        return false;
    }
    let p = at as *mut u8;
    unsafe {
        let zeroed = (0..bytes as usize).all(|k| p.add(k).read() == 0);
        p.add(bytes as usize - 1).write(0x5A);
        zeroed && p.add(bytes as usize - 1).read() == 0x5A
    }
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    for bytes in [0u32, 1, 0x1000, 0x12345] {
        let before = Regs {
            eax: bytes,
            ..Regs::SENTINEL
        };
        let mut ro = before;
        unsafe { call_original(&mut ro) };
        let mine = unsafe { virtual_alloc(bytes) };
        let preserved = ro.preserved_except(&before, crate::framework::EAX);
        let ok = match (bytes, mine) {
            (0, Ok(0)) => ro.eax == 0,
            (_, Ok(at)) => unsafe { usable(ro.eax, bytes) && usable(at, bytes) },
            _ => false,
        };
        t.check(ok && preserved && !ro.carry(), || {
            format!(
                "{bytes:#x} bytes: original {:#x} carry={} preserved={preserved}; rust {mine:x?}",
                ro.eax,
                ro.carry()
            )
        });
        unsafe {
            for at in [ro.eax, mine.unwrap_or(0)] {
                if at != 0 {
                    VirtualFree(at as *mut _, 0, MEM_RELEASE);
                }
            }
        }
    }
    t.finish()
}
