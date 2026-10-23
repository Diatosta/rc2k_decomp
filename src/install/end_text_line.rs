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
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{EBX, Regs};
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

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
/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
