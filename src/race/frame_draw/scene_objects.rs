//! The scene's objects: the kept reflections, the race slots, screen quads, flares, car lights and the drawing of the sorted entries.

use super::*;
use crate::errors::{BadInput, Refused};
use crate::physics::tables_store::PHYSICS;
use crate::pointers::GamePtr;
use crate::race::kind_store::KIND_TABLES;
use crate::race::light_records_store::LIGHT_RECORDS;
use crate::race::matrices_store::SceneMatrices;
use crate::race::owner::race;
use crate::race::switches_store::FRAME_SWITCHES;
use crate::race::trap::car_at;
use crate::race::vars_store::RACE_VARS;
use crate::race::view_store::RACE_VIEW;
use crate::race::work_store::RACE_WORK;
use crate::scene::draw_store::DRAW;
use crate::scene::light_store::LIGHT_STATE;
use crate::scene::light_work_store::LIGHT_WORK;
use crate::scene::object_store::OBJECT_WORK;
use crate::scene::object_tables_store::OBJECT_TABLES;
use crate::scene::polygon_store::POLYGON_WORK;

/// `race::start_kept_reflections` (0x451864): the 0x400 kept reflection normals cleared to
/// -999 (0xFFFFFC19) and `g_reflectionsKept` 1.
pub fn start_kept_reflections() -> Result<(), BadInput> {
    {
        for cell in &race().physics.ground.kept_reflections[..0x400] {
            cell.set(0xFFFF_FC19);
        }
        RACE_VIEW.reflections_kept.set_bits(1u32);
    }
    Ok(())
}

/// `race::end_kept_reflections` (0x451881): `g_reflectionsKept` 0.
pub fn end_kept_reflections() {
    RACE_VIEW.reflections_kept.set_bits(0u32);
}

/// `race::draw_slot_object` (0x4626D1): the race slot's object (+0, into `g_objectNumber`)
/// drawn, unless outside one player and modes 0 and 2 its car (object 0x50 + 0x14 x car)
/// is not the drawn view's (`race().frame.frame_work.listed_car_sounds`): the light (lowest 0, highest 0xFF, level
/// `STAGE_PREFS.light_level`, strength `STAGE_PREFS.light_strength`, turns -`STAGE_PREFS.sun_turn` / `STAGE_PREFS.sun_height`,
/// `LIGHT_STATE.view_kind` 1, `g_lightKind` 2), its turn (+0x10, 16 dwords to `race().frame.matrices.object_turn_matrix`) placed
/// at +4, +8 + (+0x5C), +0xC (into `OBJECT_WORK.place` ..) and moved by the view (0x5EA1CC,
/// into `race().frame.matrices.matrix_scratch`), times the view's turn into `g_madeObjectMatrix`; the reflections
/// kept around the drawing (`scene::draw_scene_object`). Refuses
/// (before anything) as `check_slot_object`.
pub fn draw_slot_object(slot: &crate::race::work_store::RaceSlot) -> Result<(), Refused> {
    let race = race();
    use crate::scene::matrix;
    check_slot_object(slot)?;
    let object = slot.part.get();
    FRAME_SWITCHES.object_number.set(object);
    let mode: u32 = RACE_VARS.run_kind.bits();
    if RACE_VIEW.players.bits() != 1 && mode != 0 && mode != 2 {
        let n = object.wrapping_sub(0x50);
        if race.frame.frame_work.listed_car_sounds[((n / 0x14) & 7) as usize].get()
            != RACE_VIEW.drawn_view.bits()
        {
            return Ok(());
        }
    }
    OBJECT_WORK.other_light.set_bits(0);
    OBJECT_WORK.light_lowest.set_bits(0);
    OBJECT_WORK.light_highest.set_bits(0xFF);
    OBJECT_WORK
        .light_level
        .set_bits(race.course.stage_prefs.light_level.bits());
    OBJECT_WORK
        .light_strength
        .set_bits(race.course.stage_prefs.light_strength.bits());
    OBJECT_WORK.light_turns[0].set_bits(race.course.stage_prefs.sun_turn.get().wrapping_neg());
    OBJECT_WORK.light_turns[1].set_bits(race.course.stage_prefs.sun_height.bits());
    LIGHT_STATE.view_kind.set_bits(1);
    LIGHT_WORK.light_kind.set_bits(2);
    crate::math::cells::copy_cells(&slot.matrix, &race.frame.matrices.object_turn_matrix, 16);
    let place = [
        slot.place[0].get(),
        slot.place[1].get().wrapping_add(slot.unknown_5c.get()),
        slot.place[2].get(),
    ];
    for (k, &p) in place.iter().enumerate() {
        OBJECT_WORK.place[k].set_bits(p);
    }
    matrix::set_place(place);
    matrix::copy_matrix_with_place(
        &race.frame.matrices.placed_matrix,
        &race.frame.matrices.view_place_matrix,
        &race.frame.matrices.matrix_scratch,
    );
    matrix::copy_matrix_with_place(
        &race.frame.matrices.object_turn_matrix,
        &race.frame.matrices.matrix_scratch,
        &race.frame.matrices.turned_matrix,
    );
    matrix::multiply_matrices(
        &race.frame.matrices.turned_matrix,
        &race.frame.matrices.view_turn_matrix,
        &race.frame.matrices.made_object_matrix,
    );
    start_kept_reflections()?;
    // esi `g_turnedMatrix`, as the matrices' calls leave it.
    crate::scene::draw_scene_object::draw_scene_object_in_game(Some(matrices_at(
        core::mem::offset_of!(SceneMatrices, turned_matrix),
    )))?;
    end_kept_reflections();
    Ok(())
}

