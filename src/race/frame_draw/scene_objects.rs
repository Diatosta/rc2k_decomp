//! The scene's objects: the kept reflections, the race slots, screen quads, flares, car lights and the drawing of the sorted entries.

use super::*;
use crate::errors::{BadInput, Refused};
use crate::race::frame_view::car_at as car_view;
use crate::race::kind_store::KIND_TABLES;
use crate::race::owner::Race;
use crate::race::owner_shared::RaceShared;
use crate::race::trap::car_at;
use crate::race::view_scratch::DrawCtx;
use crate::scene::draw_described_object::Description;
use crate::scene::polygon_store::POLYGON_WORK;

/// `race::start_kept_reflections` (0x451864): the 0x400 kept reflection normals cleared to
/// -999 (0xFFFFFC19) and `g_reflectionsKept` 1.
pub fn start_kept_reflections(view: &crate::race::view_scratch::ViewScratch) {
    for cell in &view.object.kept_reflections[..0x400] {
        cell.set(0xFFFF_FC19);
    }
    view.face.reflections_kept.set(1u32);
}

/// `race::end_kept_reflections` (0x451881): `g_reflectionsKept` 0.
pub fn end_kept_reflections(view: &crate::race::view_scratch::ViewScratch) {
    view.face.reflections_kept.set(0u32);
}

/// `race::draw_slot_object` (0x4626D1): the race slot's object (+0, into `g_objectNumber`) drawn,
/// unless outside one player and modes 0 and 2 its car (object 0x50 + 0x14 x car) is not the drawn
/// view's (`race().frame.frame_work.listed_car_sounds`): the light (lowest 0, highest 0xFF, level
/// `STAGE_PREFS.light_level`, strength `STAGE_PREFS.light_strength`, turns -`STAGE_PREFS.sun_turn`
/// / `STAGE_PREFS.sun_height`, `LIGHT_STATE.view_kind` 1, `g_lightKind` 2), its turn (+0x10, 16
/// dwords to `ctx.view.matrices.object_turn_matrix`) placed at +4, +8 + (+0x5C), +0xC (into
/// `OBJECT_WORK.place` ..) and moved by the view (0x5EA1CC, into
/// `ctx.view.matrices.matrix_scratch`), times the view's turn into `g_madeObjectMatrix`; the
/// reflections kept around the drawing (`scene::draw_scene_object`). Refuses (before anything) as
/// `check_slot_object`.
pub fn draw_slot_object(
    ctx: &crate::race::view_scratch::DrawCtx<'_>,
    slot: &crate::race::work_store::RaceSlot,
) -> Result<(), Refused> {
    let race = ctx.race;
    use crate::scene::matrix;
    check_slot_object(&race.shared, slot)?;
    let object = slot.part.get();
    ctx.view.cars.object_number.set(object);
    let mode: u32 = race.shared.vars.run_kind.get();
    if race.shared.view.players.get() != 1 && mode != 0 && mode != 2 {
        let n = object.wrapping_sub(0x50);
        if race.frame.frame_work.listed_car_sounds[((n / 0x14) & 7) as usize].get()
            != ctx.view.drawn_view(race)
        {
            return Ok(());
        }
    }
    ctx.view.light.other_light.set(0);
    ctx.view.light.light_lowest.set(0);
    ctx.view.light.light_highest.set(0xFF);
    ctx.view
        .light
        .light_level
        .set(race.course.stage_prefs.light_level.get());
    ctx.view
        .object
        .light_strength
        .set(race.course.stage_prefs.light_strength.get());
    ctx.view.light.light_turns[0].set(race.course.stage_prefs.sun_turn.get().wrapping_neg());
    ctx.view.light.light_turns[1].set(race.course.stage_prefs.sun_height.get());
    ctx.view.light.view_kind.set(1);
    ctx.view.light.light_kind.set(2);
    crate::math::cells::copy_cells(&slot.matrix, &ctx.view.matrices.object_turn_matrix, 16);
    let place = [
        slot.place[0].get(),
        slot.place[1]
            .get()
            .wrapping_add(slot.draw_height_offset.get()),
        slot.place[2].get(),
    ];
    for (k, &p) in place.iter().enumerate() {
        ctx.view.object.place[k].set(p);
    }
    matrix::set_place(&ctx.view.matrices.placed_matrix, place);
    matrix::copy_matrix_with_place(
        &ctx.view.matrices.placed_matrix,
        &ctx.view.matrices.view_place_matrix,
        &ctx.view.matrices.matrix_scratch,
    );
    matrix::copy_matrix_with_place(
        &ctx.view.matrices.object_turn_matrix,
        &ctx.view.matrices.matrix_scratch,
        &ctx.view.matrices.turned_matrix,
    );
    matrix::multiply_matrices(
        &ctx.view.matrices.turned_matrix,
        &ctx.view.matrices.view_turn_matrix,
        &ctx.view.matrices.made_object_matrix,
    );
    start_kept_reflections(ctx.view);
    // The object is placed by `g_turnedMatrix`, as the matrices' calls leave it.
    crate::scene::draw_scene_object::draw_scene_object_in_game(
        ctx,
        Some(Description::Floats(&ctx.view.matrices.turned_matrix)),
    )?;
    end_kept_reflections(ctx.view);
    Ok(())
}

