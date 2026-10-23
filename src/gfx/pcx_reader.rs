//! What the hooks of the three PCX readers share: their `dispatch` and their self-test case.
//! A reader takes the path in eax and returns with the carry flag set if it failed; it
//! reads a file and writes the picture buffer and the variables of `gfx/pcx.rs`, so in
//! shadow mode it can run twice: the Rust version first, then the original, and what each
//! left is compared.

use crate::errors::BadInput;
use crate::framework::Regs;

/// A reader's Rust version: the path in, whether it worked out.
pub type Reader = unsafe fn(*const u8) -> Result<bool, BadInput>;

/// `dispatch_reader` without its hook (a refusal is the caller's).
pub fn reader_entry(r: &mut Regs, rust: Reader) -> Result<(), crate::errors::Refused> {
    let worked = unsafe { rust(r.eax as *const u8) }?;
    r.set_carry(!worked);
    r.set_zero(worked);
    Ok(())
}