/// What `scene::draw_scene_object` would refuse for slot `at`'s object, before anything:
/// a car's object (0x50 .. 0xEF) without its block.
pub fn check_slot_object(slot: &crate::race::work_store::RaceSlot) -> Result<(), Refused> {
    let n = slot.part.get().wrapping_sub(0x50);
    if n < 0xA0 && RACE_WORK.car_block((n / 0x14) as usize) == 0 {
        return Err(BadInput("a slot's car without its block").into());
    }
    Ok(())
}

/// `race::draw_race_slots` (0x4621E5): `draw_slot_object` for each of the eight race slots
/// (0x78 bytes at `g_raceSlots`) whose +0 is not 0. Refuses (before anything) as
/// `check_slot_object` for any.
pub fn draw_race_slots() -> Result<(), Refused> {
    let slots = &RACE_WORK.race_slots;
    for slot in slots {
        if slot.part.get() != 0 {
            check_slot_object(slot)?;
        }
    }
    for slot in slots {
        if slot.part.get() != 0 {
            draw_slot_object(slot)?;
        }
    }
    Ok(())
}

/// `race::draw_screen_quad` (0x44B364): a flat rectangle on the screen from x
/// `RACE_VARS.quad_corners` .. `RACE_VARS.quad_corners[1]`, y `RACE_VARS.quad_corners[4]` .. `RACE_VARS.quad_corners[5]` (the four vertices
/// at 0x623E40, 0x20 bytes each, depth `NUMBERS.k4096_b`) in `RACE_VARS.quad_colour`'s colour: the
/// polygon's four work entries set (0x975510, 0x44 bytes each), clipped
/// (`scene::clip_polygon_to_screen`), and with three vertices or more drawn with the depth
/// buffer off (`g_polygonMode` 0, switches -1, 1, 0; `g_polygonDrawn` 1); the colour back
/// to 0xFFFFFF. Every register kept (its callers return at once). Refuses what the clip
/// refuses (after the entries are set: never seen with four vertices).
pub fn draw_screen_quad() -> Result<(), Refused> {
    DRAW.polygon_drawn.set(0u32);
    let entries = &POLYGON_WORK.entries[..4];
    for (k, e) in (0u32..).zip(entries) {
        // the quad vertex's record (the original's address of it)
        e.record.set(crate::scene::point_records::of_quad(k));
        e.u.set(0u32);
        e.v.set(0u32);
    }
    // Converted: `rc2k_core::race::loading::quad_corners`.
    let corners = rc2k_core::race::loading::quad_corners(
        RACE_VARS.quad_corners[0].bits() as i32,
        RACE_VARS.quad_corners[1].bits() as i32,
        RACE_VARS.quad_corners[4].bits() as i32,
        RACE_VARS.quad_corners[5].bits() as i32,
    );
    for (k, (x, y)) in corners.into_iter().enumerate() {
        let v = &RACE_VARS.quad_vertices[8 * k..8 * k + 8];
        v[3].set_bits(x.to_bits());
        v[7].set_bits(y.to_bits());
        v[6].set_bits(race().course.numbers.k4096_b.bits());
    }
    DRAW.scratch.set_bits(corners[2].1.to_bits());
    for e in entries {
        e.point.set(u32::MAX);
        e.light.set(0u32);
    }
    DRAW.polygon_colour.set(RACE_VARS.quad_colour[0].get());
    DRAW.polygon_vertex_count.set(4u32);
    DRAW.polygon_next_record
        .set(crate::scene::polygon::pool_start());
    for k in 0..4u32 {
        POLYGON_WORK.list[k as usize].set(k);
    }
    POLYGON_WORK.list[4].set(0u32);
    DRAW.polygon_next_entry.set(4u32);
    // No callee reads a register (the clip keeps them; the depth buffer's switches and
    // the polygon's drawing take none).
    crate::scene::clip_polygon_to_screen::clip_polygon_to_screen()?;
    if DRAW.polygon_vertex_count.get() >= 3 {
        crate::scene::depth_buffer_on::depth_buffer(false);
        DRAW.polygon_textures[0].set(u32::MAX);
        RACE_VIEW.polygon_mode.set(0u32);
        DRAW.polygon_sides[0].set(1u32);
        DRAW.polygon_sides[1].set(0u32);
        crate::scene::draw_polygon::draw_polygon_in_game()?;
        RACE_VIEW.polygon_mode.set(0u32);
        DRAW.polygon_drawn.set(1u32);
        crate::scene::depth_buffer_on::depth_buffer(true);
    }
    DRAW.polygon_colour.set(0x00FF_FFFFu32);
    Ok(())
}

