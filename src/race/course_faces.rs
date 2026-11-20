//! Draws the course's faces (`draw_tinted_face` -> `draw_course_face`). A face record is 0x40 bytes
//! at `course_faces`: +0 the kind and flags, +1 the object, +4 .. +0x10 four scene points, +0x18 /
//! +0x30 / +0x38 and +0x1C / +0x34 / +0x3C its two triangles' normals (as `face_turned_to_view`
//! reads them), +0x20 / +0x21 the second's flags and object. This file holds its leaves.

use crate::errors::{BadInput, Refused};
use crate::math::cell_x87::CellX87;
use crate::memory::session::byte_of;
use crate::memory::session::{PlainDword, PlainFloat};
use crate::race::face_work::FaceWork;
use crate::race::owner_course::CourseReads;
use crate::race::trap::Trap;
use crate::race::view_scratch::DrawCtx;
use crate::scene::polygon_store::PolygonEntry;
use rc2k_core::race::faces::Face;

/// The byte `n` of a byte table held as dwords; past it refused.
fn table_byte(
    cells: &[crate::memory::session::PlainDword],
    n: u32,
    what: &'static str,
) -> Result<u32, BadInput> {
    byte_of(cells, n as usize)
        .map(u32::from)
        .ok_or(BadInput(what))
}

/// With no texture animation (`texture_a` 0) and `polygon_mode` 4, turns the depth test off and
/// sets `depth_test` 1.
pub fn face_depth_test_off(face: &FaceWork, part: &crate::scene::workers::PartScratch) {
    {
        if !rc2k_core::race::faces::depth_test_off(face.texture_a.get(), face.polygon_mode.get()) {
            return;
        }
        crate::scene::depth_test_off::depth_test_off(part);
        face.depth_test.set(1u32);
    }
}

/// Turns depth writing back on when `face_depth_test_off` turned it off (`depth_test` set), then
/// sets `depth_test` 0. Fixed: the 1999 test was the wrong way round (on only when it was not
/// turned off); "Course faces drawn with the last face's depth setting" in `wiki/Fixed-bugs.md`.
pub fn face_depth_test_on(face: &FaceWork, part: &crate::scene::workers::PartScratch) {
    {
        if rc2k_core::race::faces::depth_test_on(face.depth_test.get()) {
            crate::scene::depth_test_on::depth_test_on(part);
        }
        face.depth_test.set(0u32);
    }
}

/// Puts a face's four points into the view scratch's corner list, the fourth -1 when it is the
/// third (a triangle). Returns the fourth.
pub fn face_corners(
    scene: &crate::scene::owner::Scene,
    view: &crate::race::view_scratch::ViewScratch,
    face: &Face,
) -> u32 {
    let corners = rc2k_core::race::faces::corners(std::array::from_fn(|n| face.point(n)));
    crate::memory::session::write_bits(&view.face.corners, corners);
    // The next stage's start-up (`race::norms`) reads the last face drawn's corners.
    scene.scratch.note_face_drawn(view.number());
    corners[3]
}

/// 1 when the face's triangle at `face`'s `+half` (half 0 or 4) faces the view (its point +8 less
/// the view's place, `view_place_f` .., dotted with its normal +0x18, +0x30, +0x38 above 0, or not
/// a number, as the x87 compares), 0 outside a race.
#[cfg(test)]
fn face_turned_to_view(
    race: &crate::race::owner::Race,
    face: &[u32],
    half: u32,
) -> Result<u32, BadInput> {
    let mut bytes = [0u8; 64];
    for (b, w) in bytes.as_chunks_mut::<4>().0.iter_mut().zip(face) {
        b.copy_from_slice(&w.to_le_bytes());
    }
    turned_to_view(
        race.course.reads(),
        race.shared.switches.get(),
        &crate::race::scene_points::no_own(),
        &Face(bytes),
        (half / 4) as usize,
        &race
            .frame
            .view_and_records
            .get()
            .view_vars
            .view_place_f
            .each_ref()
            .map(|c| PlainFloat::from_bits(c.bits())),
    )
}

/// On a read face, its triangle 0 or 1. `place`: the view's place as floats (`ViewScratch::vars`).
fn turned_to_view(
    course: CourseReads<'_>,
    switches: &crate::race::switches_store::FrameSwitches,
    points: &crate::race::scene_points::OwnPoints,
    f: &Face,
    triangle: usize,
    place: &[PlainFloat; 3],
) -> Result<u32, BadInput> {
    use crate::math::x87_float::{X, compare, int, ld};
    {
        if switches.scene_set_up.get() == 0 {
            return Ok(0);
        }
        let p = f.point(1 + triangle);
        let normal = f.normal(triangle);
        let at = crate::race::scene_points::point_number(course, points, p)?;
        let along = |k: usize, view_at: &PlainFloat, normal: u32| -> Result<_, BadInput> {
            Ok((int(at[k] as i32) - view_at.ld()) * ld(normal))
        };
        let (x, y, z) = (
            along(0, &place[0], normal[0])?,
            along(1, &place[1], normal[1])?,
            along(2, &place[2], normal[2])?,
        );
        Ok(u32::from(compare(X(0.0), x + (y + z)) & 0x0100 != 0))
    }
}

/// Which of face `face`'s triangles face the view: `face_turned` the first's, `face_turned[1]` the
/// second's, or 1 when the face has no second triangle (`corners[3]` is -1).
pub fn face_sides(ctx: &DrawCtx<'_>, face: &Face) -> Result<(), BadInput> {
    let work = &ctx.view.face;
    {
        work.turned[0].set(turned_to_view(
            ctx.reads.course(),
            ctx.reads.switches,
            &ctx.part.points,
            face,
            0,
            &ctx.view.vars.view_place_f,
        )?);
        work.turned[1].set(1u32);
        if work.corners[3].get() != u32::MAX {
            work.turned[1].set(turned_to_view(
                ctx.reads.course(),
                ctx.reads.switches,
                &ctx.part.points,
                face,
                1,
                &ctx.view.vars.view_place_f,
            )?);
        }
    }
    Ok(())
}

/// (t x f x a x b) rounded to a whole number.
fn frame_of(
    t: u32,
    f: impl crate::math::x87_float::Loadable,
    a: impl crate::math::x87_float::Loadable,
    b: impl crate::math::x87_float::Loadable,
) -> u32 {
    use crate::math::x87_float::{int, ld, to_int};
    to_int(int(t as i32) * ld(f) * ld(a) * ld(b)) as u32
}

