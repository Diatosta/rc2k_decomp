//! The scene's objects: the kept reflections, the race slots, screen quads, flares, car lights and
//! the drawing of the sorted entries.

use super::*;
use crate::errors::{BadInput, Refused};
use crate::physics::figures::GROUND_DROP;
use crate::race::lamp_work::Flare;
use crate::race::view_scratch::DrawCtx;
use crate::scene::draw_described_object::{DescribedMatrix, Description};

/// `race::start_kept_reflections`: the 0x400 kept reflection normals cleared to -999 and
/// `reflections_kept` 1.
pub fn start_kept_reflections(view: &crate::race::view_scratch::ViewScratch) {
    view.object.borrow_mut().kept_reflections[..0x400].fill(0xFFFF_FC19);
    view.face.borrow_mut().reflections_kept = 1u32;
}

/// `race::end_kept_reflections`: `reflections_kept` 0.
pub fn end_kept_reflections(view: &crate::race::view_scratch::ViewScratch) {
    view.face.borrow_mut().reflections_kept = 0u32;
}

/// `race::draw_slot_object`: the race slot's object (`part`, into `object_number`) drawn, unless
/// outside one player and modes 0 and 2 its car (object 0x50 + 0x14 x car) is not the drawn view's
/// (`race().frame.frame_work.listed_car_sounds`): the light (lowest 0, highest 0xFF, level
/// `light_level`, strength `light_strength`, turns -`sun_turn` / `sun_height`, `view_kind` 1,
/// `light_kind` 2), its turn (`matrix`, 16 dwords to `ctx.view.matrices.object_turn_matrix`) placed
/// at `place` with its y plus `draw_height_offset` (into `place` ..) and moved by the view (into
/// `ctx.view.matrices.matrix_scratch`), times the view's turn into `made_object_matrix`; the
/// reflections kept around the drawing (`scene::draw_scene_object`). Refuses (before anything) as
/// `check_slot_object`.
fn draw_slot_object(
    ctx: &crate::race::view_scratch::DrawCtx<'_>,
    slot: &crate::race::work_store::RaceSlot,
) -> Result<(), Refused> {
    let race = ctx.race;
    use crate::scene::matrix;
    check_slot_object(&ctx.reads.work.borrow(), slot)?;
    let object = slot.part;
    ctx.view.cars.borrow_mut().object_number = object;
    let mode: u32 = ctx.reads.vars.run_kind();
    if ctx.reads.view.players() != 1 && mode != 0 && mode != 2 {
        let n = object.wrapping_sub(0x50);
        if ctx.reads.frame_work.listed_car_sounds()[((n / 0x14) & 7) as usize]
            != ctx.view.drawn_view(race)
        {
            return Ok(());
        }
    }
    ctx.view.light.borrow_mut().other_light = 0;
    ctx.view.light.borrow_mut().light_lowest = 0;
    ctx.view.light.borrow_mut().light_highest = 0xFF;
    ctx.view.light.borrow_mut().light_level = ctx.reads.stage_prefs.read(|p| p.light_level);
    ctx.view.object.borrow_mut().light_strength = ctx.reads.stage_prefs.read(|p| p.light_strength);
    ctx.view.light.borrow_mut().light_turns[0] =
        ctx.reads.stage_prefs.read(|p| p.sun_turn).wrapping_neg();
    ctx.view.light.borrow_mut().light_turns[1] = ctx.reads.stage_prefs.read(|p| p.sun_height);
    ctx.view.light.borrow_mut().view_kind = 1;
    ctx.view.light.borrow_mut().light_kind = 2;
    ctx.view.matrices.borrow_mut().object_turn_matrix = matrix::matrix_of(&slot.matrix);
    let place = [
        slot.place[0],
        slot.place[1].wrapping_add(slot.draw_height_offset),
        slot.place[2],
    ];
    for (k, &p) in place.iter().enumerate() {
        ctx.view.object.borrow_mut().place[k] = p;
    }
    {
        let mut kept = ctx.view.matrices.borrow_mut();
        let m = &mut *kept;
        matrix::set_place(&mut m.placed_matrix, place);
        matrix::copy_matrix_with_place(
            &m.placed_matrix,
            &m.view_place_matrix,
            &mut m.matrix_scratch,
        );
        matrix::copy_matrix_with_place(
            &m.object_turn_matrix,
            &m.matrix_scratch,
            &mut m.turned_matrix,
        );
        matrix::multiply_matrices(
            &m.turned_matrix,
            &m.view_turn_matrix,
            &mut m.made_object_matrix,
        );
    }
    start_kept_reflections(ctx.view);
    // The object is placed by `turned_matrix`, as the matrices' calls leave it.
    crate::scene::draw_scene_object::draw_scene_object_in_game(
        ctx,
        Some(Description::Floats(
            &ctx.view.matrices,
            DescribedMatrix::Turned,
        )),
    )?;
    end_kept_reflections(ctx.view);
    Ok(())
}