/// `race::draw_view_tint` (0x456092): the whole view (0 .. `DRAW.view_size`, 0 ..
/// `DRAW.view_size[1]`) as a screen quad in 0x204080 (`RACE_VARS.quad_colour`), switches -1, 1, 0, by
/// `draw_screen_quad`.
pub fn draw_view_tint() -> Result<(), Refused> {
    RACE_VARS.quad_corners[0].set(0u32);
    RACE_VARS.quad_corners[1].set(DRAW.view_size[0].get());
    RACE_VARS.quad_corners[4].set(0u32);
    RACE_VARS.quad_corners[5].set(DRAW.view_size[1].get());
    RACE_VARS.quad_colour[0].set(0x20_4080u32);
    DRAW.polygon_textures[0].set(u32::MAX);
    DRAW.polygon_sides[0].set(1u32);
    DRAW.polygon_sides[1].set(0u32);
    draw_screen_quad()
}

/// `race::add_car_depth_entry` (0x4574A0): the place (+0x134 .., into `OBJECT_WORK.place` ..) of
/// the block of car `car` (its address) through `g_viewMatrix` (rounded into `OBJECT_WORK.turned_place`
/// ..; x and y back as floats, z the key: plus 0x2000000, shifted right 10), a depth entry
/// of the key with the car's `number` plus 0x2000000. Refuses (before anything) a depth
/// entry past the 0x1000.
pub fn add_car_depth_entry(car: u32, number: u32) -> Result<(), Refused> {
    use crate::math::x87::from_int;
    let n: u32 = RACE_VIEW.depth_entry_count.bits();
    if n >= SORT_MAX {
        return Err(BadInput("a depth entry past the 0x1000").into());
    }
    let place = crate::session::read_bits(&car_at(car)?.position);
    for (k, &p) in place.iter().enumerate() {
        OBJECT_WORK.place[k].set_bits(p);
    }
    let matrix: [u32; 16] = crate::session::read_bits(&race().frame.matrices.view_matrix);
    let t = crate::math::x87_load::rotate_rounding(place.map(|v| v as i32), &matrix);
    for (k, &v) in t.iter().enumerate() {
        OBJECT_WORK.turned_place[k].set_bits(v);
    }
    OBJECT_WORK.place[0].set(from_int(t[0] as i32));
    OBJECT_WORK.place[1].set(from_int(t[1] as i32));
    let key = t[2].wrapping_add(0x200_0000) >> 10;
    RACE_VIEW.face_depth_key.set_bits(key);
    let second = number.wrapping_add(0x200_0000);
    let at = depth_entry(n)?;
    at[0].set(key);
    at[1].set(second);
    RACE_VIEW.depth_entry_count.set_bits(n + 1);
    Ok(())
}

/// The tables' 256 entries. The original counts to 0x400, each table running over the next
/// and the last past its end; flares past the tables are not added.
pub const FLARES_MAX: u32 = 0x100;