/// Sets face `face`'s two triangles' textures: for each, its object (+1, +0x21) is animated when
/// `face_speeds` gives it a speed (car 0's +8 clock times `face_key_factors[speed]`, `ten`,
/// `quarter`, rounded, & 7 picks the frame of `face_places`), into `texture_a` / `texture_b`; the
/// kind's low three bits and bit 3 (+0, +0x20) go into `order_a` / `back_a` and `order_b` /
/// `back_b`.
pub fn face_materials(ctx: &DrawCtx<'_>, face: &Face) -> Result<(), BadInput> {
    let work = &ctx.view.face;
    {
        let clock: u32 = ctx.reads.frame_cars.car(0)?.step_clock.get();
        let (k1, k2) = (ctx.reads.numbers.ten.get(), ctx.reads.numbers.quarter.get());
        let texture = |object: u32| -> Result<u32, BadInput> {
            let speed = table_byte(
                &ctx.reads.stage_prefs.face_speeds,
                object,
                "the face speeds",
            )?;
            if speed == 0 {
                return Ok(object);
            }
            let n = frame_of(
                clock,
                ctx.reads.course_lists.face_key_factors[speed as usize].get(),
                k1,
                k2,
            );
            let (_, place) = rc2k_core::race::faces::frame_place(object, n);
            table_byte(&ctx.reads.stage_prefs.face_places, place, "the face places")
        };
        // The frame's rounding is the x87's (`rc2k_core::race::faces::{frame_place,
        // material_bits}`).
        use rc2k_core::race::faces::material_bits;
        work.texture_a.set(texture(face.object() as u32)?);
        let (order, back) = material_bits(face.kind());
        work.order_a.set(order);
        work.back_a.set(back);
        work.texture_b.set(texture(face.second_object() as u32)?);
        let (order, back) = material_bits(face.second_kind());
        work.order_b.set(order);
        work.back_b.set(back);
    }
    Ok(())
}

/// Draws a marked face: `scene::scale_texture_coordinates`, `scene::fog_polygon`, the drawing
/// (`scene::draw_polygon`) and `polygon_drawn` 1.
pub fn draw_marked_face(ctx: &DrawCtx<'_>) -> Result<(), Refused> {
    crate::scene::scale_texture_coordinates::scale_texture_coordinates(
        &ctx.part.polygon,
        &ctx.view.polygon,
    )?;
    crate::scene::fog_polygon::fog_polygon(ctx.part, ctx.view)?;
    crate::scene::draw_polygon::draw_polygon_in_game(
        ctx.view,
        ctx.part,
        ctx.reads,
        ctx.reads.screen,
    )?;
    ctx.view.polygon.polygon_drawn.set(1u32);
    Ok(())
}

/// Sets `polygon_sides` and the next polygon flag to 1 when the face's first / second texture is
/// animated (`texture_a` / `texture_b` not 0), else 0.
pub fn face_switches(poly: &crate::scene::polygon_work::PolygonWork, work: &FaceWork) {
    {
        let [first, second] =
            rc2k_core::race::faces::switches([work.texture_a.get(), work.texture_b.get()]);
        poly.polygon_sides[0].set(first);
        poly.polygon_sides[1].set(second);
    }
}

/// The light at scene point `n` (`light_colour_at`), 0 for -1.
fn point_light(ctx: &DrawCtx<'_>, n: u32) -> Result<u32, Refused> {
    let race = ctx.race;
    if n == u32::MAX {
        return Ok(0);
    }
    let [x, y, z] =
        crate::race::scene_points::point_number(ctx.reads.course(), &ctx.part.points, n)?;
    ctx.view.light.lit_place[0].set(x);
    ctx.view.light.lit_place[1].set(y);
    ctx.view.light.lit_place[2].set(z);
    crate::race::frame_draw::light_colour_at(
        ctx.reads,
        ctx.reads.net_race.get(),
        &ctx.part.points,
        &ctx.view.lamps,
        &ctx.view.light,
        ctx.view.drawn_view(race),
    )?;
    let light: u32 = ctx.view.light.light_colour.get();
    Ok(light)
}

/// The light at each of the face's four corners (`corner_lights` .. `corner_lights[3]`; 0 for a
/// missing fourth), when lamps are wanted (`lamps_wanted`) and the face is not a plain mode-4 one;
/// all 0 otherwise. Refused (before anything): what `light_colour_at` refuses.
pub fn face_lights(ctx: &DrawCtx<'_>) -> Result<(), Refused> {
    let wanted: u32 = ctx.view.lamps.lamps_wanted.get();
    let lit = wanted != 0
        && rc2k_core::race::faces::corners_lit(
            wanted,
            ctx.view.face.texture_a.get(),
            ctx.view.face.polygon_mode.get(),
        );
    if lit {
        crate::race::frame_draw::check_light_colour(
            ctx.reads.work,
            ctx.reads.view,
            &ctx.view.lamps,
        )?;
    }
    for item in ctx.view.face.corner_lights.iter().take(4) {
        item.set(0u32);
    }
    if !lit {
        return Ok(());
    }
    for k in 0..4 {
        let n: u32 = ctx.view.face.corners[k].get();
        let light = point_light(ctx, n)?;
        ctx.view.face.corner_lights[k].set(light);
    }
    Ok(())
}

/// Cuts the face's polygon by depth (when some vertices are outside, `polygon_outside_count`) and
/// to the screen, and draws it with three vertices or more (`scene::draw_polygon_in_game`);
/// `polygon_drawn` 1.
pub fn clip_and_draw_face(ctx: &DrawCtx<'_>) -> Result<(), Refused> {
    use rc2k_core::race::face_drawing::draws_polygon;
    if ctx.view.polygon.polygon_outside_count.get() != 0 {
        crate::scene::clip_polygon_by_depth::clip_polygon_by_depth(
            ctx.part,
            &ctx.view.polygon,
            ctx.reads.course(),
            ctx.reads.draw_switches,
            ctx.far_clip,
            ctx.view,
        )?;
        if !draws_polygon(ctx.view.polygon.polygon_vertex_count.get()) {
            return Ok(());
        }
    }
    crate::scene::clip_polygon_to_screen::clip_polygon_to_screen(
        ctx.part,
        &ctx.view.polygon,
        ctx.reads.course(),
    )?;
    if !draws_polygon(ctx.view.polygon.polygon_vertex_count.get()) {
        return Ok(());
    }
    crate::scene::draw_polygon::draw_polygon_in_game(
        ctx.view,
        ctx.part,
        ctx.reads,
        ctx.reads.screen,
    )?;
    ctx.view.polygon.polygon_drawn.set(1u32);
    Ok(())
}

