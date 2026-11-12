//! The course view: the depth-sorted entry lists and the grid of course faces seen from the view.

use super::*;
use crate::errors::{BadInput, Refused};
use crate::math::x87_float as bh;
use crate::race::owner::Race;
use crate::race::owner_course::CourseReads;
use crate::race::trap::Trap;
use crate::race::view_scratch::{DrawCtx, SortLists};

/// The most entries in a sort list (`race::view_scratch`).
pub const SORT_MAX: u32 = crate::race::view_scratch::ENTRIES as u32;

/// `race::add_sort_entry` (0x454D57): entry `sort_entry_count` of the view's sort entries for
/// `face`: +4 the face, +0 0, or with the face number (without its top bit) below 0x1000000 the
/// object `kind` << 16 | the face's low 16 bits; the count + 1. Refuses (before anything) an entry
/// past the 0x1000.
pub fn add_sort_entry(lists: &SortLists, kind: u32, face: u32) -> Result<(), Refused> {
    let n: u32 = lists.sort_entry_count.get();
    if n >= SORT_MAX {
        return Err(BadInput("a sort entry past the 0x1000").into());
    }
    // Converted: `rc2k_core::race::frame::sort_entry`.
    let entry = rc2k_core::race::frame::sort_entry(kind, face);
    let at = lists.sort_entry(n)?;
    at[0].set(entry.first.unwrap_or(0));
    at[1].set(face);
    lists.sort_entry_count.set(n.wrapping_add(1));
    Ok(())
}

/// `race::checkpoint_colours` (0x4548E3): the current checkpoint's (`CAR.checkpoint`) face
/// (`g_checkpointFaces`, records of 0x40 at `g_courseFaces`): its +0x10 byte's low and high nibbles
/// (0 read as 0x10) and `CHECKPOINTS.angles[2][checkpoint]` - `CAR.car_heading` (the
/// original stored the three where nothing reads them); the far cells' limit returned (the original's `LEVEL_GRID.far_cell_limit`): the low (twice the
/// distance below 0x40000000 as signed), the high (from 0xC0000000), else the larger. The tables
/// are read unchecked, as the original.
pub fn checkpoint_colours(
    race: &Race,
    course: CourseReads<'_>,
    car: &crate::physics::car_store::ViewCar,
) -> Result<u32, BadInput> {
    // Converted: `rc2k_core::race::frame::checkpoint_colours`.
    {
        let n: u32 = car.checkpoint.get();
        let face = crate::race::checkpoints_store::face(race.checkpoints(), n as usize).get();
        let byte: u8 = face_field(course, face, 0x10)?;
        let along = crate::race::checkpoints_store::angle(race.checkpoints(), n as usize).get();
        let c = rc2k_core::race::frame::checkpoint_colours(byte, along, car.car_heading.get());
        Ok(c.level)
    }
}

/// A scene point's record (0x44CF33, inline in the callers): the record at
/// `g_pointRecords` + (point & 0xFFF) x 0x20, made anew (`scene::make_scene_record`) when
/// its +0 is not the point.
pub fn point_record(
    ctx: &crate::race::view_scratch::DrawCtx<'_>,
    point: u32,
) -> Result<u32, Refused> {
    let record = crate::scene::point_records::of_scene_point(point);
    if crate::scene::point_records::field(ctx.part, record, 0)? != point {
        crate::scene::project_point::make_scene_record(
            ctx.part, point, record, ctx.reads, ctx.view,
        )?;
    }
    Ok(record)
}

/// What `point_record` needs to make a record: the records and the points.
pub fn points_there(ctx: &crate::race::view_scratch::DrawCtx<'_>) -> Result<(), Refused> {
    if !crate::scene::point_records::held(ctx.reads.course_lists)
        || ctx.reads.course_work.scene_points.get() == 0
    {
        return Err(BadInput("no point records or no points").into());
    }
    Ok(())
}