/// `sub_457332`: a flare at `OBJECT_WORK.place` .. (object `g_objectNumber`, size
/// `OBJECT_WORK.size`, colour `OBJECT_WORK.other_light`) into the six flare tables (0x100 dwords each, from
/// `LEVEL_GRID.flare_place_x`) and, turned by `g_viewMatrix` (x and y back as floats, z rounded into
/// the key), a depth entry with the flare's number + 0x1000000; none past `FLARES_MAX`.
pub(super) fn add_flare() -> Result<(), BadInput> {
    let race = race();
    let n: u32 = race.frame.frame_work.flare_count.bits();
    if n == FLARES_MAX {
        return Ok(());
    }
    let place = [
        OBJECT_WORK.place[0].get(),
        OBJECT_WORK.place[1].bits(),
        OBJECT_WORK.place[2].bits(),
    ];
    for (table, v) in [
        (
            &race.course.flare_tables.object_number,
            FRAME_SWITCHES.object_number.get(),
        ),
        (&race.course.level_grid.flare_place_x, place[0]),
        (&race.course.flare_tables.place_y, place[1]),
        (&race.course.flare_tables.place_z, place[2]),
        (&race.course.flare_tables.size, OBJECT_WORK.size.bits()),
        (
            &race.course.flare_tables.other_light,
            OBJECT_WORK.other_light.bits(),
        ),
    ] {
        table[n as usize].set(v);
    }
    let out = crate::math::x87_load::rotate_rounding_z(
        place.map(|v| v as i32),
        &crate::session::read_bits(&race.frame.matrices.view_matrix),
    );
    OBJECT_WORK.place[0].set(out[0]);
    OBJECT_WORK.place[1].set(out[1]);
    let key = out[2].wrapping_add(0x200_0000) >> 10;
    RACE_VIEW.face_depth_key.set_bits(key);
    let entry: u32 = RACE_VIEW.depth_entry_count.bits();
    let at = depth_entry(entry)?;
    at[0].set(key);
    at[1].set(n.wrapping_add(0x100_0000));
    RACE_VIEW.depth_entry_count.set_bits(entry + 1);
    race.frame.frame_work.flare_count.set_bits(n + 1);
    Ok(())
}

/// `sub_4572B0`: a flare at `point` in the frame of the car block at `car` (its matrix +0xB4,
/// each part rounded, plus its place +0x134 ..), object 0x39, by `add_flare`.
pub(super) fn place_flare(car: u32, point: [u32; 3]) -> Result<(), BadInput> {
    let t = crate::math::x87_load::rotate_rounding(
        point.map(|v| v as i32),
        &crate::session::read_bits(&car_at(car)?.crash_matrix),
    );
    DRAW.scratch.set_bits(t[2]);
    for (k, v) in t.iter().enumerate() {
        OBJECT_WORK.place[k].set_bits(v.wrapping_add(car_at(car)?.position[k].bits()));
    }
    FRAME_SWITCHES.object_number.set(0x39u32);
    add_flare()?;
    Ok(())
}

/// The car block's part switches' place (+0xAC0), from which `light_pair`'s byte offsets
/// count.
pub(super) const PART_SWITCHES: u32 = 0xAC0;

/// `sub_457241`: a pair of lights at the light record's +`at` (three dwords: none
/// when all are 0) for the car block at `car`: the right one (x as it is) when the block's byte
/// +`right` is set, the left (x negated) when +`left` is (-1: no byte), each by
/// `place_flare`; `race().frame.frame_work.light_left` / `race().frame.frame_work.light_right` the two bytes.
pub(super) fn light_pair(
    car: u32,
    record: GamePtr,
    at: u32,
    left: u32,
    right: u32,
) -> Result<(), BadInput> {
    let frame = &race().frame;
    // The bytes are the block's part switches (+0xAC0 ..).
    let switches = &car_at(car)?.part_switches;
    let byte = |o: u32| -> Result<u32, BadInput> {
        let n = o.wrapping_sub(PART_SWITCHES) as usize;
        if n >= switches.atomics().len() {
            return Err(BadInput("a light byte past the part switches"));
        }
        Ok(u32::from(switches.get(n)))
    };
    frame.frame_work.light_left.set(0u32);
    frame.frame_work.light_right.set(0u32);
    if left != u32::MAX {
        frame.frame_work.light_left.set(byte(left)?);
    }
    if right != u32::MAX {
        frame.frame_work.light_right.set(byte(right)?);
    }
    let [x, y, z]: [u32; 3] = record.offset(at).read("a light record's place")?;
    if x | y | z == 0 {
        return Ok(());
    }
    let y = y.wrapping_sub(PHYSICS.ground_drop.bits());
    if frame.frame_work.light_right.get() != 0 {
        place_flare(car, [x, y, z])?;
    }
    if frame.frame_work.light_left.get() != 0 {
        place_flare(car, [x.wrapping_neg(), y, z])?;
    }
    Ok(())
}