/// True when none of the face's corners (`corners` ..; the fourth when there is one) is in sight: a
/// corner is in sight when its scene record's depth (+0x18) is not negative and |x| (+0x10) is
/// below `near` or |x| times `slope` is below the depth. The records are made when not there.
/// Refused (before anything): no point records or points.
pub fn face_out_of_view(ctx: &DrawCtx<'_>) -> Result<bool, Refused> {
    // The in-sight test is exact in `f64` (`rc2k_core::race::face_drawing::corner_in_sight`).
    use rc2k_core::race::face_drawing::{InSight, corner_in_sight};
    crate::race::frame_draw::points_there(ctx)?;
    let near = ctx.reads.view.near.get();
    // The cone to the sides is as much wider as the view is wider than 4:3.
    let slope =
        ctx.reads.view.slope.get() / crate::race::frame_draw::side_widening(ctx.reads, ctx.view);
    for k in 0..4 {
        let n: u32 = ctx.view.face.corners[k].get();
        if k == 3 && n == u32::MAX {
            break;
        }
        let record = crate::race::frame_draw::point_record(ctx, n)?;
        let depth = crate::scene::point_records::field(ctx.part, record, 0x18)?;
        if depth & 0x8000_0000 != 0 {
            continue;
        }
        let x = crate::scene::point_records::field(ctx.part, record, 0x10)?;
        match corner_in_sight(depth, x, near, slope) {
            Some(InSight::Near) => {
                return Ok(false);
            }
            Some(InSight::Slope) => return Ok(false),
            None => {}
        }
    }
    Ok(true)
}

/// Whether the face leaves nothing on the screen: its corners all projected and past one edge
/// (`rc2k_core::race::face_drawing::off_screen`), without the face's colours, lights and polygons
/// made for nothing. The corners' records are there: `face_out_of_view` made them.
fn face_off_screen(ctx: &DrawCtx<'_>) -> Result<bool, Refused> {
    use rc2k_core::race::face_drawing::{ScreenCorner, ScreenEdges, off_screen};
    let mut corners = [ScreenCorner {
        depth: 0,
        x: 0,
        y: 0,
    }; 4];
    let mut n = 0;
    for k in 0..4 {
        let point: u32 = ctx.view.face.corners[k].get();
        if k == 3 && point == u32::MAX {
            break;
        }
        let record = crate::race::frame_draw::point_record(ctx, point)?;
        let at = |offset| crate::scene::point_records::field(ctx.part, record, offset);
        corners[n] = ScreenCorner {
            depth: at(0x18)?,
            x: at(0xC)?,
            y: at(0x1C)?,
        };
        n += 1;
    }
    let clip = &ctx.view.polygon.window.clip_screen;
    let edges = ScreenEdges {
        top: clip[0].get(),
        bottom: clip[1].get(),
        left: clip[2].get(),
        right: clip[3].get(),
    };
    Ok(off_screen(
        &corners[..n],
        edges,
        crate::scene::project_point::NEAR,
    ))
}

/// Where one of the face's two triangles keeps its texture's working values (the same code runs for
/// the first and the second, on other variables).
pub struct TextureSet<'a> {
    /// The texture (`texture_a` / `texture_b`).
    pub texture: &'a PlainDword,
    /// The kind's low bits, the corner order (`order_a` / `order_b`).
    pub order: &'a PlainDword,
    /// The texture's size word with bit 31 (`polygon_textures` / `polygon_textures[1]`).
    pub size: &'a PlainDword,
    /// The texture's mapping mode, its +0x2D signed (`mapping_a` / `mapping_b`).
    pub mapping: &'a PlainDword,
    /// 1 when the mapping is 0 (`plain_first` / the next flag).
    pub plain: &'a PlainDword,
    /// The four corners' u, v (`corner_uv` .. / `corner_uv_b` ..).
    pub uv: &'a [PlainDword; 8],
}

/// The first triangle's set.
pub fn first_set<'a>(ctx: &DrawCtx<'a>) -> TextureSet<'a> {
    let work = &ctx.view.face;
    TextureSet {
        texture: &work.texture_a,
        order: &work.order_a,
        size: &ctx.view.polygon.polygon_textures[0],
        mapping: &work.mapping_a[0],
        plain: &ctx.view.polygon.plain_first,
        uv: &work.uv_a,
    }
}

/// The second triangle's set.
pub fn second_set<'a>(ctx: &DrawCtx<'a>) -> TextureSet<'a> {
    let work = &ctx.view.face;
    TextureSet {
        texture: &work.texture_b,
        order: &work.order_b,
        size: &ctx.view.polygon.polygon_textures[1],
        mapping: &work.mapping_b[0],
        plain: &ctx.view.polygon.plain_second,
        uv: &work.uv_b,
    }
}

/// The `N` bytes (at most 4) at `offset` of texture `texture`'s info record. The texture is a byte
/// (the face's object or its animation's place), the offsets are the record's own (the size word,
/// the mapping byte, the corner table's 0 .. 12 and two more), so the bytes lie in one record; a
/// place past the records is refused.
fn info_bytes<const N: usize>(
    tables: &crate::scene::polygon_store::PolygonTables,
    texture: u32,
    offset: u32,
) -> Result<[u8; N], BadInput> {
    let infos = &tables.texture_infos;
    let at = (texture << 6).wrapping_add(offset) as usize;
    let (dword, byte) = (at / 4, at % 4);
    if let Some(info) = infos.get(dword / 16)
        && let Some(bytes) = info.words[dword % 16]
            .get()
            .to_le_bytes()
            .get(byte..byte + N)
    {
        return Ok(bytes.try_into().expect("N bytes"));
    }
    Err(BadInput("a texture info's bytes past the texture infos"))
}

