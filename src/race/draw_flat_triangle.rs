//! `race::draw_flat_triangle` (0x455912): three vertices at (1, 1, 0), texture and colour 0,
//! drawn as a fan (`scene::draw_fan`); see `race/frame_views.rs`.
//!
//! In:  nothing
//! Out: every register preserved (`pusha` .. `popa`)
//!
//! Caller: `sub_4559A6`.
//! Hook removed 2026-10-22 (direct calls): Rust callers run its `entry` (`calls::call`,
//! `routes.rs`); the registers above are the original's, for reference.

use crate::framework::Regs;
use crate::race::frame_views::draw_flat_triangle;

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    unsafe {
        draw_flat_triangle(r, &mut |_, r| {
            crate::scene::draw_fan::draw_fan(r, &mut crate::scene::draw_fan::game_part)
        })
    };
    Ok(())
}