/// `race::car_light_flares` (0x457036): the lights of the car block at `car` as flares.
/// Refuses (before anything) a flare list or depth list without room for the 24 flares a car
/// can give.
pub fn car_light_flares(car: u32) -> Result<(), Refused> {
    let race = race();
    if RACE_VIEW.depth_entry_count.bits() > SORT_MAX - 24 {
        return Err(BadInput("the depth list has no room for a car's flares").into());
    }
    if FRAME_SWITCHES.mask_kind.get() == 8 {
        OBJECT_WORK.size.set(2u32);
        let point = [
            race.frame.frame_work.part_place[0].bits(),
            race.frame.frame_work.part_place[1]
                .get()
                .wrapping_sub(PHYSICS.ground_drop.bits()),
            race.frame.frame_work.part_place[2].bits(),
        ];
        OBJECT_WORK.other_light.set(0x00F0_F0F0u32);
        place_flare(car, point)?;
        // Not the original's: the pair's other light, as `light_pair` (the debug UI).
        if point[0] != 0 {
            place_flare(car, [point[0].wrapping_neg(), point[1], point[2]])?;
        }
        return Ok(());
    }
    if FRAME_SWITCHES.other_way.bits() == 0 {
        return Ok(());
    }
    let kind = car_at(car)?.car_kind.bits();
    race.frame.frame_work.part_kind.set(kind);
    let valid = kind < 0x22 && KIND_TABLES.kind_shows_parts[kind as usize].bits() != 0;
    if valid {
        if !crate::math::x87_load::below_or_unordered(
            car_at(car)?.electrics_damage.bits(),
            race.course.numbers.hundred.bits(),
        ) {
            return Ok(());
        }
        race.frame
            .frame_work
            .parts_present
            .set(car_at(car)?.parts_present.bits());
        // A second test of the same kind follows; its error branch (a log line) is
        // never reached.
    }
    // `sub_41F4E1`: the kind held to 0 ..= 0x21 (by sign), through `KIND_TABLES.kind_light_records`.
    let index = KIND_TABLES.kind_light_records[(kind as i32).clamp(0, 0x21) as usize].bits();
    let record = LIGHT_RECORDS.record_ptr(index);
    LIGHT_STATE.view_kind.set(1u32);
    LIGHT_WORK.light_kind.set(3u32);
    OBJECT_WORK.size.set(2u32);
    OBJECT_WORK.other_light.set(0x0060_6060u32);
    let pair = |at: u32, left: u32, right: u32| light_pair(car, record, at, left, right);
    pair(0xA4, 0xAC0, 0xAC1)?;
    if LIGHT_WORK.marks_mask.get() != 0 && race.frame.frame_work.parts_present.get() >> 9 & 1 != 0 {
        pair(0xB0, 0xACA, 0xACB)?;
        pair(0xBC, 0xACC, 0xACD)?;
        pair(0xC8, 0xACE, 0xACF)?;
    }
    // `sub_45721A`: no left light for kind 0xD.
    let left = if kind == 0xD { u32::MAX } else { 0xAC6 };
    pair(0xF8, left, 0xAC7)?;
    OBJECT_WORK.other_light.set(0x00C0_0000u32);
    // `sub_457229`: none for kinds 0xD and 0xE.
    let left = if kind == 0xD || kind == 0xE {
        u32::MAX
    } else {
        0xAC8
    };
    pair(0x104, left, 0xAC9)?;
    OBJECT_WORK.other_light.set(0x0080_0000u32);
    pair(0xD4, 0xAC4, 0xAC5)?;
    pair(0xE0, 0xAD0, u32::MAX)?;
    pair(0xEC, 0xAC2, 0xAC3)?;
    Ok(())
}

