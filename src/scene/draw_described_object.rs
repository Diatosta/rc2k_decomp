//! `scene::draw_described_object`: an object drawn from its description (a course face record of
//! kind 0xFF, part R1 of stream A; also `scene::draw_scene_object`'s modes 1 and 2).
//!
//! Nothing for an object above 0x64 or a point past the scene's point count, nor in a race for
//! object 0x3F. Objects 0x38 and 0x3A ..= 0x3F are signs: upright (0x2000), full light, 0x3D / 0x3E
//! half bright when any of the course face record's flag bytes that mark them is set. Then the view
//! kind 2, `race::place_object_matrix`, the light kind 2 (1 when the object's material record has
//! its light-kind byte), `scene::draw_scene_object` (given the material record, the object and that
//! byte); after it (and in the cases of nothing) `only_object` 0 and the object place -1. Refuses
//! (before anything) what `draw_scene_object` refuses for the object (a race car without its
//! block).

use crate::errors::{BadInput, Refused};

/// What `scene::draw_scene_object` would refuse for `object`: in a race (`in_race`) a car
/// (0x50 ..) without its block.
fn check_object(
    numbers: &crate::race::numbers_store::NumberVars,
    work: &crate::race::work_store::RaceWork,
    object: u32,
) -> Result<(), Refused> {
    let car = object.wrapping_sub(0x50);
    if numbers.in_race.is_set() && car < 0xA0 && !work.has_car_block((car / 0x14) as usize) {
        return Err(BadInput("a car without its block").into());
    }
    Ok(())
}

/// Where an object's description is read (passed in): a place in game memory, or a view's sort
/// list from a flare's entry on (`race::view_scratch`).
#[derive(Clone, Copy)]
pub enum Description<'a> {
    /// The material record of an object kind (and what follows it in the tables).
    Material(u32),
    /// The view's own cells (the object matrix slots, or a sort list from a flare's entry on).
    PlainCells(&'a [crate::memory::session::PlainDword]),
    /// A matrix's floats, read as the bytes they are.
    Floats(&'a [crate::memory::session::PlainFloat]),
}

/// The bytes of a description that the drawing reads: nothing lies past them.
const HEAD_LEN: usize = 0x3C;

impl Description<'_> {
    /// The first `len` bytes of the words (the rest zero), or the refusal `past` when they are
    /// fewer.
    fn from_words(
        words: impl Iterator<Item = u32>,
        len: usize,
        past: &'static str,
    ) -> Result<[u8; HEAD_LEN], BadInput> {
        let mut bytes = [0u8; HEAD_LEN];
        let (slots, _) = bytes.as_chunks_mut::<4>();
        let mut taken = 0;
        for (slot, word) in slots[..len.div_ceil(4)].iter_mut().zip(words) {
            *slot = word.to_le_bytes();
            taken += 4;
        }
        if taken < len {
            return Err(BadInput(past));
        }
        bytes[len..].fill(0);
        Ok(bytes)
    }

    /// The description's first `len` bytes (at most `HEAD_LEN`; a material's from the polygon
    /// tables `tables`), the rest of the array zero.
    pub fn read_bytes(
        &self,
        tables: &crate::scene::polygon_store::PolygonTables,
        len: usize,
    ) -> Result<[u8; HEAD_LEN], BadInput> {
        match self {
            Description::Material(kind) => {
                let mut bytes = [0u8; HEAD_LEN];
                crate::scene::polygon::material_run(tables, *kind, 0, &mut bytes[..len])?;
                Ok(bytes)
            }
            Description::PlainCells(cells) => Self::from_words(
                cells.iter().map(crate::memory::session::PlainDword::get),
                len,
                "a described object past its cells",
            ),
            Description::Floats(cells) => Self::from_words(
                cells.iter().map(crate::memory::session::PlainFloat::bits),
                len,
                "a described object past its matrix",
            ),
        }
    }
}

/// Draws the object of the description at `d`. Its calls: `race::place_object_matrix` and
/// `scene::draw_scene_object`.
pub fn draw_described_object(
    ctx: &crate::race::view_scratch::DrawCtx<'_>,
    d: Option<Description<'_>>,
) -> Result<(), Refused> {
    let d = d.ok_or(BadInput("a described object without its description"))?;
    draw_description(ctx, &|len| {
        d.read_bytes(&ctx.reads.buffers.polygon_tables, len)
    })
}

/// The same for course face `face` (kind 0xFF), its own description.
pub fn draw_described_face(
    ctx: &crate::race::view_scratch::DrawCtx<'_>,
    face: u32,
) -> Result<(), Refused> {
    draw_description(ctx, &|len| {
        let bytes: [u8; 0x40] =
            crate::race::course_places::face_field(ctx.reads.course(), face, 0)?;
        let mut head = [0u8; HEAD_LEN];
        head[..len].copy_from_slice(&bytes[..len]);
        Ok(head)
    })
}