/// What `scene::draw_scene_object` would refuse for slot `at`'s object, before anything: a car's
/// object (0x50 .. 0xEF) without its block.
fn check_slot_object(
    work: &crate::race::work_store::RaceWork,
    slot: &crate::race::work_store::RaceSlot,
) -> Result<(), Refused> {
    let n = slot.part.wrapping_sub(0x50);
    if n < 0xA0 && !work.has_car_block((n / 0x14) as usize) {
        return Err(BadInput("a slot's car without its block").into());
    }
    Ok(())
}

/// `race::draw_race_slots`: `draw_slot_object` for each of the eight race slots (0x78 bytes at
/// `race_slots`) whose `part` is not 0. Refuses (before anything) as `check_slot_object` for any.
pub fn draw_race_slots(ctx: &crate::race::view_scratch::DrawCtx<'_>) -> Result<(), Refused> {
    let slots = &ctx.reads.work.borrow().race_slots;
    for slot in slots {
        if slot.part != 0 {
            check_slot_object(&ctx.reads.work.borrow(), slot)?;
        }
    }
    for slot in slots {
        if slot.part != 0 {
            draw_slot_object(ctx, slot)?;
        }
    }
    Ok(())
}

/// `race::draw_screen_quad`: a flat rectangle on the screen from x `quad_corners` ..
/// `quad_corners[1]`, y `quad_corners[4]` .. `quad_corners[5]` (the
/// four vertices, 0x20 bytes each, depth `k4096_b`) in `quad_colour`'s colour:
/// the polygon's four work entries set (0x44 bytes each), clipped
/// (`scene::clip_polygon_to_screen`), and with three vertices or more drawn with the depth buffer
/// off (`polygon_mode` 0, switches -1, 1, 0; `polygon_drawn` 1); the colour back to 0xFFFFFF.
/// Refuses what the clip refuses (after the entries are set: never seen with four vertices).
pub fn draw_screen_quad(
    poly: &crate::scene::polygon_work_store::PolygonWork,
    part: &crate::scene::scratch_store::PartScratch,
    face: &crate::race::face_work::FaceWork,
    reads: crate::scene::workers::DrawReads<'_>,
    screen: &crate::memory::session::Screen,
    view: &crate::race::view_scratch::ViewScratch,
) -> Result<(), Refused> {
    let entries = &part.polygon.entries[..4];
    for (k, e) in (0u32..).zip(entries) {
        // the quad vertex's record
        e.record.set(crate::scene::point_records::of_quad(k));
        e.u.set(0u32);
        e.v.set(0u32);
    }
    let corners = rc2k_core::race::loading::quad_corners(
        view.quad.borrow().corners[0] as i32,
        view.quad.borrow().corners[1] as i32,
        view.quad.borrow().corners[4] as i32,
        view.quad.borrow().corners[5] as i32,
    );
    for (k, (x, y)) in corners.into_iter().enumerate() {
        let mut quads = poly.quads.borrow_mut();
        let v = &mut quads.words[8 * k..8 * k + 8];
        v[3] = x.to_bits();
        v[7] = y.to_bits();
        v[6] = reads.numbers.k4096_b.get().to_bits();
    }
    for e in entries {
        e.point.set(u32::MAX);
        e.light.set(0u32);
    }
    poly.scalars.borrow_mut().polygon_colour = view.quad.borrow().colour[0];
    poly.scalars.borrow_mut().polygon_vertex_count = 4u32;
    poly.scalars.borrow_mut().polygon_next_record = crate::scene::polygon::pool_start();
    for k in 0..4u32 {
        part.polygon.list[k as usize].set(k);
    }
    part.polygon.list[4].set(0u32);
    poly.scalars.borrow_mut().polygon_next_entry = 4u32;
    crate::scene::clip_polygon_to_screen::clip_polygon_to_screen(part, poly, reads.course())?;
    if poly.scalars.borrow().polygon_vertex_count >= 3 {
        crate::scene::depth_buffer_on::depth_buffer(part, false);
        poly.scalars.borrow_mut().polygon_textures[0] = u32::MAX;
        face.borrow_mut().polygon_mode = 0u32;
        poly.scalars.borrow_mut().polygon_sides[0] = 1u32;
        poly.scalars.borrow_mut().polygon_sides[1] = 0u32;
        crate::scene::draw_polygon::draw_polygon_in_game(view, part, reads, screen)?;
        face.borrow_mut().polygon_mode = 0u32;
        crate::scene::depth_buffer_on::depth_buffer(part, true);
    }
    poly.scalars.borrow_mut().polygon_colour = 0x00FF_FFFFu32;
    Ok(())
}