/// `race::car_lights` (0x455CFF): with `FRAME_SWITCHES.other_way` set, for each race car: with
/// +0x6FC bit 5 (lights on, `car_light_modes`) its depth entry (`add_car_depth_entry`) and then `car_light_flares`. Refuses (before anything) no cars (the loop tests at
/// its end) or more than eight, a null block, or a depth list without room for every car's
/// entries.
pub fn car_lights() -> Result<(), Refused> {
    if FRAME_SWITCHES.other_way.bits() == 0 {
        return Ok(());
    }
    // Converted: `rc2k_core::race::lights::{check_car_lights, lights_on}` (the null
    // block's refusal between its two, as before).
    use rc2k_core::race::lights::{check_car_lights, lights_on};
    let cars: u32 = RACE_VIEW.race_cars.bits();
    check_car_lights(cars, 0, SORT_MAX)?;
    let block = |n: u32| RACE_WORK.car_block(n as usize);
    if (0..cars).any(|n| block(n) == 0) {
        return Err(BadInput("a null car block").into());
    }
    check_car_lights(cars, RACE_VIEW.depth_entry_count.bits(), SORT_MAX)?;
    for n in 0..cars {
        let b = block(n);
        // Both only with the lights on: the `jz` skips to the `popa`.
        if lights_on(car_at(b)?.state_bits.bits()) {
            add_car_depth_entry(b, n)?;
            car_light_flares(b)?;
        }
    }
    Ok(())
}

/// The end of `place_object_matrix`'s ways: the place `OBJECT_WORK.place` .. into
/// `g_placedMatrix`, moved by the view (0x5EA1CC into `race().frame.matrices.matrix_scratch`), `race().frame.matrices.object_turn_matrix` with it
/// (`g_turnedMatrix`), times the view's turn into `g_madeObjectMatrix`.
pub(super) fn placed_object_tail() {
    let frame = &race().frame;
    use crate::scene::matrix;
    let place = [
        OBJECT_WORK.place[0].get(),
        OBJECT_WORK.place[1].bits(),
        OBJECT_WORK.place[2].bits(),
    ];
    matrix::set_place(place);
    matrix::copy_matrix_with_place(
        &frame.matrices.placed_matrix,
        &frame.matrices.view_place_matrix,
        &frame.matrices.matrix_scratch,
    );
    matrix::copy_matrix_with_place(
        &frame.matrices.object_turn_matrix,
        &frame.matrices.matrix_scratch,
        &frame.matrices.turned_matrix,
    );
    matrix::multiply_matrices(
        &frame.matrices.turned_matrix,
        &frame.matrices.view_turn_matrix,
        &frame.matrices.made_object_matrix,
    );
}

/// `race().frame.matrices.object_turn_matrix`'s sixteen floats times `OBJECT_WORK.size` / 16 (through `DRAW.scratch`).
pub(super) fn scale_object_turn() -> Result<(), BadInput> {
    {
        let scale = crate::math::x87::int_over_16(OBJECT_WORK.size.get() as i32);
        DRAW.scratch.set_bits(scale);
        for cell in &race().frame.matrices.object_turn_matrix {
            cell.set_bits(crate::math::x87::mul(cell.bits(), scale));
        }
    }
    Ok(())
}