/// What `scene::draw_scene_object` would refuse for slot `at`'s object, before anything:
/// a car's object (0x50 .. 0xEF) without its block.
pub fn check_slot_object(
    shared: &RaceShared,
    slot: &crate::race::work_store::RaceSlot,
) -> Result<(), Refused> {
    let n = slot.part.get().wrapping_sub(0x50);
    if n < 0xA0 && shared.work.car_block((n / 0x14) as usize) == 0 {
        return Err(BadInput("a slot's car without its block").into());
    }
    Ok(())
}

/// `race::draw_race_slots` (0x4621E5): `draw_slot_object` for each of the eight race slots (0x78
/// bytes at `g_raceSlots`) whose +0 is not 0. Refuses (before anything) as `check_slot_object` for
/// any.
pub fn draw_race_slots(ctx: &crate::race::view_scratch::DrawCtx<'_>) -> Result<(), Refused> {
    let race = ctx.race;
    let slots = &race.shared.work.race_slots;
    for slot in slots {
        if slot.part.get() != 0 {
            check_slot_object(&race.shared, slot)?;
        }
    }
    for slot in slots {
        if slot.part.get() != 0 {
            draw_slot_object(ctx, slot)?;
        }
    }
    Ok(())
}

/// `race::draw_screen_quad` (0x44B364): a flat rectangle on the screen from x
/// `RACE_VARS.quad_corners` .. `RACE_VARS.quad_corners[1]`, y `RACE_VARS.quad_corners[4]` ..
/// `RACE_VARS.quad_corners[5]` (the four vertices at 0x623E40, 0x20 bytes each, depth
/// `NUMBERS.k4096_b`) in `RACE_VARS.quad_colour`'s colour: the polygon's four work entries set
/// (0x975510, 0x44 bytes each), clipped (`scene::clip_polygon_to_screen`), and with three vertices
/// or more drawn with the depth buffer off (`g_polygonMode` 0, switches -1, 1, 0; `g_polygonDrawn`
/// 1); the colour back to 0xFFFFFF. Refuses what the clip refuses (after the entries are set: never
/// seen with four vertices).
pub fn draw_screen_quad(
    poly: &crate::scene::polygon_work::PolygonWork,
    race: &Race,
    face: &crate::race::face_work::FaceWork,
    scene: &crate::scene::owner::Scene,
    screen: &crate::session::Screen,
    view: &crate::race::view_scratch::ViewScratch,
) -> Result<(), Refused> {
    let course = &race.course;
    let shared = &race.shared;
    poly.polygon_drawn.set(0u32);
    let entries = &POLYGON_WORK.entries[..4];
    for (k, e) in (0u32..).zip(entries) {
        // the quad vertex's record (the original's address of it)
        e.record.set(crate::scene::point_records::of_quad(k));
        e.u.set(0u32);
        e.v.set(0u32);
    }
    // Converted: `rc2k_core::race::loading::quad_corners`.
    let corners = rc2k_core::race::loading::quad_corners(
        view.quad.corners[0].get() as i32,
        view.quad.corners[1].get() as i32,
        view.quad.corners[4].get() as i32,
        view.quad.corners[5].get() as i32,
    );
    for (k, (x, y)) in corners.into_iter().enumerate() {
        let v = &poly.quad_vertices[8 * k..8 * k + 8];
        v[3].set(x.to_bits());
        v[7].set(y.to_bits());
        v[6].set(course.numbers.k4096_b.bits());
    }
    for e in entries {
        e.point.set(u32::MAX);
        e.light.set(0u32);
    }
    poly.polygon_colour.set(shared.vars.quad_colour[0].get());
    poly.polygon_vertex_count.set(4u32);
    poly.polygon_next_record
        .set(crate::scene::polygon::pool_start());
    for k in 0..4u32 {
        POLYGON_WORK.list[k as usize].set(k);
    }
    POLYGON_WORK.list[4].set(0u32);
    poly.polygon_next_entry.set(4u32);
    crate::scene::clip_polygon_to_screen::clip_polygon_to_screen(poly, race)?;
    if poly.polygon_vertex_count.get() >= 3 {
        crate::scene::depth_buffer_on::depth_buffer(false);
        poly.polygon_textures[0].set(u32::MAX);
        face.polygon_mode.set(0u32);
        poly.polygon_sides[0].set(1u32);
        poly.polygon_sides[1].set(0u32);
        crate::scene::draw_polygon::draw_polygon_in_game(poly, race, face, scene, screen)?;
        face.polygon_mode.set(0u32);
        poly.polygon_drawn.set(1u32);
        crate::scene::depth_buffer_on::depth_buffer(true);
    }
    poly.polygon_colour.set(0x00FF_FFFFu32);
    Ok(())
}

