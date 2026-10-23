//! Comparing results that hold floats (2026-10-20, the conversion to plain floats): the
//! original computes on the x87 at the precision its control word sets, the converted
//! Rust in `f64` where the x87 keeps a value on its stack and `f32` where it stores one.
//! The game runs at double precision most of the time (0x027F) and at extended (0x037F)
//! after its `finit`: at double the two agree bit for bit, at extended the last bit may
//! differ, and an integer made from a float may land one off at a rounding edge. A place
//! declared here that differs within its tolerance is a drift, logged as "DRIFT" (ten
//! lines a name) and counted in `stats`, not a mismatch; anything else that differs is
//! still a mismatch. Rust callers in modules not yet converted call the exact x87
//! versions (the converted ones are `*_plain`), so their own exact comparisons hold.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::service::harness::Rust;

/// `dispatch_state_floats` outside shadow mode, without its hook (a refusal is the
/// caller's).
pub fn floats_entry(r: &mut Regs, rust: Rust) -> Result<(), Refused> {
    rust(r)?.apply(r);
    Ok(())
}
