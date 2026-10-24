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

use crate::d3d::draw_fan_unclipped::COUNT_RVA;
use crate::errors::BadInput;
use crate::framework::{game_read, game_write, rva};
use crate::scene::clip::{
    DEPTH_CLIPPED_RVA, Edge, PLANE_RVA, clip_pass, listed_with_records, or_undone,
};

// ---- game data ----

const NEAREST: i32 = 0x1000;
const NO_FAR_RVA: usize = rva(0x621510); // dword_621510
const FAR_ON_RVA: usize = rva(0x621900); // dword_621900
const FARTHEST_RVA: usize = rva(0x624230); // dword_624230

// ---- Rust version ----

pub unsafe fn clip_polygon_by_depth() -> Result<(), BadInput> {
    unsafe {
        listed_with_records()?;
        or_undone(|| {
            game_write(DEPTH_CLIPPED_RVA, 1u32);
            game_write(PLANE_RVA, NEAREST);
            clip_pass(Edge::Depth(NEAREST), true, 0)?;
            let far = game_read::<u32>(NO_FAR_RVA) == 0 && game_read::<u32>(FAR_ON_RVA) != 0;
            if far && game_read::<i32>(COUNT_RVA).wrapping_sub(3) >= 0 {
                let farthest: i32 = game_read(FARTHEST_RVA);
                game_write(PLANE_RVA, farthest);
                clip_pass(Edge::Depth(farthest), false, 0)?;
            }
            Ok(())
        })
    }
}
