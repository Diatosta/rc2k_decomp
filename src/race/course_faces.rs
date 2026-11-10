//! The course's faces drawn (`race::draw_tinted_face` -> 0x4537E3 -> `race::draw_course_face`, part
//! R1, IDA 2026-10-18). A face record is 0x40 bytes at `g_courseFaces` (+0 the kind and flags, +1
//! the object, +4 .. +0x10 four scene points, +0x18 / +0x30 / +0x38 and +0x1C / +0x34 /
//! +0x3C its two triangles' normals, by the reading of 0x4525F8, +0x20 / +0x21 the
//! second's flags and object). Here its leaves.

use crate::errors::{BadInput, Refused};
use crate::pointers::GamePtr;
use crate::race::face_work::FaceWork;
use crate::race::owner::Race;
use crate::race::owner_course::Course;
use crate::race::owner_shared::Shared;
use crate::race::trap::Trap;
use crate::race::view_scratch::DrawCtx;
use crate::scene::polygon_store::{POLYGON_WORK, PolygonEntry};
use crate::session::byte_of;
use crate::session::{Dword, Float};
use rc2k_core::race::faces::Face;

#[cfg(all(test, target_arch = "x86"))]
fn get(a: u32) -> Result<u32, BadInput> {
    crate::pointers::GamePtr::new(a).read("a test's face, point or entry word")
}

/// The byte `n` of a byte table held as dwords; past it refused.
fn table_byte(
    cells: &[crate::session::Dword],
    n: u32,
    what: &'static str,
) -> Result<u32, BadInput> {
    byte_of(cells, n as usize)
        .map(u32::from)
        .ok_or(BadInput(what))
}

/// `race::face_depth_test_off` (0x452358): with no texture animation (`dword_63DA2C` 0)
/// and `g_polygonMode` 4, the depth test off and `dword_63D9E0` 1.
pub fn face_depth_test_off(face: &FaceWork) {
    {
        // Converted: `rc2k_core::race::faces::depth_test_off`.
        if !rc2k_core::race::faces::depth_test_off(face.texture_a.bits(), face.polygon_mode.bits())
        {
            return;
        }
        crate::scene::depth_test_off::depth_test_off();
        face.depth_test.set(1u32);
    }
}

/// `race::face_depth_test_on` (0x45237A): depth writing back on when `face_depth_test_off`
/// turned it off (`dword_63D9E0` set), then `dword_63D9E0` 0. Fixed: the original's test is
/// the wrong way round (on only when it was not turned off), `wiki/Fixed-bugs.md`.
pub fn face_depth_test_on(face: &FaceWork) {
    {
        // Converted: `rc2k_core::race::faces::depth_test_on`.
        if rc2k_core::race::faces::depth_test_on(face.depth_test.bits()) {
            crate::scene::depth_test_on::depth_test_on();
        }
        face.depth_test.set(0u32);
    }
}

/// `race::face_corners` (0x452444): face esi's four points into `dword_63D9EC` ..
/// `dword_63D9F8`, the fourth -1 when it is the third (a triangle). Leaves eax the fourth.
pub fn face_corners(view: &crate::race::view_scratch::ViewScratch, face: &Face) -> u32 {
    // Converted: `rc2k_core::race::faces::corners`.
    let corners = rc2k_core::race::faces::corners(std::array::from_fn(|n| face.point(n)));
    crate::session::write_bits(&view.face.corners, corners);
    // `race::norms` (the next stage's start-up) reads the last face drawn's.
    crate::race::face_work::note_face_drawn(view.number());
    corners[3]
}

/// 0x4525F8: 1 when the face's triangle at `face + half` (half 0 or 4) faces the view
/// (its point +8 less the view's place, `flt_6062A4` .., dotted with its normal +0x18,
/// +0x30, +0x38 above 0, or not a number; the original's x87 run), 0 outside a race.
#[cfg(all(test, target_arch = "x86"))]
fn face_turned_to_view(race: &Race, face: u32, half: u32) -> Result<u32, BadInput> {
    let bytes = crate::pointers::GamePtr::new(face).read::<[u8; 64]>("a test's face")?;
    turned_to_view(
        race,
        &Face(bytes),
        (half / 4) as usize,
        &race.frame.view_and_records.view_vars.view_place_f,
    )
}

/// 0x4525F8 on a read face, its triangle 0 or 1.
/// `place`: the view's place as floats (`ViewScratch::vars`).
fn turned_to_view(
    race: &Race,
    f: &Face,
    triangle: usize,
    place: &[Float; 3],
) -> Result<u32, BadInput> {
    use crate::math::x87_float::{X, compare, int, ld};
    {
        if race.shared.switches.other_way.get() == 0 {
            return Ok(0);
        }
        let p = f.point(1 + triangle);
        let normal = f.normal(triangle);
        let at = crate::race::scene_points::point_number(p)?;
        let along = |k: usize, view_at: &Float, normal: u32| -> Result<_, BadInput> {
            Ok((int(at[k] as i32) - ld(view_at.bits())) * ld(normal))
        };
        let (x, y, z) = (
            along(0, &place[0], normal[0])?,
            along(1, &place[1], normal[1])?,
            along(2, &place[2], normal[2])?,
        );
        Ok(u32::from(compare(X(0.0), x + (y + z)) & 0x0100 != 0))
    }
}

/// `race::face_sides` (0x4525BE): which of face `face`'s (esi) triangles face the view:
/// `dword_63D9E4` the first's (0x4525F8), `dword_63D9E8` 1, or the second's when it
/// has one (`dword_63D9F8` not -1). The registers the original leaves (eax the last
/// answer, ebx the last triangle's point, ebp the face) are read by no one: its one
/// caller's parts take only esi, the face.
pub fn face_sides(ctx: &DrawCtx<'_>, face: &Face) -> Result<(), BadInput> {
    let (race, work) = (ctx.race, &ctx.view.face);
    {
        work.turned[0].set(turned_to_view(race, face, 0, &ctx.view.vars.view_place_f)?);
        work.turned[1].set(1u32);
        if work.corners[3].get() != u32::MAX {
            work.turned[1].set(turned_to_view(race, face, 1, &ctx.view.vars.view_place_f)?);
        }
    }
    Ok(())
}

/// `fild t; fmul f; fmul a; fmul b; fistp`.
fn frame_of(t: u32, f: u32, a: u32, b: u32) -> u32 {
    use crate::math::x87_float::{int, ld, to_int};
    to_int(int(t as i32) * ld(f) * ld(a) * ld(b)) as u32
}