/// `race::draw_view_tint`: the whole view (0 .. `view_size`, 0 .. `view_size[1]`) as a
/// screen quad in 0x204080 (`quad_colour`), switches -1, 1, 0, by `draw_screen_quad`.
pub fn draw_view_tint(
    poly: &crate::scene::polygon_work_store::PolygonWork,
    part: &crate::scene::scratch_store::PartScratch,
    face: &crate::race::face_work::FaceWork,
    reads: crate::scene::workers::DrawReads<'_>,
    screen: &crate::memory::session::Screen,
    view: &crate::race::view_scratch::ViewScratch,
) -> Result<(), Refused> {
    view.quad.borrow_mut().corners[0] = 0u32;
    view.quad.borrow_mut().corners[1] = view.polygon.window.borrow().view_size[0];
    view.quad.borrow_mut().corners[4] = 0u32;
    view.quad.borrow_mut().corners[5] = view.polygon.window.borrow().view_size[1];
    view.quad.borrow_mut().colour[0] = 0x20_4080u32;
    poly.scalars.borrow_mut().polygon_textures[0] = u32::MAX;
    poly.scalars.borrow_mut().polygon_sides[0] = 1u32;
    poly.scalars.borrow_mut().polygon_sides[1] = 0u32;
    draw_screen_quad(poly, part, face, reads, screen, view)
}

/// `race::add_car_depth_entry`: the place (the car's `position`, into `place` ..) of the block of
/// car `car` through `view_matrix` (rounded into `turned_place` ..; x and
/// y back as floats, z the key: plus 0x2000000, shifted right 10), a depth entry of the key with
/// the car's `number` plus 0x2000000. Refuses (before anything) a depth entry past the 0x1000.
pub fn add_car_depth_entry(ctx: &DrawCtx, car: usize, number: u32) -> Result<(), Refused> {
    let DrawCtx { view, .. } = *ctx;
    use crate::math::x87::from_int;
    let n: u32 = view.lists.depth_entry_count.get();
    if n >= SORT_MAX {
        return Err(BadInput("a depth entry past the 0x1000").into());
    }
    let place = crate::memory::session::read_bits(&ctx.reads.frame_cars.car(car)?.position);
    for (k, &p) in place.iter().enumerate() {
        ctx.view.object.borrow_mut().place[k] = p;
    }
    let matrix: [u32; 16] = view.matrices.borrow().view_matrix.map(f32::to_bits);
    let t = crate::math::x87::rotate_rounding(place.map(|v| v as i32), &matrix);
    for (k, &v) in t.iter().enumerate() {
        view.object.borrow_mut().turned_place[k] = v;
    }
    ctx.view.object.borrow_mut().place[0] = from_int(t[0] as i32).to_bits();
    ctx.view.object.borrow_mut().place[1] = from_int(t[1] as i32).to_bits();
    let key = t[2].wrapping_add(0x200_0000) >> 10;
    view.face.borrow_mut().depth_key = key;
    let second = number.wrapping_add(0x200_0000);
    let at = view.lists.depth_entry(n)?;
    at[0].set(key);
    at[1].set(second);
    view.lists.depth_entry_count.set(n + 1);
    Ok(())
}

/// The tables' 256 entries. `add_flare`'s count runs on to 0x400 (a slip kept as is: each table
/// runs over the next and the last past its end); flares past the tables are not added.
const FLARES_MAX: u32 = 0x100;

/// A flare at `place` .. (object `object_number`, size `size`, colour
/// `other_light`) into the six flare tables (0x100 dwords each, from
/// `flare_place_x`) and, turned by `view_matrix` (x and y back as floats, z rounded into
/// the key), a depth entry with the flare's number + 0x1000000; none past `FLARES_MAX`.
pub(super) fn add_flare(ctx: &DrawCtx) -> Result<(), BadInput> {
    let DrawCtx { view, .. } = *ctx;
    let n: u32 = view.lamps.borrow().flare_count;
    if n == FLARES_MAX {
        return Ok(());
    }
    let place = ctx.view.object.borrow().place;
    let flare = Flare {
        place,
        size: ctx.view.object.borrow().size,
        other_light: ctx.view.light.borrow().other_light,
        object_number: ctx.view.cars.borrow().object_number,
    };
    view.lamps.borrow_mut().set_flare(n as usize, flare);
    let out = crate::math::x87::rotate_rounding_z(
        place.map(|v| v as i32),
        &view.matrices.borrow().view_matrix.map(f32::to_bits),
    );
    ctx.view.object.borrow_mut().place[0] = out[0];
    ctx.view.object.borrow_mut().place[1] = out[1];
    let key = out[2].wrapping_add(0x200_0000) >> 10;
    view.face.borrow_mut().depth_key = key;
    let entry: u32 = view.lists.depth_entry_count.get();
    let at = view.lists.depth_entry(entry)?;
    at[0].set(key);
    at[1].set(n.wrapping_add(0x100_0000));
    view.lists.depth_entry_count.set(entry + 1);
    view.lamps.borrow_mut().flare_count = n + 1;
    Ok(())
}

