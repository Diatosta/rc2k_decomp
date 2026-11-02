//! The course's faces drawn (`sub_44F9F3` -> `sub_4537E3` -> `sub_45232A`, part R1, IDA
//! 2026-10-18). A face record is 0x40 bytes at `g_courseFaces` (+0 the kind and flags, +1
//! the object, +4 .. +0x10 four scene points, +0x18 / +0x30 / +0x38 and +0x1C / +0x34 /
//! +0x3C its two triangles' normals, by the reading of `sub_4525F8`, +0x20 / +0x21 the
//! second's flags and object). Here its leaves.

use crate::errors::{BadInput, Refused};
use crate::pointers::GamePtr;
use crate::race::blocks_store::CAR_BLOCKS;
use crate::race::course_lists_store::COURSE_LISTS;
use crate::race::course_store::COURSE;
use crate::race::level_grid_store::LEVEL_GRID;
use crate::race::numbers_store::NUMBERS;
use crate::race::prefs_store::STAGE_PREFS;
use crate::race::switches_store::FRAME_SWITCHES;
use crate::race::trap::Trap;
use crate::race::vars_store::RACE_VARS;
use crate::race::view_store::RACE_VIEW;
use crate::race::view_vars_store::VIEW_VARS;
use crate::scene::draw_store::DRAW;
use crate::scene::light_store::LIGHT_STATE;
use crate::scene::light_tables_store::LIGHT_TABLES;
use crate::scene::light_work_store::LIGHT_WORK;
use crate::scene::polygon_store::{POLYGON_WORK, PolygonEntry};
use crate::session::byte_of;
use crate::session::{Dword, Float};
use rc2k_core::race::faces::Face;

