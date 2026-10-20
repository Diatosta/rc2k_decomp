//! `files::append_file_name` (0x403040): a path's file name, after its last `/`, `\` or
//! `:`, appended at the cursor (no NUL). The search goes back from the NUL and stops
//! before the path's first byte (as `files::append_directory`'s): without a separator
//! after it the whole path is appended.
//!
//! In:  edx = the path, ebx = the cursor
//! Out: ebx past what was appended; eax, ecx, edx kept; the flags of `test cl, cl` on the
//!      NUL (zero set, carry clear)
//!
//! One caller, `install::run_after_exit`. Refuses (before anything) a null path. Shadow
//! mode compares both (`append_directory`'s).
//! Self-test: the original and the Rust version on the paths of `append_directory`'s test.

use crate::errors::BadInput;
use crate::files::append_directory::{shadow_compare, test_paths, text_of};
use crate::framework::{CARRY_FLAG, RawHook, Regs, ZERO_FLAG, hook_asm, rva, shadow};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("files::append_file_name", rva(0x403040), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// The Rust version: the cursor after.
pub unsafe fn append_file_name(path: *const u8, cursor: *mut u8) -> Result<*mut u8, BadInput> {
    if path.is_null() {
        return Err(BadInput("a null path"));
    }
    // Converted: `rc2k_core::paths::file_name`.
    let name = rc2k_core::paths::file_name(unsafe { text_of(path) });
    unsafe {
        std::ptr::copy(name.as_ptr(), cursor, name.len());
        Ok(cursor.add(name.len()))
    }
}

/// The registers it leaves (with the flags).
pub unsafe fn append_file_name_regs(r: &Regs) -> Result<Regs, BadInput> {
    let cursor = unsafe { append_file_name(r.edx as *const u8, r.ebx as *mut u8) }?;
    let mut g = Regs {
        ebx: cursor as u32,
        ..*r
    };
    g.eflags = g.eflags & !(CARRY_FLAG | ZERO_FLAG) | ZERO_FLAG;
    Ok(g)
}

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        shadow_compare(&HOOK, r, call_original, append_file_name_regs);
        return;
    }
    match unsafe { append_file_name_regs(r) } {
        Ok(g) => *r = Regs { esp: r.esp, ..g },
        Err(e) => unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) },
    }
}

fn self_test() -> (usize, usize) {
    test_paths(&HOOK, call_original, append_file_name_regs)
}