/// The drawing from a description's first `len` bytes as `read` gives them.
fn draw_description(
    ctx: &crate::race::view_scratch::DrawCtx<'_>,
    read: &dyn Fn(usize) -> Result<[u8; HEAD_LEN], BadInput>,
) -> Result<(), Refused> {
    let head = read(8)?;
    let object = head[1] as u32;
    let place = u32::from_le_bytes([head[4], head[5], head[6], head[7]]);
    let shown = object <= 0x64
        && place < ctx.reads.course_work.get().scene_point_count
        && !(ctx.reads.switches.scene_set_up.get() != 0 && object == 0x3F);
    if shown {
        check_object(ctx.reads.number_vars, ctx.reads.work, object)?;
    }
    ctx.view.cars.object_number.set(object);
    let end = || {
        ctx.view.object.borrow_mut().only_object = 0u32;
        ctx.view.object.borrow_mut().object_place = u32::MAX;
    };
    if object > 0x64 {
        end();
        return Ok(());
    }
    if place >= ctx.reads.course_work.get().scene_point_count {
        end();
        return Ok(());
    }
    ctx.view.object.borrow_mut().object_place = place;
    if !shown {
        end();
        return Ok(());
    }
    // Everything the description gives from here on lies in its first 0x3C bytes.
    let desc = read(HEAD_LEN)?;
    let byte = |o: u32| desc[o as usize] as u32;
    let get = |o: usize| u32::from_le_bytes(desc[o..o + 4].try_into().unwrap());
    let at: [u32; 3] =
        crate::race::scene_points::point_number(ctx.reads.course(), &ctx.part.points, place)?;
    ctx.view.object.borrow_mut().place[0] = at[0].wrapping_add(get(0x30));
    ctx.view.object.borrow_mut().place[1] = at[1].wrapping_add(get(0x34));
    ctx.view.object.borrow_mut().place[2] = at[2].wrapping_add(get(0x38));
    ctx.view.object.borrow_mut().angles[0] = (byte(8) << 8) & 0xFFFF;
    ctx.view.object.borrow_mut().angles[1] = byte(0xC) << 8;
    ctx.view.object.borrow_mut().angles[2] = byte(0x10) << 8;
    ctx.view.object.borrow_mut().described_turns[1] = 0u32;
    ctx.view.object.borrow_mut().described_turns[0] = 0u32;
    ctx.view.object.borrow_mut().described_turns[2] = 0u32;
    if byte(0x11) != 0 {
        ctx.view.object.borrow_mut().described_turns[2] = byte(9) << 8;
        ctx.view.object.borrow_mut().described_turns[0] = byte(0x12) << 8;
        ctx.view.object.borrow_mut().described_turns[1] = byte(0x13) << 8;
    }
    ctx.view.light.borrow_mut().other_light = get(0x2C);
    ctx.view.object.borrow_mut().only_object = (byte(0x25) >> 1) & 3;
    let light = |lowest: u32, highest: u32, level: u32, strength: u32| {
        ctx.view.light.borrow_mut().light_lowest = lowest;
        ctx.view.light.borrow_mut().light_highest = highest;
        ctx.view.light.borrow_mut().light_level = level;
        ctx.view.object.borrow_mut().light_strength = strength;
    };
    if object == 0x38 || (0x3A..0x40).contains(&object) {
        ctx.view.object.borrow_mut().angles[0] = 0x2000u32;
        ctx.view.object.borrow_mut().angles[1] = 0u32;
        ctx.view.object.borrow_mut().angles[2] = 0u32;
        ctx.view.object.borrow_mut().size = byte(0x18);
        light(0xFF, 0xFF, 0xFF, 0);
        ctx.view.light.borrow_mut().light_turns[0] = 0u32;
        ctx.view.light.borrow_mut().light_turns[1] = 0u32;
        if (object == 0x3E || object == 0x3D)
            && [0x28u32, 0x29, 0x2A, 0x2B, 0x19, 0x1A, 0x1B, 0x20]
                .iter()
                .any(|&o| byte(o) != 0)
        {
            ctx.view.light.borrow_mut().light_highest = 0x80u32;
        }
    } else {
        ctx.view.object.borrow_mut().size = byte(0x18);
        light(byte(0x14), byte(0x15), byte(0x16), byte(0x17));
        ctx.view.light.borrow_mut().light_turns[0] = byte(0x22);
        ctx.view.light.borrow_mut().light_turns[1] = byte(0x23);
        if byte(0x24) != 0 {
            ctx.view.light.borrow_mut().light_turns[0] =
                ctx.reads.stage_prefs.sun_turn.get().wrapping_neg();
            ctx.view.light.borrow_mut().light_turns[1] = ctx.reads.stage_prefs.sun_height.get();
            ctx.view.light.borrow_mut().light_level = ctx.reads.stage_prefs.light_level.get();
            ctx.view.object.borrow_mut().light_strength =
                ctx.reads.stage_prefs.light_strength_kept.get();
            ctx.view.light.borrow_mut().light_lowest = 0u32;
            ctx.view.light.borrow_mut().light_highest = 0xFFu32;
        }
    }
    ctx.view.light.borrow_mut().view_kind = 2u32;
    crate::race::frame_draw::place_object_matrix(ctx)?;
    ctx.view.light.borrow_mut().light_kind = 2u32;
    let object: u32 = ctx.view.cars.object_number.get();
    let kind: u32 = ctx.reads.object_tables.object_kind(object);
    if crate::scene::polygon::material_run_byte(&ctx.reads.buffers.polygon_tables, kind, 0x1F)? != 0
    {
        ctx.view.light.borrow_mut().light_kind = 1u32;
    }
    crate::scene::draw_scene_object::draw_scene_object_in_game(
        ctx,
        Some(Description::Material(kind)),
    )?;
    end();
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::memory::session::PlainDword;

    #[test]
    fn a_description_is_read_to_its_length_and_refused_past_its_cells() {
        let cells: Vec<PlainDword> = (1..=3).map(PlainDword::new).collect();
        let description = Description::PlainCells(&cells);
        let tables: crate::scene::polygon_store::PolygonTables =
            crate::memory::cells::zeroed_cells();
        let head = description.read_bytes(&tables, 8).unwrap();
        assert_eq!(head[..8], [1, 0, 0, 0, 2, 0, 0, 0]);
        assert!(head[8..].iter().all(|&b| b == 0));
        assert!(description.read_bytes(&tables, 16).is_err());
        assert!(description.read_bytes(&tables, 12).is_ok());
    }
}
