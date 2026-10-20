//! `settings::copy_value` (0x4115F5): copies a value out of a settings text.
//!
//! In:  eax = where the value starts (0: nothing is done), ebx = cursor
//! Out: ebx = the cursor after the value; everything else preserved
//! Copies up to a NUL, a carriage return, a line feed or a tab; a value that starts with
//! `"` is copied without it and up to the next `"` instead of a tab. No NUL is written.
//! Three callers. Manual patch with a private copy (0x2E bytes): a short jump in the
//! first bytes.

use crate::bridge::{Raw, read_through};
use crate::framework::{EBX, RawHook, Regs, SelfTest, hook_asm, rva, shadow};
use crate::settings::find_key::TEST_TEXTS;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new("settings::copy_value", rva(0x4115F5), thunk, &ORIGINAL)
    .private_copy(0x2E)
    .manual_patch()
    .self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- Rust version ----

/// The value's bytes; none for a null pointer.
pub unsafe fn value_at(value: *const u8) -> Vec<u8> {
    if value.is_null() {
        return Vec::new();
    }
    // Converted: `rc2k_core::files::settings::value`.
    let text = unsafe { Raw::new(value) };
    read_through(rc2k_core::files::settings::value(&text, 0))
}

/// The cursor after the value.
pub unsafe fn copy_value(value: *const u8, cursor: *mut u8) -> *mut u8 {
    unsafe {
        let bytes = value_at(value);
        std::ptr::copy(bytes.as_ptr(), cursor, bytes.len());
        cursor.add(bytes.len())
    }
}

// ---- hook ----

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    let (value, cursor) = (r.eax as *const u8, r.ebx as *mut u8);
    if !shadow() {
        r.ebx = unsafe { copy_value(value, cursor) } as u32;
        return;
    }
    let mine = unsafe { value_at(value) };
    let mut o = *r;
    unsafe { call_original(&mut o) };
    let written = o.ebx.wrapping_sub(r.ebx) as usize;
    let same = written == mine.len()
        && unsafe { std::slice::from_raw_parts(cursor, written) } == &mine[..];
    if !same || !o.preserved_except(r, EBX) {
        HOOK.mismatch(|| {
            format!(
                "value={:#x}: original wrote {written} bytes; rust {:?}",
                r.eax,
                String::from_utf8_lossy(&mine)
            )
        });
    }
    r.ebx = o.ebx;
    r.eflags = o.eflags;
}

// ---- self-test ----

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let more: [&[u8]; 6] = [
        b"\"\0",
        b"\"open\0",
        b"\"a \"b\"\0",
        b"one\ttwo\0",
        b"\"one\ttwo\"\0",
        b"a b c\r\n\0",
    ];
    for text in TEST_TEXTS.iter().chain(&more) {
        // From every place in the text, and once from nowhere.
        for from in 0..text.len() + 1 {
            let value = if from == text.len() {
                std::ptr::null()
            } else {
                unsafe { text.as_ptr().add(from) }
            };
            let run = |rust: bool| {
                let mut out = [0xAAu8; 0x40];
                let before = Regs {
                    eax: value as u32,
                    ebx: out.as_mut_ptr() as u32 + 4,
                    ..Regs::SENTINEL
                };
                let length = if rust {
                    Some(unsafe { copy_value(value, out.as_mut_ptr().add(4)) } as u32)
                } else {
                    let mut ro = before;
                    unsafe { call_original(&mut ro) };
                    ro.preserved_except(&before, EBX).then_some(ro.ebx)
                }
                .map(|end| end.wrapping_sub(out.as_ptr() as u32 + 4));
                (length, out)
            };
            let (theirs, mine) = (run(false), run(true));
            t.check(theirs == mine && theirs.0.is_some(), || {
                format!(
                    "text={:?} from {from}: original {:?} {:?}; rust {:?} {:?}",
                    String::from_utf8_lossy(text),
                    theirs.0,
                    crate::strings::utils::bytes_str(&theirs.1),
                    mine.0,
                    crate::strings::utils::bytes_str(&mine.1)
                )
            });
        }
    }
    t.finish()
}
