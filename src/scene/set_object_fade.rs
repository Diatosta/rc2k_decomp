//! `scene::set_object_fade`: how see-through an object is drawn by how far off it is, and
//! whether another, simpler object is drawn in its place.
//!
//! From the record of the object's kind (0x80 bytes in `object_materials`) two words: +0x1D to
//! the first `fade_words` entry (another kind to draw instead) and +0x22 to its second (a
//! distance). The object's `position` x and depth = 0, `object_fading` = 0 (no fading), `fade`
//! = 0x100 (not see-through). Nothing more if both words are 0 or the object's `object_place`
//! (a point of the scene) is -1. Else that point's record (made if it is not there, the same
//! as `scene::project_point` makes one) gives the `position`: its x and depth in the view as
//! whole numbers. With the depth beyond the distance times 65536: both `fade` cells = (the
//! distance times 65536 less the depth, plus 0x100000, not under 0) over 0x1000,
//! `object_fading` = 1, and if the first word is not 0 and an object of that kind exists
//! (`scene::find_object_of_kind`: the first of the objects 1 ..= 0xFF with that kind), the
//! current object becomes that object. One caller, `scene::draw_object`. Refuses an object's
//! number past the tables and, where a record has to be made, no records or no points.

use crate::errors::BadInput;
use crate::math::x87;
use crate::scene::point_records::field;
use crate::scene::project_point::make_scene_record;

// ---- game data ----
const OBJECTS: u32 = 0x100;

fn record_at(lists: &crate::race::course_lists_store::CourseLists, place: u32) -> Option<u32> {
    crate::scene::point_records::held(lists)
        .then(|| crate::scene::point_records::of_scene_point(place))
}

/// The two words of the object's kind, and whether the object's place is looked at;
/// None for an object's number past the tables.
fn words(
    tables: &crate::scene::polygon_store::PolygonTables,
    reads: crate::workers::DrawReads<'_>,
    cars: &crate::race::car_draw_state::CarDrawState,
    place: u32,
) -> Result<Option<(u32, u32, bool)>, BadInput> {
    {
        let object: u32 = cars.object_number.get();
        if object >= OBJECTS {
            return Ok(None);
        }
        let kind: u32 = reads.object_tables.object_kind(object);
        let word =
            |at: u32| crate::scene::polygon::material_run_word(tables, kind, at).map(u32::from);
        let (other_kind, distance) = (word(0x1D)?, word(0x22)?);
        let looked_at = (other_kind | distance) != 0 && place != u32::MAX;
        Ok(Some((other_kind, distance, looked_at)))
    }
}

pub fn set_object_fade(ctx: &crate::race::view_scratch::DrawCtx<'_>) -> Result<(), BadInput> {
    let place: u32 = ctx.view.object.object_place.get();
    let Some((other_kind, distance, needs_record)) = words(
        &ctx.reads.buffers.polygon_tables,
        ctx.reads,
        &ctx.view.cars,
        place,
    )?
    else {
        return Err(BadInput("an object's number past the tables"));
    };
    // What can refuse is asked before anything is written.
    let record = match (needs_record, record_at(ctx.reads.course_lists, place)) {
        (false, _) => 0,
        (true, None) => return Err(BadInput("no records for the points")),
        (true, Some(record)) => {
            if field(ctx.part, record, 0)? != place
                && !crate::race::scene_points::held(&ctx.reads.stage.course)
            {
                return Err(BadInput("no points"));
            }
            record
        }
    };
    crate::session::write_bits(&ctx.view.cars.fade_words, [other_kind, distance]);
    ctx.view.object.position[0].set(0u32);
    ctx.view.object.position[2].set(0u32);
    ctx.view.object.fading.set(0u32);
    ctx.view.object.fade[0].set(0x100u32);
    if !needs_record {
        return Ok(());
    }
    if field(ctx.part, record, 0)? != place {
        make_scene_record(ctx.part, place, record, ctx.reads, ctx.view)?;
    }
    ctx.view.object.position[0].set(x87::to_int(field(ctx.part, record, 0x10)?) as u32);
    let depth = x87::to_int(field(ctx.part, record, 0x18)?);
    ctx.view.object.position[2].set(depth as u32);
    let near = ((distance << 16) as i32).wrapping_sub(depth);
    if near >= 0 {
        return Ok(());
    }
    let fade = (near.wrapping_add(0x10_0000).max(0) as u32) >> 12;
    crate::session::write_bits(&ctx.view.object.fade, [fade, fade]);
    ctx.view.object.fading.set(1u32);
    if other_kind != 0 {
        let kinds: [u32; 0x100] = crate::session::read_bits(&ctx.reads.object_tables.object_kinds);
        if let Some(other) = (1..0x100).find(|&n| kinds[n] == other_kind) {
            ctx.view.cars.object_number.set(other as u32);
        }
    }
    Ok(())
}