/// `race::face_hidden` (0x454D92): whether the face record is hidden from the shown view:
/// when the view's `race().frame.view_and_records.view_vars.view_hides` entry is 0, `FRAME_SWITCHES.scene_set_up` is set and the face's +2 word
/// has a bit of `race().frame.frame_work.face_hide_mask`; else by its first point's record (+4): `OBJECT_WORK.object_position` = |x|
/// and `OBJECT_WORK.face_depth` = depth + `LEVEL_GRID.depth_shift` as whole numbers, hidden when the depth is below
/// |x| (the sign of the difference). Refuses (before anything) no point records or points.
pub fn face_hidden(
    ctx: &crate::race::view_scratch::DrawCtx<'_>,
    face: &rc2k_core::race::faces::Face,
) -> Result<bool, Refused> {
    let race = ctx.race;
    // Converted: `rc2k_core::race::faces::{hidden_by_switches, hidden_by_depth}`; the
    // depths' roundings the x87's.
    use rc2k_core::race::faces::{hidden_by_depth, hidden_by_switches};
    let view: u32 = ctx.view.car.camera_view.get();
    // race().frame.view_and_records.view_vars.view_hides[]: by camera view. Its 34 entries cover views 0 .. 0x21; the views up
    // to 0x33 (the viewer's cameras) read on into the variables after it, as the original
    // (notes/bugs.md); 0x33 is still inside `race().frame.view_and_records.view_vars`.
    let shared = &ctx.reads.view_and_records.view_vars;
    let shows: u32 =
        crate::session::read_on_in(shared, &shared.view_hides, view as usize, "the view hides")?;
    let hiding = if shows == 0 {
        race.shared.switches.scene_set_up.get()
    } else {
        0
    };
    let (_, hidden) = hidden_by_switches(shows, hiding, || {
        face.switches() as u32 & ctx.view.face.face_hide_mask.get()
    });
    if hidden {
        return Ok(true);
    }
    points_there(ctx)?;
    let record = point_record(ctx, face.point(0))?;
    let x = bh::to_int(bh::ld(crate::scene::point_records::field(ctx.part, record, 0x10)?).abs());
    ctx.view.object.position[0].set(x as u32);
    let depth = crate::math::x87::add_to_int(
        crate::scene::point_records::field(ctx.part, record, 0x18)?,
        ctx.reads.level_grid.depth_shift.get(),
    );
    ctx.view.object.face_depth.set(depth as u32);
    Ok(hidden_by_depth(depth as u32, x as u32))
}

/// `race::face_depth` (0x4552D8): the depth key of `face` (records of 0x40 at `g_courseFaces`) into
/// `RACE_VIEW.face_depth_key`: for a kind (+0) of 0x10 or more, its first point's depth (`fistp`;
/// and `RACE_VIEW.face_side` its x); else, with its object's (+1) record +0xE bit 1, the largest of
/// its four points' depths (+4 .. +0x10, each `fistp`), otherwise their float sum `fistp`'d and
/// shifted right 2. Then + 0x2000000, shifted right 10 (logical). Refuses (before anything) no
/// point records or points.
pub fn face_depth(ctx: &crate::race::view_scratch::DrawCtx<'_>, face: u32) -> Result<(), Refused> {
    let race = ctx.race;
    use crate::math::x87::{add, to_int};
    points_there(ctx)?;
    let depth = |k: u32| -> Result<u32, Refused> {
        let record = point_record(ctx, face_field(ctx.reads.course(), face, 4 + 4 * k)?)?;
        Ok(crate::scene::point_records::field(ctx.part, record, 0x18)?)
    };
    // Converted: `rc2k_core::race::faces::{depth_by, deeper, average_key, depth_key}`;
    // the roundings and the sum the x87's.
    use rc2k_core::race::faces::{DepthBy, average_key, deeper, depth_by};
    let kind: u8 = face_field(ctx.reads.course(), face, 0)?;
    let trap = Trap::new();
    let by = depth_by(kind, || {
        let object = trap.keep_read(face_field::<u8>(ctx.reads.course(), face, 1)) as u32;
        // +0x2E of the object's 0x40-byte info (`object` a byte: inside the 0x100).
        race.polygon_tables().texture_infos[object as usize].words[11]
            .get()
            .to_le_bytes()[2]
    });
    trap.finish()?;
    let key = match by {
        DepthBy::First => {
            let record = point_record(ctx, face_field(ctx.reads.course(), face, 4)?)?;
            to_int(crate::scene::point_records::field(ctx.part, record, 0x18)?) as u32
        }
        DepthBy::Deepest => {
            let mut key = to_int(depth(0)?) as u32;
            for k in 1..4 {
                let d = to_int(depth(k)?) as u32;
                key = deeper(key, d);
            }
            key
        }
        DepthBy::Average => {
            let mut sum = depth(0)?;
            for k in 1..3 {
                sum = add(depth(k)?, sum).to_bits();
            }
            average_key(crate::math::x87::add_to_int(depth(3)?, sum))
        }
    };
    let key = rc2k_core::race::faces::depth_key(key);
    ctx.view.face.depth_key.set(key);
    Ok(())
}