/// `race::face_materials` (0x45265C): face `face`'s two triangles' textures: for each, its
/// object (+1, +0x21) animated when `byte_609C4C` gives it a speed (car 0's +8 clock times
/// `g_faceKeyFactors[speed]`, `flt_5C5D54`, `flt_5C5DA4`, rounded, & 7 picks the frame of
/// `byte_609384`; the first's frame also to `dword_63DA08`), into `dword_63DA2C` /
/// `dword_63DA38`; the kind's low three bits and bit 3 (+0, +0x20) into `dword_63DA30` /
/// `dword_63DA34` and `dword_63DA3C` / `dword_63DA40`. The registers it leaves are read
/// by no one.
pub fn face_materials(ctx: &DrawCtx<'_>, face: &Face) -> Result<(), BadInput> {
    let (race, work) = (ctx.race, &ctx.view.face);
    let course = &race.course;
    let shared = &race.shared;
    {
        let clock: u32 = crate::race::frame_view::car(0)?.step_clock.bits();
        let (k1, k2) = (course.numbers.ten.bits(), course.numbers.quarter.bits());
        let texture = |object: u32, first: bool| -> Result<u32, BadInput> {
            let speed = table_byte(&course.stage_prefs.face_speeds, object, "the face speeds")?;
            if speed == 0 {
                return Ok(object);
            }
            let n = frame_of(
                clock,
                course.lists.face_key_factors[speed as usize].bits(),
                k1,
                k2,
            );
            let (frame, place) = rc2k_core::race::faces::frame_place(object, n);
            if first {
                shared.view.face_frame.set(frame);
            }
            table_byte(&course.stage_prefs.face_places, place, "the face places")
        };
        // Converted: `rc2k_core::race::faces::{frame_place, material_bits}` (the frame's
        // rounding the x87's).
        use rc2k_core::race::faces::material_bits;
        work.texture_a.set(texture(face.object() as u32, true)?);
        let (order, back) = material_bits(face.kind());
        work.order_a.set(order);
        work.back_a.set(back);
        work.texture_b
            .set(texture(face.second_object() as u32, false)?);
        let (order, back) = material_bits(face.second_kind());
        work.order_b.set(order);
        work.back_b.set(back);
    }
    Ok(())
}

/// `race::draw_marked_face` (0x44CD90): `scene::scale_texture_coordinates`,
/// `scene::fog_polygon`, the drawing (`scene::draw_polygon`; the original through
/// `scene::draw_marked_polygon`, whose mark nothing reads: dropped), `g_polygonDrawn` 1.
pub fn draw_marked_face(ctx: &DrawCtx<'_>) -> Result<(), Refused> {
    let (race, scene) = (ctx.race, ctx.scene);
    crate::scene::scale_texture_coordinates::scale_texture_coordinates(&ctx.view.polygon)?;
    crate::scene::fog_polygon::fog_polygon(ctx.view)?;
    crate::scene::draw_polygon::draw_polygon_in_game(
        &ctx.view.polygon,
        race,
        &ctx.view.face,
        scene,
    )?;
    ctx.view.polygon.polygon_drawn.set(1u32);
    Ok(())
}

/// `race::face_switches` (0x45322C): `dword_6240E0` / `dword_6240E4` 1 when the face's
/// first / second texture is animated (`dword_63DA2C` / `dword_63DA38` not 0), else 0.
pub fn face_switches(poly: &crate::scene::polygon_work::PolygonWork, work: &FaceWork) {
    // Converted: `rc2k_core::race::faces::switches`.
    {
        let [first, second] =
            rc2k_core::race::faces::switches([work.texture_a.bits(), work.texture_b.bits()]);
        poly.polygon_sides[0].set(first);
        poly.polygon_sides[1].set(second);
    }
}

/// 0x453786: the light at scene point `n` (`race::light_colour_at`; the original's store of
/// it to `dword_62400C` is not kept), 0 for -1.
fn point_light(ctx: &DrawCtx<'_>, n: u32) -> Result<u32, Refused> {
    let (race, scene) = (ctx.race, ctx.scene);
    if n == u32::MAX {
        return Ok(0);
    }
    let [x, y, z] = crate::race::scene_points::point_number(n)?;
    ctx.view.light.lit_place[0].set(x);
    ctx.view.light.lit_place[1].set(y);
    ctx.view.light.lit_place[2].set(z);
    crate::race::frame_draw::light_colour_at(
        race,
        scene,
        &ctx.view.lamps,
        &ctx.view.light,
        ctx.view.drawn_view(race),
    )?;
    let light: u32 = ctx.view.light.light_colour.get();
    Ok(light)
}