/// `race::draw_view_tint` (0x456092): the whole view (0 .. `DRAW.view_size`, 0 ..
/// `DRAW.view_size[1]`) as a screen quad in 0x204080 (`RACE_VARS.quad_colour`), switches -1, 1, 0,
/// by `draw_screen_quad`.
pub fn draw_view_tint(
    poly: &crate::scene::polygon_work::PolygonWork,
    race: &Race,
    face: &crate::race::face_work::FaceWork,
    scene: &crate::scene::owner::Scene,
    screen: &crate::session::Screen,
    view: &crate::race::view_scratch::ViewScratch,
) -> Result<(), Refused> {
    let shared = &race.shared;
    view.quad.corners[0].set(0u32);
    view.quad.corners[1].set(view.polygon.window.view_size[0].get());
    view.quad.corners[4].set(0u32);
    view.quad.corners[5].set(view.polygon.window.view_size[1].get());
    shared.vars.quad_colour[0].set(0x20_4080u32);
    poly.polygon_textures[0].set(u32::MAX);
    poly.polygon_sides[0].set(1u32);
    poly.polygon_sides[1].set(0u32);
    draw_screen_quad(poly, race, face, scene, screen, view)
}

/// `race::add_car_depth_entry` (0x4574A0): the place (+0x134 .., into `OBJECT_WORK.place` ..) of
/// the block of car `car` (its address) through `g_viewMatrix` (rounded into
/// `OBJECT_WORK.turned_place` ..; x and y back as floats, z the key: plus 0x2000000, shifted right
/// 10), a depth entry of the key with the car's `number` plus 0x2000000. Refuses (before anything)
/// a depth entry past the 0x1000.
pub fn add_car_depth_entry(ctx: &DrawCtx, car: u32, number: u32) -> Result<(), Refused> {
    let DrawCtx { view, .. } = *ctx;
    use crate::math::x87::from_int;
    let n: u32 = view.lists.depth_entry_count.get();
    if n >= SORT_MAX {
        return Err(BadInput("a depth entry past the 0x1000").into());
    }
    let place = crate::session::read_bits(&car_view(car)?.position);
    for (k, &p) in place.iter().enumerate() {
        ctx.view.object.place[k].set(p);
    }
    let matrix: [u32; 16] = crate::session::read_bits(&view.matrices.view_matrix);
    let t = crate::math::x87::rotate_rounding(place.map(|v| v as i32), &matrix);
    for (k, &v) in t.iter().enumerate() {
        view.object.turned_place[k].set(v);
    }
    ctx.view.object.place[0].set(from_int(t[0] as i32).to_bits());
    ctx.view.object.place[1].set(from_int(t[1] as i32).to_bits());
    let key = t[2].wrapping_add(0x200_0000) >> 10;
    view.face.depth_key.set(key);
    let second = number.wrapping_add(0x200_0000);
    let at = view.lists.depth_entry(n)?;
    at[0].set(key);
    at[1].set(second);
    view.lists.depth_entry_count.set(n + 1);
    Ok(())
}

/// The tables' 256 entries. The original counts to 0x400, each table running over the next
/// and the last past its end; flares past the tables are not added.
pub const FLARES_MAX: u32 = 0x100;

/// 0x457332: a flare at `OBJECT_WORK.place` .. (object `g_objectNumber`, size
/// `OBJECT_WORK.size`, colour `OBJECT_WORK.other_light`) into the six flare tables (0x100 dwords each, from
/// `LEVEL_GRID.flare_place_x`) and, turned by `g_viewMatrix` (x and y back as floats, z rounded into
/// the key), a depth entry with the flare's number + 0x1000000; none past `FLARES_MAX`.
pub(super) fn add_flare(ctx: &DrawCtx) -> Result<(), BadInput> {
    let DrawCtx { view, .. } = *ctx;
    let n: u32 = view.lamps.flare_count.get();
    if n == FLARES_MAX {
        return Ok(());
    }
    let place = [
        ctx.view.object.place[0].get(),
        ctx.view.object.place[1].get(),
        ctx.view.object.place[2].get(),
    ];
    for (table, v) in [
        (
            &view.lamps.flares.object_number,
            ctx.view.cars.object_number.get(),
        ),
        (&view.lamps.flare_place_x, place[0]),
        (&view.lamps.flares.place_y, place[1]),
        (&view.lamps.flares.place_z, place[2]),
        (&view.lamps.flares.size, ctx.view.object.size.get()),
        (
            &view.lamps.flares.other_light,
            ctx.view.light.other_light.get(),
        ),
    ] {
        table[n as usize].set(v);
    }
    let out = crate::math::x87::rotate_rounding_z(
        place.map(|v| v as i32),
        &crate::session::read_bits(&view.matrices.view_matrix),
    );
    ctx.view.object.place[0].set(out[0]);
    ctx.view.object.place[1].set(out[1]);
    let key = out[2].wrapping_add(0x200_0000) >> 10;
    view.face.depth_key.set(key);
    let entry: u32 = view.lists.depth_entry_count.get();
    let at = view.lists.depth_entry(entry)?;
    at[0].set(key);
    at[1].set(n.wrapping_add(0x100_0000));
    view.lists.depth_entry_count.set(entry + 1);
    view.lamps.flare_count.set(n + 1);
    Ok(())
}