/// `race::add_depth_entry` (0x454D2A): `face_depth` of `face`, then entry `depth_entry_count` of
/// the view's depth entries: +0 the key, +4 the face; the count + 1. Refuses (before anything) an
/// entry past the 0x1000, or what `face_depth` refuses.
pub fn add_depth_entry(ctx: &DrawCtx, face: u32) -> Result<(), Refused> {
    let DrawCtx { view, .. } = *ctx;
    let lists = &view.lists;
    // Converted: `rc2k_core::race::faces::depth_entry_room`.
    let n: u32 = lists.depth_entry_count.get();
    rc2k_core::race::faces::depth_entry_room(n)?;
    face_depth(ctx, face)?;
    let key: u32 = ctx.view.face.depth_key.get();
    let at = lists.depth_entry(n)?;
    at[0].set(key);
    at[1].set(face);
    lists.depth_entry_count.set(n.wrapping_add(1));
    Ok(())
}
// LEVEL_GRID.course_grid[0x400]: 32 x 32 course records

/// `race::course_grid` (0x45497E): the course records around the view in a 32 x 32 grid. The half
/// size `LEVEL_GRID.grid_half` = `hold_to_limit(STAGE_PREFS.grid_reach)`, the span
/// `LEVEL_GRID.grid_span` twice that; the corner is the view's place
/// (`race().frame.view_and_records.view_vars.view_x` /
/// `race().frame.view_and_records.view_vars.view_z` >> 21, arithmetic) less the half. Unchanged
/// corner: nothing more. Else the corner kept, the grid cleared to 0x0FFFFFFF, and each record
/// (0x20 bytes at `g_courseRecords`, +0 two 11-bit cell numbers, -1 none; each into
/// `COURSE.record_place` / `COURSE.record_place[1]`) whose cell less the corner (& 0x7FF) is within
/// the span on both axes goes in at row z, column x. Refuses (before anything is written) no
/// records when the grid is to be made (the original's loop would run on: it tests `jnz` after the
/// first). `vars`: the view's variables (its place, `ViewScratch::vars`).
pub fn course_grid(
    race: &Race,
    course: CourseReads<'_>,
    view: &crate::race::view_grid::ViewGrid,
    vars: &crate::race::view_vars_store::ViewVarsCopy,
) -> Result<(), Refused> {
    // Converted: `rc2k_core::race::frame::{hold_to_limit, grid_corner, record_cell,
    // grid_cell}`.
    use rc2k_core::race::frame::{GRID_EMPTY, GRID_SIDE, grid_cell, grid_corner, record_cell};
    let half = hold_to_limit(&race.shared, course.stage_prefs.grid_reach.get());
    let span = half.wrapping_add(half);
    let (gx, gz) = grid_corner(vars.view_x.get(), vars.view_z.get(), half);
    let unchanged = gx == view.x.get() && gz == view.z.get();
    let count: u32 = course.course.course_record_count.get();
    if !unchanged && count == 0 {
        return Err(BadInput("no course records (the loop would not end)?").into());
    }
    view.half.set(half);
    view.span.set(span);
    if unchanged {
        return Ok(());
    }
    view.x.set(gx);
    view.z.set(gz);
    let grid = &view.cells;
    for k in 0..GRID_SIDE * GRID_SIDE {
        grid[k as usize].set(GRID_EMPTY);
    }
    for n in 0..count {
        let Some((cx, cz)) = record_cell(record_field(course, n, 0)?) else {
            continue;
        };
        if let Some(cell) = grid_cell(cx, cz, (gx, gz), span) {
            view.cell(cell)?.set(n);
        }
    }
    Ok(())
}

/// `object_bytes`' outputs (byte tables, walked by pointer: absolute addresses).
/// Byte `n` of a face table (an object number: one past the 0x100 is refused, where the
/// original reads on into the next table).
pub(super) fn byte_at(
    mine: &crate::race::view_course::ViewCourse,
    which: usize,
    n: u32,
) -> Result<u32, BadInput> {
    if n >= 0x100 {
        return Err(BadInput("an object number past the face tables"));
    }
    Ok(mine.tables()[which].get(n as usize) as u32)
}

/// A course record's faces (+8 first, +0xC last; -1 none), or None.
pub(super) fn record_faces(
    course: CourseReads<'_>,
    record: u32,
) -> Result<Option<(u32, u32)>, BadInput> {
    let (first, last) = (
        record_field(course, record, 8)?,
        record_field(course, record, 0xC)?,
    );
    Ok((first != u32::MAX && first <= last).then_some((first, last)))
}