#[cfg(all(test, target_arch = "x86"))]
fn get(a: u32) -> Result<u32, BadInput> {
    crate::race::course_places::read(a, "a course face, point or entry word")
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
pub fn face_depth_test_off() {
    {
        // Converted: `rc2k_core::race::faces::depth_test_off`.
        if !rc2k_core::race::faces::depth_test_off(
            RACE_VIEW.texture_a.bits(),
            RACE_VIEW.polygon_mode.bits(),
        ) {
            return;
        }
        crate::scene::depth_test_off::depth_test_off();
        RACE_VIEW.depth_test.set(1u32);
    }
}

/// `race::face_depth_test_on` (0x45237A): the depth test on unless `dword_63D9E0` is set,
/// then `dword_63D9E0` 0. (It turns the test on when it was not turned off: the test looks
/// the wrong way round, `notes/bugs.md`.)
pub fn face_depth_test_on() {
    {
        // Converted: `rc2k_core::race::faces::depth_test_on`.
        if rc2k_core::race::faces::depth_test_on(RACE_VIEW.depth_test.bits()) {
            crate::scene::depth_test_on::depth_test_on();
        }
        RACE_VIEW.depth_test.set(0u32);
    }
}

/// `race::face_corners` (0x452444): face esi's four points into `dword_63D9EC` ..
/// `dword_63D9F8`, the fourth -1 when it is the third (a triangle). Leaves eax the fourth.
pub fn face_corners(face: &Face) -> u32 {
    // Converted: `rc2k_core::race::faces::corners`.
    let corners = rc2k_core::race::faces::corners(std::array::from_fn(|n| face.point(n)));
    crate::session::write_bits(&RACE_VIEW.corners, corners);
    corners[3]
}

/// `sub_4525F8`: 1 when the face's triangle at `face + half` (half 0 or 4) faces the view
/// (its point +8 less the view's place, `flt_6062A4` .., dotted with its normal +0x18,
/// +0x30, +0x38 above 0, or not a number; the original's x87 run), 0 outside a race.
#[cfg(all(test, target_arch = "x86"))]
fn face_turned_to_view(face: u32, half: u32) -> Result<u32, BadInput> {
    turned_to_view(
        &crate::race::course_places::face(face, "a course face")?,
        (half / 4) as usize,
    )
}

/// `sub_4525F8` on a read face, its triangle 0 or 1.
fn turned_to_view(f: &Face, triangle: usize) -> Result<u32, BadInput> {
    use crate::math::x87_float::{X, compare, int, ld};
    {
        if FRAME_SWITCHES.other_way.get() == 0 {
            return Ok(0);
        }
        let p = f.point(1 + triangle);
        let normal = f.normal(triangle);
        let along = |k: u32, view_at: &Float, normal: u32| -> Result<_, BadInput> {
            Ok(
                (int(crate::race::scene_points::point_word(p, k)? as i32) - ld(view_at.bits()))
                    * ld(normal),
            )
        };
        let place = &VIEW_VARS.view_place_f;
        let (x, y, z) = (
            along(0, &place[0], normal[0])?,
            along(1, &place[1], normal[1])?,
            along(2, &place[2], normal[2])?,
        );
        Ok(u32::from(compare(X(0.0), x + (y + z)) & 0x0100 != 0))
    }
}

/// `race::face_sides` (0x4525BE): which of face `face`'s (esi) triangles face the view:
/// `dword_63D9E4` the first's (`sub_4525F8`), `dword_63D9E8` 1, or the second's when it
/// has one (`dword_63D9F8` not -1). The registers the original leaves (eax the last
/// answer, ebx the last triangle's point, ebp the face) are read by no one: its one
/// caller's parts take only esi, the face.
pub fn face_sides(face: &Face) -> Result<(), BadInput> {
    {
        RACE_VIEW.face_turned[0].set(turned_to_view(face, 0)?);
        RACE_VIEW.face_turned[1].set(1u32);
        if RACE_VIEW.corners[3].get() != u32::MAX {
            RACE_VIEW.face_turned[1].set(turned_to_view(face, 1)?);
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
pub fn face_materials(face: &Face) -> Result<(), BadInput> {
    {
        let clock: u32 = CAR_BLOCKS.cars[0].step_clock.bits();
        let (k1, k2) = (NUMBERS.ten.bits(), NUMBERS.quarter.bits());
        let texture = |object: u32, first: bool| -> Result<u32, BadInput> {
            let speed = table_byte(&STAGE_PREFS.face_speeds, object, "the face speeds")?;
            if speed == 0 {
                return Ok(object);
            }
            let n = frame_of(
                clock,
                COURSE_LISTS.face_key_factors[speed as usize].bits(),
                k1,
                k2,
            );
            DRAW.scratch.set(n);
            let (frame, place) = rc2k_core::race::faces::frame_place(object, n);
            if first {
                RACE_VIEW.face_frame.set(frame);
            }
            table_byte(&STAGE_PREFS.face_places, place, "the face places")
        };
        // Converted: `rc2k_core::race::faces::{frame_place, material_bits}` (the frame's
        // rounding the x87's).
        use rc2k_core::race::faces::material_bits;
        RACE_VIEW
            .texture_a
            .set(texture(face.object() as u32, true)?);
        let (order, back) = material_bits(face.kind());
        RACE_VIEW.order_a.set(order);
        RACE_VIEW.back_a.set(back);
        RACE_VIEW
            .texture_b
            .set(texture(face.second_object() as u32, false)?);
        let (order, back) = material_bits(face.second_kind());
        RACE_VIEW.order_b.set(order);
        RACE_VIEW.back_b.set(back);
    }
    Ok(())
}

/// `race::draw_marked_face` (0x44CD90): `scene::scale_texture_coordinates`,
/// `scene::fog_polygon`, the drawing (`scene::draw_marked_polygon`), `g_polygonDrawn` 1.
pub fn draw_marked_face() -> Result<(), Refused> {
    crate::scene::scale_texture_coordinates::scale_texture_coordinates()?;
    crate::scene::fog_polygon::fog_polygon()?;
    crate::scene::draw_marked_polygon::draw_marked_polygon(&mut || {
        crate::scene::draw_polygon::draw_polygon_in_game()
    })?;
    DRAW.polygon_drawn.set(1u32);
    Ok(())
}

/// `race::face_switches` (0x45322C): `dword_6240E0` / `dword_6240E4` 1 when the face's
/// first / second texture is animated (`dword_63DA2C` / `dword_63DA38` not 0), else 0.
pub fn face_switches() {
    // Converted: `rc2k_core::race::faces::switches`.
    {
        let [first, second] = rc2k_core::race::faces::switches([
            RACE_VIEW.texture_a.bits(),
            RACE_VIEW.texture_b.bits(),
        ]);
        DRAW.polygon_sides[0].set(first);
        DRAW.polygon_sides[1].set(second);
    }
}

/// `sub_453786`: the light at scene point `n` (`race::light_colour_at`, through
/// `dword_62400C`), 0 for -1.
fn point_light(n: u32) -> Result<u32, Refused> {
    if n == u32::MAX {
        return Ok(0);
    }
    let [x, y, z] = crate::race::scene_points::point_number(n)?;
    LIGHT_STATE.lit_place[0].set(x);
    LIGHT_STATE.lit_place[1].set(y);
    LIGHT_STATE.lit_place[2].set(z);
    crate::race::frame_draw::light_colour_at()?;
    let light: u32 = LIGHT_WORK.light_colour.get();
    DRAW.scratch.set(light);
    Ok(light)
}

/// `race::face_lights` (0x453715): the light at each of the face's four corners
/// (`dword_63DA54` .. `dword_63DA60`; 0 for a missing fourth), when lamps are wanted
/// (`g_lampsWanted`) and the face is not a plain mode-4 one; all 0 otherwise. Refuses
/// (before anything) what `light_colour_at` refuses.
pub fn face_lights() -> Result<(), Refused> {
    // Converted: `rc2k_core::race::faces::corners_lit`.
    let wanted: u32 = RACE_VIEW.lamps_wanted.get();
    let lit = wanted != 0
        && rc2k_core::race::faces::corners_lit(
            wanted,
            RACE_VIEW.texture_a.bits(),
            RACE_VIEW.polygon_mode.bits(),
        );
    if lit {
        crate::race::frame_draw::check_light_colour()?;
    }
    for k in 0..4 {
        RACE_VIEW.corner_lights[k].set(0u32);
    }
    if !lit {
        return Ok(());
    }
    for k in 0..4 {
        let n: u32 = RACE_VIEW.corners[k].get();
        let light = point_light(n)?;
        RACE_VIEW.corner_lights[k].set(light);
    }
    Ok(())
}

/// `race::clip_and_draw_face` (0x454187): the face's polygon cut by depth (when some
/// vertices are outside, `g_polygonOutsideCount`) and to the screen, and with three
/// vertices or more drawn (`scene::draw_polygon_in_game`), `g_polygonDrawn` 1.
pub fn clip_and_draw_face() -> Result<(), Refused> {
    // Converted: `rc2k_core::race::face_drawing::draws_polygon`.
    use rc2k_core::race::face_drawing::draws_polygon;
    if DRAW.polygon_outside_count.get() != 0 {
        crate::scene::clip_polygon_by_depth::clip_polygon_by_depth()?;
        if !draws_polygon(DRAW.polygon_vertex_count.bits()) {
            return Ok(());
        }
    }
    crate::scene::clip_polygon_to_screen::clip_polygon_to_screen()?;
    if !draws_polygon(DRAW.polygon_vertex_count.bits()) {
        return Ok(());
    }
    crate::scene::draw_polygon::draw_polygon_in_game()?;
    DRAW.polygon_drawn.set(1u32);
    Ok(())
}

/// `race::face_out_of_view` (0x45246F): true (the original's carry) when none of the
/// face's corners (`dword_63D9EC` ..; the fourth when there is one) is in sight: a corner
/// is in sight when its scene record's depth (+0x18) is not negative and |x| (+0x10) is
/// below `flt_63DA00` (|x| then left in `dword_62400C`) or |x| times `flt_63D9FC` is below
/// the depth. The records are made when not there (as `sub_44CF33`). The eax and edx the
/// original leaves are read by no one. Refuses (before anything) no point records or
/// points.
pub fn face_out_of_view() -> Result<bool, Refused> {
    // Converted: `rc2k_core::race::face_drawing::corner_in_sight` (exact in `f64`).
    use rc2k_core::race::face_drawing::{InSight, corner_in_sight};
    crate::race::frame_draw::points_there()?;
    let near: u32 = RACE_VIEW.near.bits();
    let slope: u32 = RACE_VIEW.slope.bits();
    for k in 0..4 {
        let n: u32 = RACE_VIEW.corners[k].get();
        if k == 3 && n == u32::MAX {
            break;
        }
        let record = crate::race::frame_draw::point_record(n)?;
        let depth = crate::scene::point_records::field(record, 0x18)?;
        if depth & 0x8000_0000 != 0 {
            continue;
        }
        let x = crate::scene::point_records::field(record, 0x10)?;
        match corner_in_sight(depth, x, near, slope) {
            Some(InSight::Near) => {
                DRAW.scratch.set(x & 0x7FFF_FFFF);
                return Ok(false);
            }
            Some(InSight::Slope) => return Ok(false),
            None => {}
        }
    }
    Ok(true)
}

/// Where one of the face's two triangles keeps its texture's working values
/// (`sub_453267` for the first, `sub_45349F` for the second: the same code on other
/// variables).
pub struct TextureSet {
    /// The texture (`dword_63DA2C` / `dword_63DA38`).
    pub texture: &'static Dword,
    /// The kind's low bits, the corner order (`dword_63DA30` / `dword_63DA3C`).
    pub order: &'static Dword,
    /// The texture's size word with bit 31 (`dword_6240BC` / `dword_6240C0`).
    pub size: &'static Dword,
    /// The texture's mapping mode, its +0x2D signed (`dword_623DD8` / `dword_623DE0`).
    pub mapping: &'static Dword,
    /// 1 when the mapping is 0 (`dword_63E168` / `dword_63E16C`).
    pub plain: &'static Dword,
    /// The four corners' u, v (`dword_63DA0C` .. / `dword_63DAB8` ..).
    pub uv: &'static [Dword; 8],
}

/// The first triangle's set.
pub fn first_set() -> TextureSet {
    TextureSet {
        texture: &RACE_VIEW.texture_a,
        order: &RACE_VIEW.order_a,
        size: &DRAW.polygon_textures[0],
        mapping: &RACE_VARS.mapping_a[0],
        plain: &LEVEL_GRID.plain_first,
        uv: &RACE_VIEW.corner_uv,
    }
}

/// The second triangle's set.
pub fn second_set() -> TextureSet {
    TextureSet {
        texture: &RACE_VIEW.texture_b,
        order: &RACE_VIEW.order_b,
        size: &DRAW.polygon_textures[1],
        mapping: &RACE_VARS.mapping_b[0],
        plain: &LEVEL_GRID.plain_second,
        uv: &RACE_VIEW.corner_uv_b,
    }
}

/// The `N` bytes (at most 4) at `offset` of texture `texture`'s info record: the records
/// lie one after another, and the original's sums of addresses may leave the record, or
/// the records (then read by address, checked).
fn info_bytes<const N: usize>(texture: u32, offset: u32) -> Result<[u8; N], BadInput> {
    let infos = &POLYGON_WORK.texture_infos;
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
    let base = infos[0].words[0].addr();
    GamePtr::new(base.wrapping_add(at as u32)).read("a texture info's bytes")
}

/// `race::face_texture_coordinates` (0x453267) / `race::face_texture_coordinates_2`
/// (0x45349F): the triangle's texture's size word (+0x20, bit 31 set) and mapping (+0x2D);
/// the four corners' u, v from the texture's corner table for the order
/// (`dword_63DAD8[order * 4 + k]` points at a u16 pair inside the texture record, << 8,
/// doubled for sizes 0x190 .. 0x31F); with a mapping above 0 the corners' scene x and z
/// shifted by (mapping & 3) + 2 instead, less the first corner's top byte (the fourth
/// corner read even when it is -1: `notes/bugs.md`). The registers the original leaves
/// (eax the size word, ebx the texture record) are read by no one.
pub fn face_texture_coordinates(set: &TextureSet) -> Result<(), BadInput> {
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
            let table = RACE_VIEW.corner_uv_offsets[(order << 2) as usize + k].get();
            *pair_out = (
                u16::from_le_bytes(info_bytes(texture, table)?),
                u16::from_le_bytes(info_bytes(texture, table.wrapping_add(2))?),
            );
        }
        let trap = Trap::new();
        let c = texture_coordinates(texture_info, pairs, |k| {
            let n: u32 = RACE_VIEW.corners[k].get();
            let w = |k: u32| match crate::race::scene_points::point_word(n, k) {
                Ok(v) => v,
                Err(e) => {
                    trap.keep(e);
                    0
                }
            };
            (w(0), w(2))
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
pub fn face_corners_to_new_points() -> Result<(), Refused> {
    // Converted: `rc2k_core::race::faces::new_points`.
    {
        crate::race::frame_draw::points_there()?;
        let count: u32 = COURSE.scene_point_count.get();
        let corners: [u32; 4] = crate::session::read_bits(&RACE_VIEW.corners);
        let (marked, copies) = rc2k_core::race::faces::new_points(count, corners);
        for n in marked {
            crate::scene::point_records::put_field(
                crate::scene::point_records::at_offset(n << 5),
                0,
                u32::MAX,
            )?;
        }
        for (k, (from, to)) in copies.into_iter().enumerate() {
            crate::race::scene_points::copy_point(from, to)?;
            RACE_VIEW.corners[k].set(to);
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
fn fill_entry(slot: u32, corner: u32) -> Result<(), BadInput> {
    {
        let e = entry(slot);
        let c = corner as usize;
        e.point.set(RACE_VIEW.corners[c].get());
        e.u.set(RACE_VIEW.corner_uv[2 * c].get());
        e.v.set(RACE_VIEW.corner_uv[2 * c + 1].get());
        e.light.set(RACE_VIEW.corner_colours[c].get());
        e.lamp_light.set(RACE_VIEW.corner_lights[c].get());
        e.u2.set(RACE_VIEW.corner_uv_b[2 * c].get());
        e.v2.set(RACE_VIEW.corner_uv_b[2 * c + 1].get());
        e.amount.set(RACE_VIEW.corner_alphas[c].get() as u8);
    }
    Ok(())
}

/// The light's third byte (bits 16 .. 23) of entry `slot`: the lamps' part.
fn lamp_byte(slot: u32) -> Result<u8, BadInput> {
    Ok(entry(slot).byte(0x1E))
}

/// `g_polygonList` 0 .. count - 1 and a closing 0, `g_polygonVertexCount` count,
/// `g_polygonNextEntry` `next`.
fn plain_list(count: u32, next: u32) {
    {
        DRAW.polygon_vertex_count.set(count);
        for k in 0..count {
            crate::race::checkpoints_store::cell_in(&POLYGON_WORK.list, k as usize).set(k);
        }
        crate::race::checkpoints_store::cell_in(&POLYGON_WORK.list, count as usize).set(0);
        DRAW.polygon_next_entry.set(next);
    }
}

/// `race::draw_whole_quad` (0x452F3D): the face drawn as one quad when it can be: the
/// entries 1, 2, 3, 0 from corners 0, 1, 3, 2; carry (draw it as triangles) when a corner
/// has lamp light or some vertex is outside after `scene::project_polygon` or off the
/// screen's inside (the screen edges `dword_6241D4` .. less one; each vertex's x and y
/// rounded into `dword_62400C`, compared by the sign of the difference); nothing when
/// all are outside; else the quad drawn (`race::draw_marked_face`). Gives the carry
/// (true: draw it as triangles).
pub fn draw_whole_quad() -> Result<bool, Refused> {
    // Converted: `rc2k_core::race::face_drawing::{QUAD_ENTRIES, quad_by_outside,
    // screen_inside, inside}` (the roundings the x87's).
    use rc2k_core::race::face_drawing::{QUAD_ENTRIES, quad_by_outside, screen_inside};
    for (slot, corner) in QUAD_ENTRIES {
        fill_entry(slot, corner)?;
    }
    for s in 0..4 {
        if lamp_byte(s)? != 0 {
            return Ok(true);
        }
    }
    plain_list(4, 4);
    crate::scene::project_polygon::project_polygon()?;
    if let Some(triangles) = quad_by_outside(DRAW.polygon_outside_count.bits()) {
        return Ok(triangles);
    }
    let [left, right, top, bottom] = screen_inside(
        DRAW.clip_edges[2].bits(),
        DRAW.clip_edges[3].bits(),
        DRAW.clip_edges[0].bits(),
        DRAW.clip_edges[1].bits(),
    );
    let inside = |v: u32, low: u32, high: u32| {
        let n = crate::math::x87::to_int(v) as u32;
        DRAW.scratch.set(n);
        rc2k_core::race::face_drawing::inside(n, low, high)
    };
    for slot in 0..4 {
        let v = entry(slot).record.get();
        if !inside(crate::scene::point_records::vertex(v, 0xC), left, right)
            || !inside(crate::scene::point_records::vertex(v, 0x1C), top, bottom)
        {
            return Ok(true);
        }
    }
    DRAW.polygon_vertex_count.set(4u32);
    draw_marked_face()?;
    Ok(false)
}

/// `race::draw_face_triangles` (0x452CF3): the face's two triangles each drawn
/// (`race::draw_lit_triangle`) unless facing away: the first (corners 0, 1, 2) when
/// `dword_63D9E4` is 0, the second (3, 2, 1) when there is one and `dword_63D9E8` is 0.
pub fn draw_face_triangles() -> Result<(), Refused> {
    // Converted: `rc2k_core::race::face_drawing::{draws_first, draws_second,
    // FIRST_TRIANGLE, SECOND_TRIANGLE}`.
    use rc2k_core::race::face_drawing::{
        FIRST_TRIANGLE, SECOND_TRIANGLE, draws_first, draws_second,
    };
    if draws_first(RACE_VIEW.face_turned[0].bits()) {
        for (slot, corner) in FIRST_TRIANGLE {
            fill_entry(slot, corner)?;
        }
        DRAW.polygon_vertex_count.set(3u32);
        draw_lit_triangle()?;
    }
    let fourth: u32 = RACE_VIEW.corners[3].get();
    if fourth != u32::MAX && draws_second(fourth, RACE_VIEW.face_turned[1].bits()) {
        for (slot, corner) in SECOND_TRIANGLE {
            fill_entry(slot, corner)?;
        }
        DRAW.polygon_vertex_count.set(3u32);
        draw_lit_triangle()?;
    }
    Ok(())
}

/// `race::draw_face_polygons` (0x452CCF): in a race with both triangles facing the view
/// (`dword_63D9E4` + `dword_63D9E8` 0) the face as one quad (`race::draw_whole_quad`, its
/// carry asking for triangles), else as triangles (`race::draw_face_triangles`).
pub fn draw_face_polygons() -> Result<(), Refused> {
    // Converted: `rc2k_core::race::face_drawing::tries_quad`.
    let in_race: u32 = FRAME_SWITCHES.other_way.get();
    if in_race != 0
        && rc2k_core::race::face_drawing::tries_quad(
            in_race,
            RACE_VIEW.face_turned[0].bits(),
            RACE_VIEW.face_turned[1].bits(),
        )
        && !draw_whole_quad()?
    {
        return Ok(());
    }
    draw_face_triangles()
}

/// `race::draw_face` (0x452C6F): the face's textures (`race::face_switches`, the first's
/// coordinates when animated; `g_polygonMode` 2, or with a second texture the mode its
/// record's +0x2C picks in `dword_63D9D0` and the second's coordinates), the corners'
/// lights (`race::face_lights`), then the depth test off, the polygons
/// (`race::draw_face_polygons`), the depth test on. The original keeps esi (the face);
/// the rest it leaves no caller reads. Refuses (before anything) what `face_lights`
/// refuses.
pub fn draw_face() -> Result<(), Refused> {
    // Converted: `rc2k_core::race::face_drawing::{face_mode, PLAIN_MODE}`, the lights'
    // test `rc2k_core::race::faces::corners_lit`.
    use rc2k_core::race::face_drawing::{PLAIN_MODE, face_mode};
    let second: u32 = RACE_VIEW.texture_b.get();
    let trap = Trap::new();
    let picked = || {
        let record = POLYGON_WORK.texture_infos[0].words[0]
            .addr()
            .wrapping_add(second << 6);
        let pick = trap.read::<u8>(record + 0x2C, "a texture info's kind") as usize;
        // dword_63D9D0: indexed by a byte (not bounded by the field's 4 dwords), so
        // read by address.
        trap.read::<u32>(
            RACE_VIEW.surface_kinds[0].addr() + (4 * pick) as u32,
            "the surface kinds table",
        )
    };
    let mode = face_mode(second, picked);
    trap.finish()?;
    let wanted: u32 = RACE_VIEW.lamps_wanted.get();
    if wanted != 0 && rc2k_core::race::faces::corners_lit(wanted, RACE_VIEW.texture_a.bits(), mode)
    {
        crate::race::frame_draw::check_light_colour()?;
    }
    face_switches();
    if RACE_VIEW.texture_a.get() != 0 {
        face_texture_coordinates(&first_set())?;
    }
    RACE_VIEW.polygon_mode.set(PLAIN_MODE);
    if second != 0 {
        RACE_VIEW.polygon_mode.set(face_mode(second, picked));
        trap.finish()?;
        face_texture_coordinates(&second_set())?;
    }
    face_lights()?;
    face_depth_test_off();
    draw_face_polygons()?;
    face_depth_test_on();
    Ok(())
}

/// `sub_4537E9`'s new vertex between two entries' vertices `a` and `b` into `v`: the
/// view floats (+0x10 .. +0x18) halfway (b - a) x `flt_5C5E40` + a, the screen place
/// (+0xC, +0x1C) 0, or when the depth is not below `flt_5C5E18` projected (by
/// `flt_624088` over the depth, or over `dword_5CCD80` + 1 when `dword_60932C` is set,
/// the depth then kept in `dword_62400C`; centre `flt_62409C`, `flt_6240A0`).
fn halfway_vertex(a: u32, b: u32, v: u32) -> Result<(), BadInput> {
    use crate::scene::point_records::{get, set};
    let mut made = get(v)?;
    halfway_record(&get(a)?, &get(b)?, &mut made);
    set(v, made)
}

/// `halfway_vertex` on the records themselves (`a`, `b` point records of any kind).
fn halfway_record(
    a: &crate::scene::point_records::Record,
    b: &crate::scene::point_records::Record,
    v: &mut crate::scene::point_records::Record,
) {
    use crate::math::x87_float::{ld, st};
    use crate::math::x87_load::below_or_unordered;
    let half = ld(NUMBERS.half.bits());
    for k in [4, 5, 6] {
        let (from, to) = (ld(a[k]), ld(b[k]));
        v[k] = st((to - from) * half + from);
    }
    (v[3], v[7]) = (0, 0);
    if below_or_unordered(v[6], NUMBERS.k4096_a.bits()) {
        return;
    }
    crate::race::skidmarks::project_record(v);
}

/// `sub_4537E9`'s new entry between entries `first` and `second` into `to`: the texture
/// floats (+8 .. +0x14) (second + first) x `flt_5C5E40`, the bytes +0x20, +0x21, +0x18 ..
/// +0x1A the halves of their sums (+0x1B left).
fn halfway_entry(first: &PolygonEntry, second: &PolygonEntry, to: &PolygonEntry) {
    use crate::math::x87_float::{ld, st};
    let half = ld(NUMBERS.half.bits());
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
pub fn draw_lit_triangle() -> Result<(), Refused> {
    use crate::race::frame_draw::add_colours;
    let mut lit = false;
    for s in 0..3 {
        if lamp_byte(s)? != 0 {
            lit = true;
            break;
        }
    }
    if !lit {
        return crate::scene::draw_clipped_polygon::draw_clipped_polygon_in_game();
    }
    crate::race::frame_draw::points_there()?;
    crate::race::frame_draw::check_light_colour()?;
    DRAW.polygon_drawn.set(0u32);
    DRAW.polygon_next_record
        .set(crate::scene::polygon::pool_start());
    plain_list(DRAW.polygon_vertex_count.bits(), 6);
    crate::scene::project_polygon::project_polygon()?;
    if DRAW.polygon_outside_count.get() == DRAW.polygon_vertex_count.get() {
        return Ok(());
    }
    crate::scene::scale_texture_coordinates::scale_texture_coordinates()?;
    crate::scene::fog_polygon::fog_polygon()?;
    for k in 0..3 {
        entry(3 + k)
            .record
            .set(crate::scene::point_records::of_quad(k));
    }
    for (k, (first, second)) in [(0u32, 1u32), (1, 2), (2, 0)].into_iter().enumerate() {
        let v = crate::scene::point_records::of_quad(k as u32);
        let pa = entry(first).point.get();
        let pb = entry(second).point.get();
        for (n, lit) in LIGHT_STATE.lit_place.iter().enumerate() {
            let o = 4 * n as u32;
            let (a, b) = (
                crate::race::scene_points::point_word(pa, n as u32)?,
                crate::race::scene_points::point_word(pb, n as u32)?,
            );
            let m = ((b.wrapping_sub(a) as i32) >> 1) as u32;
            let m = m.wrapping_add(a);
            crate::scene::point_records::put_field(v, o, m)?;
            lit.set(m);
        }
        halfway_vertex(entry(first).record.get(), entry(second).record.get(), v)?;
        let to = entry(3 + k as u32);
        halfway_entry(entry(first), entry(second), to);
        crate::race::frame_draw::light_colour_at()?;
        let light: u32 = LIGHT_WORK.light_colour.get();
        DRAW.scratch.set(light);
        to.lamp_light.set(light);
    }
    let mode5 = RACE_VIEW.polygon_mode.get() == 5;
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
        DRAW.polygon_vertex_count.set(3u32);
        DRAW.polygon_next_entry.set(6u32);
        clip_and_draw_face()?;
    }
    Ok(())
}

/// `race::face_colours` (0x452887): the corners' colours from `g_lightTable` by the face's
/// bytes +0x14 .. +0x17, with flag 4 of +2 plus `dword_635518[+0x28 ..]` and
/// `dword_635918[+0x2C ..]` (`add_colours`, MMX or not as the original); the fog bytes
/// +0x22 .. +0x25 into `dword_63DA64` ..; then the face drawn (`race::draw_face`).
pub fn face_colours(face: &Face) -> Result<(), Refused> {
    // Converted: `rc2k_core::race::lights::{corner_colour, EXTRA_LIGHTS}`.
    use rc2k_core::race::lights::{EXTRA_LIGHTS, corner_colour};
    let lit = face.switches() & EXTRA_LIGHTS != 0;
    for k in 0..4 {
        let c = LIGHT_TABLES.light_table[face.corner_light(k) as usize].get();
        RACE_VIEW.corner_colours[k].set(c);
        if lit {
            let extra = || {
                let (a, b) = face.lamps(k);
                let a = LIGHT_TABLES.lamp_table_a[a as usize].get();
                let b = LIGHT_TABLES.lamp_table_b[b as usize].get();
                (a, b)
            };
            let c = corner_colour(c, lit, extra, crate::race::frame_draw::has_mmx());
            RACE_VIEW.corner_colours[k].set(c);
        }
    }
    for k in 0..4 {
        RACE_VIEW.corner_alphas[k].set(face.corner_fog(k) as u32);
    }
    draw_face()
}

/// The corners turned over (0 with 1, and 2 with 3 when there is a fourth) and the first
/// texture's order by `byte_63DA7C` (its bit 3 kept): `sub_452B58`'s step, done twice.
fn turn_over() -> Result<(), BadInput> {
    {
        let swap = |a: usize, b: usize| {
            let (x, y) = (RACE_VIEW.corners[a].get(), RACE_VIEW.corners[b].get());
            RACE_VIEW.corners[a].set(y);
            RACE_VIEW.corners[b].set(x);
        };
        // Converted: `rc2k_core::race::face_drawing::{turns_last_pair, turned_order}`.
        use rc2k_core::race::face_drawing::{turned_order, turns_last_pair};
        swap(0, 1);
        if turns_last_pair(RACE_VIEW.corners[3].bits()) {
            swap(2, 3);
        }
        let order: u32 = RACE_VIEW.order_a.get();
        let trap = Trap::new();
        let turned = turned_order(order, |n| {
            // `turned_order` asks 0 ..= 7.
            byte_of(&RACE_VIEW.turned_corner_order, n as usize).unwrap_or_else(|| {
                trap.keep(BadInput("past the turned corner order"));
                0
            })
        });
        trap.finish()?;
        RACE_VIEW.order_a.set(turned);
    }
    Ok(())
}

/// `race::draw_back_face` (0x452B58): a two-sided face's back (when `dword_63DA34`, the
/// first kind's bit 3): the corners turned over, the colours from `g_lightTable` by
/// +0x28 .. +0x2B and the fog bytes +0x2C .. +0x2F, drawn (`race::draw_face`), the
/// corners turned back.
pub fn draw_back_face(face: &Face) -> Result<(), Refused> {
    // Converted: `rc2k_core::race::face_drawing::{has_back, turns_last_pair,
    // turned_order}`.
    if !rc2k_core::race::face_drawing::has_back(RACE_VIEW.back_a.bits()) {
        return Ok(());
    }
    turn_over()?;
    for k in 0..4 {
        RACE_VIEW.corner_colours[k]
            .set(LIGHT_TABLES.light_table[face.back_light(k) as usize].get());
    }
    for k in 0..4 {
        RACE_VIEW.corner_alphas[k].set(face.back_fog(k) as u32);
    }
    draw_face()?;
    turn_over()?;
    Ok(())
}

/// `race::draw_face_sides` (0x452393): outside a race both sides (`race::face_colours`,
/// `race::draw_back_face`) with neither facing away; in a race, a face with objects (+1
/// or +0x21): one-sided (kind bit 3 clear) its front unless both triangles face away;
/// two-sided its front unless both face away, then the facings flipped and its back
/// unless both (now) face away. Each after `race::face_materials`. `face` is esi.
pub fn draw_face_sides(face: &Face) -> Result<(), Refused> {
    // Converted: `rc2k_core::race::face_drawing::{has_objects, two_sided, side_drawn,
    // flipped}`.
    use rc2k_core::race::face_drawing::{flipped, has_objects, side_drawn, two_sided};
    if FRAME_SWITCHES.other_way.get() == 0 {
        face_materials(face)?;
        RACE_VIEW.face_turned[0].set(0u32);
        RACE_VIEW.face_turned[1].set(0u32);
        face_colours(face)?;
        RACE_VIEW.face_turned[0].set(0u32);
        RACE_VIEW.face_turned[1].set(0u32);
        return draw_back_face(face);
    }
    let facings = || -> bool {
        side_drawn(
            RACE_VIEW.face_turned[0].bits(),
            RACE_VIEW.face_turned[1].bits(),
        )
    };
    if !has_objects(face.object(), face.second_object()) {
        return Ok(());
    }
    if !two_sided(face.kind()) {
        if !facings() {
            return Ok(());
        }
        face_materials(face)?;
        return face_colours(face);
    }
    face_materials(face)?;
    if facings() {
        face_colours(face)?;
    }
    RACE_VIEW.face_turned[0].set(flipped(RACE_VIEW.face_turned[0].bits()));
    RACE_VIEW.face_turned[1].set(flipped(RACE_VIEW.face_turned[1].bits()));
    if facings() {
        draw_back_face(face)?;
    }
    Ok(())
}

/// `race::draw_course_face` (0x45232A): face `face` (esi): its corners
/// (`race::face_corners`; with flag 0x200 of +2 copied to new points,
/// `race::face_corners_to_new_points`), and unless none is in sight
/// (`race::face_out_of_view`) its facings (`race::face_sides`), the depth test off, its
/// sides (`race::draw_face_sides`), the depth test on. Refuses (before anything) no point
/// records or points.
pub fn draw_course_face(face: u32) -> Result<(), Refused> {
    crate::race::frame_draw::points_there()?;
    // The face read once for its corners, normals, kinds, objects and colours: drawing it
    // writes no face.
    let face = crate::race::course_places::face_number(face, "a course face")?;
    face_corners(&face);
    if face.switches() & rc2k_core::race::faces::REPEATED != 0 {
        face_corners_to_new_points()?;
    }
    if face_out_of_view()? {
        return Ok(());
    }
    face_sides(&face)?;
    face_depth_test_off();
    draw_face_sides(&face)?;
    face_depth_test_on();
    Ok(())
}

/// `race::draw_tinted_face` (0x44F9F3): face `face` (esi) drawn (`sub_4537E3`, that is
/// `race::draw_course_face`) with `g_polygonColour` times its object's tint
/// (`dword_60A880[+1]`, when not 0; byte by byte (a x b) >> 8, all four with MMX, the low
/// three without), the colour put back after. Of the registers its one caller
/// (`race::draw_list_entry`) gives, the drawing reads only esi.
pub fn draw_tinted_face(face: u32) -> Result<(), Refused> {
    // Converted: `rc2k_core::race::face_drawing::{tints, tinted}`.
    use rc2k_core::race::face_drawing::{tinted, tints};
    let colour: u32 = DRAW.polygon_colour.get();
    let object = crate::race::course_places::face_field::<u8>(face, 1, "a course face")? as u32;
    // `object` is a byte: inside the 256 tints.
    let tint = STAGE_PREFS.object_tints[object as usize].get();
    if tints(tint) {
        DRAW.polygon_colour
            .set(tinted(tint, colour, crate::race::frame_draw::has_mmx()));
    }
    let done = draw_course_face(face);
    DRAW.polygon_colour.set(colour);
    done
}

#[cfg(all(test, target_arch = "x86"))]
mod asm_reference {
    use super::*;

    // The reference blocks read and write the test's own memory plainly.
    fn get(a: u32) -> u32 {
        super::get(a).expect("the test's memory is registered")
    }
    use crate::framework::{game_write, rva};
    use crate::race::asm_test::{Rng, Saved, at_each_control_word, lock, same};

    // The original functions, verbatim.
    pub(super) unsafe fn face_turned_to_view_asm(face: u32, half: u32) -> u32 {
        unsafe {
            if FRAME_SWITCHES.other_way.get() == 0 {
                return 0;
            }
            let e = face + half;
            let p = get(e + 8)
                .wrapping_mul(12)
                .wrapping_add(COURSE.scene_points.bits());
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
            if below_or_unordered(get(v + 0x18), NUMBERS.k4096_a.bits()) {
                return;
            }
            if STAGE_PREFS.flat_depth.get() != 0 {
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
        let _l = lock();
        let _saved = Saved::new(&[0x621480, 0x5CCCC0, 0x6062A4, 0x6062A8, 0x6062AC]);
        let mut r = Rng(11);
        let mut points = [0u32; 12];
        let mut face = [0u32; 0x20];
        at_each_control_word(|| {
            for _ in 0..20000 {
                FRAME_SWITCHES.other_way.set(r.below(8));
                COURSE
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
                    face_turned_to_view(at, half as u32).unwrap()
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
        let _l = lock();
        let _saved = Saved::new(&[
            0x62400C, 0x5CCD80, 0x624088, 0x6240A0, 0x62409C, 0x60932C, 0x5C5E40, 0x5C5E18,
        ]);
        let mut r = Rng(13);
        at_each_control_word(|| {
            for n in 0..20000 {
                DRAW.scratch.set(0xAAAA_AAAAu32);
                STAGE_PREFS.flat_depth.set((n % 2) as u32);
                COURSE.flat_depth.set(r.int() as u32);
                DRAW.view_scale.set_bits(r.float());
                DRAW.view_centre[1].set_bits(r.float());
                DRAW.view_centre[0].set_bits(r.float());
                NUMBERS.half.set_bits(if r.below(2) == 0 {
                    0x3F00_0000
                } else {
                    r.float()
                });
                // mostly a depth the projection takes
                NUMBERS.k4096_a.set_bits(if r.below(4) == 0 {
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
                let kept = DRAW.scratch.get();
                DRAW.scratch.set_bits(0xAAAA_AAAAu32);
                halfway_record(&a, &b, &mut v2);
                same_all(&v1, &v2, "vertex");
                assert!(same(kept, DRAW.scratch.get()), "depth kept");
            }
        });
    }

    #[test]
    fn halfway_entry_matches() {
        let _l = lock();
        let _saved = Saved::new(&[0x5C5E40]);
        let mut r = Rng(14);
        at_each_control_word(|| {
            for _ in 0..20000 {
                NUMBERS.half.set_bits(if r.below(2) == 0 {
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
                halfway_entry(&f, &s, &t2);
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