/// Sets the triangle's texture's size word (+0x20, bit 31 set) and mapping (+0x2D) and the four
/// corners' u, v from the texture's corner table for the order (`corner_uv_offsets[order * 4 + k]`
/// points at a u16 pair inside the texture record, << 8, doubled for sizes 0x190 .. 0x31F); with a
/// mapping above 0 the corners' scene x and z are shifted by (mapping & 3) + 2 instead, less the
/// first corner's top byte (the fourth corner is read even when it is -1). Used for both triangles.
pub fn face_texture_coordinates(
    course: CourseReads<'_>,
    points: &crate::race::scene_points::OwnPoints,
    buffers: &crate::race::buffers_store::BufferReads,
    race_view: &crate::race::view_store::RaceView,
    work: &FaceWork,
    set: &TextureSet<'_>,
) -> Result<(), BadInput> {
    use rc2k_core::race::faces::{TextureInfo, texture_coordinates};
    {
        let texture: u32 = set.texture.get();
        let texture_info = TextureInfo {
            size: u32::from_le_bytes(info_bytes(&buffers.polygon_tables, texture, 0x20)?),
            mapping: i8::from_le_bytes(info_bytes(&buffers.polygon_tables, texture, 0x2D)?),
        };
        let order: u32 = set.order.get();
        let mut pairs = [(0u16, 0u16); 4];
        for (k, pair_out) in pairs.iter_mut().enumerate() {
            let table = race_view.corner_uv_offsets[(order << 2) as usize + k].get();
            *pair_out = (
                u16::from_le_bytes(info_bytes(&buffers.polygon_tables, texture, table)?),
                u16::from_le_bytes(info_bytes(
                    &buffers.polygon_tables,
                    texture,
                    table.wrapping_add(2),
                )?),
            );
        }
        let trap = Trap::new();
        let c = texture_coordinates(texture_info, pairs, |k| {
            let n: u32 = work.corners[k].get();
            crate::race::scene_points::point_xz(course, points, n).unwrap_or_else(|e| {
                trap.keep(e);
                (0, 0)
            })
        });
        trap.finish()?;
        set.size.set(c.size);
        set.mapping.set(c.mapping);
        set.plain.set(c.plain);
        crate::memory::session::write_bits(set.uv, c.uv);
    }
    Ok(())
}

/// Copies the face's corners to the four scene points after the last (`scene_point_count` + 0 .. 3,
/// their records marked unmade, all four), the corners then naming the copies (the fourth only when
/// there is one); the count is left as it is. Refused (before anything): no point records or
/// points.
pub fn face_corners_to_new_points(ctx: &DrawCtx<'_>) -> Result<(), Refused> {
    let work = &ctx.view.face;
    {
        crate::race::frame_draw::points_there(ctx)?;
        let count: u32 = ctx.reads.course_work.scene_point_count.get();
        let corners: [u32; 4] = crate::memory::session::read_bits(&work.corners);
        let (marked, copies) = rc2k_core::race::faces::new_points(count, corners);
        for n in marked {
            crate::scene::point_records::put_field(
                ctx.part,
                crate::scene::point_records::at_offset(n << 5),
                0,
                u32::MAX,
            )?;
        }
        for (k, (from, to)) in copies.into_iter().enumerate() {
            crate::race::scene_points::copy_point(ctx.reads.course(), &ctx.part.points, from, to)?;
            work.corners[k].set(to);
        }
        Ok(())
    }
}

/// The work table's entry `slot`.
fn entry(polygon_work: &crate::scene::polygon_store::PolygonWork, slot: u32) -> &PolygonEntry {
    &polygon_work.entries[slot as usize]
}

/// One polygon entry filled from one of the face's corners: +0 its point, +8 / +0xC the first
/// texture's u, v, +0x10 / +0x14 the second's, +0x18 its colour, +0x1C its light, +0x21 its fog
/// byte (`corner_alphas` ..).
fn fill_entry(
    polygon_work: &crate::scene::polygon_store::PolygonWork,
    work: &FaceWork,
    slot: u32,
    corner: u32,
) -> Result<(), BadInput> {
    {
        let e = entry(polygon_work, slot);
        let c = corner as usize;
        e.point.set(work.corners[c].get());
        e.u.set(work.uv_a[2 * c].get());
        e.v.set(work.uv_a[2 * c + 1].get());
        e.light.set(work.corner_colours[c].get());
        e.lamp_light.set(work.corner_lights[c].get());
        e.u2.set(work.uv_b[2 * c].get());
        e.v2.set(work.uv_b[2 * c + 1].get());
        e.amount.set(work.corner_alphas[c].get() as u8);
    }
    Ok(())
}

/// The light's third byte (bits 16 .. 23) of entry `slot`: the lamps' part.
fn lamp_byte(
    polygon_work: &crate::scene::polygon_store::PolygonWork,
    slot: u32,
) -> Result<u8, BadInput> {
    Ok(entry(polygon_work, slot).byte(0x1E))
}

/// The polygon list 0 .. count - 1 and a closing 0, `polygon_vertex_count` count,
/// `polygon_next_entry` `next`.
fn plain_list(
    polygon_work: &crate::scene::polygon_store::PolygonWork,
    poly: &crate::scene::polygon_work::PolygonWork,
    count: u32,
    next: u32,
) {
    {
        poly.polygon_vertex_count.set(count);
        for k in 0..count {
            crate::scene::polygon::list_cell(polygon_work, k as usize).set(k);
        }
        crate::scene::polygon::list_cell(polygon_work, count as usize).set(0);
        poly.polygon_next_entry.set(next);
    }
}

/// Draws the face as one quad when it can be: the entries 1, 2, 3, 0 from corners 0, 1, 3, 2. It
/// carries (the caller draws triangles) when a corner has lamp light or some vertex is outside
/// after `scene::project_polygon` or off the screen's inside (the screen edges `clip_edges` .. less
/// one; each vertex's x and y rounded, compared by the sign of the difference); nothing is drawn
/// when all are outside; else the quad is drawn (`draw_marked_face`).
pub fn draw_whole_quad(ctx: &DrawCtx<'_>) -> Result<bool, Refused> {
    // The roundings are the x87's.
    use rc2k_core::race::face_drawing::{QUAD_ENTRIES, quad_by_outside, screen_inside};
    for (slot, corner) in QUAD_ENTRIES {
        fill_entry(&ctx.part.polygon, &ctx.view.face, slot, corner)?;
    }
    for s in 0..4 {
        if lamp_byte(&ctx.part.polygon, s)? != 0 {
            return Ok(true);
        }
    }
    plain_list(&ctx.part.polygon, &ctx.view.polygon, 4, 4);
    crate::scene::project_polygon::project_polygon(
        ctx.part,
        &ctx.view.polygon,
        ctx.reads,
        ctx.view,
    )?;
    if let Some(triangles) = quad_by_outside(ctx.view.polygon.polygon_outside_count.get()) {
        return Ok(triangles);
    }
    let [left, right, top, bottom] = screen_inside(
        ctx.view.polygon.window.clip_edges[2].get(),
        ctx.view.polygon.window.clip_edges[3].get(),
        ctx.view.polygon.window.clip_edges[0].get(),
        ctx.view.polygon.window.clip_edges[1].get(),
    );
    let inside = |v: u32, low: u32, high: u32| {
        let n = crate::math::x87::to_int(v) as u32;
        rc2k_core::race::face_drawing::inside(n, low, high)
    };
    for slot in 0..4 {
        let v = entry(&ctx.part.polygon, slot).record.get();
        if !inside(
            crate::scene::point_records::vertex(ctx.part, &ctx.view.polygon.quad_vertices, v, 0xC),
            left,
            right,
        ) || !inside(
            crate::scene::point_records::vertex(ctx.part, &ctx.view.polygon.quad_vertices, v, 0x1C),
            top,
            bottom,
        ) {
            return Ok(true);
        }
    }
    ctx.view.polygon.polygon_vertex_count.set(4u32);
    draw_marked_face(ctx)?;
    Ok(false)
}