/// `race::gather_record_faces` (0x454C99): each face of course record `record` (+8 .. +0xC),
/// `COURSE.current_course_face` the face: a kind (+0) of 0xFF, or below 0x10, not `face_hidden`,
/// goes to `add_depth_entry` when its object's flag says so (`LEVEL_GRID.face_table_b` /
/// `LEVEL_GRID.face_table_a`), else to `add_sort_entry` with the object's kind byte
/// (`LEVEL_GRID.face_table_d` / `LEVEL_GRID.face_table_c`). For kinds below 0x10 the object is +1,
/// or +0x21 when +1 is 0. Refuses a last face of -1 (the loop would not end); the full tables and
/// missing points are refused where they are met.
pub fn gather_record_faces(ctx: &DrawCtx, record: u32) -> Result<(), Refused> {
    let DrawCtx { view, .. } = *ctx;
    let Some((first, last)) = record_faces(ctx.reads.course(), record)? else {
        return Ok(());
    };
    if last == u32::MAX {
        return Err(BadInput("a course record's last face is -1").into());
    }
    let mut face = first;
    loop {
        ctx.view.face.course_face.set(face);
        // The face read once for its kind, objects, switches and first point.
        let read = crate::race::course_places::face_number(ctx.reads.course(), face)?;
        let kind = read.kind();
        // Converted: `rc2k_core::race::faces::{gathered, face_object}`.
        use rc2k_core::race::faces::{Gathered, face_object, gathered};
        let how = gathered(kind);
        if how != Gathered::No && !face_hidden(ctx, &read)? {
            let object = face_object(how, read.object(), || read.second_object());
            let (depth, kind) = if how == Gathered::Lit { (1, 3) } else { (0, 2) };
            if byte_at(&view.course, depth, object)? != 0 {
                add_depth_entry(ctx, face)?;
            } else {
                add_sort_entry(&view.lists, byte_at(&view.course, kind, object)?, face)?;
            }
        }
        face = face.wrapping_add(1);
        if face > record_field(ctx.reads.course(), record, 0xC)? {
            break;
        }
    }
    Ok(())
}

/// `race::gather_visible_faces` (0x454B71): `gather_record_faces` of each record in `g_courseGrid`
/// (numbers below 0x0FFFFFFF), then of the last record once more. Refuses as `gather_record_faces`.
pub fn gather_visible_faces(ctx: &DrawCtx) -> Result<(), Refused> {
    let todo = visible_records(ctx.reads.course(), &ctx.view.grid)?;
    for &record in &todo {
        gather_record_faces(ctx, record)?;
    }
    Ok(())
}

/// `race::face_hide_mask` (0x456056): `physics::probe_ground`, then, unless the face under the car
/// (`g_car` +0xE00) is -1, `race().frame.frame_work.face_hide_mask` = its +2 word's bits 1 and 3
/// flipped when any is set (0xA), or'd with bits 4 and 5 likewise (0x30).
pub fn face_hide_mask(
    race: &Race,
    reads: crate::workers::DrawReads<'_>,
    view: &crate::race::view_scratch::ViewScratch,
    part: &crate::workers::PartScratch,
) -> Result<(), Refused> {
    crate::physics::probe_ground::probe_ground_in_view(
        &crate::physics::probe_ground::ProbeInputs {
            numbers: reads.numbers,
            tables: &race.shared.physics_tables,
            course: reads.course(),
            engine_tables: &race.shared.buffers.engine_tables,
        },
        &view.car,
        reads.draw_switches,
        &part.points,
    )?;
    let ground: u32 = view.car.probe_point_face[4].get();
    if ground == u32::MAX {
        return Ok(());
    }
    // Converted: `rc2k_core::race::faces::hide_mask`.
    let word: u16 = face_field(reads.course(), ground, 2)?;
    view.face
        .face_hide_mask
        .set(rc2k_core::race::faces::hide_mask(word));
    Ok(())
}

