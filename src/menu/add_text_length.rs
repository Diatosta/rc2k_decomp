//! `menu::add_text_length` (0x46F4CE): adds the length of the NUL-terminated text at edi to
//! the text entry's length. Its callers use it with the length at 0, to set it to the length
//! of a text just restored.
//!
//! In:  edi = text
//! Out: ecx = ecx + 1 (an `inc ecx` after the `popa`); everything else preserved
//! Refuses a text with no NUL in its first 0x1000 bytes.

use crate::errors::BadInput;
use crate::errors::Refused;
use crate::framework::ECX;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::menu::text_entry::length;
use crate::menu::text_entry::{LENGTH_RVA, TEXT_LEN, backup, backup_region, set_length};
use crate::screens::harness::Left;
use crate::service::harness::{Region, at, capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("menu::add_text_length", rva(0x46F4CE), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- Rust version ----

pub unsafe fn add_text_length(text: *const u8) -> Result<(), BadInput> {
    let Some(len) = (0..0x1000).find(|&i| unsafe { text.add(i).read() } == 0) else {
        return Err(BadInput("text without a NUL"));
    };
    set_length(length().wrapping_add(len as i32));
    Ok(())
}

// ---- hook ----

fn regions(field: u32) -> [Region; 3] {
    [(field, 0x28), backup_region(), at(LENGTH_RVA)]
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    unsafe { add_text_length(r.edi as *const u8) }?;
    Ok(Left::nothing().with(ECX, r.ecx.wrapping_add(1)))
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &[at(LENGTH_RVA)], &rust);
}

// ---- self-test: a text field of its own ----

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let mut field = [0u8; 0x28];
    let regions = regions(field.as_mut_ptr() as u32);
    let inputs = [backup_region(), at(LENGTH_RVA)];
    let saved = capture(&inputs);
    let before = Regs {
        edi: field.as_mut_ptr() as u32,
        ..Regs::SENTINEL
    };
    for (n, text) in [
        &b"old name\0"[..],
        &b"\0"[..],
        &b"a much longer name, 30 chars..\0"[..],
    ]
    .into_iter()
    .enumerate()
    {
        let field_ptr = field.as_mut_ptr();
        check_state(
            &mut t,
            &format!("text {n}"),
            call_original,
            before,
            &|| unsafe {
                field_ptr.write_bytes(0xAA, 0x28);
                std::ptr::copy_nonoverlapping(text.as_ptr(), field_ptr, text.len());
                backup().write_bytes(0xBB, TEXT_LEN);
                std::ptr::copy_nonoverlapping(b"backup".as_ptr(), backup(), 6);
                set_length(3);
            },
            &regions,
            &rust,
        );
    }
    restore(&inputs, &saved);
    t.finish()
}