/// Draws the face's two triangles (`draw_lit_triangle`) unless facing away: the first (corners 0,
/// 1, 2) when `face_turned` is 0, the second (3, 2, 1) when there is one and `face_turned[1]` is 0.
pub fn draw_face_triangles(ctx: &DrawCtx<'_>) -> Result<(), Refused> {
    use rc2k_core::race::face_drawing::{
        FIRST_TRIANGLE, SECOND_TRIANGLE, draws_first, draws_second,
    };
    if draws_first(ctx.view.face.turned[0].get()) {
        for (slot, corner) in FIRST_TRIANGLE {
            fill_entry(&ctx.part.polygon, &ctx.view.face, slot, corner)?;
        }
        ctx.view.polygon.polygon_vertex_count.set(3u32);
        draw_lit_triangle(ctx)?;
    }
    let fourth: u32 = ctx.view.face.corners[3].get();
    if fourth != u32::MAX && draws_second(fourth, ctx.view.face.turned[1].get()) {
        for (slot, corner) in SECOND_TRIANGLE {
            fill_entry(&ctx.part.polygon, &ctx.view.face, slot, corner)?;
        }
        ctx.view.polygon.polygon_vertex_count.set(3u32);
        draw_lit_triangle(ctx)?;
    }
    Ok(())
}

/// In a race with both triangles facing the view (`face_turned` + `face_turned[1]` 0) draws the
/// face as one quad, else as triangles (`draw_face_triangles`).
pub fn draw_face_polygons(ctx: &DrawCtx<'_>) -> Result<(), Refused> {
    let in_race: u32 = ctx.reads.switches.scene_set_up.get();
    if in_race != 0
        && rc2k_core::race::face_drawing::tries_quad(
            in_race,
            ctx.view.face.turned[0].get(),
            ctx.view.face.turned[1].get(),
        )
        && !draw_whole_quad(ctx)?
    {
        return Ok(());
    }
    draw_face_triangles(ctx)
}

/// Draws a face: its textures (`face_switches`, the first's coordinates when animated;
/// `polygon_mode` 2, or with a second texture the mode its record's +0x2C picks in `surface_kinds`
/// and the second's coordinates), the corners' lights (`face_lights`), then the depth test off, the
/// polygons (`draw_face_polygons`), the depth test on. Refused (before anything): what
/// `face_lights` refuses.
pub fn draw_face(ctx: &DrawCtx<'_>) -> Result<(), Refused> {
    use rc2k_core::race::face_drawing::{PLAIN_MODE, face_mode};
    let second: u32 = ctx.view.face.texture_b.get();
    let trap = Trap::new();
    let picked = || {
        // The record's kind byte picks one of the four surface kinds: 0 .. 3 in every texture info
        // file shipped; a byte past them is refused.
        let pick = trap.keep_read(info_bytes::<1>(
            &ctx.reads.buffers.polygon_tables,
            second,
            0x2C,
        ))[0] as usize;
        match ctx.reads.view.surface_kinds.get(pick) {
            Some(kind) => kind.get(),
            None => {
                trap.keep(BadInput("a texture's surface kind past the four kinds"));
                0
            }
        }
    };
    let mode = face_mode(second, picked);
    trap.finish()?;
    let wanted: u32 = ctx.view.lamps.lamps_wanted.get();
    if wanted != 0
        && rc2k_core::race::faces::corners_lit(wanted, ctx.view.face.texture_a.get(), mode)
    {
        crate::race::frame_draw::check_light_colour(
            ctx.reads.work,
            ctx.reads.view,
            &ctx.view.lamps,
        )?;
    }
    face_switches(&ctx.view.polygon, &ctx.view.face);
    if ctx.view.face.texture_a.get() != 0 {
        face_texture_coordinates(
            ctx.reads.course(),
            &ctx.part.points,
            ctx.reads.buffers,
            ctx.reads.view,
            &ctx.view.face,
            &first_set(ctx),
        )?;
    }
    ctx.view.face.polygon_mode.set(PLAIN_MODE);
    if second != 0 {
        ctx.view.face.polygon_mode.set(face_mode(second, picked));
        trap.finish()?;
        face_texture_coordinates(
            ctx.reads.course(),
            &ctx.part.points,
            ctx.reads.buffers,
            ctx.reads.view,
            &ctx.view.face,
            &second_set(ctx),
        )?;
    }
    face_lights(ctx)?;
    face_depth_test_off(&ctx.view.face, ctx.part);
    draw_face_polygons(ctx)?;
    face_depth_test_on(&ctx.view.face, ctx.part);
    Ok(())
}

/// The new vertex of `draw_lit_triangle` between two entries' vertices `a` and `b`, into `v`: the
/// view floats (+0x10 .. +0x18) halfway (b - a) x `half` + a, the screen place (+0xC, +0x1C) 0, or
/// when the depth is not below `k4096_a` projected (by `view_scale` over the depth, or over
/// `flat_depth` + 1 when `flat_depth` is set; centre `view_centre`, the next).
fn halfway_vertex(
    course: CourseReads<'_>,
    part: &crate::scene::workers::PartScratch,
    quad: &[crate::memory::session::PlainDword; 32],
    a: u32,
    b: u32,
    v: u32,
    window: &crate::scene::view_window::ViewWindow,
) -> Result<(), BadInput> {
    use crate::scene::point_records::{get_in, set_in};
    let quad = Some(quad);
    let mut made = get_in(part, quad, v)?;
    halfway_record(
        course,
        &get_in(part, quad, a)?,
        &get_in(part, quad, b)?,
        &mut made,
        window,
    );
    set_in(part, quad, v, made)
}

/// `halfway_vertex` on the records themselves (`a`, `b` point records of any kind).
fn halfway_record(
    course: CourseReads<'_>,
    a: &crate::scene::point_records::Record,
    b: &crate::scene::point_records::Record,
    v: &mut crate::scene::point_records::Record,
    window: &crate::scene::view_window::ViewWindow,
) {
    use crate::math::x87::below_or_unordered;
    use crate::math::x87_float::{ld, st};
    let half = course.numbers.half.ld();
    for k in [4, 5, 6] {
        let (from, to) = (ld(a[k]), ld(b[k]));
        v[k] = st((to - from) * half + from);
    }
    (v[3], v[7]) = (0, 0);
    if below_or_unordered(v[6], course.numbers.k4096_a.get()) {
        return;
    }
    crate::race::skidmarks::project_record(course.stage_prefs, course.course, v, window);
}

