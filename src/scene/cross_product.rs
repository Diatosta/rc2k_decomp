//! `scene::cross_product` (0x420F8D): the cross product of two vectors of three floats.
//!
//! In:  ebx, ecx = the two vectors, edx = where the result goes
//! Out: every register and the flags preserved
//!
//! out[0] = c[2] b[1] - b[2] c[1], out[1] = c[0] b[2] - b[0] c[2],
//! out[2] = c[1] b[0] - b[1] c[0] (b = ebx's, c = ecx's), each stored before the next
//! is worked out. (The x87 version: the original's instructions.)
//! Ten callers.
//! Refuses a null pointer.
//! Self-test: vectors by lot, the result apart and in the place of either.
//!
//! Converted 2026-10-21: the hook's Rust version is `cross_product_plain`, through
//! `rc2k_core::scene::matrices::cross` (plain floats; the x87 version stays for the Rust
//! callers in unconverted code). Shadow mode and the self-test compare within
//! `service::floats`' tolerances (the self-test at both test precisions, exact at double).
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::{BadInput, Refused};
use crate::framework::Regs;
use crate::scene::view_matrix;
use crate::screens::harness::Left;
use crate::service::floats::*;

fn rust(r: &Regs) -> Result<Left, Refused> {
    if r.ebx == 0 || r.ecx == 0 || r.edx == 0 {
        return Err(BadInput("a null vector").into());
    }
    unsafe { view_matrix::cross_product_plain(r.ebx as usize, r.ecx as usize, r.edx as usize) };
    Ok(Left::nothing())
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    floats_entry(r, &rust)
}