/// `race::place_object_matrix` (0x42739A): `g_madeObjectMatrix` for an object placed at
/// `OBJECT_WORK.place` .. (after `scene::set_light_direction`). With `LIGHT_STATE.light_turn_off` 0: turned
/// about y, x and z (16 bits a turn) by `OBJECT_WORK.angles[1]`, `OBJECT_WORK.angles`, `OBJECT_WORK.angles[2]`
/// negated, unscaled. Else (nothing more with `LIGHT_STATE.view_kind` 4) by the object's
/// material (+0x20 of its kind's 0x80-byte record at `g_objectMaterials`): 1 and 2 facing
/// the view (`g_viewBackMatrix`); 5 turned about y by `race().frame.view_and_records.view_vars.view_angles[3]` and the view's y
/// angle (fine turns); else about y by -`OBJECT_WORK.angles`, then with `OBJECT_WORK.angles[1]` tilted
/// about x round the y of `OBJECT_WORK.angles[2]`, and with `OBJECT_WORK.described_cleared[1]` / `OBJECT_WORK.described_cleared[2]` again
/// round `OBJECT_WORK.described_cleared`; then scaled by `OBJECT_WORK.size` / 16.
pub fn place_object_matrix() -> Result<(), BadInput> {
    let frame = &race().frame;
    use crate::scene::matrix::{
        multiply_matrices, set_light_direction, set_turn_x, set_turn_y, set_turn_z_16,
    };
    use crate::scene::view_matrix::{Axis, set_fine_turn};
    let (turn_y, turn_x, turn_z) = (
        &frame.matrices.turn_y_matrix,
        &frame.matrices.turn_x_matrix,
        &frame.matrices.turn_z_matrix,
    );
    let (turned, lit, object) = (
        &frame.matrices.turned_matrix,
        &frame.matrices.lit_matrix,
        &frame.matrices.object_turn_matrix,
    );
    set_light_direction();
    if LIGHT_STATE.light_turn_off.get() == 0 {
        set_turn_y(OBJECT_WORK.angles[1].bits().wrapping_neg());
        set_turn_x(OBJECT_WORK.angles[0].bits().wrapping_neg());
        multiply_matrices(turn_y, turn_x, turned);
        set_turn_z_16(OBJECT_WORK.angles[2].bits().wrapping_neg());
        multiply_matrices(turned, turn_z, object);
        placed_object_tail();
        return Ok(());
    }
    if LIGHT_STATE.view_kind.get() == 4 {
        return Ok(());
    }
    let kind = OBJECT_TABLES.object_kind(FRAME_SWITCHES.object_number.get());
    let material = POLYGON_WORK
        .object_materials
        .get(kind as usize)
        .ok_or(BadInput("an object kind past the 1200 materials"))?
        .words[8]
        .get() as u8; // the record's byte +0x20
    match material {
        1 | 2 => {
            crate::math::cells::copy_cells(&frame.matrices.view_back_matrix, &object, 16);
        }
        5 => {
            crate::math::cells::copy_cells(&frame.matrices.identity_matrix, &object, 16);
            set_fine_turn(
                Axis::Y,
                frame.view_and_records.view_vars.view_angles[3].get(),
            );
            multiply_matrices(object, turn_y, turned);
            set_fine_turn(
                Axis::Y,
                frame.view_and_records.view_vars.view_angles[0].get(),
            );
            multiply_matrices(turned, turn_y, object);
        }
        _ => {
            set_turn_y(OBJECT_WORK.angles[0].bits().wrapping_neg());
            crate::math::cells::copy_cells(&turn_y, &object, 16);
            if OBJECT_WORK.angles[1].get() != 0 {
                set_turn_y(OBJECT_WORK.angles[2].bits().wrapping_neg());
                multiply_matrices(object, turn_y, lit);
                set_turn_x(OBJECT_WORK.angles[1].bits().wrapping_neg());
                multiply_matrices(lit, turn_x, turned);
                set_turn_y(OBJECT_WORK.angles[2].get());
                multiply_matrices(turned, turn_y, object);
            }
            if OBJECT_WORK.described_cleared[2].get() != 0
                || OBJECT_WORK.described_cleared[1].get() != 0
            {
                set_turn_y(OBJECT_WORK.described_cleared[2].bits().wrapping_neg());
                multiply_matrices(object, turn_y, lit);
                set_turn_y(OBJECT_WORK.described_cleared[0].bits().wrapping_neg());
                multiply_matrices(lit, turn_y, turned);
                set_turn_x(OBJECT_WORK.described_cleared[1].bits().wrapping_neg());
                multiply_matrices(turned, turn_x, lit);
                set_turn_y(OBJECT_WORK.described_cleared[0].get());
                multiply_matrices(lit, turn_y, object);
            }
        }
    }
    scale_object_turn()?;
    placed_object_tail();
    Ok(())
}

/// `race::draw_flare` (0x455189): flare `flare` of the six flare tables back into
/// `OBJECT_WORK.place` .., `OBJECT_WORK.size`, `OBJECT_WORK.other_light`, `g_objectNumber`; `LIGHT_STATE.view_kind` 2,
/// `place_object_matrix`, `g_lightKind` 0 and the drawing (`scene::draw_scene_object`,
/// described by `entry`, the draw list entry the flare came from).
pub fn draw_flare(flare: u32, entry: GamePtr) -> Result<(), Refused> {
    let course = &race().course;
    for (table, to) in [
        (&course.level_grid.flare_place_x, &OBJECT_WORK.place[0]),
        (&course.flare_tables.place_y, &OBJECT_WORK.place[1]),
        (&course.flare_tables.place_z, &OBJECT_WORK.place[2]),
        (&course.flare_tables.size, &OBJECT_WORK.size),
        (&course.flare_tables.other_light, &OBJECT_WORK.other_light),
        (
            &course.flare_tables.object_number,
            &FRAME_SWITCHES.object_number,
        ),
    ] {
        to.set_bits(
            table
                .get(flare as usize)
                .ok_or(BadInput("a flare past the tables"))?
                .bits(),
        );
    }
    LIGHT_STATE.view_kind.set(2u32);
    place_object_matrix()?;
    LIGHT_WORK.light_kind.set(0u32);
    crate::scene::draw_scene_object::draw_scene_object_in_game(Some(entry))
}