/// The new entry of `draw_lit_triangle` between entries `first` and `second`, into `to`: the
/// texture floats (+8 .. +0x14) (second + first) x `half`, the bytes +0x20, +0x21, +0x18 .. +0x1A
/// the halves of their sums (+0x1B left as it is).
fn halfway_entry(
    course: CourseReads<'_>,
    first: &PolygonEntry,
    second: &PolygonEntry,
    to: &PolygonEntry,
) {
    use crate::math::x87_float::{ld, st};
    let half = course.numbers.half.ld();
    for o in [8, 0xC, 0x10, 0x14] {
        let sum = ld(second.word(o)) + ld(first.word(o));
        to.set_word(o, st(sum * half));
    }
    for o in [0x20, 0x21, 0x18, 0x19, 0x1A] {
        let sum = second.byte(o) as u32 + first.byte(o) as u32;
        to.set_byte(o, (sum >> 1) as u8);
    }
}

/// Draws the triangle in entries 0 .. 2. Without lamp light at its corners it is drawn as it is
/// (`scene::draw_clipped_polygon`). Else (unless `scene::project_polygon` finds it all outside:
/// `polygon_drawn` 0, the record pool restarted, `scene::scale_texture_coordinates`,
/// `scene::fog_polygon`) it is cut in four: for each side (0-1, 1-2, 2-0) a new scene point halfway
/// (vertices; its light by `light_colour_at`) and entry (3 .. 5); in mode 5 each entry's fog byte
/// less twice its lamp light (not below 0); each entry's colour plus its light (`add_colours`);
/// then the triangles (0, 3, 5), (1, 4, 3), (2, 5, 4), (3, 4, 5) are each clipped and drawn
/// (`clip_and_draw_face`). Refused (before anything): no points, or what `light_colour_at` refuses.
pub fn draw_lit_triangle(ctx: &DrawCtx<'_>) -> Result<(), Refused> {
    let race = ctx.race;
    use crate::race::frame_draw::add_colours;
    let mut lit = false;
    for s in 0..3 {
        if lamp_byte(&ctx.part.polygon, s)? != 0 {
            lit = true;
            break;
        }
    }
    if !lit {
        return crate::scene::draw_clipped_polygon::draw_clipped_polygon_in_game(ctx);
    }
    crate::race::frame_draw::points_there(ctx)?;
    crate::race::frame_draw::check_light_colour(ctx.reads.work, ctx.reads.view, &ctx.view.lamps)?;
    ctx.view.polygon.polygon_drawn.set(0u32);
    ctx.view
        .polygon
        .polygon_next_record
        .set(crate::scene::polygon::pool_start());
    plain_list(
        &ctx.part.polygon,
        &ctx.view.polygon,
        ctx.view.polygon.polygon_vertex_count.get(),
        6,
    );
    crate::scene::project_polygon::project_polygon(
        ctx.part,
        &ctx.view.polygon,
        ctx.reads,
        ctx.view,
    )?;
    if ctx.view.polygon.polygon_outside_count.get() == ctx.view.polygon.polygon_vertex_count.get() {
        return Ok(());
    }
    crate::scene::scale_texture_coordinates::scale_texture_coordinates(
        &ctx.part.polygon,
        &ctx.view.polygon,
    )?;
    crate::scene::fog_polygon::fog_polygon(ctx.part, ctx.view)?;
    for k in 0..3 {
        entry(&ctx.part.polygon, 3 + k)
            .record
            .set(crate::scene::point_records::of_quad(k));
    }
    for (k, (first, second)) in [(0u32, 1u32), (1, 2), (2, 0)].into_iter().enumerate() {
        let v = crate::scene::point_records::of_quad(k as u32);
        let pa = entry(&ctx.part.polygon, first).point.get();
        let pb = entry(&ctx.part.polygon, second).point.get();
        let (at_a, at_b) = (
            crate::race::scene_points::point_number(ctx.reads.course(), &ctx.part.points, pa)?,
            crate::race::scene_points::point_number(ctx.reads.course(), &ctx.part.points, pb)?,
        );
        for (n, lit) in ctx.view.light.lit_place.iter().enumerate() {
            let o = 4 * n as u32;
            let (a, b) = (at_a[n], at_b[n]);
            let m = ((b.wrapping_sub(a) as i32) >> 1) as u32;
            let m = m.wrapping_add(a);
            crate::scene::point_records::put_field_in(
                ctx.part,
                Some(&ctx.view.polygon.quad_vertices),
                v,
                o,
                m,
            )?;
            lit.set(m);
        }
        halfway_vertex(
            ctx.reads.course(),
            ctx.part,
            &ctx.view.polygon.quad_vertices,
            entry(&ctx.part.polygon, first).record.get(),
            entry(&ctx.part.polygon, second).record.get(),
            v,
            &ctx.view.polygon.window,
        )?;
        let to = entry(&ctx.part.polygon, 3 + k as u32);
        halfway_entry(
            ctx.reads.course(),
            entry(&ctx.part.polygon, first),
            entry(&ctx.part.polygon, second),
            to,
        );
        crate::race::frame_draw::light_colour_at(
            ctx.reads,
            ctx.reads.net_race.get(),
            &ctx.part.points,
            &ctx.view.lamps,
            &ctx.view.light,
            ctx.view.drawn_view(race),
        )?;
        let light: u32 = ctx.view.light.light_colour.get();
        to.lamp_light.set(light);
    }
    let mode5 = ctx.view.face.polygon_mode.get() == 5;
    for slot in 0..6 {
        let e = entry(&ctx.part.polygon, slot);
        if mode5 {
            let fog = e.amount.get() as i32 - 2 * lamp_byte(&ctx.part.polygon, slot)? as i32;
            e.amount.set(fog.max(0) as u8);
        }
        e.light.set(add_colours(e.light.get(), e.lamp_light.get()));
    }
    for list in [[0u32, 3, 5], [1, 4, 3], [2, 5, 4], [3, 4, 5]] {
        for (k, n) in list.iter().enumerate() {
            crate::scene::polygon::list_cell(&ctx.part.polygon, k).set(*n);
        }
        ctx.part.polygon.list[3].set(list[0]);
        ctx.view.polygon.polygon_vertex_count.set(3u32);
        ctx.view.polygon.polygon_next_entry.set(6u32);
        clip_and_draw_face(ctx)?;
    }
    Ok(())
}