/// `race::face_lights` (0x453715): the light at each of the face's four corners
/// (`dword_63DA54` .. `dword_63DA60`; 0 for a missing fourth), when lamps are wanted
/// (`g_lampsWanted`) and the face is not a plain mode-4 one; all 0 otherwise. Refuses
/// (before anything) what `light_colour_at` refuses.
pub fn face_lights(ctx: &DrawCtx<'_>) -> Result<(), Refused> {
    let race = ctx.race;
    // Converted: `rc2k_core::race::faces::corners_lit`.
    let wanted: u32 = ctx.view.lamps.lamps_wanted.get();
    let lit = wanted != 0
        && rc2k_core::race::faces::corners_lit(
            wanted,
            ctx.view.face.texture_a.bits(),
            ctx.view.face.polygon_mode.bits(),
        );
    if lit {
        crate::race::frame_draw::check_light_colour(&race.shared, &ctx.view.lamps)?;
    }
    for k in 0..4 {
        ctx.view.face.corner_lights[k].set(0u32);
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

/// `race::clip_and_draw_face` (0x454187): the face's polygon cut by depth (when some
/// vertices are outside, `g_polygonOutsideCount`) and to the screen, and with three
/// vertices or more drawn (`scene::draw_polygon_in_game`), `g_polygonDrawn` 1.
pub fn clip_and_draw_face(ctx: &DrawCtx<'_>) -> Result<(), Refused> {
    let (race, scene) = (ctx.race, ctx.scene);
    // Converted: `rc2k_core::race::face_drawing::draws_polygon`.
    use rc2k_core::race::face_drawing::draws_polygon;
    if ctx.view.polygon.polygon_outside_count.get() != 0 {
        crate::scene::clip_polygon_by_depth::clip_polygon_by_depth(
            &ctx.view.polygon,
            race,
            scene,
            ctx.far_clip,
            ctx.view,
        )?;
        if !draws_polygon(ctx.view.polygon.polygon_vertex_count.bits()) {
            return Ok(());
        }
    }
    crate::scene::clip_polygon_to_screen::clip_polygon_to_screen(&ctx.view.polygon, race)?;
    if !draws_polygon(ctx.view.polygon.polygon_vertex_count.bits()) {
        return Ok(());
    }
    crate::scene::draw_polygon::draw_polygon_in_game(
        &ctx.view.polygon,
        race,
        &ctx.view.face,
        scene,
    )?;
    ctx.view.polygon.polygon_drawn.set(1u32);
    Ok(())
}

/// `race::face_out_of_view` (0x45246F): true (the original's carry) when none of the
/// face's corners (`dword_63D9EC` ..; the fourth when there is one) is in sight: a corner
/// is in sight when its scene record's depth (+0x18) is not negative and |x| (+0x10) is
/// below `flt_63DA00` (the original's store of |x| to `dword_62400C` is not kept) or |x| times `flt_63D9FC` is below
/// the depth. The records are made when not there (as 0x44CF33). The eax and edx the
/// original leaves are read by no one. Refuses (before anything) no point records or
/// points.
pub fn face_out_of_view(ctx: &DrawCtx<'_>) -> Result<bool, Refused> {
    let race = ctx.race;
    let course = &race.course;
    let shared = &race.shared;
    // Converted: `rc2k_core::race::face_drawing::corner_in_sight` (exact in `f64`).
    use rc2k_core::race::face_drawing::{InSight, corner_in_sight};
    crate::race::frame_draw::points_there(course)?;
    let near: u32 = shared.view.near.bits();
    let slope: u32 = shared.view.slope.bits();
    for k in 0..4 {
        let n: u32 = ctx.view.face.corners[k].get();
        if k == 3 && n == u32::MAX {
            break;
        }
        let record = crate::race::frame_draw::point_record(ctx, n)?;
        let depth = crate::scene::point_records::field(record, 0x18)?;
        if depth & 0x8000_0000 != 0 {
            continue;
        }
        let x = crate::scene::point_records::field(record, 0x10)?;
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

/// Whether the face leaves nothing on the screen, its corners all projected and past one
/// edge (`rc2k_core::race::face_drawing::off_screen`; not the original's, which cuts each of
/// its polygons away at the end: the same picture, without the face's colours, lights and
/// polygons made for nothing). The corners' records are there: `face_out_of_view` made them.
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
        let at = |offset| crate::scene::point_records::field(record, offset);
        corners[n] = ScreenCorner {
            depth: at(0x18)?,
            x: at(0xC)?,
            y: at(0x1C)?,
        };
        n += 1;
    }
    let clip = &ctx.view.polygon.window.clip_screen;
    let edges = ScreenEdges {
        top: clip[0].bits(),
        bottom: clip[1].bits(),
        left: clip[2].bits(),
        right: clip[3].bits(),
    };
    Ok(off_screen(
        &corners[..n],
        edges,
        crate::scene::project_point::NEAR,
    ))
}

/// Where one of the face's two triangles keeps its texture's working values
/// (`race::face_texture_coordinates` for the first, 0x45349F for the second: the same code on other
/// variables).
pub struct TextureSet<'a> {
    /// The texture (`dword_63DA2C` / `dword_63DA38`).
    pub texture: &'a Dword,
    /// The kind's low bits, the corner order (`dword_63DA30` / `dword_63DA3C`).
    pub order: &'a Dword,
    /// The texture's size word with bit 31 (`dword_6240BC` / `dword_6240C0`).
    pub size: &'a Dword,
    /// The texture's mapping mode, its +0x2D signed (`dword_623DD8` / `dword_623DE0`).
    pub mapping: &'a Dword,
    /// 1 when the mapping is 0 (`dword_63E168` / `dword_63E16C`).
    pub plain: &'a Dword,
    /// The four corners' u, v (`dword_63DA0C` .. / `dword_63DAB8` ..).
    pub uv: &'a [Dword; 8],
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

/// The `N` bytes (at most 4) at `offset` of texture `texture`'s info record: the records
/// lie one after another, and the original's sums of addresses may leave the record, or
/// the records (then read by address, checked).
fn info_bytes<const N: usize>(texture: u32, offset: u32) -> Result<[u8; N], BadInput> {
    let infos = &crate::scene::polygon_store::POLYGON_TABLES.texture_infos;
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
    // A run-on past the infos, as the original (a byte offset into the info words).
    GamePtr::to(&infos[0].words)
        .unbounded()
        .offset(at as u32)
        .read("a texture info's bytes")
}

/// `race::face_texture_coordinates` (0x453267) / `race::face_texture_coordinates_2`
/// (0x45349F): the triangle's texture's size word (+0x20, bit 31 set) and mapping (+0x2D);
/// the four corners' u, v from the texture's corner table for the order
/// (`dword_63DAD8[order * 4 + k]` points at a u16 pair inside the texture record, << 8,
/// doubled for sizes 0x190 .. 0x31F); with a mapping above 0 the corners' scene x and z
/// shifted by (mapping & 3) + 2 instead, less the first corner's top byte (the fourth
/// corner read even when it is -1: `notes/bugs.md`). The registers the original leaves
/// (eax the size word, ebx the texture record) are read by no one.
pub fn face_texture_coordinates(
    shared: &Shared,
    work: &FaceWork,
    set: &TextureSet<'_>,
) -> Result<(), BadInput> {
    // Converted: `rc2k_core::race::faces::texture_coordinates`.
    use rc2k_core::race::faces::{TextureInfo, texture_coordinates};
    {
        let texture: u32 = set.texture.get();
        let texture_info = TextureInfo {
            size: u32::from_le_bytes(info_bytes(texture, 0x20)?),
            mapping: i8::from_le_bytes(info_bytes(texture, 0x2D)?),
        };
        let order: u32 = set.order.get();
        let mut pairs = [(0u16, 0u16); 4];
        for (k, pair_out) in pairs.iter_mut().enumerate() {
            let table = shared.view.corner_uv_offsets[(order << 2) as usize + k].get();
            *pair_out = (
                u16::from_le_bytes(info_bytes(texture, table)?),
                u16::from_le_bytes(info_bytes(texture, table.wrapping_add(2))?),
            );
        }
        let trap = Trap::new();
        let c = texture_coordinates(texture_info, pairs, |k| {
            let n: u32 = work.corners[k].get();
            crate::race::scene_points::point_xz(n).unwrap_or_else(|e| {
                trap.keep(e);
                (0, 0)
            })
        });
        trap.finish()?;
        set.size.set(c.size);
        set.mapping.set(c.mapping);
        set.plain.set(c.plain);
        crate::session::write_bits(set.uv, c.uv);
    }
    Ok(())
}

/// `race::face_corners_to_new_points` (0x452722): the face's corners copied to the four
/// scene points after the last (`g_scenePointCount` + 0 .. 3, their records marked unmade,
/// all four), the corners then naming the copies (the fourth only when there is one); the
/// count is left. Refuses (before anything) no point records or points.
pub fn face_corners_to_new_points(ctx: &DrawCtx<'_>) -> Result<(), Refused> {
    let (race, work) = (ctx.race, &ctx.view.face);
    let course = &race.course;
    // Converted: `rc2k_core::race::faces::new_points`.
    {
        crate::race::frame_draw::points_there(course)?;
        let count: u32 = course.course.scene_point_count.get();
        let corners: [u32; 4] = crate::session::read_bits(&work.corners);
        let (marked, copies) = rc2k_core::race::faces::new_points(count, corners);
        for n in marked {
            crate::scene::point_records::put_field(
                crate::scene::point_records::at_offset(n << 5),
                0,
                u32::MAX,
            )?;
        }
        for (k, (from, to)) in copies.into_iter().enumerate() {
            crate::race::scene_points::copy_point(course, from, to)?;
            work.corners[k].set(to);
        }
        Ok(())
    }
}

/// The work table's entry `slot`.
fn entry(slot: u32) -> &'static PolygonEntry {
    &POLYGON_WORK.entries[slot as usize]
}

/// One polygon entry filled from one of the face's corners: +0 its point, +8 / +0xC the
/// first texture's u, v, +0x10 / +0x14 the second's, +0x18 its colour, +0x1C its light,
/// +0x21 its fog byte (`dword_63DA64` ..).
fn fill_entry(work: &FaceWork, slot: u32, corner: u32) -> Result<(), BadInput> {
    {
        let e = entry(slot);
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
fn lamp_byte(slot: u32) -> Result<u8, BadInput> {
    Ok(entry(slot).byte(0x1E))
}

/// `g_polygonList` 0 .. count - 1 and a closing 0, `g_polygonVertexCount` count,
/// `g_polygonNextEntry` `next`.
fn plain_list(poly: &crate::scene::polygon_work::PolygonWork, count: u32, next: u32) {
    {
        poly.polygon_vertex_count.set(count);
        for k in 0..count {
            crate::race::checkpoints_store::cell_in(&POLYGON_WORK.list, k as usize).set(k);
        }
        crate::race::checkpoints_store::cell_in(&POLYGON_WORK.list, count as usize).set(0);
        poly.polygon_next_entry.set(next);
    }
}

/// `race::draw_whole_quad` (0x452F3D): the face drawn as one quad when it can be: the
/// entries 1, 2, 3, 0 from corners 0, 1, 3, 2; carry (draw it as triangles) when a corner
/// has lamp light or some vertex is outside after `scene::project_polygon` or off the
/// screen's inside (the screen edges `dword_6241D4` .. less one; each vertex's x and y
/// rounded, compared by the sign of the difference); nothing when
/// all are outside; else the quad drawn (`race::draw_marked_face`). Gives the carry
/// (true: draw it as triangles).
pub fn draw_whole_quad(ctx: &DrawCtx<'_>) -> Result<bool, Refused> {
    let (race, scene) = (ctx.race, ctx.scene);
    // Converted: `rc2k_core::race::face_drawing::{QUAD_ENTRIES, quad_by_outside,
    // screen_inside, inside}` (the roundings the x87's).
    use rc2k_core::race::face_drawing::{QUAD_ENTRIES, quad_by_outside, screen_inside};
    for (slot, corner) in QUAD_ENTRIES {
        fill_entry(&ctx.view.face, slot, corner)?;
    }
    for s in 0..4 {
        if lamp_byte(s)? != 0 {
            return Ok(true);
        }
    }
    plain_list(&ctx.view.polygon, 4, 4);
    crate::scene::project_polygon::project_polygon(&ctx.view.polygon, race, scene, ctx.view)?;
    if let Some(triangles) = quad_by_outside(ctx.view.polygon.polygon_outside_count.bits()) {
        return Ok(triangles);
    }
    let [left, right, top, bottom] = screen_inside(
        ctx.view.polygon.window.clip_edges[2].bits(),
        ctx.view.polygon.window.clip_edges[3].bits(),
        ctx.view.polygon.window.clip_edges[0].bits(),
        ctx.view.polygon.window.clip_edges[1].bits(),
    );
    let inside = |v: u32, low: u32, high: u32| {
        let n = crate::math::x87::to_int(v) as u32;
        rc2k_core::race::face_drawing::inside(n, low, high)
    };
    for slot in 0..4 {
        let v = entry(slot).record.get();
        if !inside(
            crate::scene::point_records::vertex(&ctx.view.polygon.quad_vertices, v, 0xC),
            left,
            right,
        ) || !inside(
            crate::scene::point_records::vertex(&ctx.view.polygon.quad_vertices, v, 0x1C),
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

/// `race::draw_face_triangles` (0x452CF3): the face's two triangles each drawn
/// (`race::draw_lit_triangle`) unless facing away: the first (corners 0, 1, 2) when
/// `dword_63D9E4` is 0, the second (3, 2, 1) when there is one and `dword_63D9E8` is 0.
pub fn draw_face_triangles(ctx: &DrawCtx<'_>) -> Result<(), Refused> {
    // Converted: `rc2k_core::race::face_drawing::{draws_first, draws_second,
    // FIRST_TRIANGLE, SECOND_TRIANGLE}`.
    use rc2k_core::race::face_drawing::{
        FIRST_TRIANGLE, SECOND_TRIANGLE, draws_first, draws_second,
    };
    if draws_first(ctx.view.face.turned[0].bits()) {
        for (slot, corner) in FIRST_TRIANGLE {
            fill_entry(&ctx.view.face, slot, corner)?;
        }
        ctx.view.polygon.polygon_vertex_count.set(3u32);
        draw_lit_triangle(ctx)?;
    }
    let fourth: u32 = ctx.view.face.corners[3].get();
    if fourth != u32::MAX && draws_second(fourth, ctx.view.face.turned[1].bits()) {
        for (slot, corner) in SECOND_TRIANGLE {
            fill_entry(&ctx.view.face, slot, corner)?;
        }
        ctx.view.polygon.polygon_vertex_count.set(3u32);
        draw_lit_triangle(ctx)?;
    }
    Ok(())
}

/// `race::draw_face_polygons` (0x452CCF): in a race with both triangles facing the view
/// (`dword_63D9E4` + `dword_63D9E8` 0) the face as one quad (`race::draw_whole_quad`, its
/// carry asking for triangles), else as triangles (`race::draw_face_triangles`).
pub fn draw_face_polygons(ctx: &DrawCtx<'_>) -> Result<(), Refused> {
    let race = ctx.race;
    // Converted: `rc2k_core::race::face_drawing::tries_quad`.
    let in_race: u32 = race.shared.switches.other_way.get();
    if in_race != 0
        && rc2k_core::race::face_drawing::tries_quad(
            in_race,
            ctx.view.face.turned[0].bits(),
            ctx.view.face.turned[1].bits(),
        )
        && !draw_whole_quad(ctx)?
    {
        return Ok(());
    }
    draw_face_triangles(ctx)
}

/// `race::draw_face` (0x452C6F): the face's textures (`race::face_switches`, the first's
/// coordinates when animated; `g_polygonMode` 2, or with a second texture the mode its
/// record's +0x2C picks in `dword_63D9D0` and the second's coordinates), the corners'
/// lights (`race::face_lights`), then the depth test off, the polygons
/// (`race::draw_face_polygons`), the depth test on. The original keeps esi (the face);
/// the rest it leaves no caller reads. Refuses (before anything) what `face_lights`
/// refuses.
pub fn draw_face(ctx: &DrawCtx<'_>) -> Result<(), Refused> {
    let race = ctx.race;
    // Converted: `rc2k_core::race::face_drawing::{face_mode, PLAIN_MODE}`, the lights'
    // test `rc2k_core::race::faces::corners_lit`.
    use rc2k_core::race::face_drawing::{PLAIN_MODE, face_mode};
    let second: u32 = ctx.view.face.texture_b.get();
    let trap = Trap::new();
    let picked = || {
        // A run-on: the picked info number is not bounded, as the original.
        let record =
            GamePtr::to(&crate::scene::polygon_store::POLYGON_TABLES.texture_infos[0].words)
                .unbounded()
                .offset(second << 6);
        let pick = trap.read_at::<u8>(record.offset(0x2C), "a texture info's kind") as usize;
        // dword_63D9D0: indexed by a byte (not bounded by the field's 4 dwords), so
        // read by address.
        trap.read_at::<u32>(
            GamePtr::to(&race.shared.view.surface_kinds)
                .unbounded()
                .offset((4 * pick) as u32),
            "the surface kinds table",
        )
    };
    let mode = face_mode(second, picked);
    trap.finish()?;
    let wanted: u32 = ctx.view.lamps.lamps_wanted.get();
    if wanted != 0
        && rc2k_core::race::faces::corners_lit(wanted, ctx.view.face.texture_a.bits(), mode)
    {
        crate::race::frame_draw::check_light_colour(&race.shared, &ctx.view.lamps)?;
    }
    face_switches(&ctx.view.polygon, &ctx.view.face);
    if ctx.view.face.texture_a.get() != 0 {
        face_texture_coordinates(&race.shared, &ctx.view.face, &first_set(ctx))?;
    }
    ctx.view.face.polygon_mode.set(PLAIN_MODE);
    if second != 0 {
        ctx.view.face.polygon_mode.set(face_mode(second, picked));
        trap.finish()?;
        face_texture_coordinates(&race.shared, &ctx.view.face, &second_set(ctx))?;
    }
    face_lights(ctx)?;
    face_depth_test_off(&ctx.view.face);
    draw_face_polygons(ctx)?;
    face_depth_test_on(&ctx.view.face);
    Ok(())
}

/// `race::draw_lit_triangle`'s new vertex between two entries' vertices `a` and `b` into `v`: the
/// view floats (+0x10 .. +0x18) halfway (b - a) x `flt_5C5E40` + a, the screen place
/// (+0xC, +0x1C) 0, or when the depth is not below `flt_5C5E18` projected (by
/// `flt_624088` over the depth, or over `dword_5CCD80` + 1 when `dword_60932C` is set,
/// the original's store of the depth to `dword_62400C` is not kept; centre `flt_62409C`, `flt_6240A0`).
fn halfway_vertex(
    course: &Course,
    quad: &[crate::session::Dword; 32],
    a: u32,
    b: u32,
    v: u32,
    window: &crate::scene::view_window::ViewWindow,
) -> Result<(), BadInput> {
    use crate::scene::point_records::{get_in, set_in};
    let quad = Some(quad);
    let mut made = get_in(quad, v)?;
    halfway_record(
        course,
        &get_in(quad, a)?,
        &get_in(quad, b)?,
        &mut made,
        window,
    );
    set_in(quad, v, made)
}

/// `halfway_vertex` on the records themselves (`a`, `b` point records of any kind).
fn halfway_record(
    course: &Course,
    a: &crate::scene::point_records::Record,
    b: &crate::scene::point_records::Record,
    v: &mut crate::scene::point_records::Record,
    window: &crate::scene::view_window::ViewWindow,
) {
    use crate::math::x87_float::{ld, st};
    use crate::math::x87_load::below_or_unordered;
    let half = ld(course.numbers.half.bits());
    for k in [4, 5, 6] {
        let (from, to) = (ld(a[k]), ld(b[k]));
        v[k] = st((to - from) * half + from);
    }
    (v[3], v[7]) = (0, 0);
    if below_or_unordered(v[6], course.numbers.k4096_a.bits()) {
        return;
    }
    crate::race::skidmarks::project_record(course, v, window);
}

/// `race::draw_lit_triangle`'s new entry between entries `first` and `second` into `to`: the
/// texture floats (+8 .. +0x14) (second + first) x `flt_5C5E40`, the bytes +0x20, +0x21, +0x18 ..
/// +0x1A the halves of their sums (+0x1B left).
fn halfway_entry(course: &Course, first: &PolygonEntry, second: &PolygonEntry, to: &PolygonEntry) {
    use crate::math::x87_float::{ld, st};
    let half = ld(course.numbers.half.bits());
    for o in [8, 0xC, 0x10, 0x14] {
        let sum = ld(second.word(o)) + ld(first.word(o));
        to.set_word(o, st(sum * half));
    }
    for o in [0x20, 0x21, 0x18, 0x19, 0x1A] {
        let sum = second.byte(o) as u32 + first.byte(o) as u32;
        to.set_byte(o, (sum >> 1) as u8);
    }
}

/// `race::draw_lit_triangle` (0x4537E9): the triangle in entries 0 .. 2. Without lamp
/// light at its corners drawn as it is (`scene::draw_clipped_polygon`). Else
/// (unless `scene::project_polygon` finds it all outside; `g_polygonDrawn` 0, the record
/// pool restarted, `scene::scale_texture_coordinates`, `scene::fog_polygon`) cut in four:
/// for each side (0-1, 1-2, 2-0) a new scene point halfway (vertices 0x623E40 ..; its
/// light by `race::light_colour_at`) and entry (3 .. 5); in mode 5 each entry's fog byte
/// less twice its lamp light (not below 0); each entry's colour plus its light
/// (`add_colours`); then the triangles (0, 3, 5), (1, 4, 3), (2, 5, 4), (3, 4, 5) each
/// clipped and drawn (`race::clip_and_draw_face`). Refuses (before anything) no points,
/// or what `light_colour_at` refuses.
pub fn draw_lit_triangle(ctx: &DrawCtx<'_>) -> Result<(), Refused> {
    let (race, scene) = (ctx.race, ctx.scene);
    use crate::race::frame_draw::add_colours;
    let mut lit = false;
    for s in 0..3 {
        if lamp_byte(s)? != 0 {
            lit = true;
            break;
        }
    }
    if !lit {
        return crate::scene::draw_clipped_polygon::draw_clipped_polygon_in_game(ctx);
    }
    crate::race::frame_draw::points_there(&race.course)?;
    crate::race::frame_draw::check_light_colour(&race.shared, &ctx.view.lamps)?;
    ctx.view.polygon.polygon_drawn.set(0u32);
    ctx.view
        .polygon
        .polygon_next_record
        .set(crate::scene::polygon::pool_start());
    plain_list(
        &ctx.view.polygon,
        ctx.view.polygon.polygon_vertex_count.bits(),
        6,
    );
    crate::scene::project_polygon::project_polygon(&ctx.view.polygon, race, scene, ctx.view)?;
    if ctx.view.polygon.polygon_outside_count.get() == ctx.view.polygon.polygon_vertex_count.get() {
        return Ok(());
    }
    crate::scene::scale_texture_coordinates::scale_texture_coordinates(&ctx.view.polygon)?;
    crate::scene::fog_polygon::fog_polygon(ctx.view)?;
    for k in 0..3 {
        entry(3 + k)
            .record
            .set(crate::scene::point_records::of_quad(k));
    }
    for (k, (first, second)) in [(0u32, 1u32), (1, 2), (2, 0)].into_iter().enumerate() {
        let v = crate::scene::point_records::of_quad(k as u32);
        let pa = entry(first).point.get();
        let pb = entry(second).point.get();
        let (at_a, at_b) = (
            crate::race::scene_points::point_number(pa)?,
            crate::race::scene_points::point_number(pb)?,
        );
        for (n, lit) in ctx.view.light.lit_place.iter().enumerate() {
            let o = 4 * n as u32;
            let (a, b) = (at_a[n], at_b[n]);
            let m = ((b.wrapping_sub(a) as i32) >> 1) as u32;
            let m = m.wrapping_add(a);
            crate::scene::point_records::put_field_in(
                Some(&ctx.view.polygon.quad_vertices),
                v,
                o,
                m,
            )?;
            lit.set(m);
        }
        halfway_vertex(
            &race.course,
            &ctx.view.polygon.quad_vertices,
            entry(first).record.get(),
            entry(second).record.get(),
            v,
            &ctx.view.polygon.window,
        )?;
        let to = entry(3 + k as u32);
        halfway_entry(&race.course, entry(first), entry(second), to);
        crate::race::frame_draw::light_colour_at(
            race,
            scene,
            &ctx.view.lamps,
            &ctx.view.light,
            ctx.view.drawn_view(race),
        )?;
        let light: u32 = ctx.view.light.light_colour.get();
        to.lamp_light.set(light);
    }
    let mode5 = ctx.view.face.polygon_mode.get() == 5;
    for slot in 0..6 {
        let e = entry(slot);
        if mode5 {
            let fog = e.amount.get() as i32 - 2 * lamp_byte(slot)? as i32;
            e.amount.set(fog.max(0) as u8);
        }
        e.light.set(add_colours(e.light.get(), e.lamp_light.get()));
    }
    for list in [[0u32, 3, 5], [1, 4, 3], [2, 5, 4], [3, 4, 5]] {
        for (k, n) in list.iter().enumerate() {
            crate::race::checkpoints_store::cell_in(&POLYGON_WORK.list, k).set(*n);
        }
        POLYGON_WORK.list[3].set(list[0]);
        ctx.view.polygon.polygon_vertex_count.set(3u32);
        ctx.view.polygon.polygon_next_entry.set(6u32);
        clip_and_draw_face(ctx)?;
    }
    Ok(())
}

/// `race::face_colours` (0x452887): the corners' colours from `g_lightTable` by the face's
/// bytes +0x14 .. +0x17, with flag 4 of +2 plus `dword_635518[+0x28 ..]` and
/// `dword_635918[+0x2C ..]` (`add_colours`, MMX or not as the original); the fog bytes
/// +0x22 .. +0x25 into `dword_63DA64` ..; then the face drawn (`race::draw_face`).
pub fn face_colours(ctx: &DrawCtx<'_>, face: &Face) -> Result<(), Refused> {
    let scene = ctx.scene;
    let light_tables = &scene.light_tables;
    // Converted: `rc2k_core::race::lights::{corner_colour, EXTRA_LIGHTS}`.
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
            let c = corner_colour(
                c,
                lit,
                extra,
                crate::race::frame_draw::has_mmx(crate::system::owner::system()),
            );
            ctx.view.face.corner_colours[k].set(c);
        }
    }
    for k in 0..4 {
        ctx.view.face.corner_alphas[k].set(face.corner_fog(k) as u32);
    }
    draw_face(ctx)
}

/// The corners turned over (0 with 1, and 2 with 3 when there is a fourth) and the first
/// texture's order by `byte_63DA7C` (its bit 3 kept): `race::draw_back_face`'s step, done twice.
fn turn_over(ctx: &DrawCtx<'_>) -> Result<(), BadInput> {
    let (shared, work) = (&ctx.race.shared, &ctx.view.face);
    {
        let swap = |a: usize, b: usize| {
            let (x, y) = (work.corners[a].get(), work.corners[b].get());
            work.corners[a].set(y);
            work.corners[b].set(x);
        };
        // Converted: `rc2k_core::race::face_drawing::{turns_last_pair, turned_order}`.
        use rc2k_core::race::face_drawing::{turned_order, turns_last_pair};
        swap(0, 1);
        if turns_last_pair(work.corners[3].bits()) {
            swap(2, 3);
        }
        let order: u32 = work.order_a.get();
        let trap = Trap::new();
        let turned = turned_order(order, |n| {
            // `turned_order` asks 0 ..= 7.
            byte_of(&shared.view.turned_corner_order, n as usize).unwrap_or_else(|| {
                trap.keep(BadInput("past the turned corner order"));
                0
            })
        });
        trap.finish()?;
        work.order_a.set(turned);
    }
    Ok(())
}

/// `race::draw_back_face` (0x452B58): a two-sided face's back (when `dword_63DA34`, the
/// first kind's bit 3): the corners turned over, the colours from `g_lightTable` by
/// +0x28 .. +0x2B and the fog bytes +0x2C .. +0x2F, drawn (`race::draw_face`), the
/// corners turned back.
pub fn draw_back_face(ctx: &DrawCtx<'_>, face: &Face) -> Result<(), Refused> {
    let scene = ctx.scene;
    // Converted: `rc2k_core::race::face_drawing::{has_back, turns_last_pair,
    // turned_order}`.
    if !rc2k_core::race::face_drawing::has_back(ctx.view.face.back_a.bits()) {
        return Ok(());
    }
    turn_over(ctx)?;
    for k in 0..4 {
        ctx.view.face.corner_colours[k]
            .set(scene.light_tables.light_table[face.back_light(k) as usize].get());
    }
    for k in 0..4 {
        ctx.view.face.corner_alphas[k].set(face.back_fog(k) as u32);
    }
    draw_face(ctx)?;
    turn_over(ctx)?;
    Ok(())
}

/// `race::draw_face_sides` (0x452393): outside a race both sides (`race::face_colours`,
/// `race::draw_back_face`) with neither facing away; in a race, a face with objects (+1
/// or +0x21): one-sided (kind bit 3 clear) its front unless both triangles face away;
/// two-sided its front unless both face away, then the facings flipped and its back
/// unless both (now) face away. Each after `race::face_materials`. `face` is esi.
pub fn draw_face_sides(ctx: &DrawCtx<'_>, face: &Face) -> Result<(), Refused> {
    let race = ctx.race;
    // Converted: `rc2k_core::race::face_drawing::{has_objects, two_sided, side_drawn,
    // flipped}`.
    use rc2k_core::race::face_drawing::{flipped, has_objects, side_drawn, two_sided};
    if race.shared.switches.other_way.get() == 0 {
        face_materials(ctx, face)?;
        ctx.view.face.turned[0].set(0u32);
        ctx.view.face.turned[1].set(0u32);
        face_colours(ctx, face)?;
        ctx.view.face.turned[0].set(0u32);
        ctx.view.face.turned[1].set(0u32);
        return draw_back_face(ctx, face);
    }
    let facings = || -> bool {
        side_drawn(
            ctx.view.face.turned[0].bits(),
            ctx.view.face.turned[1].bits(),
        )
    };
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
    ctx.view.face.turned[0].set(flipped(ctx.view.face.turned[0].bits()));
    ctx.view.face.turned[1].set(flipped(ctx.view.face.turned[1].bits()));
    if facings() {
        draw_back_face(ctx, face)?;
    }
    Ok(())
}

/// `race::draw_course_face` (0x45232A): face `face` (esi): its corners
/// (`race::face_corners`; with flag 0x200 of +2 copied to new points,
/// `race::face_corners_to_new_points`), and unless none is in sight
/// (`race::face_out_of_view`) its facings (`race::face_sides`) and its sides
/// (`race::draw_face_sides`). Refuses (before anything) no point records or points.
/// Fixed: the original wraps the sides in `face_depth_test_off` / `on` too, judged by the
/// previous face's texture and mode (this face's are set inside `draw_face_sides`), so a
/// face after a mode-4 face could be drawn without writing depth; each side's own
/// `race::draw_face` does it with its own (`wiki/Fixed-bugs.md`).
pub fn draw_course_face(ctx: &DrawCtx<'_>, face: u32) -> Result<(), Refused> {
    let race = ctx.race;
    crate::race::frame_draw::points_there(&race.course)?;
    // The face read once for its corners, normals, kinds, objects and colours: drawing it
    // writes no face.
    let face = crate::race::course_places::face_number(face, "a course face")?;
    face_corners(ctx.view, &face);
    if face.switches() & rc2k_core::race::faces::REPEATED != 0 {
        face_corners_to_new_points(ctx)?;
    }
    if face_out_of_view(ctx)? || face_off_screen(ctx)? {
        return Ok(());
    }
    face_sides(ctx, &face)?;
    draw_face_sides(ctx, &face)
}

/// `race::draw_tinted_face` (0x44F9F3): face `face` (esi) drawn (0x4537E3, that is
/// `race::draw_course_face`) with `g_polygonColour` times its object's tint
/// (`dword_60A880[+1]`, when not 0; byte by byte (a x b) >> 8, all four with MMX, the low
/// three without), the colour put back after. Of the registers its one caller
/// (`race::draw_list_entry`) gives, the drawing reads only esi.
pub fn draw_tinted_face(ctx: &DrawCtx<'_>, face: u32) -> Result<(), Refused> {
    let race = ctx.race;
    // Converted: `rc2k_core::race::face_drawing::{tints, tinted}`.
    use rc2k_core::race::face_drawing::{tinted, tints};
    let colour: u32 = ctx.view.polygon.polygon_colour.get();
    let object = crate::race::course_places::face_field::<u8>(face, 1, "a course face")? as u32;
    // `object` is a byte: inside the 256 tints.
    let tint = race.course.stage_prefs.object_tints[object as usize].get();
    if tints(tint) {
        ctx.view.polygon.polygon_colour.set(tinted(
            tint,
            colour,
            crate::race::frame_draw::has_mmx(crate::system::owner::system()),
        ));
    }
    let done = draw_course_face(ctx, face);
    ctx.view.polygon.polygon_colour.set(colour);
    done
}

#[cfg(all(test, target_arch = "x86"))]
mod asm_reference {
    use super::*;
    use crate::race::owner::race;

    // The reference blocks read and write the test's own memory plainly.
    fn get(a: u32) -> u32 {
        super::get(a).expect("the test's memory is registered")
    }
    use crate::framework::{game_write, rva};
    use crate::race::asm_test::{Rng, Saved, at_each_control_word, lock, same};

    // The original functions, verbatim.
    pub(super) unsafe fn face_turned_to_view_asm(face: u32, half: u32) -> u32 {
        unsafe {
            if crate::race::owner::race().shared.switches.other_way.get() == 0 {
                return 0;
            }
            let e = face + half;
            let p = get(e + 8)
                .wrapping_mul(12)
                .wrapping_add(race().course.course.scene_points.bits());
            let status: u16;
            let old = crate::race::store::OldPlaces::lay(&[0x6062a4, 0x6062a8, 0x6062ac]);
            core::arch::asm!(
                "fild dword ptr [{p}]",
                "fsub dword ptr [{rc} + 0x2062A4]",
                "fld dword ptr [{e} + 0x18]",
                ".byte 0xDE, 0xC9",
                "fild dword ptr [{p} + 4]",
                "fsub dword ptr [{rc} + 0x2062A8]",
                "fld dword ptr [{e} + 0x30]",
                ".byte 0xDE, 0xC9",
                "fild dword ptr [{p} + 8]",
                "fsub dword ptr [{rc} + 0x2062AC]",
                "fld dword ptr [{e} + 0x38]",
                ".byte 0xDE, 0xC9",
                ".byte 0xDE, 0xC1",
                ".byte 0xDE, 0xC1",
                ".byte 0xD9, 0xEE",
                ".byte 0xDE, 0xD9",
                "fnstsw ax",
                rc = in(reg) crate::race::store::asm_base(),
                p = in(reg) p,
                e = in(reg) e,
                out("ax") status,
                out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
                out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
                options(nostack),
            );
            drop(old);
            u32::from(status & 0x0100 != 0)
        }
    }

    pub(super) fn frame_of_asm(t: u32, f: u32, a: u32, b: u32) -> u32 {
        let (v, mut out) = ([t, f, a, b], 0u32);
        unsafe {
            core::arch::asm!(
                "fild dword ptr [{v}]",
                "fmul dword ptr [{v} + 4]",
                "fmul dword ptr [{v} + 8]",
                "fmul dword ptr [{v} + 12]",
                "fistp dword ptr [{o}]",
                v = in(reg) v.as_ptr(),
                o = in(reg) &raw mut out,
                out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
                out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
                options(nostack),
            );
        }
        out
    }

    pub(super) unsafe fn halfway_vertex_asm(a: u32, b: u32, v: u32) {
        let course = &race().course;
        use crate::math::x87_load::below_or_unordered;
        unsafe {
            let old = crate::race::store::OldPlaces::lay(&[0x5c5e40]);
            core::arch::asm!(
                "fld dword ptr [{b} + 0x10]",
                "fsub dword ptr [{a} + 0x10]",
                "fmul dword ptr [{rc} + 0x1C5E40]",
                "fadd dword ptr [{a} + 0x10]",
                "fstp dword ptr [{v} + 0x10]",
                "fld dword ptr [{b} + 0x14]",
                "fsub dword ptr [{a} + 0x14]",
                "fmul dword ptr [{rc} + 0x1C5E40]",
                "fadd dword ptr [{a} + 0x14]",
                "fstp dword ptr [{v} + 0x14]",
                "fld dword ptr [{b} + 0x18]",
                "fsub dword ptr [{a} + 0x18]",
                "fmul dword ptr [{rc} + 0x1C5E40]",
                "fadd dword ptr [{a} + 0x18]",
                "fstp dword ptr [{v} + 0x18]",
                rc = in(reg) crate::race::store::asm_base(),
                a = in(reg) a,
                b = in(reg) b,
                v = in(reg) v,
                out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
                out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
                options(nostack),
            );
            drop(old);
            crate::pointers::host(v + 0xC)
                .cast::<u32>()
                .write_unaligned(0);
            crate::pointers::host(v + 0x1C)
                .cast::<u32>()
                .write_unaligned(0);
            if below_or_unordered(get(v + 0x18), course.numbers.k4096_a.bits()) {
                return;
            }
            if course.stage_prefs.flat_depth.get() != 0 {
                let old = crate::race::store::OldPlaces::lay(&[
                    0x5ccd80, 0x62400c, 0x624088, 0x62409c, 0x6240a0,
                ]);
                core::arch::asm!(
                    "fld dword ptr [{v} + 0x10]",
                    "fld dword ptr [{v} + 0x14]",
                    "fld dword ptr [{v} + 0x18]",
                    "fstp dword ptr [{rc} + 0x22400C]",
                    "fild dword ptr [{rc} + 0x1CCD80]",
                    ".byte 0xD9, 0xE8",
                    ".byte 0xDE, 0xC1",
                    "fld dword ptr [{rc} + 0x224088]",
                    ".byte 0xDE, 0xF1",
                    ".byte 0xDC, 0xC9",
                    ".byte 0xDE, 0xCA",
                    ".byte 0xD9, 0xE0",
                    "fadd dword ptr [{rc} + 0x2240A0]",
                    ".byte 0xD9, 0xC9",
                    "fadd dword ptr [{rc} + 0x22409C]",
                    "fstp dword ptr [{v} + 0xC]",
                    "fstp dword ptr [{v} + 0x1C]",
                    rc = in(reg) crate::race::store::asm_base(),
                    v = in(reg) v,
                    out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
                    out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
                    options(nostack),
                );
                drop(old);
            } else {
                let old = crate::race::store::OldPlaces::lay(&[0x624088, 0x62409c, 0x6240a0]);
                core::arch::asm!(
                    "fld dword ptr [{v} + 0x10]",
                    "fld dword ptr [{v} + 0x14]",
                    "fld dword ptr [{v} + 0x18]",
                    "fld dword ptr [{rc} + 0x224088]",
                    ".byte 0xDE, 0xF1",
                    ".byte 0xDC, 0xC9",
                    ".byte 0xDE, 0xCA",
                    ".byte 0xD9, 0xE0",
                    "fadd dword ptr [{rc} + 0x2240A0]",
                    ".byte 0xD9, 0xC9",
                    "fadd dword ptr [{rc} + 0x22409C]",
                    "fstp dword ptr [{v} + 0xC]",
                    "fstp dword ptr [{v} + 0x1C]",
                    rc = in(reg) crate::race::store::asm_base(),
                    v = in(reg) v,
                    out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
                    out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
                    options(nostack),
                );
                drop(old);
            }
        }
    }

    pub(super) unsafe fn halfway_entry_asm(first: u32, second: u32, to: u32) {
        unsafe {
            let old = crate::race::store::OldPlaces::lay(&[0x5c5e40]);
            core::arch::asm!(
                "fld dword ptr [{s} + 8]",
                "fadd dword ptr [{f} + 8]",
                "fmul dword ptr [{rc} + 0x1C5E40]",
                "fstp dword ptr [{t} + 8]",
                "fld dword ptr [{s} + 0xC]",
                "fadd dword ptr [{f} + 0xC]",
                "fmul dword ptr [{rc} + 0x1C5E40]",
                "fstp dword ptr [{t} + 0xC]",
                "fld dword ptr [{s} + 0x10]",
                "fadd dword ptr [{f} + 0x10]",
                "fmul dword ptr [{rc} + 0x1C5E40]",
                "fstp dword ptr [{t} + 0x10]",
                "fld dword ptr [{s} + 0x14]",
                "fadd dword ptr [{f} + 0x14]",
                "fmul dword ptr [{rc} + 0x1C5E40]",
                "fstp dword ptr [{t} + 0x14]",
                rc = in(reg) crate::race::store::asm_base(),
                s = in(reg) second,
                f = in(reg) first,
                t = in(reg) to,
                out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
                out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
                options(nostack),
            );
            drop(old);
            for o in [0x20u32, 0x21, 0x18, 0x19, 0x1A] {
                let sum = crate::pointers::host(second + o).cast_const().read() as u32
                    + crate::pointers::host(first + o).cast_const().read() as u32;
                crate::pointers::host(to + o).write((sum >> 1) as u8);
            }
        }
    }

    fn same_all(a: &[u32], b: &[u32], what: &str) {
        for (k, (x, y)) in a.iter().zip(b).enumerate() {
            assert!(
                same(*x, *y),
                "{what}: word {k}: {x:#x} {y:#x} in {a:x?} {b:x?}"
            );
        }
    }

    #[test]
    fn face_turned_to_view_matches() {
        let race = crate::race::owner::race();
        let _l = lock();
        let _saved = Saved::new(&[0x621480, 0x5CCCC0, 0x6062A4, 0x6062A8, 0x6062AC]);
        let mut r = Rng(11);
        let mut points = [0u32; 12];
        let mut face = [0u32; 0x20];
        at_each_control_word(|| {
            for _ in 0..20000 {
                race.shared.switches.other_way.set(r.below(8));
                race.course
                    .course
                    .scene_points
                    .set(crate::pointers::game_address(points.as_ptr()));
                for a in [0x6062A4, 0x6062A8, 0x6062AC] {
                    game_write(rva(a), r.float());
                }
                for q in points.iter_mut() {
                    *q = r.int() as u32;
                }
                for q in face.iter_mut() {
                    *q = r.float();
                }
                let half = if r.below(2) == 0 { 0 } else { 4 };
                face[(half + 8) / 4] = r.below(4);
                let at = crate::pointers::game_address(face.as_ptr());
                let (_r1, _r2) = (
                    crate::race::trap::test_region(&face),
                    crate::race::trap::test_region(&points),
                );
                let want = unsafe { face_turned_to_view_asm(at, half as u32) };
                let got = crate::race::scene_points::with_test_points(&points, || {
                    face_turned_to_view(race, at, half as u32).unwrap()
                });
                assert_eq!(want, got, "{face:x?} {points:x?}");
            }
        });
    }

    #[test]
    fn frame_of_matches() {
        let _l = lock();
        let mut r = Rng(12);
        at_each_control_word(|| {
            for _ in 0..20000 {
                let (t, f, a, b) = (r.int() as u32, r.float(), r.float(), r.float());
                assert_eq!(
                    frame_of_asm(t, f, a, b),
                    frame_of(t, f, a, b),
                    "{t} {f:#x} {a:#x} {b:#x}"
                );
            }
        });
    }

    fn vertex(r: &mut Rng) -> [u32; 8] {
        let mut v = [0xAAAA_AAAAu32; 8];
        for q in v.iter_mut().skip(4).take(3) {
            *q = r.float();
        }
        v
    }

    #[test]
    fn halfway_vertex_matches() {
        let drawing = &crate::scene::owner::scene().drawing;
        let course = &race().course;
        let _l = lock();
        let _saved = Saved::new(&[
            0x62400C, 0x5CCD80, 0x624088, 0x6240A0, 0x62409C, 0x60932C, 0x5C5E40, 0x5C5E18,
        ]);
        let mut r = Rng(13);
        let win = crate::scene::view_window::ViewWindow::new();
        at_each_control_word(|| {
            for n in 0..20000 {
                course.stage_prefs.flat_depth.set((n % 2) as u32);
                course.course.flat_depth.set(r.int() as u32);
                // The assembly reads the old window; the Rust version the window it is given.
                for (old, own) in [
                    (&drawing.scene.view_scale, &win.view_scale),
                    (&drawing.scene.view_centre[1], &win.view_centre[1]),
                    (&drawing.scene.view_centre[0], &win.view_centre[0]),
                ] {
                    let value = r.float();
                    old.set_bits(value);
                    own.set_bits(value);
                }
                course.numbers.half.set_bits(if r.below(2) == 0 {
                    0x3F00_0000
                } else {
                    r.float()
                });
                // mostly a depth the projection takes
                course.numbers.k4096_a.set_bits(if r.below(4) == 0 {
                    r.float()
                } else {
                    0x4580_0000
                });
                let (a, b) = (vertex(&mut r), vertex(&mut r));
                let (mut v1, mut v2) = (vertex(&mut r), [0u32; 8]);
                v2.copy_from_slice(&v1);
                let (_r1, _r2, _r3) = (
                    crate::race::trap::test_region(&a),
                    crate::race::trap::test_region(&b),
                    crate::race::trap::test_region(&v1),
                );
                unsafe {
                    halfway_vertex_asm(
                        crate::pointers::game_address(a.as_ptr()),
                        crate::pointers::game_address(b.as_ptr()),
                        crate::pointers::game_address(v1.as_mut_ptr()),
                    );
                };
                halfway_record(course, &a, &b, &mut v2, &win);
                same_all(&v1, &v2, "vertex");
                // The assembly's depth store to 0x62400C is not kept by the Rust version.
            }
        });
    }

    #[test]
    fn halfway_entry_matches() {
        let course = &race().course;
        let _l = lock();
        let _saved = Saved::new(&[0x5C5E40]);
        let mut r = Rng(14);
        at_each_control_word(|| {
            for _ in 0..20000 {
                race().course.numbers.half.set_bits(if r.below(2) == 0 {
                    0x3F00_0000
                } else {
                    r.float()
                });
                let entry = |r: &mut Rng| {
                    let e = PolygonEntry::zeroed();
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
                let (t1, t2) = (PolygonEntry::zeroed(), PolygonEntry::zeroed());
                for k in 0..10 {
                    t1.set_word(4 * k, 0xAAAA_AAAA);
                    t2.set_word(4 * k, 0xAAAA_AAAA);
                }
                let at = |e: &PolygonEntry| {
                    crate::pointers::game_address(std::ptr::from_ref::<PolygonEntry>(e))
                };
                let (_r1, _r2, _r3) = (
                    crate::race::trap::test_region(std::slice::from_ref(&f)),
                    crate::race::trap::test_region(std::slice::from_ref(&s)),
                    crate::race::trap::test_region(std::slice::from_ref(&t1)),
                );
                unsafe { halfway_entry_asm(at(&f), at(&s), at(&t1)) };
                halfway_entry(course, &f, &s, &t2);
                // the bytes compare as they are, the floats as floats
                for k in 0..10 {
                    let (a, b) = (t1.word(4 * k), t2.word(4 * k));
                    if (2..6).contains(&k) {
                        assert!(same(a, b), "word {k}: {a:#x} {b:#x}");
                    } else {
                        assert_eq!(a, b, "word {k}");
                    }
                }
            }
        });
    }
}