/// `race::mark_far_cells` (0x454A5B): every `g_courseGrid` cell's top bits cleared (& 0x0FFFFFFF);
/// then each cell (column x, row z, each less the half `LEVEL_GRID.grid_half`) turned by
/// `g_viewTurnMatrix` (the length of the middle taken): not marked when the length is below 2.0;
/// marked (bit 31) when the turned z is below 0, or when |x / 2| - 2 is not below z, or when the
/// length is not below `limit` (as a float). The original kept the limit in
/// `LEVEL_GRID.far_cell_limit` and stored the turned x and z (`LEVEL_GRID.turned_x` / `turned_z`)
/// and the limit as a float (`DRAW.work_float_b`), which nothing read after.
pub fn mark_far_cells(
    numbers: &crate::race::numbers_store::Numbers,
    scratch: &crate::race::view_scratch::ViewScratch,
    limit: u32,
) -> Result<(), BadInput> {
    let view = &scratch.grid;
    use crate::math::x87::{below_or_unordered, float_hypot, scaled_abs_less_below};
    let grid = &view.cells;
    for k in 0..0x400 {
        grid[k as usize].set(grid[k as usize].get() & 0x0FFF_FFFF);
    }
    let half: u32 = view.half.get();
    // Nothing in the loop writes the matrix: its floats read once.
    let matrix = scratch
        .matrices
        .view_turn_matrix
        .each_ref()
        .map(crate::session::Float::get);
    let two = numbers.two.get();
    let zero = numbers.zero.get();
    let point_five = numbers.half.get();
    let mut out = [0u32; 8];
    for cell in 0..0x400u32 {
        let x = (cell & 0x1F).wrapping_sub(half);
        let z = (cell >> 5).wrapping_sub(half);
        let point = [x, 0, z];
        crate::scene::project_point::rotate(&point, &matrix, &mut out);
        let (x, z) = (out[4], out[6]);
        let length = float_hypot(x, z);
        let limit = crate::math::x87::from_int(limit as i32);
        if below_or_unordered(length, two) {
            continue;
        }
        let mark = below_or_unordered(z, zero)
            || !scaled_abs_less_below(x, point_five, two, z)
            || !below_or_unordered(length, limit);
        if mark {
            grid[cell as usize].set(grid[cell as usize].get() | 0x8000_0000);
        }
    }
    Ok(())
}

/// `race::prepare_course_view` (0x4548A8): `face_hide_mask`, `course_grid`; with
/// `FRAME_SWITCHES.scene_set_up` set, `mark_far_cells` with the limit `checkpoint_colours` gives with
/// two checkpoints or more (`g_checkpointCount`), else 0x10 (the original kept it in
/// `LEVEL_GRID.far_cell_limit`, set to 0x10 first, which nothing else read). Refuses (before
/// anything) what `course_grid` would.
pub fn prepare_course_view(
    race: &Race,
    reads: crate::workers::DrawReads<'_>,
    view: &crate::race::view_scratch::ViewScratch,
    part: &crate::workers::PartScratch,
) -> Result<(), Refused> {
    check_course_grid(reads.course())?;
    face_hide_mask(race, reads, view, part)?;
    course_grid(race, reads.course(), &view.grid, &view.vars)?;
    if race.shared.switches.scene_set_up.get() == 0 {
        return Ok(());
    }
    let limit = if race.shared.vars.checkpoint_count.get() >= 2 {
        checkpoint_colours(race, reads.course(), &view.car)?
    } else {
        0x10
    };
    mark_far_cells(reads.numbers, view, limit)?;
    Ok(())
}

/// `course_grid`'s refusal, before anything.
pub(super) fn check_course_grid(course: CourseReads<'_>) -> Result<(), Refused> {
    if course.course.course_record_count.get() == 0 {
        return Err(BadInput("no course records").into());
    }
    Ok(())
}

/// The records `gather_visible_faces` gathers, as it would.
pub fn visible_records(
    course: CourseReads<'_>,
    view: &crate::race::view_grid::ViewGrid,
) -> Result<Vec<u32>, BadInput> {
    // Converted: `rc2k_core::race::view::visible_records`.
    let grid = &view.cells;
    let list = rc2k_core::race::view::visible_records(
        grid.iter().map(crate::session::Dword::get),
        course.course.course_record_count.get(),
    )
    .into_iter()
    .collect();
    Ok(list)
}

/// `race::sort_sort_entries` (0x454F62) / `race::sort_depth_entries`: with two entries or more, the
/// list's quicksort (0x454F85 / its twin 0x454EF7: 8-byte entries by their first dword, the largest
/// first) from its first entry to its last. Refuses (before anything) a sort that would go past the
/// list. `entries` the list's cells.
pub fn sort_entries(
    count: &crate::session::Dword,
    entries: &[crate::session::Dword],
) -> Result<(), Refused> {
    // Converted: `rc2k_core::race::quick_sort::quick_sort` (through `sort_cells`).
    use rc2k_core::race::quick_sort::Order;
    let n: u32 = count.get();
    crate::race::sorting::sort_cells::<2>(entries, n, 0, Order::Down)
}