/// Works out the corners' colours from `light_table` by the face's bytes +0x14 .. +0x17, with flag
/// 4 of +2 plus `lamp_table_a[+0x28 ..]` and `lamp_table_b[+0x2C ..]` (`add_colours`);
/// the fog bytes +0x22 .. +0x25 go into `corner_alphas` ..; then the face is drawn (`draw_face`).
pub fn face_colours(ctx: &DrawCtx<'_>, face: &Face) -> Result<(), Refused> {
    let light_tables = ctx.reads.light_tables;
    use rc2k_core::race::lights::{EXTRA_LIGHTS, corner_colour};
    let lit = face.switches() & EXTRA_LIGHTS != 0;
    for k in 0..4 {
        let c = light_tables.light_table[face.corner_light(k) as usize].get();
        ctx.view.face.corner_colours[k].set(c);
        if lit {
            let extra = || {
                let (a, b) = face.lamps(k);
                let a = light_tables.lamp_table_a[a as usize].get();
                let b = light_tables.lamp_table_b[b as usize].get();
                (a, b)
            };
            let c = corner_colour(c, lit, extra);
            ctx.view.face.corner_colours[k].set(c);
        }
    }
    for (k, cell) in ctx.view.face.corner_alphas.iter().enumerate().take(4) {
        cell.set(face.corner_fog(k) as u32);
    }
    draw_face(ctx)
}

/// Turns the corners over (0 with 1, and 2 with 3 when there is a fourth) and the first texture's
/// order by `turned_corner_order` (its bit 3 kept); `draw_back_face` does this twice.
fn turn_over(ctx: &DrawCtx<'_>) -> Result<(), BadInput> {
    let work = &ctx.view.face;
    {
        let swap = |a: usize, b: usize| {
            let (x, y) = (work.corners[a].get(), work.corners[b].get());
            work.corners[a].set(y);
            work.corners[b].set(x);
        };
        use rc2k_core::race::face_drawing::{turned_order, turns_last_pair};
        swap(0, 1);
        if turns_last_pair(work.corners[3].get()) {
            swap(2, 3);
        }
        let order: u32 = work.order_a.get();
        let trap = Trap::new();
        let turned = turned_order(order, |n| {
            // `turned_order` asks 0 ..= 7.
            byte_of(&ctx.reads.view.turned_corner_order, n as usize).unwrap_or_else(|| {
                trap.keep(BadInput("past the turned corner order"));
                0
            })
        });
        trap.finish()?;
        work.order_a.set(turned);
    }
    Ok(())
}

/// Draws a two-sided face's back (when `back_a`, the first kind's bit 3): the corners turned over,
/// the colours from `light_table` by +0x28 .. +0x2B and the fog bytes +0x2C .. +0x2F, drawn
/// (`draw_face`), the corners turned back.
pub fn draw_back_face(ctx: &DrawCtx<'_>, face: &Face) -> Result<(), Refused> {
    if !rc2k_core::race::face_drawing::has_back(ctx.view.face.back_a.get()) {
        return Ok(());
    }
    turn_over(ctx)?;
    for k in 0..4 {
        ctx.view.face.corner_colours[k]
            .set(ctx.reads.light_tables.light_table[face.back_light(k) as usize].get());
    }
    for (k, cell) in ctx.view.face.corner_alphas.iter().enumerate().take(4) {
        cell.set(face.back_fog(k) as u32);
    }
    draw_face(ctx)?;
    turn_over(ctx)?;
    Ok(())
}

/// Outside a race both sides (`face_colours`, `draw_back_face`) are drawn with neither facing away;
/// in a race, a face with objects (+1 or +0x21): one-sided (kind bit 3 clear) its front unless both
/// triangles face away; two-sided its front unless both face away, then the facings flipped and its
/// back unless both (now) face away. Each after `face_materials`. `face` is the face drawn.
pub fn draw_face_sides(ctx: &DrawCtx<'_>, face: &Face) -> Result<(), Refused> {
    use rc2k_core::race::face_drawing::{flipped, has_objects, side_drawn, two_sided};
    if ctx.reads.switches.scene_set_up.get() == 0 {
        face_materials(ctx, face)?;
        ctx.view.face.turned[0].set(0u32);
        ctx.view.face.turned[1].set(0u32);
        face_colours(ctx, face)?;
        ctx.view.face.turned[0].set(0u32);
        ctx.view.face.turned[1].set(0u32);
        return draw_back_face(ctx, face);
    }
    let facings =
        || -> bool { side_drawn(ctx.view.face.turned[0].get(), ctx.view.face.turned[1].get()) };
    if !has_objects(face.object(), face.second_object()) {
        return Ok(());
    }
    if !two_sided(face.kind()) {
        if !facings() {
            return Ok(());
        }
        face_materials(ctx, face)?;
        return face_colours(ctx, face);
    }
    face_materials(ctx, face)?;
    if facings() {
        face_colours(ctx, face)?;
    }
    ctx.view.face.turned[0].set(flipped(ctx.view.face.turned[0].get()));
    ctx.view.face.turned[1].set(flipped(ctx.view.face.turned[1].get()));
    if facings() {
        draw_back_face(ctx, face)?;
    }
    Ok(())
}

/// Draws face `face`: its corners, and unless none is in sight (`face_out_of_view`) its facings
/// (`face_sides`) and its sides (`draw_face_sides`). Refused (before anything): no point records or
/// points. Fixed: the 1999 code wrapped the sides in `face_depth_test_off` / `on` too, judged by
/// the previous face's texture and mode (this face's are set inside `draw_face_sides`), so a face
/// after a mode-4 face could be drawn without writing depth; each side's own `draw_face` does it
/// with its own ("Course faces drawn with the last face's depth setting" in `wiki/Fixed-bugs.md`).
pub fn draw_course_face(ctx: &DrawCtx<'_>, face: u32) -> Result<(), Refused> {
    crate::race::frame_draw::points_there(ctx)?;
    // The face is read once for its corners, normals, kinds, objects and colours: drawing it writes
    // no face.
    let face = crate::race::course_places::face_number(ctx.reads.course(), face)?;
    face_corners(ctx.scene, ctx.view, &face);
    if face.switches() & rc2k_core::race::faces::REPEATED != 0 {
        face_corners_to_new_points(ctx)?;
    }
    if face_out_of_view(ctx)? || face_off_screen(ctx)? {
        return Ok(());
    }
    face_sides(ctx, &face)?;
    draw_face_sides(ctx, &face)
}

