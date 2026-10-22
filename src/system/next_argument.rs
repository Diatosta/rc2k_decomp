//! `system::next_argument` (0x4107B8): takes the next word off a command line.
//!
//! In:  eax = where to go on from
//! Out: eax = the word's start, ecx = its length, ebx = where to go on from next;
//!      everything else preserved
//! Spaces and tabs before the word are passed over (`sub_4107AA`). A word ends at a space,
//! a tab or a carriage return, which is passed over, or at the text's end. A word that
//! starts with `"` is what follows up to the next `"` or carriage return (passed over) or
//! the text's end. Twelve callers: the command line's reader and `sub_4162DB`.
//! Manual patch with a private copy (0x43 bytes, its call relocated): the call is the
//! first five bytes.

use crate::framework::{EAX, EBX, ECX, RawHook, Regs, SelfTest, hook_asm, rva, shadow};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new("system::next_argument", rva(0x4107B8), thunk, &ORIGINAL)
    .private_copy(0x43)
    .relocate_calls(&[0x00])
    .manual_patch()
    .self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- Rust version ----

/// (the word's start, its length, where to go on from).
pub unsafe fn next_argument(text: *const u8) -> (*const u8, u32, *const u8) {
    // Converted: `rc2k_core::strings::words::next_argument`, on the text in game memory
    // (whose NUL the original stops at too).
    let raw = unsafe { crate::bridge::Raw::new(text) };
    // `Raw` answers every byte, so the core never meets the end of its bytes.
    let w = rc2k_core::strings::words::next_argument(&raw, 0)
        .unwrap_or_else(|_| unreachable!("game memory answers every byte"));
    (
        text.wrapping_add(w.start),
        w.len as u32,
        text.wrapping_add(w.next),
    )
}

// ---- hook ----

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    let (start, len, next) = unsafe { next_argument(r.eax as *const u8) };
    if !shadow() {
        (r.eax, r.ecx, r.ebx) = (start as u32, len, next as u32);
        return;
    }
    let mut o = *r;
    unsafe { call_original(&mut o) };
    if (o.eax, o.ecx, o.ebx) != (start as u32, len, next as u32)
        || !o.preserved_except(r, EAX | ECX | EBX)
    {
        HOOK.mismatch(|| {
            format!(
                "text={:?}: original start={:#x} len={} next={:#x}; rust start={:#x} len={len} next={:#x}",
                unsafe { std::ffi::CStr::from_ptr(r.eax as *const _) },
                o.eax,
                o.ecx,
                o.ebx,
                start as u32,
                next as u32
            )
        });
    }
    let esp = r.esp;
    *r = o;
    r.esp = esp;
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    let (start, len, next) = unsafe { next_argument(r.eax as *const u8) };
    (r.eax, r.ecx, r.ebx) = (start as u32, len, next as u32);
    Ok(())
}

// ---- self-test ----

pub const TEST_LINES: [&[u8]; 12] = [
    b"\0",
    b"   \t \0",
    b"word\0",
    b"  two words\0",
    b"\"C:\\Program Files\\ral.exe\" -a -c 1\0",
    b"\"open quote\0",
    b"\"\" after\0",
    b"a\rb\tc d\0",
    b"\"quoted\rreturn\" x\0",
    b"x\"y z\" w\0",
    b"-i name \"second name\" 3 5 -u\0",
    b" \r \r\0",
];

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    for line in TEST_LINES {
        for from in 0..line.len() {
            let text = unsafe { line.as_ptr().add(from) };
            let before = Regs {
                eax: text as u32,
                ..Regs::SENTINEL
            };
            let mut ro = before;
            unsafe { call_original(&mut ro) };
            let (start, len, next) = unsafe { next_argument(text) };
            let preserved = ro.preserved_except(&before, EAX | ECX | EBX);
            let base = line.as_ptr() as u32;
            t.check(
                (ro.eax, ro.ecx, ro.ebx) == (start as u32, len, next as u32) && preserved,
                || {
                    format!(
                        "line={:?} from {from}: original start={} len={} next={} preserved={preserved}; rust start={} len={len} next={}",
                        String::from_utf8_lossy(line),
                        ro.eax.wrapping_sub(base),
                        ro.ecx,
                        ro.ebx.wrapping_sub(base),
                        (start as u32).wrapping_sub(base),
                        (next as u32).wrapping_sub(base)
                    )
                },
            );
        }
    }
    t.finish()
}
