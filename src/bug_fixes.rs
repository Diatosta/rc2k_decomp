//! The game's own bugs fixed in RAL.EXE's code, for the hooked game: where the Rust version
//! has the fix but the game's callers now run the original (conversion phase 4 removed the
//! hooks), the original's bytes are mended at start-up. Each fix checks the bytes first and
//! logs what it did; the standalone program needs none (it runs the Rust versions).
//! Every fix is in `notes/bugs.md` and `wiki/Fixed-bugs.md`.

use crate::framework::log;
use windows_sys::Win32::System::Diagnostics::Debug::FlushInstructionCache;
use windows_sys::Win32::System::Memory::{PAGE_EXECUTE_READWRITE, VirtualProtect};
use windows_sys::Win32::System::Threading::GetCurrentProcess;

/// One fix: where (as IDA shows it), the bytes there, what they become, what it mends.
struct Fix {
    at: usize,
    was: &'static [u8],
    now: &'static [u8],
    what: &'static str,
}

const FIXES: [Fix; 2] = [
    // `marks::emit_bursts` and `marks::emit_bursts_2`: the exit taken when the particle
    // ring fills inside a burst took only the loop's `push esi` off, not the counter
    // under it, so `popa` restored shifted registers and `retn` jumped to a register's
    // value (the crash driving over mud; seen 2026-10-23). `lea esp, [esp+4]` ->
    // `lea esp, [esp+8]`.
    Fix {
        at: 0x4CD741,
        was: &[0x8D, 0x64, 0x24, 0x04],
        now: &[0x8D, 0x64, 0x24, 0x08],
        what: "marks::emit_bursts: the full ring's exit",
    },
    Fix {
        at: 0x4CDC6E,
        was: &[0x8D, 0x64, 0x24, 0x04],
        now: &[0x8D, 0x64, 0x24, 0x08],
        what: "marks::emit_bursts_2: the full ring's exit",
    },
];

/// Applies every fix to the module at `base` (the image loaded at its own base, 0x400000).
pub fn apply(base: usize) {
    for fix in &FIXES {
        let addr = base + (fix.at - 0x40_0000);
        let here = unsafe { std::slice::from_raw_parts(addr as *const u8, fix.was.len()) };
        if here == fix.now {
            continue;
        }
        if here != fix.was {
            log(&format!(
                "bug_fixes: {} ({:#x}): bytes {here:02x?}, not {:02x?}: left",
                fix.what, fix.at, fix.was
            ));
            continue;
        }
        unsafe {
            let mut old = 0u32;
            VirtualProtect(
                addr as *const _,
                fix.now.len(),
                PAGE_EXECUTE_READWRITE,
                &mut old,
            );
            std::ptr::copy_nonoverlapping(fix.now.as_ptr(), addr as *mut u8, fix.now.len());
            VirtualProtect(addr as *const _, fix.now.len(), old, &mut old);
            FlushInstructionCache(GetCurrentProcess(), addr as *const _, fix.now.len());
        }
        log(&format!("bug_fixes: {} ({:#x}) mended", fix.what, fix.at));
    }
}