/// Draws face `face` (`draw_course_face`) with `polygon_colour` times its object's tint
/// (`object_tints[+1]`, when not 0; byte by byte (a x b) >> 8); the colour is put back after.
pub fn draw_tinted_face(ctx: &DrawCtx<'_>, face: u32) -> Result<(), Refused> {
    use rc2k_core::race::face_drawing::{tinted, tints};
    let colour: u32 = ctx.view.polygon.polygon_colour.get();
    let object = crate::race::course_places::face_field::<u8>(ctx.reads.course(), face, 1)? as u32;
    // `object` is a byte: inside the 256 tints.
    let tint = ctx.reads.stage_prefs.object_tints[object as usize].get();
    if tints(tint) {
        ctx.view.polygon.polygon_colour.set(tinted(tint, colour));
    }
    let done = draw_course_face(ctx, face);
    ctx.view.polygon.polygon_colour.set(colour);
    done
}

#[cfg(test)]
mod reference_vectors {
    use super::*;
    use crate::race::asm_test::{Rng, at_each_control_word};
    use crate::test_support::vectors::{Recorder, word};

    // The recorded block.

    #[test]
    fn frame_of_matches() {
        let mut r = Rng(12);
        let mut file = Recorder::replay("race.course_faces.frame_of");
        at_each_control_word(|| {
            for _ in 0..20000 {
                let (t, f, a, b) = (r.int() as u32, r.float(), r.float(), r.float());
                for w in [t, f, a, b] {
                    file.input(&w.to_le_bytes());
                }
                assert_eq!(
                    word(&mut file),
                    frame_of(t, f, a, b),
                    "{t} {f:#x} {a:#x} {b:#x}"
                );
            }
        });
        file.finish();
    }
}

/// The blocks that read the race's state against the recorded outputs
/// (`test_support::race_vectors`).
#[cfg(test)]
mod vector_tests {
    use super::*;
    use crate::race::asm_test::{Rng, at_each_control_word, same};
    use crate::test_support::own::race;
    use crate::test_support::race_vectors::{Kind, Vectors};
    use crate::test_support::state::Snapshot;

    #[test]
    fn face_turned_to_view_is_the_originals() {
        let race = race();
        let kept = Snapshot::of(&[
            "owners.race.shared.switches",
            "owners.race.course.course",
            "owners.race.frame.view_and_records",
        ]);
        let mut vectors = Vectors::open("race.course_faces.face_turned_to_view", Kind::Exact, 1);
        let mut r = Rng(11);
        let mut points = [0u32; 12];
        let mut face = [0u32; 0x20];
        let place = &race.frame.view_and_records.get().view_vars.view_place_f;
        at_each_control_word(|| {
            for _ in 0..20000 {
                race.shared.switches.get().scene_set_up.set(r.below(8));
                for cell in place {
                    cell.set(f32::from_bits(r.float()));
                }
                for q in points.iter_mut() {
                    *q = r.int() as u32;
                }
                for q in face.iter_mut() {
                    *q = r.float();
                }
                let half = if r.below(2) == 0 { 0 } else { 4 };
                face[(half + 8) / 4] = r.below(4);
                let got = crate::race::scene_points::with_test_points(
                    &race.course.stage.course,
                    &points,
                    || face_turned_to_view(race, &face, half as u32).unwrap(),
                );
                vectors.check(&[got], |a, b| a == b, || format!("{face:x?} {points:x?}"));
            }
        });
        kept.restore();
        vectors.finish();
    }

    fn vertex(r: &mut Rng) -> [u32; 8] {
        let mut v = [0xAAAA_AAAAu32; 8];
        for q in v.iter_mut().skip(4).take(3) {
            *q = r.float();
        }
        v
    }

    #[test]
    fn halfway_vertex_is_the_originals() {
        let drawing = &crate::test_support::own::scene().drawing;
        let course = &race().course;
        let kept = Snapshot::of(&[
            "owners.scene.drawing.scene",
            "owners.race.course.course",
            "owners.race.course.stage_prefs",
            "owners.race.course.numbers",
        ]);
        let mut vectors = Vectors::open("race.course_faces.halfway_vertex", Kind::Exact, 8);
        let mut r = Rng(13);
        let win = crate::scene::view_window::ViewWindow::new();
        at_each_control_word(|| {
            for n in 0..20000 {
                course.stage_prefs.get().flat_depth.set((n % 2) as u32);
                course.course.get().flat_depth.set(r.int() as u32);
                // The recorded run read the scene's window; this code reads the window it is given.
                for (old, own) in [
                    (&drawing.scene.get().view_scale, &win.view_scale),
                    (&drawing.scene.get().view_centre[1], &win.view_centre[1]),
                    (&drawing.scene.get().view_centre[0], &win.view_centre[0]),
                ] {
                    let value = f32::from_bits(r.float());
                    old.set(value);
                    own.set(value);
                }
                course.numbers.get().half.set(if r.below(2) == 0 {
                    0.5
                } else {
                    f32::from_bits(r.float())
                });
                // mostly a depth the projection takes
                course.numbers.get().k4096_a.set(if r.below(4) == 0 {
                    f32::from_bits(r.float())
                } else {
                    4096.0
                });
                let (a, b) = (vertex(&mut r), vertex(&mut r));
                let from = vertex(&mut r);
                let mut got = from;
                halfway_record(course.reads(), &a, &b, &mut got, &win);
                // The depth's store to the scene's scratch is not kept.
                vectors.check(&got, same, || format!("{a:x?} {b:x?} {from:x?}"));
            }
        });
        kept.restore();
        vectors.finish();
    }

    #[test]
    fn halfway_entry_is_the_originals() {
        let course = &race().course;
        let kept = Snapshot::of(&["owners.race.course.numbers"]);
        let mut vectors = Vectors::open("race.course_faces.halfway_entry", Kind::Exact, 10);
        let mut r = Rng(14);
        at_each_control_word(|| {
            for _ in 0..20000 {
                course.numbers.get().half.set(if r.below(2) == 0 {
                    0.5
                } else {
                    f32::from_bits(r.float())
                });
                let entry = |r: &mut Rng| {
                    let e = crate::memory::cells::zeroed_cells::<PolygonEntry>();
                    for k in 0..10 {
                        e.set_word(
                            4 * k,
                            if (2..6).contains(&k) {
                                r.float()
                            } else {
                                r.u32()
                            },
                        );
                    }
                    e
                };
                let (f, s) = (entry(&mut r), entry(&mut r));
                let to = crate::memory::cells::zeroed_cells::<PolygonEntry>();
                for k in 0..10 {
                    to.set_word(4 * k, 0xAAAA_AAAA);
                }
                halfway_entry(course.reads(), &f, &s, &to);
                let got: [u32; 10] = std::array::from_fn(|k| to.word(4 * k));
                // The floats (words 2 .. 6) compare as floats, the bytes as they are.
                vectors.check(&got, same, String::new);
            }
        });
        kept.restore();
        vectors.finish();
    }
}
