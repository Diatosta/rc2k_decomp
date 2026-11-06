//! `scene::draw_clipped_polygon` (0x44CCEF): a polygon's whole way from its points to
//! the screen.
//!
//! In:  nothing; the polygon's entries are the work table's first, in order
//! Out: the original leaves every register changed; the Rust version keeps them
//!
//! `dword_624120`, `dword_624124`, `dword_624134` = 0; the pool of records for new vertices
//! (`dword_6240A4`) = `unk_977F10`; the list = 0, 1, 2 .. and a 0 after it; the next free entry
//! (`dword_624114`) = the number of vertices. Then `scene::project_polygon`; nothing more if
//! every point is outside the depths; `scene::scale_texture_coordinates`; `scene::fog_polygon`;
//! if some point is outside, `scene::clip_polygon_by_depth`, and nothing more with fewer than
//! three vertices left; `scene::clip_polygon_to_screen`, and the same;
//! `scene::draw_marked_polygon`; `dword_624134` = 1 (the polygon was drawn). Four callers
//! (`sub_451F69` and three more: the objects' polygons). Refuses a polygon of no vertices (the
//! original goes through 2^32) or of more than 256, and what its parts refuse. A refusal after
//! the texture coordinates were scaled hands the original a polygon it scales once more.

use crate::errors::{BadInput, Refused};
use crate::race::owner::race;
use crate::scene::clip_polygon_by_depth::clip_polygon_by_depth;
use crate::scene::clip_polygon_to_screen::clip_polygon_to_screen;
use crate::scene::draw_marked_polygon::draw_marked_polygon;
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
    step: &mut dyn FnMut(Step) -> Result<(), Refused>,
) -> Result<(), Refused> {
    {
        let count: u32 = crate::scene::owner::scene()
            .drawing
            .scene
            .polygon_vertex_count
            .bits();
        if count == 0 || count > MOST {
            return Err(BadInput("a polygon of no vertices, or of more than 256").into());
        }
        crate::scene::owner::scene()
            .drawing
            .scene
            .screen_clipped
            .set_bits(0u32);
        crate::scene::owner::scene()
            .drawing
            .scene
            .depth_clipped
            .set_bits(0u32);
        crate::scene::owner::scene()
            .drawing
            .scene
            .polygon_drawn
            .set_bits(0u32);
        crate::scene::owner::scene()
            .drawing
            .scene
            .polygon_next_record
            .set_bits(crate::scene::polygon::pool_start());
        for n in 0..count {
            crate::scene::polygon::list_cell(n as usize).set(n);
        }
        crate::scene::polygon::list_cell(count as usize).set(0u32);
        crate::scene::owner::scene()
            .drawing
            .scene
            .polygon_next_entry
            .set_bits(count);
        step(Step::Project)?;
        let outside: u32 = crate::scene::owner::scene()
            .drawing
            .scene
            .polygon_outside_count
            .bits();
        if outside
            == crate::scene::owner::scene()
                .drawing
                .scene
                .polygon_vertex_count
                .bits()
        {
            return Ok(());
        }
        step(Step::ScaleTextureCoordinates)?;
        step(Step::Fog)?;
        if crate::scene::owner::scene()
            .drawing
            .scene
            .polygon_outside_count
            .bits()
            != 0
        {
            step(Step::ClipByDepth)?;
            if crate::scene::owner::scene()
                .drawing
                .scene
                .polygon_vertex_count
                .bits()
                < 3
            {
                return Ok(());
            }
        }
        step(Step::ClipToScreen)?;
        if crate::scene::owner::scene()
            .drawing
            .scene
            .polygon_vertex_count
            .bits()
            < 3
        {
            return Ok(());
        }
        step(Step::Draw)?;
        crate::scene::owner::scene()
            .drawing
            .scene
            .polygon_drawn
            .set_bits(1u32);
        Ok(())
    }
}

/// The function with its parts' Rust versions, drawing for real.
pub fn draw_clipped_polygon_in_game() -> Result<(), Refused> {
    let race = race();
    let course = &race.course;
    // The list stays as it is from the projection to the fog: listed once for the three.
    let mut entries = None;
    draw_clipped_polygon(&mut |step| {
        match step {
            Step::Project => {
                let listed = entries.insert(crate::scene::polygon::listed()?);
                project_entries(course, listed)?;
            }
            Step::ScaleTextureCoordinates => match &entries {
                Some(listed) if !listed.is_empty() => scale_entries(listed)?,
                _ => scale_texture_coordinates()?,
            },
            Step::Fog => match entries.take() {
                Some(listed) => fog_entries(&listed)?,
                None => fog_polygon()?,
            },
            Step::ClipByDepth => clip_polygon_by_depth()?,
            Step::ClipToScreen => clip_polygon_to_screen()?,
            Step::Draw => draw_marked_polygon(&mut || draw_polygon(race, &mut Game))?,
        }
        Ok(())
    })
}
