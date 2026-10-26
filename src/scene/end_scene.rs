//! `scene::end_scene` (0x44E952): the 3D part of a frame ends.
//!
//! In:  nothing
//! Out: as `d3d::end_scene` (0x41AB8A: Direct3D's EndScene, platform) leaves them
//!
//! A call of `d3d::end_scene` and nothing more. Nine callers.
//! Hook removed 2026-10-22 (direct calls): Rust callers run its `entry` (`calls::call`,
//! `routes.rs`); the registers above are the original's, for reference.

use crate::framework::{Regs, rva};

const LAYER_RVA: usize = rva(0x41AB8A); // d3d::end_scene
// ---- Rust version ----

/// `end` stands for `d3d::end_scene`.
pub fn end_scene(r: &mut Regs, end: &mut dyn FnMut(&mut Regs)) {
    end(r);
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    end_scene(r, &mut |r| crate::calls::call(LAYER_RVA, r));
    Ok(())
}