/// 0x4572B0: a flare at `point` in the frame of the car block at `car` (its matrix +0xB4,
/// each part rounded, plus its place +0x134 ..), object 0x39, by `add_flare`.
pub(super) fn place_flare(ctx: &DrawCtx, car: u32, point: [u32; 3]) -> Result<(), BadInput> {
    let t = crate::math::x87::rotate_rounding(
        point.map(|v| v as i32),
        &crate::session::read_bits(&car_view(car)?.crash_matrix),
    );
    for (k, v) in t.iter().enumerate() {
        ctx.view.object.place[k].set(v.wrapping_add(car_view(car)?.position[k].get()));
    }
    ctx.view.cars.object_number.set(0x39u32);
    add_flare(ctx)?;
    Ok(())
}

/// The car block's part switches' place (+0xAC0), from which `light_pair`'s byte offsets
/// count.
pub(super) const PART_SWITCHES: u32 = 0xAC0;

/// 0x457241: a pair of lights at the light record's +`at` (three dwords: none
/// when all are 0) for the car block at `car`: the right one (x as it is) when the block's byte
/// +`right` is set, the left (x negated) when +`left` is (-1: no byte), each by
/// `place_flare`; `race().frame.frame_work.light_left` / `race().frame.frame_work.light_right` the two bytes.
pub(super) fn light_pair(
    ctx: &DrawCtx,
    car: u32,
    record: u32,
    at: u32,
    left: u32,
    right: u32,
) -> Result<(), BadInput> {
    let race = ctx.race;
    // The bytes are the block's part switches (+0xAC0 ..).
    let switches = &car_at(car)?.part_switches;
    let byte = |o: u32| -> Result<u32, BadInput> {
        let n = o.wrapping_sub(PART_SWITCHES) as usize;
        if n >= switches.atomics().len() {
            return Err(BadInput("a light byte past the part switches"));
        }
        Ok(u32::from(switches.get(n)))
    };
    ctx.view.cars.light_left.set(0u32);
    ctx.view.cars.light_right.set(0u32);
    if left != u32::MAX {
        ctx.view.cars.light_left.set(byte(left)?);
    }
    if right != u32::MAX {
        ctx.view.cars.light_right.set(byte(right)?);
    }
    let [x, y, z]: [u32; 3] = crate::pointers::load_value(
        &race.frame.lights_and_gear,
        record.wrapping_add(at) as usize,
    )?;
    if x | y | z == 0 {
        return Ok(());
    }
    let y = y.wrapping_sub(race.shared.physics_tables.ground_drop.get());
    if ctx.view.cars.light_right.get() != 0 {
        place_flare(ctx, car, [x, y, z])?;
    }
    if ctx.view.cars.light_left.get() != 0 {
        place_flare(ctx, car, [x.wrapping_neg(), y, z])?;
    }
    Ok(())
}