/// A flare at `point` in the frame of the car block at `car` (its `crash_matrix`, each part
/// rounded, plus its `position`), object 0x39, by `add_flare`.
pub(super) fn place_flare(ctx: &DrawCtx, car: usize, point: [u32; 3]) -> Result<(), BadInput> {
    let t = crate::math::x87::rotate_rounding(
        point.map(|v| v as i32),
        &crate::memory::session::read_bits(&ctx.reads.frame_cars.car(car)?.crash_matrix),
    );
    for (k, v) in t.iter().enumerate() {
        ctx.view.object.borrow_mut().place[k] =
            v.wrapping_add(ctx.reads.frame_cars.car(car)?.position[k].get());
    }
    ctx.view.cars.borrow_mut().object_number = 0x39u32;
    add_flare(ctx)?;
    Ok(())
}

/// The car block's part switches' byte offset (0xAC0), from which `light_pair`'s byte offsets
/// count.
pub(super) const PART_SWITCHES: u32 = 0xAC0;

/// A pair of lights at byte `at` of the light record (three dwords: none when all are 0) for the
/// car block at `car`: the right one (x as it is) when the block's byte `right` is set, the left (x
/// negated) when `left` is (-1: no byte), each by `place_flare`;
/// `race().frame.frame_work.light_left` / `race().frame.frame_work.light_right` the two bytes.
pub(super) fn light_pair(
    ctx: &DrawCtx,
    car: usize,
    record: usize,
    at: u32,
    left: u32,
    right: u32,
) -> Result<(), BadInput> {
    // The bytes are the car's `part_switches`.
    let switches = ctx.reads.car_parts.car(car).part_switches;
    let byte = |o: u32| -> Result<u32, BadInput> {
        let n = o.wrapping_sub(PART_SWITCHES) as usize;
        if n >= crate::race::car_parts_store::PART_SWITCHES_LEN {
            return Err(BadInput("a light byte past the part switches"));
        }
        Ok(u32::from(switches[n]))
    };
    ctx.view.cars.borrow_mut().light_left = 0u32;
    ctx.view.cars.borrow_mut().light_right = 0u32;
    if left != u32::MAX {
        ctx.view.cars.borrow_mut().light_left = byte(left)?;
    }
    if right != u32::MAX {
        ctx.view.cars.borrow_mut().light_right = byte(right)?;
    }
    let lights = ctx
        .reads
        .light_records
        .read(|l| l.records.get(record).map(|r| r.lights))
        .ok_or(BadInput("a light record past the 34 models'"))?;
    // `at` counts from the record's start; the lights lie from byte 0xA4 and every pair is inside.
    let first = (at as usize).wrapping_sub(0xA4) / 4;
    let [x, y, z]: [u32; 3] = match lights.get(first..first.wrapping_add(3)) {
        Some(&[x, y, z]) => [x, y, z],
        _ => return Err(BadInput("a light pair outside the record's lights")),
    };
    if x | y | z == 0 {
        return Ok(());
    }
    let y = y.wrapping_sub(GROUND_DROP);
    if ctx.view.cars.borrow().light_right != 0 {
        place_flare(ctx, car, [x, y, z])?;
    }
    if ctx.view.cars.borrow().light_left != 0 {
        place_flare(ctx, car, [x.wrapping_neg(), y, z])?;
    }
    Ok(())
}

