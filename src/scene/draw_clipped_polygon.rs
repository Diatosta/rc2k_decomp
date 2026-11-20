//! `scene::draw_clipped_polygon`: a polygon's whole way from its points to the screen.
//!
//! The polygon's entries are the work table's first, in order. Three clip cells and the drawn flag
//! = 0; the pool of records for new vertices = the pool's start; the list = 0, 1, 2 .. and a 0
//! after it; the next free entry = the number of vertices. Then `scene::project_polygon`; nothing
//! more if every point is outside the depths; `scene::scale_texture_coordinates`;
//! `scene::fog_polygon`; if some point is outside, `scene::clip_polygon_by_depth`, and nothing more
//! with fewer than three vertices left; `scene::clip_polygon_to_screen`, and the same;
//! `scene::draw_polygon`; the drawn flag = 1 (the polygon was drawn). Refuses a polygon of no
//! vertices or of more than 256, and what its parts refuse. A refusal after the texture coordinates
//! were scaled leaves a polygon that would be scaled once more.

use crate::errors::{BadInput, Refused};
use crate::scene::clip_polygon_by_depth::clip_polygon_by_depth;
use crate::scene::clip_polygon_to_screen::clip_polygon_to_screen;
use crate::scene::draw_polygon::{Game, draw_polygon};
use crate::scene::fog_polygon::{fog_entries, fog_polygon};
use crate::scene::polygon::MOST;
use crate::scene::project_polygon::project_entries;
use crate::scene::scale_texture_coordinates::{scale_entries, scale_texture_coordinates};

// ---- game data ----

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
    polygon_work: &crate::scene::polygon_store::PolygonWork,
    poly: &crate::scene::polygon_work_store::PolygonWork,
    step: &mut dyn FnMut(Step) -> Result<(), Refused>,
) -> Result<(), Refused> {
    {
        let count: u32 = poly.polygon_vertex_count.get();
        if count == 0 || count > MOST {
            return Err(BadInput("a polygon of no vertices, or of more than 256").into());
        }
        poly.screen_clipped.set(0u32);
        poly.depth_clipped.set(0u32);
        poly.polygon_drawn.set(0u32);
        poly.polygon_next_record
            .set(crate::scene::polygon::pool_start());
        for n in 0..count {
            crate::scene::polygon::list_cell(polygon_work, n as usize).set(n);
        }
        crate::scene::polygon::list_cell(polygon_work, count as usize).set(0u32);
        poly.polygon_next_entry.set(count);
        step(Step::Project)?;
        let outside: u32 = poly.polygon_outside_count.get();
        if outside == poly.polygon_vertex_count.get() {
            return Ok(());
        }
        step(Step::ScaleTextureCoordinates)?;
        step(Step::Fog)?;
        if poly.polygon_outside_count.get() != 0 {
            step(Step::ClipByDepth)?;
            if poly.polygon_vertex_count.get() < 3 {
                return Ok(());
            }
        }
        step(Step::ClipToScreen)?;
        if poly.polygon_vertex_count.get() < 3 {
            return Ok(());
        }
        step(Step::Draw)?;
        poly.polygon_drawn.set(1u32);
        Ok(())
    }
}

/// The function with its parts, drawing for real.
pub fn draw_clipped_polygon_in_game(
    ctx: &crate::race::view_scratch::DrawCtx<'_>,
) -> Result<(), Refused> {
    // The list stays as it is from the projection to the fog: listed once for the three.
    let mut entries = None;
    let poly = &ctx.view.polygon;
    draw_clipped_polygon(&ctx.part.polygon, poly, &mut |step| {
        match step {
            Step::Project => {
                let listed =
                    entries.insert(crate::scene::polygon::listed(&ctx.part.polygon, poly)?);
                project_entries(ctx.part, poly, listed, ctx.reads, ctx.view)?;
            }
            Step::ScaleTextureCoordinates => match &entries {
                Some(listed) if !listed.is_empty() => scale_entries(poly, listed)?,
                _ => scale_texture_coordinates(&ctx.part.polygon, poly)?,
            },
            Step::Fog => match entries.take() {
                Some(listed) => fog_entries(ctx.part, &listed, ctx.view)?,
                None => fog_polygon(ctx.part, ctx.view)?,
            },
            Step::ClipByDepth => {
                clip_polygon_by_depth(
                    ctx.part,
                    poly,
                    ctx.reads.course(),
                    ctx.reads.draw_switches,
                    ctx.far_clip,
                    ctx.view,
                )?;
            }
            Step::ClipToScreen => clip_polygon_to_screen(ctx.part, poly, ctx.reads.course())?,
            Step::Draw => {
                draw_polygon(
                    poly,
                    ctx.part,
                    &ctx.view.face,
                    &mut Game(ctx.part, ctx.view),
                    ctx.reads,
                    ctx.reads.screen,
                )?;
            }
        }
        Ok(())
    })
}