/// `race::car_light_flares` (0x457036): the lights of the car block at `car` as flares. Refuses
/// (before anything) a flare list or depth list without room for the 24 flares a car can give.
pub fn car_light_flares(ctx: &DrawCtx, car: u32) -> Result<(), Refused> {
    let DrawCtx {
        race, scene, view, ..
    } = *ctx;
    if view.lists.depth_entry_count.get() > SORT_MAX - 24 {
        return Err(BadInput("the depth list has no room for a car's flares").into());
    }
    if race.shared.switches.mask_kind.get() == 8 {
        ctx.view.object.size.set(2u32);
        let point = [
            race.frame.frame_work.part_place[0].get(),
            race.frame.frame_work.part_place[1]
                .get()
                .wrapping_sub(race.shared.physics_tables.ground_drop.get()),
            race.frame.frame_work.part_place[2].get(),
        ];
        ctx.view.light.other_light.set(0x00F0_F0F0u32);
        place_flare(ctx, car, point)?;
        // Not the original's: the pair's other light, as `light_pair` (the debug UI).
        if point[0] != 0 {
            place_flare(ctx, car, [point[0].wrapping_neg(), point[1], point[2]])?;
        }
        return Ok(());
    }
    if race.shared.switches.scene_set_up.get() == 0 {
        return Ok(());
    }
    let kind = car_view(car)?.car_kind.get();
    let valid = kind < 0x22 && KIND_TABLES.kind_shows_parts[kind as usize].get() != 0;
    if valid {
        if !crate::math::x87::below_or_unordered(
            car_view(car)?.electrics_damage.bits(),
            race.course.numbers.hundred.bits(),
        ) {
            return Ok(());
        }
        ctx.view
            .cars
            .parts_present
            .set(car_view(car)?.parts_present.get());
        // A second test of the same kind follows; its error branch (a log line) is
        // never reached.
    }
    // 0x41F4E1: the kind held to 0 ..= 0x21 (by sign), through `KIND_TABLES.kind_light_records`.
    let index = KIND_TABLES.kind_light_records[(kind as i32).clamp(0, 0x21) as usize].get();
    // The index is not bounded: the original reads on past the 34 records into the gear
    // figures (the group `lights_and_gear` bounds the read).
    let record = (core::mem::offset_of!(crate::race::groups_store::LightsAndGear, light_records)
        as u32)
        .wrapping_add(index.wrapping_mul(0x134));
    ctx.view.light.view_kind.set(1u32);
    ctx.view.light.light_kind.set(3u32);
    ctx.view.object.size.set(2u32);
    ctx.view.light.other_light.set(0x0060_6060u32);
    let pair = |at: u32, left: u32, right: u32| light_pair(ctx, car, record, at, left, right);
    pair(0xA4, 0xAC0, 0xAC1)?;
    if scene.lights.work.marks_mask.get() != 0 && ctx.view.cars.parts_present.get() >> 9 & 1 != 0 {
        pair(0xB0, 0xACA, 0xACB)?;
        pair(0xBC, 0xACC, 0xACD)?;
        pair(0xC8, 0xACE, 0xACF)?;
    }
    // 0x45721A: no left light for kind 0xD.
    let left = if kind == 0xD { u32::MAX } else { 0xAC6 };
    pair(0xF8, left, 0xAC7)?;
    ctx.view.light.other_light.set(0x00C0_0000u32);
    // 0x457229: none for kinds 0xD and 0xE.
    let left = if kind == 0xD || kind == 0xE {
        u32::MAX
    } else {
        0xAC8
    };
    pair(0x104, left, 0xAC9)?;
    ctx.view.light.other_light.set(0x0080_0000u32);
    pair(0xD4, 0xAC4, 0xAC5)?;
    pair(0xE0, 0xAD0, u32::MAX)?;
    pair(0xEC, 0xAC2, 0xAC3)?;
    Ok(())
}

/// `race::car_lights` (0x455CFF): with `FRAME_SWITCHES.scene_set_up` set, for each race car: with
/// +0x6FC bit 5 (lights on, `car_light_modes`) its depth entry (`add_car_depth_entry`) and then
/// `car_light_flares`. Refuses (before anything) no cars (the loop tests at its end) or more than
/// eight, a null block, or a depth list without room for every car's entries.
pub fn car_lights(ctx: &DrawCtx) -> Result<(), Refused> {
    let race = ctx.race;
    if race.shared.switches.scene_set_up.get() == 0 {
        return Ok(());
    }
    // Converted: `rc2k_core::race::lights::{check_car_lights, lights_on}` (the null
    // block's refusal between its two, as before).
    use rc2k_core::race::lights::{check_car_lights, lights_on};
    let cars: u32 = race.shared.view.race_cars.get();
    check_car_lights(cars, 0, SORT_MAX)?;
    let block = |n: u32| race.shared.work.car_block(n as usize);
    if (0..cars).any(|n| block(n) == 0) {
        return Err(BadInput("a null car block").into());
    }
    check_car_lights(cars, ctx.view.lists.depth_entry_count.get(), SORT_MAX)?;
    for n in 0..cars {
        let b = block(n);
        // Both only with the lights on: the `jz` skips to the `popa`.
        if lights_on(ctx.view.cars.state_of(b)?.get()) {
            add_car_depth_entry(ctx, b, n)?;
            car_light_flares(ctx, b)?;
        }
    }
    Ok(())
}

/// The end of `place_object_matrix`'s ways: the place `OBJECT_WORK.place` .. into
/// `g_placedMatrix`, moved by the view (0x5EA1CC into `ctx.view.matrices.matrix_scratch`), `ctx.view.matrices.object_turn_matrix` with it
/// (`g_turnedMatrix`), times the view's turn into `g_madeObjectMatrix`.
pub(super) fn placed_object_tail(ctx: &DrawCtx) {
    use crate::scene::matrix;
    let place = [
        ctx.view.object.place[0].get(),
        ctx.view.object.place[1].get(),
        ctx.view.object.place[2].get(),
    ];
    matrix::set_place(&ctx.view.matrices.placed_matrix, place);
    matrix::copy_matrix_with_place(
        &ctx.view.matrices.placed_matrix,
        &ctx.view.matrices.view_place_matrix,
        &ctx.view.matrices.matrix_scratch,
    );
    matrix::copy_matrix_with_place(
        &ctx.view.matrices.object_turn_matrix,
        &ctx.view.matrices.matrix_scratch,
        &ctx.view.matrices.turned_matrix,
    );
    matrix::multiply_matrices(
        &ctx.view.matrices.turned_matrix,
        &ctx.view.matrices.view_turn_matrix,
        &ctx.view.matrices.made_object_matrix,
    );
}

