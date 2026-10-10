//! `menu::backup_text` (0x46D5CC): copies the 0x1F bytes of a text field to the text entry's
//! backup, before the field is edited.
//!
//! In:  edi = the text field
//! Out: every register preserved

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::menu::text_entry::{LENGTH_RVA, TEXT_LEN, backup, backup_region, set_length};
use crate::screens::harness::Left;
use crate::service::harness::{Region, at, capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("menu::backup_text", rva(0x46D5CC), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- Rust version ----

pub unsafe fn backup_text(field: *const u8) {
    unsafe { std::ptr::copy(field, backup(), TEXT_LEN) };
}

// ---- hook ----

fn regions(field: u32) -> [Region; 3] {
    [(field, 0x28), backup_region(), at(LENGTH_RVA)]
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    unsafe { backup_text(r.edi as *const u8) };
    Ok(Left::nothing())
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &regions(r.edi)[1..], &rust);
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
    for (n, text) in [&b"old name\0"[..], &b"\0"[..], &[b'x'; 0x28][..]]
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