/// `race::car_light_flares`: the lights of the car block at `car` as flares. Refuses (before
/// anything) a flare list or depth list without room for the 24 flares a car can give.
pub fn car_light_flares(ctx: &DrawCtx, car: usize) -> Result<(), Refused> {
    let DrawCtx { race, view, .. } = *ctx;
    if view.lists.depth_entry_count.get() > SORT_MAX - 24 {
        return Err(BadInput("the depth list has no room for a car's flares").into());
    }
    if ctx.reads.switches.mask_kind() == 8 {
        ctx.view.object.borrow_mut().size = 2u32;
        let place = ctx.reads.frame_work.part_place();
        let point = [place[0], place[1].wrapping_sub(GROUND_DROP), place[2]];
        ctx.view.light.borrow_mut().other_light = 0x00F0_F0F0u32;
        place_flare(ctx, car, point)?;
        // the pair's other light, as `light_pair` (the debug UI).
        if point[0] != 0 {
            place_flare(ctx, car, [point[0].wrapping_neg(), point[1], point[2]])?;
        }
        return Ok(());
    }
    if ctx.reads.switches.scene_set_up() == 0 {
        return Ok(());
    }
    let kind = ctx.reads.frame_cars.car(car)?.car_kind.get();
    let valid = kind < 0x22 && race.kind_tables().kind_shows_parts[kind as usize] != 0;
    if valid {
        if !crate::math::x87::below_or_unordered(
            ctx.reads.frame_cars.car(car)?.electrics_damage.get(),
            ctx.reads.numbers.hundred.get(),
        ) {
            return Ok(());
        }
        let present = ctx.reads.frame_cars.car(car)?.parts_present.get();
        ctx.view.cars.borrow_mut().parts_present = present;
        // A second test of the same kind follows; it can never fail here.
    }
    // The kind held to 0 ..= 0x21 (by sign), through the kind tables' `kind_light_records`.
    let index = race.kind_tables().kind_light_records[(kind as i32).clamp(0, 0x21) as usize];
    // The table is the image's and nothing writes it: its entries are 0 .. 33, the 34 records'.
    let record = index as usize;
    ctx.view.light.borrow_mut().view_kind = 1u32;
    ctx.view.light.borrow_mut().light_kind = 3u32;
    ctx.view.object.borrow_mut().size = 2u32;
    ctx.view.light.borrow_mut().other_light = 0x0060_6060u32;
    let pair = |at: u32, left: u32, right: u32| light_pair(ctx, car, record, at, left, right);
    pair(0xA4, 0xAC0, 0xAC1)?;
    if ctx.reads.light_work.marks_mask != 0 && ctx.view.cars.borrow().parts_present >> 9 & 1 != 0 {
        pair(0xB0, 0xACA, 0xACB)?;
        pair(0xBC, 0xACC, 0xACD)?;
        pair(0xC8, 0xACE, 0xACF)?;
    }
    // No left light for kind 0xD.
    let left = if kind == 0xD { u32::MAX } else { 0xAC6 };
    pair(0xF8, left, 0xAC7)?;
    ctx.view.light.borrow_mut().other_light = 0x00C0_0000u32;
    // None for kinds 0xD and 0xE.
    let left = if kind == 0xD || kind == 0xE {
        u32::MAX
    } else {
        0xAC8
    };
    pair(0x104, left, 0xAC9)?;
    ctx.view.light.borrow_mut().other_light = 0x0080_0000u32;
    pair(0xD4, 0xAC4, 0xAC5)?;
    pair(0xE0, 0xAD0, u32::MAX)?;
    pair(0xEC, 0xAC2, 0xAC3)?;
    Ok(())
}

/// `race::car_lights`: with `scene_set_up` set, for each race car: with bit 5 of its state bits
/// (lights on, `car_light_modes`) its depth entry (`add_car_depth_entry`) and then
/// `car_light_flares`. Refuses (before anything) no cars (the loop tests at its end) or more than
/// eight, a null block, or a depth list without room for every car's entries.
pub fn car_lights(ctx: &DrawCtx) -> Result<(), Refused> {
    if ctx.reads.switches.scene_set_up() == 0 {
        return Ok(());
    }
    // The null block's refusal comes between the two checks.
    use rc2k_core::race::lights::{check_car_lights, lights_on};
    let cars: u32 = ctx.reads.view.race_cars();
    check_car_lights(cars, 0, SORT_MAX)?;
    if (0..cars).any(|n| !ctx.reads.work.has_car_block(n as usize)) {
        return Err(BadInput("a null car block").into());
    }
    check_car_lights(cars, ctx.view.lists.depth_entry_count.get(), SORT_MAX)?;
    for n in 0..cars {
        let b = ctx.reads.work.block_number(n as usize)?;
        // Both only with the lights on:
        if lights_on(ctx.view.cars.borrow().state_of(b)?) {
            add_car_depth_entry(ctx, b, n)?;
            car_light_flares(ctx, b)?;
        }
    }
    Ok(())
}

/// The end of `place_object_matrix`'s ways: the place `place` .. into `placed_matrix`,
/// moved by the view (into `ctx.view.matrices.matrix_scratch`),
/// `ctx.view.matrices.object_turn_matrix` with it (`turned_matrix`), times the view's turn into
/// `made_object_matrix`.
pub(super) fn placed_object_tail(ctx: &DrawCtx) {
    use crate::scene::matrix;
    let place = ctx.view.object.borrow().place;
    {
        let mut kept = ctx.view.matrices.borrow_mut();
        let m = &mut *kept;
        matrix::set_place(&mut m.placed_matrix, place);
        matrix::copy_matrix_with_place(
            &m.placed_matrix,
            &m.view_place_matrix,
            &mut m.matrix_scratch,
        );
        matrix::copy_matrix_with_place(
            &m.object_turn_matrix,
            &m.matrix_scratch,
            &mut m.turned_matrix,
        );
        matrix::multiply_matrices(
            &m.turned_matrix,
            &m.view_turn_matrix,
            &mut m.made_object_matrix,
        );
    }
}