/// `ctx.view.matrices.object_turn_matrix`'s sixteen floats times `OBJECT_WORK.size` / 16.
pub(super) fn scale_object_turn(ctx: &DrawCtx) -> Result<(), BadInput> {
    {
        let scale = crate::math::x87::int_over_16(ctx.view.object.size.get() as i32);
        for cell in &ctx.view.matrices.object_turn_matrix {
            cell.set(crate::math::x87::mul(cell, scale));
        }
    }
    Ok(())
}

/// `race::place_object_matrix` (0x42739A): `g_madeObjectMatrix` for an object placed at
/// `OBJECT_WORK.place` .. (after `scene::set_light_direction`). With `LIGHT_STATE.light_turn_off`
/// 0: turned about y, x and z (16 bits a turn) by `OBJECT_WORK.angles[1]`, `OBJECT_WORK.angles`,
/// `OBJECT_WORK.angles[2]` negated, unscaled. Else (nothing more with `LIGHT_STATE.view_kind` 4) by
/// the object's material (+0x20 of its kind's 0x80-byte record at `g_objectMaterials`): 1 and 2
/// facing the view (`g_viewBackMatrix`); 5 turned about y by
/// `race().frame.view_and_records.view_vars.view_angles[3]` and the view's y angle (fine turns);
/// else about y by -`OBJECT_WORK.angles`, then with `OBJECT_WORK.angles[1]` tilted about x round
/// the y of `OBJECT_WORK.angles[2]`, and with `OBJECT_WORK.described_cleared[1]` /
/// `OBJECT_WORK.described_cleared[2]` again round `OBJECT_WORK.described_cleared`; then scaled by
/// `OBJECT_WORK.size` / 16.
pub fn place_object_matrix(ctx: &crate::race::view_scratch::DrawCtx<'_>) -> Result<(), BadInput> {
    let (race, scene) = (ctx.race, ctx.scene);
    let frame = &race.frame;
    use crate::scene::matrix::{
        multiply_matrices, set_light_direction, set_turn_x, set_turn_y, set_turn_z_16,
    };
    use crate::scene::view_matrix::{Axis, set_fine_turn};
    let (turn_y, turn_x, turn_z) = (
        &ctx.view.matrices.turn_y_matrix,
        &ctx.view.matrices.turn_x_matrix,
        &ctx.view.matrices.turn_z_matrix,
    );
    let (turned, lit, object) = (
        &ctx.view.matrices.turned_matrix,
        &ctx.view.matrices.lit_matrix,
        &ctx.view.matrices.object_turn_matrix,
    );
    set_light_direction(
        &ctx.view.matrices,
        &ctx.view.light.light_direction,
        &ctx.view.matrices.view_turn_matrix,
        &ctx.view.light,
    );
    if scene.lights.state.light_turn_off.get() == 0 {
        set_turn_y(
            &ctx.view.matrices.turn_y_matrix,
            &ctx.view.matrices.turn_y_work,
            ctx.view.object.angles[1].get().wrapping_neg(),
        );
        set_turn_x(
            &ctx.view.matrices.turn_x_matrix,
            &ctx.view.matrices.turn_x_work,
            ctx.view.object.angles[0].get().wrapping_neg(),
        );
        multiply_matrices(turn_y, turn_x, turned);
        set_turn_z_16(
            &ctx.view.matrices.turn_z_matrix,
            &ctx.view.matrices.turn_z_work,
            ctx.view.object.angles[2].get().wrapping_neg(),
        );
        multiply_matrices(turned, turn_z, object);
        placed_object_tail(ctx);
        return Ok(());
    }
    if ctx.view.light.view_kind.get() == 4 {
        return Ok(());
    }
    let number = ctx.view.cars.object_number.get();
    let kind = scene.objects.tables.object_kind(number);
    let material = crate::scene::polygon_store::POLYGON_TABLES
        .object_materials
        .get(kind as usize)
        .ok_or_else(|| {
            crate::framework::log(&format!(
                "race::place_object_matrix: object {number:#x} has kind {kind:#x}, past the 1200 materials"
            ));
            BadInput("an object kind past the 1200 materials")
        })?
        .words[8]
        .get() as u8; // the record's byte +0x20
    match material {
        1 | 2 => {
            crate::math::cells::copy_cells(&ctx.view.matrices.view_back_matrix, &object, 16);
        }
        5 => {
            crate::math::cells::copy_cells(&frame.matrices.identity_matrix, &object, 16);
            set_fine_turn(
                &race.course,
                &ctx.view.matrices,
                Axis::Y,
                ctx.view.state.view_angles[3].get(),
            );
            multiply_matrices(object, turn_y, turned);
            set_fine_turn(
                &race.course,
                &ctx.view.matrices,
                Axis::Y,
                ctx.view.state.view_angles[0].get(),
            );
            multiply_matrices(turned, turn_y, object);
        }
        _ => {
            set_turn_y(
                &ctx.view.matrices.turn_y_matrix,
                &ctx.view.matrices.turn_y_work,
                ctx.view.object.angles[0].get().wrapping_neg(),
            );
            crate::math::cells::copy_cells(&turn_y, &object, 16);
            if ctx.view.object.angles[1].get() != 0 {
                set_turn_y(
                    &ctx.view.matrices.turn_y_matrix,
                    &ctx.view.matrices.turn_y_work,
                    ctx.view.object.angles[2].get().wrapping_neg(),
                );
                multiply_matrices(object, turn_y, lit);
                set_turn_x(
                    &ctx.view.matrices.turn_x_matrix,
                    &ctx.view.matrices.turn_x_work,
                    ctx.view.object.angles[1].get().wrapping_neg(),
                );
                multiply_matrices(lit, turn_x, turned);
                set_turn_y(
                    &ctx.view.matrices.turn_y_matrix,
                    &ctx.view.matrices.turn_y_work,
                    ctx.view.object.angles[2].get(),
                );
                multiply_matrices(turned, turn_y, object);
            }
            if ctx.view.object.described_turns[2].get() != 0
                || ctx.view.object.described_turns[1].get() != 0
            {
                set_turn_y(
                    &ctx.view.matrices.turn_y_matrix,
                    &ctx.view.matrices.turn_y_work,
                    ctx.view.object.described_turns[2].get().wrapping_neg(),
                );
                multiply_matrices(object, turn_y, lit);
                set_turn_y(
                    &ctx.view.matrices.turn_y_matrix,
                    &ctx.view.matrices.turn_y_work,
                    ctx.view.object.described_turns[0].get().wrapping_neg(),
                );
                multiply_matrices(lit, turn_y, turned);
                set_turn_x(
                    &ctx.view.matrices.turn_x_matrix,
                    &ctx.view.matrices.turn_x_work,
                    ctx.view.object.described_turns[1].get().wrapping_neg(),
                );
                multiply_matrices(turned, turn_x, lit);
                set_turn_y(
                    &ctx.view.matrices.turn_y_matrix,
                    &ctx.view.matrices.turn_y_work,
                    ctx.view.object.described_turns[0].get(),
                );
                multiply_matrices(lit, turn_y, object);
            }
        }
    }
    scale_object_turn(ctx)?;
    placed_object_tail(ctx);
    Ok(())
}

