//! `scene::widen_object_radius` (0x426A7F): the stage objects' radius (`dword_5CCE14`)
//! raised to a point's distance from the middle when that is larger.
//!
//! In:  eax and ecx the point (whole numbers)
//! Out: eax the distance (`fistp` of the root of eax^2 + ecx^2, through `dword_62400C`);
//!      everything else preserved
//! The radius takes it when distance - radius is not negative (`js`). One caller,
//! 0x42687C (`scene::load_stage_objects`'s point loop).
//! Shadow mode: both, compared.
//! Self-test: the original against the Rust version on points of the test's (the two
//! variables put back).
//!
//! Converted 2026-10-21: the Rust version is `widen_object_radius_plain`, through
//! `rc2k_core::scene::distance::{radius_distance, widened}` (an `f64` root; the x87
//! version, with no Rust callers, went). Shadow mode and the self-test hold the whole
//! numbers to one either way at extended precision, exact at double.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{EAX, Regs, game_read, game_write, rva};
use crate::screens::harness::Left;
use crate::service::floats::floats_entry;

/// dword_62400C the scratch, dword_5CCE14 the radius.
const SCRATCH_RVA: usize = rva(0x62400C);
const RADIUS_RVA: usize = rva(0x5CCE14);

// ---- Rust version ----

// Converted: `rc2k_core::scene::distance::{radius_distance, widened}`
pub fn widen_object_radius_plain(x: u32, z: u32) -> u32 {
    use rc2k_core::scene::distance::{radius_distance, widened};
    let d = radius_distance(x as i32, z as i32) as u32;
    unsafe {
        game_write(SCRATCH_RVA, d);
        if let Some(radius) = widened(d, game_read(RADIUS_RVA)) {
            game_write(RADIUS_RVA, radius);
        }
    }
    d
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(Left::nothing().with(EAX, widen_object_radius_plain(r.eax, r.ecx)))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    floats_entry(r, &rust)
}