/// `ctx.view.matrices.object_turn_matrix`'s sixteen floats times `size` / 16.
pub(super) fn scale_object_turn(ctx: &DrawCtx) -> Result<(), BadInput> {
    let scale = crate::math::x87::int_over_16(ctx.view.object.borrow().size as i32);
    for cell in &mut ctx.view.matrices.borrow_mut().object_turn_matrix {
        *cell = crate::math::x87::mul(*cell, scale);
    }
    Ok(())
}

/// `race::place_object_matrix`: `made_object_matrix` for an object placed at `place` ..
/// (after `scene::set_light_direction`). With `light_turn_off` 0: turned about y, x and
/// z (16 bits a turn) by `angles[1]`, `angles`, `angles[2]`
/// negated, unscaled. Else (nothing more with `view_kind` 4) by the object's material
/// (a field of its kind's 0x80-byte record at `object_materials`): 1 and 2 facing the view
/// (`view_back_matrix`); 5 turned about y by
/// `race().frame.view_and_records.view_vars.view_angles[3]` and the view's y angle (fine turns);
/// else about y by -`angles`, then with `angles[1]` tilted about x round
/// the y of `angles[2]`, and with `described_cleared[1]` /
/// `described_cleared[2]` again round `described_cleared`; then scaled by
/// `size` / 16.
pub fn place_object_matrix(ctx: &crate::race::view_scratch::DrawCtx<'_>) -> Result<(), BadInput> {
    use crate::scene::matrix::{
        multiply_matrices, set_light_direction, set_turn_x, set_turn_y, set_turn_z_16,
    };
    use crate::scene::view_matrix::{Axis, set_fine_turn};
    let tables = ctx.reads.engine_tables;
    let object_place = &ctx.view.object;
    {
        let mut kept = ctx.view.matrices.borrow_mut();
        let view_turn = kept.view_turn_matrix;
        set_light_direction(tables, &mut kept, &view_turn, &ctx.view.light);
    }
    if ctx.reads.light_work.light_turn_off == 0 {
        {
            let mut kept = ctx.view.matrices.borrow_mut();
            let m = &mut *kept;
            set_turn_y(
                tables,
                &mut m.turn_y_matrix,
                &mut m.turn_y_work,
                object_place.borrow().angles[1].wrapping_neg(),
            );
            set_turn_x(
                tables,
                &mut m.turn_x_matrix,
                &mut m.turn_x_work,
                object_place.borrow().angles[0].wrapping_neg(),
            );
            multiply_matrices(&m.turn_y_matrix, &m.turn_x_matrix, &mut m.turned_matrix);
            set_turn_z_16(
                tables,
                &mut m.turn_z_matrix,
                &mut m.turn_z_work,
                object_place.borrow().angles[2].wrapping_neg(),
            );
            multiply_matrices(
                &m.turned_matrix,
                &m.turn_z_matrix,
                &mut m.object_turn_matrix,
            );
        }
        placed_object_tail(ctx);
        return Ok(());
    }
    if ctx.view.light.borrow().view_kind == 4 {
        return Ok(());
    }
    let number = ctx.view.cars.borrow().object_number;
    let kind = ctx.reads.object_tables.object_kind(number);
    // An object whose slot holds no kind (-1) takes the material record before the first.
    let material = crate::scene::polygon::material_run_byte(ctx.reads.polygon_tables, kind, 0x20)?;
    match material {
        1 | 2 => {
            let mut kept = ctx.view.matrices.borrow_mut();
            kept.object_turn_matrix = kept.view_back_matrix;
        }
        5 => {
            ctx.view.matrices.borrow_mut().object_turn_matrix =
                crate::race::matrices_store::identity_matrix();
            set_fine_turn(
                ctx.reads.course(),
                &ctx.view.matrices,
                Axis::Y,
                ctx.view.state.borrow().view_angles[3],
            );
            {
                let mut kept = ctx.view.matrices.borrow_mut();
                let m = &mut *kept;
                multiply_matrices(
                    &m.object_turn_matrix,
                    &m.turn_y_matrix,
                    &mut m.turned_matrix,
                );
            }
            set_fine_turn(
                ctx.reads.course(),
                &ctx.view.matrices,
                Axis::Y,
                ctx.view.state.borrow().view_angles[0],
            );
            {
                let mut kept = ctx.view.matrices.borrow_mut();
                let m = &mut *kept;
                multiply_matrices(
                    &m.turned_matrix,
                    &m.turn_y_matrix,
                    &mut m.object_turn_matrix,
                );
            }
        }
        _ => {
            let mut kept = ctx.view.matrices.borrow_mut();
            let m = &mut *kept;
            set_turn_y(
                tables,
                &mut m.turn_y_matrix,
                &mut m.turn_y_work,
                object_place.borrow().angles[0].wrapping_neg(),
            );
            m.object_turn_matrix = m.turn_y_matrix;
            if object_place.borrow().angles[1] != 0 {
                set_turn_y(
                    tables,
                    &mut m.turn_y_matrix,
                    &mut m.turn_y_work,
                    object_place.borrow().angles[2].wrapping_neg(),
                );
                multiply_matrices(&m.object_turn_matrix, &m.turn_y_matrix, &mut m.lit_matrix);
                set_turn_x(
                    tables,
                    &mut m.turn_x_matrix,
                    &mut m.turn_x_work,
                    object_place.borrow().angles[1].wrapping_neg(),
                );
                multiply_matrices(&m.lit_matrix, &m.turn_x_matrix, &mut m.turned_matrix);
                set_turn_y(
                    tables,
                    &mut m.turn_y_matrix,
                    &mut m.turn_y_work,
                    object_place.borrow().angles[2],
                );
                multiply_matrices(
                    &m.turned_matrix,
                    &m.turn_y_matrix,
                    &mut m.object_turn_matrix,
                );
            }
            if object_place.borrow().described_turns[2] != 0
                || object_place.borrow().described_turns[1] != 0
            {
                set_turn_y(
                    tables,
                    &mut m.turn_y_matrix,
                    &mut m.turn_y_work,
                    object_place.borrow().described_turns[2].wrapping_neg(),
                );
                multiply_matrices(&m.object_turn_matrix, &m.turn_y_matrix, &mut m.lit_matrix);
                set_turn_y(
                    tables,
                    &mut m.turn_y_matrix,
                    &mut m.turn_y_work,
                    object_place.borrow().described_turns[0].wrapping_neg(),
                );
                multiply_matrices(&m.lit_matrix, &m.turn_y_matrix, &mut m.turned_matrix);
                set_turn_x(
                    tables,
                    &mut m.turn_x_matrix,
                    &mut m.turn_x_work,
                    object_place.borrow().described_turns[1].wrapping_neg(),
                );
                multiply_matrices(&m.turned_matrix, &m.turn_x_matrix, &mut m.lit_matrix);
                set_turn_y(
                    tables,
                    &mut m.turn_y_matrix,
                    &mut m.turn_y_work,
                    object_place.borrow().described_turns[0],
                );
                multiply_matrices(&m.lit_matrix, &m.turn_y_matrix, &mut m.object_turn_matrix);
            }
        }
    }
    scale_object_turn(ctx)?;
    placed_object_tail(ctx);
    Ok(())
}

