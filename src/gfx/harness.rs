//! `service::harness::dispatch_state` for functions that write whole pictures: the same
//! steps, but a mismatch line names the first byte that differs and does not print the
//! regions, which run to megabytes.

use crate::framework::Regs;
use crate::service::harness::Rust;

/// `dispatch_pixels` outside shadow mode, without its hook (a refusal is the caller's).
pub fn pixels_entry(r: &mut Regs, rust: Rust) -> Result<(), crate::errors::Refused> {
    rust(r)?.apply(r);
    Ok(())
}