/// `race::draw_flare` (0x455189): flare `flare` of the six flare tables back into
/// `OBJECT_WORK.place` .., `OBJECT_WORK.size`, `OBJECT_WORK.other_light`, `g_objectNumber`;
/// `LIGHT_STATE.view_kind` 2, `place_object_matrix`, `g_lightKind` 0 and the drawing
/// (`scene::draw_scene_object`, described by `entry`, the draw list entry the flare came from).
pub fn draw_flare(
    ctx: &crate::race::view_scratch::DrawCtx<'_>,
    flare: u32,
    entry: Description<'_>,
) -> Result<(), Refused> {
    for (table, to) in [
        (&ctx.view.lamps.flare_place_x, &ctx.view.object.place[0]),
        (&ctx.view.lamps.flares.place_y, &ctx.view.object.place[1]),
        (&ctx.view.lamps.flares.place_z, &ctx.view.object.place[2]),
        (&ctx.view.lamps.flares.size, &ctx.view.object.size),
        (
            &ctx.view.lamps.flares.other_light,
            &ctx.view.light.other_light,
        ),
        (
            &ctx.view.lamps.flares.object_number,
            &ctx.view.cars.object_number,
        ),
    ] {
        to.set(
            table
                .get(flare as usize)
                .ok_or(BadInput("a flare past the tables"))?
                .get(),
        );
    }
    ctx.view.light.view_kind.set(2u32);
    place_object_matrix(ctx)?;
    ctx.view.light.light_kind.set(0u32);
    crate::scene::draw_scene_object::draw_scene_object_in_game(ctx, Some(entry))
}