/// `race::draw_flare`: flare `flare` of the six flare tables back into `place` ..,
/// `size`, `other_light`, `object_number`; `view_kind` 2,
/// `place_object_matrix`, `light_kind` 0 and the drawing (`scene::draw_scene_object`, described by
/// `entry`, the draw list entry the flare came from).
pub fn draw_flare(
    ctx: &crate::race::view_scratch::DrawCtx<'_>,
    flare: u32,
    entry: Description<'_>,
) -> Result<(), Refused> {
    let view = ctx.view;
    let held = view.lamps.borrow().flare(flare);
    let held = held.ok_or(BadInput("a flare past the tables"))?;
    view.object.borrow_mut().place = held.place;
    view.object.borrow_mut().size = held.size;
    view.light.borrow_mut().other_light = held.other_light;
    view.cars.borrow_mut().object_number = held.object_number;
    ctx.view.light.borrow_mut().view_kind = 2u32;
    place_object_matrix(ctx)?;
    ctx.view.light.borrow_mut().light_kind = 0u32;
    crate::scene::draw_scene_object::draw_scene_object_in_game(ctx, Some(entry))
}

/// `race::draw_lamp_flare`: lamp `lamp` (`lamps`): its point (`scene_points`) into
/// `place` .., its strength x 8 the size, its colour, object
/// `marker_object`; `view_kind` 2, `place_object_matrix`, `light_kind` 0
/// and the drawing.
fn draw_lamp_flare(ctx: &crate::race::view_scratch::DrawCtx<'_>, lamp: u32) -> Result<(), Refused> {
    let lamp_record = lights::own_lamp(&ctx.view.lamps, lamp)?;
    let [x, y, z] = crate::race::scene_points::point_number(
        ctx.reads.course(),
        &ctx.part.points,
        lamp_record[0],
    )?;
    ctx.view.object.borrow_mut().place[0] = x;
    ctx.view.object.borrow_mut().place[1] = y;
    ctx.view.object.borrow_mut().place[2] = z;
    ctx.view.object.borrow_mut().size = lamp_record[1] << 3;
    ctx.view.light.borrow_mut().other_light = lamp_record[2];
    let marker = ctx.reads.switches.marker_object();
    ctx.view.cars.borrow_mut().object_number = marker;
    ctx.view.light.borrow_mut().view_kind = 2u32;
    place_object_matrix(ctx)?;
    ctx.view.light.borrow_mut().light_kind = 0u32;
    // The object is placed by `turned_matrix`, as `place_object_matrix` leaves it
    // (`view_kind` is 2 here, so it always ends by placing the object).
    crate::scene::draw_scene_object::draw_scene_object_in_game(
        ctx,
        Some(Description::Floats(
            &ctx.view.matrices,
            DescribedMatrix::Turned,
        )),
    )
}

