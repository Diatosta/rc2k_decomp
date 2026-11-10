//! `scene::draw_clipped_polygon` (0x44CCEF): a polygon's whole way from its points to
//! the screen.
//!
//! In:  nothing; the polygon's entries are the work table's first, in order
//!
//! `dword_624120`, `dword_624124`, `dword_624134` = 0; the pool of records for new vertices
//! (`dword_6240A4`) = `unk_977F10`; the list = 0, 1, 2 .. and a 0 after it; the next free entry
//! (`dword_624114`) = the number of vertices. Then `scene::project_polygon`; nothing more if
//! every point is outside the depths; `scene::scale_texture_coordinates`; `scene::fog_polygon`;
//! if some point is outside, `scene::clip_polygon_by_depth`, and nothing more with fewer than
//! three vertices left; `scene::clip_polygon_to_screen`, and the same;
//! `scene::draw_polygon` (the original through `scene::draw_marked_polygon`, whose mark
//! nothing reads: dropped, `notes/bugs.md`); `dword_624134` = 1 (the polygon was drawn). Four callers
//! (`scene::draw_object_polygon` and three more: the objects' polygons). Refuses a polygon of no
//! vertices (the original goes through 2^32) or of more than 256, and what its parts refuse. A
//! refusal after the texture coordinates were scaled hands the original a polygon it scales once
//! more.

use crate::errors::{BadInput, Refused};
use crate::scene::clip_polygon_by_depth::clip_polygon_by_depth;
use crate::scene::clip_polygon_to_screen::clip_polygon_to_screen;
use crate::scene::draw_polygon::{Game, draw_polygon};
use crate::scene::fog_polygon::{fog_entries, fog_polygon};
use crate::scene::polygon::MOST;
use crate::scene::project_polygon::project_entries;
use crate::scene::scale_texture_coordinates::{scale_entries, scale_texture_coordinates};

// ---- game data ----

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

pub fn draw_clipped_polygon(
    poly: &crate::scene::polygon_work::PolygonWork,
    step: &mut dyn FnMut(Step) -> Result<(), Refused>,
) -> Result<(), Refused> {
    {
        let count: u32 = poly.polygon_vertex_count.bits();
        if count == 0 || count > MOST {
            return Err(BadInput("a polygon of no vertices, or of more than 256").into());
        }
        poly.screen_clipped.set_bits(0u32);
        poly.depth_clipped.set_bits(0u32);
        poly.polygon_drawn.set_bits(0u32);
        poly.polygon_next_record
            .set_bits(crate::scene::polygon::pool_start());
        for n in 0..count {
            crate::scene::polygon::list_cell(n as usize).set(n);
        }
        crate::scene::polygon::list_cell(count as usize).set(0u32);
        poly.polygon_next_entry.set_bits(count);
        step(Step::Project)?;
        let outside: u32 = poly.polygon_outside_count.bits();
        if outside == poly.polygon_vertex_count.bits() {
            return Ok(());
        }
        step(Step::ScaleTextureCoordinates)?;
        step(Step::Fog)?;
        if poly.polygon_outside_count.bits() != 0 {
            step(Step::ClipByDepth)?;
            if poly.polygon_vertex_count.bits() < 3 {
                return Ok(());
            }
        }
        step(Step::ClipToScreen)?;
        if poly.polygon_vertex_count.bits() < 3 {
            return Ok(());
        }
        step(Step::Draw)?;
        poly.polygon_drawn.set_bits(1u32);
        Ok(())
    }
}

/// The function with its parts' Rust versions, drawing for real.
pub fn draw_clipped_polygon_in_game(
    ctx: &crate::race::view_scratch::DrawCtx<'_>,
) -> Result<(), Refused> {
    let (race, scene) = (ctx.race, ctx.scene);
    // The list stays as it is from the projection to the fog: listed once for the three.
    let mut entries = None;
    let poly = &ctx.view.polygon;
    draw_clipped_polygon(poly, &mut |step| {
        match step {
            Step::Project => {
                let listed = entries.insert(crate::scene::polygon::listed(poly)?);
                project_entries(poly, race, listed, scene, ctx.view)?;
            }
            Step::ScaleTextureCoordinates => match &entries {
                Some(listed) if !listed.is_empty() => scale_entries(poly, listed)?,
                _ => scale_texture_coordinates(poly)?,
            },
            Step::Fog => match entries.take() {
                Some(listed) => fog_entries(&listed, ctx.view)?,
                None => fog_polygon(ctx.view)?,
            },
            Step::ClipByDepth => clip_polygon_by_depth(poly, race, scene, ctx.far_clip, ctx.view)?,
            Step::ClipToScreen => clip_polygon_to_screen(poly, race)?,
            Step::Draw => {
                draw_polygon(poly, race, &ctx.view.face, &mut Game, scene)?;
            }
        }
        Ok(())
    })
}
