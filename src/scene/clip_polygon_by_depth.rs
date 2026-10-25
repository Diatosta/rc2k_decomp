//! `scene::clip_polygon_by_depth` (0x44DB4F): cuts the polygon to the depths that can
//! be drawn.
//!
//! In:  nothing
//! Out: the original leaves every register changed (ebx the count left, as its passes end);
//!      its callers go on by the number of vertices alone; the Rust version gives ebx the
//!      count and keeps the rest
//!
//! `dword_624124` = 1; `dword_62411C` = 0x1000 and one pass of `scene/clip.rs` that
//! keeps what is deeper than that (`sub_44DB93`); then, with `dword_621510` at 0,
//! `dword_621900` not 0 and three vertices or more left, `dword_62411C` =
//! `dword_624230` and a pass that keeps what is not deeper (`sub_44DEAA`).
//! Seven callers. Refuses as `scene::clip_polygon_to_screen`.

use crate::errors::BadInput;
use crate::framework::{game_read, rva};
use crate::scene::clip::{Edge, clip_pass, listed_with_records, or_undone};
use crate::scene::draw_store::DRAW;

// ---- game data ----

const NEAREST: i32 = 0x1000;
const NO_FAR_RVA: usize = rva(0x621510); // dword_621510
const FAR_ON_RVA: usize = rva(0x621900); // dword_621900

// ---- Rust version ----

pub unsafe fn clip_polygon_by_depth() -> Result<(), BadInput> {
    unsafe {
        listed_with_records()?;
        or_undone(|| {
            DRAW.depth_clipped.set_bits(1u32);
            DRAW.clip_plane.set_bits(NEAREST as u32);
            clip_pass(Edge::Depth(NEAREST), true, 0)?;
            let far = game_read::<u32>(NO_FAR_RVA) == 0 && game_read::<u32>(FAR_ON_RVA) != 0;
            if far && (DRAW.polygon_vertex_count.bits() as i32).wrapping_sub(3) >= 0 {
                let farthest = DRAW.clip_farthest.bits() as i32;
                DRAW.clip_plane.set_bits(farthest as u32);
                clip_pass(Edge::Depth(farthest), false, 0)?;
            }
            Ok(())
        })
    }
}
