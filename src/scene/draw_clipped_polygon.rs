//! `scene::draw_clipped_polygon` (0x44CCEF): a polygon's whole way from its points to
//! the screen.
//!
//! In:  nothing; the polygon's entries are the work table's first, in order
//! Out: the original leaves every register changed; the Rust version keeps them
//!
//! `dword_624120`, `dword_624124`, `dword_624134` = 0; the pool of records for new
//! vertices (`dword_6240A4`) = `unk_977F10`; the list = 0, 1, 2 .. and a 0 after it;
//! the next free entry (`dword_624114`) = the number of vertices. Then
//! `scene::project_polygon`; nothing more if every point is outside the depths;
//! `scene::scale_texture_coordinates`; `scene::fog_polygon`; if some point is outside,
//! `scene::clip_polygon_by_depth`, and nothing more with fewer than three vertices
//! left; `scene::clip_polygon_to_screen`, and the same; `scene::draw_marked_polygon`;
//! `dword_624134` = 1 (the polygon was drawn).
//! Four callers (`sub_451F69` and three more: the objects' polygons).
//! Refuses a polygon of no vertices (the original goes through 2^32) or of more than
//! 256, and what its parts refuse. A refusal after the texture coordinates were
//! scaled hands the original a polygon it scales once more.
//! Shadow mode runs the original alone (its parts are compared in their own hooks).
//! Self-test: a redirected copy whose six calls follow a script.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::d3d::draw_fan_unclipped::COUNT_RVA;
use crate::errors::{BadInput, Refused};
use crate::framework::{Regs, game_addr, game_read, game_write, rva};
use crate::scene::clip::{
    DEPTH_CLIPPED_RVA, NEXT_ENTRY_RVA, NEXT_RECORD_RVA, POOL_RVA, SCREEN_CLIPPED_RVA,
};
use crate::scene::clip_polygon_by_depth::clip_polygon_by_depth;
use crate::scene::clip_polygon_to_screen::clip_polygon_to_screen;
use crate::scene::draw_marked_polygon::draw_marked_polygon;
use crate::scene::draw_polygon::{Game, draw_polygon};
use crate::scene::fog_polygon::fog_polygon;
use crate::scene::polygon::{LIST_RVA, MOST};
use crate::scene::project_polygon::{OUTSIDE_RVA, project_polygon};
use crate::scene::scale_texture_coordinates::scale_texture_coordinates;
use crate::scene::submit_polygon;

// ---- game data ----

const DRAWN_RVA: usize = rva(0x624134); // dword_624134

// ---- Rust version ----

/// The function's six calls, in the order they can come.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Step {
    Project,
    ScaleTextureCoordinates,
    Fog,
    ClipByDepth,
    ClipToScreen,
    Draw,
}

pub unsafe fn draw_clipped_polygon(
    step: &mut dyn FnMut(Step) -> Result<(), Refused>,
) -> Result<(), Refused> {
    unsafe {
        let count: u32 = game_read(COUNT_RVA);
        if count == 0 || count > MOST {
            return Err(BadInput("a polygon of no vertices, or of more than 256").into());
        }
        game_write(SCREEN_CLIPPED_RVA, 0u32);
        game_write(DEPTH_CLIPPED_RVA, 0u32);
        game_write(DRAWN_RVA, 0u32);
        game_write(NEXT_RECORD_RVA, game_addr(POOL_RVA) as u32);
        for n in 0..count {
            game_write(LIST_RVA + n as usize * 4, n);
        }
        game_write(LIST_RVA + count as usize * 4, 0u32);
        game_write(NEXT_ENTRY_RVA, count);
        step(Step::Project)?;
        let outside: u32 = game_read(OUTSIDE_RVA);
        if outside == game_read::<u32>(COUNT_RVA) {
            return Ok(());
        }
        step(Step::ScaleTextureCoordinates)?;
        step(Step::Fog)?;
        if game_read::<u32>(OUTSIDE_RVA) != 0 {
            step(Step::ClipByDepth)?;
            if game_read::<u32>(COUNT_RVA) < 3 {
                return Ok(());
            }
        }
        step(Step::ClipToScreen)?;
        if game_read::<u32>(COUNT_RVA) < 3 {
            return Ok(());
        }
        step(Step::Draw)?;
        game_write(DRAWN_RVA, 1u32);
        Ok(())
    }
}

/// The function with its parts' Rust versions, drawing for real. `entry`: the registers
/// the Direct3D layer is called with.
pub unsafe fn draw_clipped_polygon_in_game(entry: &Regs) -> Result<(), Refused> {
    unsafe {
        draw_clipped_polygon(&mut |step| {
            match step {
                Step::Project => project_polygon().map(|_| ())?,
                Step::ScaleTextureCoordinates => scale_texture_coordinates().map(|_| ())?,
                Step::Fog => fog_polygon().map(|_| ())?,
                Step::ClipByDepth => clip_polygon_by_depth()?,
                Step::ClipToScreen => clip_polygon_to_screen()?,
                Step::Draw => draw_marked_polygon(&mut || {
                    draw_polygon(&mut Game(submit_polygon::Game(entry)))
                })?,
            }
            Ok(())
        })
    }
}