/// `race::draw_lamp_flare` (0x4551F4): lamp `lamp` (`g_lamps`): its point (`g_scenePoints`) into
/// `OBJECT_WORK.place` .., its +4 x 8 the size, +8 the colour, object
/// `FRAME_SWITCHES.marker_object`; `LIGHT_STATE.view_kind` 2, `place_object_matrix`, `g_lightKind`
/// 0 and the drawing.
pub fn draw_lamp_flare(
    ctx: &crate::race::view_scratch::DrawCtx<'_>,
    lamp: u32,
) -> Result<(), Refused> {
    let race = ctx.race;
    let edi = lights::own_lamp(&ctx.view.lamps, lamp)?;
    let [x, y, z] = crate::race::scene_points::point_number(&race.course, edi[0].get())?;
    ctx.view.object.place[0].set(x);
    ctx.view.object.place[1].set(y);
    ctx.view.object.place[2].set(z);
    ctx.view.object.size.set(edi[1].get() << 3);
    ctx.view.light.other_light.set(edi[2].get());
    ctx.view
        .cars
        .object_number
        .set(race.shared.switches.marker_object.get());
    ctx.view.light.view_kind.set(2u32);
    place_object_matrix(ctx)?;
    ctx.view.light.light_kind.set(0u32);
    // The object is placed by `g_turnedMatrix`, as `place_object_matrix` leaves it (`LIGHT_STATE.view_kind` is 2 here,
    // so it always ends by placing the object).
    crate::scene::draw_scene_object::draw_scene_object_in_game(
        ctx,
        Some(Description::Floats(&ctx.view.matrices.turned_matrix)),
    )
}

/// `race::draw_list_entry` (0x454FF0): the entry whose cells from it on are `at`, whose second word
/// is `flags_word` (the original kept it in `g_polygonFlags`, which nothing reads to effect: not
/// kept; the top bit dropped for the kind): below 0x1000000 a course face (`g_currentCourseFace`):
/// kind 0xFF part 1, below 0x10 part 2, each with `LEVEL_GRID.smooth` .. `LEVEL_GRID.plain_second`
/// 1 around it; 0x1000000 .. a flare, 0x2000000 .. a car, 0x3000000 .. a lamp, each by its low 24
/// bits; then `LIGHT_STATE.view_kind` 0. Refuses (before anything) a word of 0x4000000 or more (the
/// original would take it for a face far past the table).
pub fn draw_list_entry(
    ctx: &crate::race::view_scratch::DrawCtx<'_>,
    at: Description<'_>,
    flags_word: u32,
) -> Result<(), Refused> {
    let word = flags_word & 0x7FFF_FFFF;
    if word >= 0x400_0000 {
        return Err(BadInput("a list entry of 0x4000000 or more").into());
    }
    let low = word & 0xFF_FFFF;
    match word >> 24 {
        0 => {
            ctx.view.face.course_face.set(word);
            let face = word;
            let kind = crate::race::course_places::face_field::<u8>(&ctx.race.course, face, 0)?;
            if kind == 0xFF || kind < 0x10 {
                ctx.view.polygon.smooth.set(1);
                ctx.view.polygon.plain_first.set(1);
                ctx.view.polygon.plain_second.set(1);
                if kind == 0xFF {
                    // The description is the face.
                    crate::scene::draw_described_object::draw_described_face(ctx, face)?;
                } else {
                    crate::race::course_faces::draw_tinted_face(ctx, face)?;
                }
                ctx.view.polygon.smooth.set(0);
                ctx.view.polygon.plain_first.set(0);
                ctx.view.polygon.plain_second.set(0);
            }
        }
        1 => draw_flare(ctx, low, at)?,
        2 => crate::race::car_draw::draw_car_entry(ctx, low)?,
        _ => draw_lamp_flare(ctx, low)?,
    }
    ctx.view.light.view_kind.set(0u32);
    Ok(())
}

/// `race::draw_sorted_entries` (0x454EAB) / `race::draw_depth_entries`: each entry of the list
/// (`count` its count, `entries` its 8-byte entries) by `draw_list_entry`. Refuses (before
/// anything) what `draw_list_entry` would for any. `from(n)` the list's cells from entry `n` on (a
/// flare's entry is read there as a description, past its own 8 bytes).
pub fn draw_entries<'a>(
    ctx: &crate::race::view_scratch::DrawCtx<'_>,
    count: &crate::session::Dword,
    from: impl Fn(usize) -> &'a [crate::session::Dword] + Sync,
) -> Result<(), Refused> {
    let entries = from(0)
        .get(..2 * count.get() as usize)
        .ok_or(BadInput("a draw list entry past the list"))?;
    let (entries, _) = entries.as_chunks::<2>();
    if entries
        .iter()
        .any(|entry| entry[1].get() & 0x7FFF_FFFF >= 0x400_0000)
    {
        return Err(BadInput("a list entry of 0x4000000 or more").into());
    }
    for (k, entry) in entries.iter().enumerate() {
        draw_list_entry(ctx, Description::Cells(from(k)), entry[1].get())?;
    }
    Ok(())
}
