//! `install::end_text_line` (0x415304; IDA had it as code outside any function, made one
//! 2026-10-19): a line feed and a NUL at ebx (`strings::append_lf`, `append_nul`), ebx past
//! them. In five of the installer's tables (0x51D7B4 ..); the installer's Rust driver does it
//! inline (`installer.rs`), this hook is for the game's own calls.
//!
//! In:  ebx = the cursor
//! Out: ebx + 2, the rest kept
//!
//! Its first instruction is a call: a private copy of the whole 0xB bytes, both calls
//! relocated. Compared in shadow mode by state. Self-test: the original against the Rust
//! version at two places in a buffer of the test's.

use crate::errors::Refused;
use crate::framework::{EBX, RawHook, Regs, SelfTest, hook_asm, rva};
use crate::screens::harness::Left;
use crate::service::harness::{check_state, dispatch_state, state_entry};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);
pub static HOOK: RawHook = RawHook::new("install::end_text_line", rva(0x415304), thunk, &ORIGINAL)
    .private_copy(0xB)
    .relocate_calls(&[0, 5])
    .manual_patch()
    .self_test(self_test);
hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);
/// The Rust version: the cursor after the two bytes.
pub unsafe fn end_text_line(cursor: u32) -> u32 {
    unsafe {
        (cursor as *mut u8).write(b'\n');
        ((cursor + 1) as *mut u8).write(0);
    }
    cursor + 2
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(Left::nothing().with(EBX, unsafe { end_text_line(r.ebx) }))
}
extern "C" fn dispatch(r: &mut Regs) {
    let regions = [(r.ebx, 2)];
    dispatch_state(&HOOK, r, call_original, &regions, &rust);
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let mut buffer = [0xAAu8; 8]; // lives to the end of the test
    let b = buffer.as_mut_ptr() as u32;
    for at in [0u32, 3] {
        check_state(
            &mut t,
            &format!("at {at}"),
            call_original,
            Regs {
                ebx: b + at,
                ..Regs::SENTINEL
            },
            &|| unsafe { std::ptr::write_bytes(b as *mut u8, 0xAA, 8) },
            &[(b, 8)],
            &rust,
        );
    }
    t.finish()
}
