//! `strings::text_length` (0x4022B8): the length of a NUL-terminated text.
//!
//! In:  ebx = the text
//! Out: ecx = its length without the NUL; the flags of the last compare (zero set, carry
//!      clear); everything else preserved
//! 14 call sites (the picture loader, the extension swap, path handling). There is no limit:
//! it runs until it finds a NUL.
//! Private copy (0x0C bytes, nothing to relocate): the loop comes back to the third byte,
//! inside what the patch overwrites.
//! Refuses a null pointer.

use crate::errors::BadInput;
use crate::framework::{ECX, RawHook, Regs, SelfTest, hook_asm, rva, shadow};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new("strings::text_length", rva(0x4022B8), thunk, &ORIGINAL)
    .private_copy(0x0C)
    .self_test(self_test);

// The original returns from `cmp byte ptr [ecx + ebx], 0` on the NUL: the same compare again
// leaves the same flags.
hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL,
    epilogue = "cmp byte ptr [ecx + ebx], 0"
);

// ---- Rust version ----

pub unsafe fn text_length(text: *const u8) -> Result<u32, BadInput> {
    if text.is_null() {
        return Err(BadInput("no text"));
    }
    let mut length = 0u32;
    while unsafe { text.add(length as usize).read() } != 0 {
        length = length.wrapping_add(1);
    }
    Ok(length)
}

// ---- hook ----

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    let mine = match unsafe { text_length(r.ebx as *const u8) } {
        Ok(length) => length,
        Err(e) => {
            unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) };
            return;
        }
    };
    if !shadow() {
        r.ecx = mine;
        return;
    }
    let mut o = *r;
    unsafe { call_original(&mut o) };
    if o.ecx != mine || !o.preserved_except(r, ECX) {
        HOOK.mismatch(|| format!("text={:#x}: original {} rust {mine}", r.ebx, o.ecx));
    }
    r.ecx = o.ecx;
}

// ---- self-test ----

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let long = [b'x'; 300];
    let texts: [&[u8]; 5] = [b"", b"a", b"files\\g\\x.png", b"\xFF\x01 two", &long];
    for text in texts {
        let mut buffer = text.to_vec();
        buffer.extend_from_slice(b"\0after\0");
        let before = Regs {
            ebx: buffer.as_ptr() as u32,
            ..Regs::SENTINEL
        };
        let mut ro = before;
        unsafe { call_original(&mut ro) };
        let mine = unsafe { text_length(buffer.as_ptr()) };
        let preserved = ro.preserved_except(&before, ECX);
        t.check(
            mine == Ok(ro.ecx) && preserved && !ro.carry() && ro.ecx == text.len() as u32,
            || {
                format!(
                    "a text of {} bytes: original {} carry={} preserved={preserved}; rust {mine:?}",
                    text.len(),
                    ro.ecx,
                    ro.carry()
                )
            },
        );
    }
    t.finish()
}