/// A list entry that is a course face (`face`, below 0x1000000) drawn: kind 0xFF the described
/// way, below 0x10 tinted, each with the texture filter's switches 1 around it and 0 after;
/// any other kind draws nothing.
fn draw_face_entry(ctx: &crate::race::view_scratch::DrawCtx<'_>, face: u32) -> Result<(), Refused> {
    ctx.view.face.borrow_mut().course_face = face;
    let kind = crate::race::course_places::face_field::<u8>(ctx.reads.course(), face, 0)?;
    if kind == 0xFF || kind < 0x10 {
        ctx.view.polygon.scalars.borrow_mut().smooth = 1;
        ctx.view.polygon.scalars.borrow_mut().plain_first = 1;
        ctx.view.polygon.scalars.borrow_mut().plain_second = 1;
        if kind == 0xFF {
            // The description is the face.
            crate::scene::draw_described_object::draw_described_face(ctx, face)?;
        } else {
            crate::race::course_faces::draw_tinted_face(ctx, face)?;
        }
        ctx.view.polygon.scalars.borrow_mut().smooth = 0;
        ctx.view.polygon.scalars.borrow_mut().plain_first = 0;
        ctx.view.polygon.scalars.borrow_mut().plain_second = 0;
    }
    Ok(())
}

/// The course faces `faces` (list entries' second words, each below 0x1000000 without its top
/// bit) drawn as `draw_list_entry` draws each.
pub fn draw_face_entries(
    ctx: &crate::race::view_scratch::DrawCtx<'_>,
    faces: &[u32],
) -> Result<(), Refused> {
    for &word in faces {
        draw_face_entry(ctx, word & 0x7FFF_FFFF)?;
        ctx.view.light.borrow_mut().view_kind = 0u32;
    }
    Ok(())
}

/// `race::draw_list_entry`: the entry whose cells from it on are `at`, whose second word is
/// `flags_word`: below 0x1000000 a course face (`current_course_face`): kind 0xFF part 1, below
/// 0x10 part 2, each with `smooth` .. `plain_second` 1 around it; 0x1000000
/// .. a flare, 0x2000000 .. a car, 0x3000000 .. a lamp, each by its low 24 bits; then
/// `view_kind` 0. Refuses (before anything) a word of 0x4000000 or more.
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
        0 => draw_face_entry(ctx, word)?,
        1 => draw_flare(ctx, low, at)?,
        2 => crate::race::car_draw::draw_car_entry(ctx, low)?,
        _ => draw_lamp_flare(ctx, low)?,
    }
    ctx.view.light.borrow_mut().view_kind = 0u32;
    Ok(())
}

/// Draws each entry of the list (`count` its count, `entries` its 8-byte entries) by
/// `draw_list_entry`. Refused (before anything): what `draw_list_entry` would refuse for any.
/// `from(n)` the list's cells from entry `n` on (a flare's entry is read there as a description,
/// past its own 8 bytes).
pub fn draw_entries<'a>(
    ctx: &crate::race::view_scratch::DrawCtx<'_>,
    count: &crate::memory::session::PlainDword,
    from: impl Fn(usize) -> &'a [crate::memory::session::PlainDword],
) -> Result<(), Refused> {
    draw_entries_around(ctx, count, from, None)
}

/// `draw_entries`, the view's own car's entry (its second word without the top bit,
/// `race::car_part::CarAside::entry_word`) taken from `car`, which drew it aside.
pub fn draw_entries_around<'a>(
    ctx: &crate::race::view_scratch::DrawCtx<'_>,
    count: &crate::memory::session::PlainDword,
    from: impl Fn(usize) -> &'a [crate::memory::session::PlainDword],
    car: Option<&crate::race::car_part::CarAside<'_, '_>>,
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
        let word = entry[1].get();
        match car {
            Some(car) if word & 0x7FFF_FFFF == car.entry_word() => car.draw(ctx)?,
            _ => draw_list_entry(ctx, Description::PlainCells(from(k)), word)?,
        }
    }
    Ok(())
}