/// `race::draw_lamp_flare` (0x4551F4): lamp `lamp` (`g_lamps`): its point (`g_scenePoints`)
/// into `OBJECT_WORK.place` .., its +4 x 8 the size, +8 the colour, object `FRAME_SWITCHES.marker_object`;
/// `LIGHT_STATE.view_kind` 2, `place_object_matrix`, `g_lightKind` 0 and the drawing. Every
/// register kept.
pub fn draw_lamp_flare(lamp: u32) -> Result<(), Refused> {
    let edi = self::lamp(lamp)?;
    let [x, y, z] = crate::race::scene_points::point_number(edi[0].get())?;
    OBJECT_WORK.place[0].set(x);
    OBJECT_WORK.place[1].set(y);
    OBJECT_WORK.place[2].set(z);
    OBJECT_WORK.size.set(edi[1].get() << 3);
    OBJECT_WORK.other_light.set(edi[2].get());
    FRAME_SWITCHES
        .object_number
        .set(FRAME_SWITCHES.marker_object.get());
    LIGHT_STATE.view_kind.set(2u32);
    place_object_matrix()?;
    LIGHT_WORK.light_kind.set(0u32);
    // esi `g_turnedMatrix`, as `place_object_matrix` leaves it (`LIGHT_STATE.view_kind` is 2 here,
    // so it always ends by placing the object).
    crate::scene::draw_scene_object::draw_scene_object_in_game(Some(matrices_at(
        core::mem::offset_of!(SceneMatrices, turned_matrix),
    )))
}

/// `race::draw_list_entry` (0x454FF0): the entry at `at`, whose second word is `flags_word`
/// (into `COURSE.polygon_flags`; the top bit dropped for the kind): below 0x1000000 a course face
/// (`g_currentCourseFace`): kind 0xFF part 1, below 0x10 part 2, each with `LEVEL_GRID.smooth` ..
/// `LEVEL_GRID.plain_second` 1 around it; 0x1000000 .. a flare, 0x2000000 .. a car, 0x3000000 .. a
/// lamp, each by its low 24 bits; then `LIGHT_STATE.view_kind` 0. Refuses (before anything) a word
/// of 0x4000000 or more (the original would take it for a face far past the table).
pub fn draw_list_entry(at: GamePtr, flags_word: u32) -> Result<(), Refused> {
    let course = &race().course;
    let word = flags_word & 0x7FFF_FFFF;
    if word >= 0x400_0000 {
        return Err(BadInput("a list entry of 0x4000000 or more").into());
    }
    course.course.polygon_flags.set(flags_word);
    let low = word & 0xFF_FFFF;
    match word >> 24 {
        0 => {
            course.course.current_course_face.set_bits(word);
            let face = word;
            let kind = crate::race::course_places::face_field::<u8>(face, 0, "a course face")?;
            if kind == 0xFF || kind < 0x10 {
                course.level_grid.smooth.set_bits(1);
                course.level_grid.plain_first.set_bits(1);
                course.level_grid.plain_second.set_bits(1);
                if kind == 0xFF {
                    // The description is the face.
                    crate::scene::draw_described_object::draw_described_face(face)?;
                } else {
                    crate::race::course_faces::draw_tinted_face(face)?;
                }
                course.level_grid.smooth.set_bits(0);
                course.level_grid.plain_first.set_bits(0);
                course.level_grid.plain_second.set_bits(0);
            }
        }
        1 => draw_flare(low, at)?,
        2 => crate::race::car_draw::draw_car_entry(low)?,
        _ => draw_lamp_flare(low)?,
    }
    LIGHT_STATE.view_kind.set(0u32);
    Ok(())
}

/// `race::draw_sorted_entries` (0x454EAB) / `race::draw_depth_entries` (0x454E84): each
/// entry of the list (`count` its count, `entries` its 8-byte entries) by
/// `draw_list_entry`. Refuses (before anything) what `draw_list_entry` would for any.
/// `cells` the list's cells; `list` the same list as a place (a flare's entry is read there
/// as a description, past its own 8 bytes).
pub fn draw_entries(
    count: &crate::session::Dword,
    cells: &[crate::session::Dword],
    list: GamePtr,
) -> Result<(), Refused> {
    let entries = cells
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
        draw_list_entry(list.offset(8 * k as u32), entry[1].get())?;
    }
    Ok(())
}

/// The scene matrices from byte `off` on, as the esi the original leaves for a drawing
/// (`scene::draw_described_object` reads a description there): bounded by the whole
/// store, since a description's 0x3C bytes run past one matrix.
pub(super) fn matrices_at(off: usize) -> GamePtr {
    GamePtr::to(&race().frame.matrices).offset(off as u32)
}
