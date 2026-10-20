//! `files::append_directory` (0x403070): a path's directory, up to and with its last `/`,
//! `\` or `:`, appended at the cursor (no NUL). The search goes back from the NUL and stops
//! before the path's first byte: a separator there alone is not found, and a path without
//! one appends nothing.
//!
//! In:  edx = the path, ebx = the cursor
//! Out: ebx past what was appended; eax, ecx, edx kept; the flags of the last `cmp eax,
//!      edx`: carry set (and zero clear) after a copy, zero set (carry clear) without one
//!
//! Five callers in the installer and `install::run_after_exit`. `files::append_file_name`
//! is its other half. Refuses (before anything) a null path. Self-test: the original and
//! the Rust version on paths of the test's (0xAA-filled buffers compared whole), the
//! registers and the two flags. Shadow mode compares both (the Rust version writing into a
//! buffer of its own).

use crate::errors::BadInput;
use crate::framework::{CARRY_FLAG, RawHook, Regs, SelfTest, ZERO_FLAG, hook_asm, rva, shadow};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("files::append_directory", rva(0x403070), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// Converted: `rc2k_core::files::paths`.
pub use rc2k_core::files::paths::{is_separator, last_separator};

/// The bytes of a NUL-terminated text, without the NUL (an unbounded scan, as the
/// original's).
pub unsafe fn text_of<'a>(text: *const u8) -> &'a [u8] {
    unsafe { std::ffi::CStr::from_ptr(text.cast()).to_bytes() }
}

/// The Rust version: the cursor after, and whether a directory was found.
pub unsafe fn append_directory(
    path: *const u8,
    cursor: *mut u8,
) -> Result<(*mut u8, bool), BadInput> {
    if path.is_null() {
        return Err(BadInput("a null path"));
    }
    // Converted: `rc2k_core::files::paths::directory`.
    let Some(directory) = rc2k_core::files::paths::directory(unsafe { text_of(path) }) else {
        return Ok((cursor, false));
    };
    unsafe {
        std::ptr::copy(directory.as_ptr(), cursor, directory.len());
        Ok((cursor.add(directory.len()), true))
    }
}

/// The registers it leaves (with the flags).
pub unsafe fn append_directory_regs(r: &Regs) -> Result<Regs, BadInput> {
    let (cursor, found) = unsafe { append_directory(r.edx as *const u8, r.ebx as *mut u8) }?;
    let mut g = Regs {
        ebx: cursor as u32,
        ..*r
    };
    g.eflags = g.eflags & !(CARRY_FLAG | ZERO_FLAG) | if found { CARRY_FLAG } else { ZERO_FLAG };
    Ok(g)
}

/// Shadow mode for both: the original, then the Rust version into a buffer of its own;
/// what each wrote and the registers and flags compared.
pub fn shadow_compare(
    hook: &RawHook,
    r: &mut Regs,
    original: unsafe extern "C" fn(*mut Regs),
    rust: unsafe fn(&Regs) -> Result<Regs, BadInput>,
) {
    let entry = *r;
    unsafe { original(r) };
    if entry.edx == 0 {
        return;
    }
    let len = unsafe { text_of(entry.edx as *const u8) }.len();
    let mut mine = vec![0u8; len + 1];
    let scratch = mine.as_mut_ptr() as u32;
    let Ok(mut g) = (unsafe {
        rust(&Regs {
            ebx: scratch,
            ..entry
        })
    }) else {
        return;
    };
    let n = g.ebx.wrapping_sub(scratch) as usize;
    g.ebx = entry.ebx.wrapping_add(n as u32);
    let theirs = unsafe {
        std::slice::from_raw_parts(
            entry.ebx as *const u8,
            r.ebx.wrapping_sub(entry.ebx) as usize,
        )
    };
    if seen(&g) != seen(r) || theirs != &mine[..n.min(mine.len())] {
        hook.mismatch(|| {
            format!(
                "{}: original wrote {}, rust {}",
                crate::strings::utils::bytes_str(unsafe { text_of(entry.edx as *const u8) }),
                crate::strings::utils::bytes_str(theirs),
                crate::strings::utils::bytes_str(&mine[..n.min(mine.len())])
            )
        });
    }
}

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        shadow_compare(&HOOK, r, call_original, append_directory_regs);
        return;
    }
    match unsafe { append_directory_regs(r) } {
        Ok(g) => *r = Regs { esp: r.esp, ..g },
        Err(e) => unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) },
    }
}

/// The paths both tests use.
pub const TEST_PATHS: [&[u8]; 12] = [
    b"",
    b"a",
    b"\\",
    b"\\a",
    b"ereg\\eregip.exe",
    b"c:\\games\\rally\\ral.exe",
    b"c:",
    b"c:ral.exe",
    b"dir/sub\\",
    b"a/b/c",
    b"..\\x",
    b"name.txt",
];

/// The registers and flags compared: the seven, carry and zero.
pub fn seen(r: &Regs) -> ([u32; 7], u32) {
    (
        crate::sound::turn_parts::regs7(r),
        r.eflags & (CARRY_FLAG | ZERO_FLAG),
    )
}

/// A test of one of the two: `rust` and the original on each path, buffers compared.
pub fn test_paths(
    hook: &RawHook,
    original: unsafe extern "C" fn(*mut Regs),
    rust: unsafe fn(&Regs) -> Result<Regs, BadInput>,
) -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(hook) else {
        return (0, 0);
    };
    for path in TEST_PATHS {
        let mut text = path.to_vec();
        text.push(0);
        let mut mine = vec![0xAAu8; 0x40];
        let mut theirs = mine.clone();
        let entry = Regs {
            edx: text.as_ptr() as u32,
            ..Regs::SENTINEL
        };
        let g = unsafe {
            rust(&Regs {
                ebx: mine.as_mut_ptr() as u32 + 3,
                ..entry
            })
        };
        let mut o = Regs {
            ebx: theirs.as_mut_ptr() as u32 + 3,
            ..entry
        };
        unsafe { original(&mut o) };
        let ok = g.is_ok_and(|g| {
            let mut g = g;
            g.ebx = g.ebx.wrapping_sub(mine.as_ptr() as u32);
            let mut o = o;
            o.ebx = o.ebx.wrapping_sub(theirs.as_ptr() as u32);
            seen(&g) == seen(&o)
        });
        t.check(ok && mine == theirs, || {
            format!(
                "{}: wrote {} / {}",
                crate::strings::utils::bytes_str(path),
                crate::strings::utils::bytes_str(&theirs),
                crate::strings::utils::bytes_str(&mine)
            )
        });
    }
    t.finish()
}

fn self_test() -> (usize, usize) {
    test_paths(&HOOK, call_original, append_directory_regs)
}
