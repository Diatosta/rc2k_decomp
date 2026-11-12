//! The cars drawn (`race::draw_list_entry` place 5: `race::draw_car_entry` ->
//! `race::draw_car_shined` ->
//! `race::draw_car`). A car is a block (`g_carBlocks`, `CarBlock`):
//! +0x24 its model, +0x134 .. +0x13C its place, +0x298 its parts' switches, +0x6FC its
//! state bits, +0x740 .. +0x8BF six matrices (four parts', the body's, the turn), +0xF04 the
//! shown parts' mask.

use crate::effects::cell_x87::CellX87;
use crate::errors::{BadInput, Refused};
use crate::race::blocks_store::CAR_BLOCKS;
use crate::race::car_draw_state::CarDrawState;
use crate::race::frame_view::car as car_view;
use crate::race::light_records_store::LIGHT_RECORDS;
use crate::race::owner::Race;
use crate::race::owner_course::Course;
use crate::race::view_vars_store::ViewVars;
use crate::race::work_store::RaceWork;
use crate::scene::draw_described_object::Description;
use crate::scene::matrix::multiply_matrices;
use crate::scene::view_matrices::ViewMatrices;
use crate::session::Float;

/// The drawn car's turn (`placed_matrices`: its +0x880), the view's own.
fn car_turn(cars: &CarDrawState) -> &'_ [Float] {
    &cars.car_turn
}
/// The drawn car's four parts' matrices (`part_matrices` .., its +0x740 ..), the view's own.
pub fn part_matrices(cars: &CarDrawState) -> [&'_ [Float]; 4] {
    [0, 16, 32, 48].map(|k| &cars.part_matrices[k..k + 16])
}

/// `race::car_part_matrix_1` .. `_4` (0x461F12, 0x461F58, 0x461F9E, 0x461FE4): matrix `a` times the
/// body's turn (`body_turn_matrix`) into `g_turnedMatrix`, times the car's turn (`placed_matrices`) into
/// `g_litMatrix`, times `g_viewMatrix` into `g_madeObjectMatrix`.
pub fn car_part_matrix(cars: &CarDrawState, m: &ViewMatrices, a: &[Float]) {
    multiply_matrices(a, &m.body_turn_matrix, &m.turned_matrix);
    multiply_matrices(&m.turned_matrix, car_turn(cars), &m.lit_matrix);
    multiply_matrices(
        &m.lit_matrix,
        &m.view_matrix, // g_viewMatrix
        &m.made_object_matrix,
    );
}

/// `race::car_body_matrix` (0x461EA1): the turns about x and z by 0 (`scene::set_fine_turn_x`,
/// `scene::set_turn_z`) multiplied into `g_litMatrix`, then as `car_part_matrix` on it.
pub fn car_body_matrix(course: &Course, cars: &CarDrawState, m: &ViewMatrices) {
    use crate::scene::view_matrix::{Axis, set_fine_turn};
    set_fine_turn(course, m, Axis::X, 0);
    set_fine_turn(course, m, Axis::Z, 0);
    multiply_matrices(&m.turn_x_matrix, &m.turn_z_matrix, &m.lit_matrix);
    car_part_matrix(cars, m, &m.lit_matrix);
}

/// The byte at `offset` of the number texture names. The table holds 32 names of 0x30 bytes;
/// kinds 32 and 33 read on into the fields after it, in their order: the drawn
/// car's nearness and word, the tens' textures, the parts' break limits, the still matrix.
/// Past those is refused.
fn number_name_byte(work: &RaceWork, offset: u32) -> Result<u8, BadInput> {
    let dword = (offset / 4) as usize;
    let bits = match dword.checked_sub(work.number_texture_names.len()) {
        None => work.number_texture_names[dword].bits(),
        Some(0) => work.car_nearness.bits(),
        Some(1) => work.car_draw_word.bits(),
        Some(n @ 2..9) => work.number_tens_textures[n - 2].bits(),
        Some(n @ 9..16) => work.part_break_limits[n - 9].bits(),
        Some(n @ 16..32) => work.part_matrix_still[n - 16].bits(),
        Some(_) => return Err(BadInput("a car kind's number texture name past the table")),
    };
    Ok(bits.to_le_bytes()[(offset % 4) as usize])
}

/// `race::car_number_textures` (0x46203D): car `car`'s marks' textures into `marks` (the original's
/// `car_mark_textures` ..: `scene::car_marks`): +0x1B8 / +0x1BC (|x| not below `decal_limit`: 1) into
/// `ObjectTables.car_mark_textures[1]`, `ObjectTables.car_mark_textures[4]` / `ObjectTables.car_mark_textures[2]`, `ObjectTables.car_mark_textures[3]`; the letter of its model's name
/// (`number_texture_names` + 0x30 x model) at +0x718 (rounded, held to 0 .. 0x2EDF) / 250, as a digit or
/// letter number, into `ObjectTables.car_mark_textures[5]` (and `car_letter_b`, which nothing read: not kept); its
/// number +0xAA8 as tens and units (`ObjectTables.car_mark_textures[6]`, `ObjectTables.car_mark_textures[7]`, 999 for none; `ObjectTables.car_mark_textures[9]`
/// the units alone below ten); the stage's (held to 11 .. 0x50) tens' plate `number_tens_textures`[]` into
/// `ObjectTables.car_mark_textures[8]`; and while +0x310 is below `hundred` its 17 bytes +0xAC0 .. into the places
/// `car_decal_places`[]` names (marks 0xA .. 0x1B; one past the marks is refused).
pub fn car_number_textures(
    race: &Race,
    car: usize,
    marks: &crate::scene::car_marks::CarMarks,
) -> Result<(), BadInput> {
    // Converted: `rc2k_core::race::car_marks` (the rounding of +0x718 the x87's, under the
    // game's control word).
    use rc2k_core::race::car_marks::{
        decals_shown, lamp_side, letter_number, letter_place, number_digits, stage_plate,
    };
    let block = car_view(car)?;
    let float = |c: &crate::session::Float| c.get();
    {
        let limit = race.frame.frame_work.decal_limit.get();
        let side = lamp_side(float(&block.drive_speed_a), limit);
        marks.mark(1).set(side);
        marks.mark(4).set(side);
        let side = lamp_side(float(&block.drive_speed_b), limit);
        marks.mark(2).set(side);
        marks.mark(3).set(side);
        let name = car_view(car)?.car_kind.get().wrapping_mul(0x30);
        let n = crate::math::x87::to_int(&race.car_blocks().cars[car].shown_engine_speed);
        let (place, _) = letter_place(n);
        let letter = letter_number(number_name_byte(
            &race.shared.work,
            name.wrapping_add(place),
        )?) as u32;
        marks.mark(5).set(letter);
        let d = number_digits(car_view(car)?.start_slot_value.get());
        marks.mark(6).set(d.tens);
        marks.mark(7).set(d.units);
        marks.mark(9).set(d.alone);
        let (tens, _) = stage_plate(race.shared.vars.stage_number.get());
        marks
            .mark(8)
            .set(race.shared.work.number_tens_textures[tens as usize].get());
        if decals_shown(
            float(&block.electrics_damage),
            race.course.numbers.hundred.get(),
        ) {
            for k in 0..0x11u32 {
                let byte = race.car_blocks().cars[car].part_switches.get(k as usize) as u32;
                let to = race.frame.frame_work.car_decal_places[k as usize].get() as usize;
                if to >= crate::scene::car_marks::MARKS {
                    return Err(BadInput("a decal place past the car's marks"));
                }
                marks.mark(to).set(byte);
            }
        }
    }
    Ok(())
}

/// The group number at byte `offset` of model `model`'s groups (the original does not
/// bound the model: one past the 34 records is refused).
fn model_field(model: u32, offset: u32) -> Result<u32, BadInput> {
    LIGHT_RECORDS
        .records
        .get(model as usize)
        .and_then(|record| record.groups.get((offset / 4) as usize))
        .map(crate::session::Dword::get)
        .ok_or(BadInput("a model's record"))
}

/// The model's bit at +`offset` of its groups (`LightsAndGear.light_records.records[0].groups[0]` ..): 1 << it, 0 for -1.
fn model_bit(model: u32, offset: u32) -> Result<u32, BadInput> {
    let c = model_field(model, offset)?;
    Ok(if c == u32::MAX { 0 } else { 1u32 << (c & 31) })
}

fn below(a: u32, b: u32) -> bool {
    (a.wrapping_sub(b) as i32) < 0
}

/// `bts value, bit` (bit mod 32).
fn with_bit(value: u32, bit: u32) -> u32 {
    value | 1u32 << (bit & 31)
}

/// A mask slot of the view's group masks (`CarDrawState::group_masks`) and the index into the model's `groups` whose bit it
/// takes, in the order they are set.
const BODY_GROUPS: [(usize, usize); 5] = [(0, 0), (1, 1), (2, 2), (3, 3), (6, 5)];

/// The mask slot of the lamps and the model's group they come from.
const LAMPS: (usize, usize) = (9, 0xD);

/// Models whose lamp mask gets extra bits, and the bits.
const EXTRA_LAMP_BITS: [(&[u32], u32); 2] = [(&[4, 7], 0x600_0000), (&[0xD], 0x300_0000)];

/// The mask slots set after the lamps, as `BODY_GROUPS`.
const OTHER_GROUPS: [(usize, usize); 5] = [(10, 0xE), (7, 0xB), (8, 0xC), (11, 0xA), (12, 0xF)];

/// A part's switch bit (`parts_present`): when it is clear the part's group, in this mask
/// slot, is not shown.
const PART_SWITCHES: [(u32, usize); 7] = [
    (9, 6),
    (12, 11),
    (13, 9),
    (14, 10),
    (15, 12),
    (16, 7),
    (17, 8),
];

/// The course's group masks from the model's part groups (an unknown model is refused).
fn set_group_masks(masks: &[crate::session::Dword; 18], model: u32) -> Result<(), BadInput> {
    for (to, group) in BODY_GROUPS {
        masks[to].set(model_bit(model, group as u32 * 4)?);
    }
    let (slot, group) = LAMPS;
    let mut lamps = model_bit(model, group as u32 * 4)?;
    if lamps != 0 {
        for (models, bits) in EXTRA_LAMP_BITS {
            if models.contains(&model) {
                lamps |= bits;
            }
        }
    }
    masks[slot].set(lamps);
    for (to, group) in OTHER_GROUPS {
        masks[to].set(model_bit(model, group as u32 * 4)?);
    }
    Ok(())
}

/// The groups of the parts still on the car: all but those of the parts whose switch is off.
fn shown_groups(masks: &[crate::session::Dword; 18], switches: u32) -> u32 {
    let mut shown = u32::MAX;
    for (bit, slot) in PART_SWITCHES {
        if switches & (1 << bit) == 0 {
            shown &= !masks[slot].get();
        }
    }
    shown
}

/// `race::draw_car` (0x4616AA): car `car` drawn: its marks' textures (`race::car_number_textures`);
/// its six matrices (+0x740 ..) to `part_matrices` .. / `body_turn_matrix` / `placed_matrices`; its model's
/// parts' bits (`group_masks` ..; the shown mask `group_mask` by +0x298, `group_mask` by
/// +0x6FC, the view's depth `car_depth` and the stage's fog, `car_nearness` the nearness (0 .. 1)
/// dimming `g_shine`), `fade_alpha_moved`, +0xF04; all parts for the drawn view's own car when
/// `ViewAndRecords.view_vars.view_car_flag_a`; the light and place; `scene::set_light_direction`; the body (`g_objectNumber`
/// 0x50 + 0x14 x car, `race::car_body_matrix`, `scene::draw_scene_object` between the kept
/// reflections), and nearer than 0x3C0000 its four parts by +0x298's bits 5 .. 8 (each
/// `car_part_matrix`, `scene::draw_scene_object`); then `race::clear_car_marks` (here the marks are
/// the drawing's own: `scene::car_marks`). Each drawing gets `g_litMatrix`, as the matrices
/// leave it (the object's description, read only on the described way). Refuses (before anything)
/// what `scene::draw_scene_object` would for the car's objects: in a race (`g_inRace`) car
/// `drawn_car` (below eight) without its block.
pub fn draw_car(ctx: &crate::race::view_scratch::DrawCtx<'_>, car: usize) -> Result<(), Refused> {
    let race = ctx.race;
    let n: u32 = ctx.view.state.drawn_car.get();
    if race.course.numbers.in_race.is_set() && n < 8 && !race.shared.work.has_car_block(n as usize)
    {
        return Err(BadInput("the drawn car without its block").into());
    }
    // The car's marks, its own while it is drawn (the original's, cleared after it by
    // `race::clear_car_marks`: `scene::car_marks`).
    let marks = crate::scene::car_marks::CarMarks::new();
    car_number_textures(race, car, &marks)?;
    let ctx = &crate::race::view_scratch::DrawCtx {
        marks: &marks,
        ..*ctx
    };
    let [p0, p1, p2, p3] = part_matrices(&ctx.view.cars);
    let c = &CAR_BLOCKS.cars[car];
    let kept = &c.part_matrices;
    for (from, to) in [
        &kept[..16],
        &kept[16..32],
        &kept[32..48],
        &kept[48..],
        &c.body_matrix,
        &c.place_matrix,
    ]
    .into_iter()
    .zip([
        p0,
        p1,
        p2,
        p3,
        &ctx.view.matrices.body_turn_matrix,
        car_turn(&ctx.view.cars),
    ]) {
        crate::math::cells::copy_cells(from, to, 16);
    }
    ctx.view
        .cars
        .groups_which
        .set(CAR_BLOCKS.cars[car].mode_on.get());
    let model = car_view(car)?.car_kind.get();
    ctx.view.matrices.known_object.set(model);
    set_group_masks(&ctx.view.cars.group_masks, model)?;
    let switches = car_view(car)?.parts_present.get();
    let shown = shown_groups(&ctx.view.cars.group_masks, switches);
    ctx.view.cars.shown_groups.set(shown);
    let field = |offset: u32| model_field(model, offset);
    ctx.view.face.faded_bits.set(field(0x10)?);
    ctx.view.cars.group_masks[4].set(
        u32::MAX
            .wrapping_sub(ctx.view.cars.group_masks[0].get())
            .wrapping_sub(ctx.view.cars.group_masks[1].get())
            .wrapping_sub(ctx.view.cars.group_masks[2].get())
            .wrapping_sub(ctx.view.cars.group_masks[3].get()),
    );
    ctx.view.cars.car_draw_kept.set(0x210_0000);
    for (to, offset) in [
        (13usize, 0x18u32),
        (14, 0x1C),
        (15, 0x20),
        (16, 0x24),
        (17, 0x14),
    ] {
        ctx.view.cars.group_masks[to].set(field(offset)?);
    }
    let state = ctx.view.cars.state_of(car)?.get();
    let bit = ctx.view.cars.group_masks[17].get();
    ctx.view.cars.group_mask.set(u32::MAX);
    if state & 2 != 0 {
        ctx.view.cars.group_mask.set(with_bit(field(0x48)?, bit));
    }
    if state & 4 != 0 {
        ctx.view.cars.group_mask.set(with_bit(field(0x44)?, bit));
    }
    ctx.view
        .cars
        .car_nearness
        .set(race.course.numbers.zero.get());
    let mut stage = ctx.view.state.view_car_stage.get();
    if (stage as i32) < 0 {
        stage = 0;
    }
    if !below(stage, 0x33) {
        stage = 0x33;
    }
    let record = ViewVars::record_of(stage as usize)?;
    if record.fixed_zoom.get() != 0 {
        ctx.view.cars.car_depth.set(0x5_0000);
    }
    let and = |mask: u32| {
        let view = &ctx.view.cars;
        view.group_mask.set(view.group_mask.get() & mask);
    };
    if state & 0x10 == 0 {
        nearness(race, &ctx.view.cars, &ctx.view.course);
        let depth = ctx.view.cars.car_depth.get();
        if !below(depth, 0x14_0000) {
            and(with_bit(field(0x40)?, bit));
            for var in [13usize, 14, 15, 16] {
                and(!(1u32 << (ctx.view.cars.group_masks[var].get() & 31)));
            }
            and(!ctx.view.cars.group_masks[7].get());
            and(!ctx.view.cars.group_masks[8].get());
        }
        if !below(depth, 0x28_0000) {
            and(!ctx.view.cars.group_masks[9].get());
            and(!ctx.view.cars.group_masks[10].get());
            and(!ctx.view.cars.group_masks[6].get());
        }
        if !below(depth, 0x3C_0000) {
            and(!ctx.view.cars.group_masks[12].get());
            and(!ctx.view.cars.group_masks[11].get());
        }
    }
    let mut ecx = 0x60u32.wrapping_add(near_size(&ctx.view.cars));
    ctx.view.cars.car_draw_word.set(ecx);
    if state & 0x100 != 0 {
        if state & 6 == 0 {
            and(with_bit(field(0x40)?, bit));
        }
        ecx = 0xF0;
    }
    ctx.view.face.fade_alpha.set(ecx);
    // The original stored the group mask into the car's block +0xF04 here (0x461CD8),
    // which nothing ever reads: dropped (`notes/bugs.md`).
    // `view_cars[drawn_view]`: past the two, read on by address as the original does.
    let view_car = crate::session::read_on_in(
        &race.shared.view,
        &race.shared.view.view_cars,
        ctx.view.drawn_view(race) as usize,
        "the view cars",
    )?;
    if view_car == ctx.view.state.drawn_car.get() && ctx.view.state.view_car_flag_a.get() != 0 {
        ctx.view.cars.group_mask.set(u32::MAX);
    }
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
    ctx.view.light.light_kind.set(3);
    let this_car = car_view(car)?;
    let place = || {
        ctx.view.object.place[0].set(this_car.position[0].get());
        ctx.view.object.place[1].set(this_car.position[1].get());
        ctx.view.object.place[2].set(this_car.position[2].get());
    };
    place();
    ctx.view.object.size.set(0x10);
    crate::scene::matrix::set_light_direction(
        &ctx.view.matrices,
        &ctx.view.light.light_direction,
        &ctx.view.matrices.view_turn_matrix,
        &ctx.view.light,
    );
    place();
    ctx.view.object.car_switches.set(switches);
    ctx.view.light.other_light.set(0);
    ctx.view.light.light_kind.set(3);
    ctx.view.light.light_kind_extra.set(0);
    ctx.view.cars.object_number.set(
        ctx.view
            .state
            .drawn_car
            .get()
            .wrapping_mul(0x14)
            .wrapping_add(0x50),
    );
    car_body_matrix(&race.course, &ctx.view.cars, &ctx.view.matrices);
    crate::race::frame_draw::start_kept_reflections(ctx.view);
    crate::scene::draw_scene_object::draw_scene_object_in_game(
        ctx,
        Some(Description::Floats(&ctx.view.matrices.lit_matrix)),
    )?;
    crate::race::frame_draw::end_kept_reflections(ctx.view);
    ctx.view.light.light_kind_extra.set(1);
    if below(ctx.view.cars.car_depth.get(), 0x3C_0000) {
        for (bit, matrix) in [0x20u32, 0x40, 0x80, 0x100]
            .into_iter()
            .zip(part_matrices(&ctx.view.cars))
        {
            ctx.view
                .cars
                .object_number
                .set(ctx.view.cars.object_number.get().wrapping_add(1));
            if ctx.view.object.car_switches.get() & bit != 0 {
                car_part_matrix(&ctx.view.cars, &ctx.view.matrices, matrix);
                crate::scene::draw_scene_object::draw_scene_object_in_game(
                    ctx,
                    Some(Description::Floats(&ctx.view.matrices.lit_matrix)),
                )?;
            }
        }
    }
    ctx.view.face.faded_bits.set(u32::MAX);
    Ok(())
}

/// `race::draw_car`'s nearness: `car_nearness` = (`car_depth` - 0xA0000) / 0x140000 held to 0 ..
/// 1 (`zero`, `one`), `g_shine` (and `shine_whole`) times 1 less it (the
/// original's stores to `scratch` are not kept).
fn nearness(race: &Race, cars: &CarDrawState, mine: &crate::race::view_course::ViewCourse) {
    use crate::math::x87::below_or_unordered;
    use crate::math::x87_float::{ONE, int, to_int};
    {
        let depth = int(cars.car_depth.get() as i32);
        cars.car_nearness
            .st((depth - int(0xA_0000)) / int(0x14_0000));
        let near = cars.car_nearness.get();
        // `fld zero; fld near; fcompp`: near below 0 (or no number) -> 0.
        if below_or_unordered(near, race.course.numbers.zero.get()) {
            cars.car_nearness.set(race.course.numbers.zero.get());
        }
        let near = cars.car_nearness.get();
        if !below_or_unordered(near, race.course.numbers.one.get()) {
            cars.car_nearness.set(race.course.numbers.one.get());
        }
        let shine = (ONE - cars.car_nearness.ld()) * int(mine.shine.get() as i32);
        let whole = to_int(shine);
        mine.shine.set(whole as u32);
    }
}

/// `race::draw_car`'s size: 0x90 times `car_nearness`, rounded (the original's stores to
/// `scratch` are not kept).
fn near_size(cars: &CarDrawState) -> u32 {
    use crate::math::x87_float::{int, to_int};
    {
        let size = to_int(int(0x90) * cars.car_nearness.ld());
        size as u32
    }
}

/// `race::draw_car_shined` (0x461669): `g_shine` for the car (0x3FCE0 less `mud_thrown_counts`, held to
/// 0 .. 0x40000, times `shine_scale`, >> 18), the car `car` drawn (`race::draw_car`), `g_shine`
/// put back.
pub fn draw_car_shined(
    ctx: &crate::race::view_scratch::DrawCtx<'_>,
    car: usize,
) -> Result<(), Refused> {
    let race = ctx.race;
    let course = &race.course;
    let shine: u32 = ctx.view.course.shine.get();
    let mut eax = 0x3_FCE0u32.wrapping_sub(race.shared.vars.mud_thrown_counts[0].get());
    if (eax as i32) < 0 {
        eax = 0;
    }
    if !below(eax, 0x4_0000) {
        eax = 0x4_0000;
    }
    let product = eax as u64 * course.stage_prefs.shine_scale.get() as u64;
    ctx.view.course.shine.set((product as u32) >> 18);
    let done = draw_car(ctx, car);
    ctx.view.course.shine.set(shine);
    done
}

/// `race::draw_car_entry` (0x455105): car `entry` & 7 (into `drawn_car`; `view_light_base` its
/// +0x240): its place turned by `g_viewMatrix` (`scene::rotate_point`) into `car_kept` ..
/// `car_depth`; drawn (`race::draw_car_shined`, counted in `cars_drawn`) unless behind
/// (depth + 0x40000 below 0) or aside (|x| - 0x80000 not below the depth), counted in
/// `cars_culled`. Refuses (before anything) a car without its block.
pub fn draw_car_entry(
    ctx: &crate::race::view_scratch::DrawCtx<'_>,
    entry: u32,
) -> Result<(), Refused> {
    let race = ctx.race;
    let frame = &race.frame;
    use crate::math::x87::tbyte_to_int;
    use crate::race::skidmarks::int_80;
    let n = entry & 7;
    if !race.shared.work.has_car_block(n as usize) {
        return Err(BadInput("a car without its block").into());
    }
    let car = race.shared.work.block_number(n as usize)?;
    ctx.view.vars.drawn_car.set(n);
    ctx.view.state.drawn_car.set(n);
    ctx.view
        .light
        .car_light
        .set(car_view(car)?.light_colour.get());
    let t = crate::scene::rotate_point::rotate_point(
        &ctx.view.matrices.view_matrix,
        [
            int_80(car_view(car)?.position[0].get()),
            int_80(car_view(car)?.position[1].get()),
            int_80(car_view(car)?.position[2].get()),
        ],
    );
    let place = t.map(tbyte_to_int);
    ctx.view.cars.car_kept[0].set(place[0]);
    ctx.view.cars.car_kept[1].set(place[1]);
    ctx.view.cars.car_depth.set(place[2]);
    let depth = place[2];
    let mut x = place[0];
    if below(depth, 0xFFFC_0000) {
        frame
            .frame_work
            .cars_culled
            .set(frame.frame_work.cars_culled.get().wrapping_add(1));
        return Ok(());
    }
    if (x as i32) < 0 {
        x = x.wrapping_neg();
    }
    x = x.wrapping_sub(0x8_0000);
    if !below(x, depth) {
        frame
            .frame_work
            .cars_culled
            .set(frame.frame_work.cars_culled.get().wrapping_add(1));
        return Ok(());
    }
    ctx.view
        .cars
        .cars_drawn
        .set(ctx.view.cars.cars_drawn.get().wrapping_add(1));
    draw_car_shined(ctx, car)
}

/// The blocks against the original's outputs, recorded (`test_support::race_vectors`).
#[cfg(test)]
mod vector_tests {
    use super::*;
    use crate::race::asm_test::{Rng, at_each_control_word, lock};
    use crate::race::owner::race;
    use crate::scene::owner::scene;
    use crate::test_support::race_vectors::{Kind, Vectors};
    use crate::test_support::state::Snapshot;

    /// The stores the test writes.
    const STORES: [&str; 5] = [
        "scene.owner.SCENE.drawing.scene",
        "race.owner.RACE.frame.frame_work",
        "race.owner.RACE.shared.work",
        "race.owner.RACE.course.course",
        "race.owner.RACE.course.numbers",
    ];

    /// The values the blocks read and write, without `scratch` (the assembly's spill dword)
    /// and `shine_whole` (the shine as a whole number, read by nothing): neither kept by the
    /// plain versions.
    fn state() -> [u32; 5] {
        let race = race();
        [
            race.frame.frame_work.car_depth.get(),
            race.shared.work.car_nearness.bits(),
            race.course.course.shine.get(),
            race.course.numbers.zero.bits(),
            race.course.numbers.one.bits(),
        ]
    }

    fn fill(r: &mut Rng) {
        let race = race();
        scene().drawing.scene.scratch.set(0xAAAA_AAAAu32);
        // the depth is a whole number around the view's reach
        race.frame
            .frame_work
            .car_depth
            .set((r.int() as u32) & 0x3F_FFFF);
        race.shared.work.car_nearness.set_bits(r.float());
        race.course.course.shine.set(r.int() as u32);
        race.course
            .numbers
            .zero
            .set_bits(if r.below(4) == 0 { r.float() } else { 0 });
        race.course.numbers.one.set_bits(if r.below(4) == 0 {
            r.float()
        } else {
            0x3F80_0000
        });
    }

    /// The state after the nearness, then the size.
    fn plain() -> Vec<u32> {
        let race = race();
        let cars = CarDrawState::new();
        cars.car_depth.set(race.frame.frame_work.car_depth.get());
        let mine = crate::race::view_course::ViewCourse::new();
        mine.shine.set(race.course.course.shine.get());
        nearness(race, &cars, &mine);
        race.course.course.shine.set(mine.shine.get());
        race.shared.work.car_nearness.set(cars.car_nearness.get());
        let mut out = state().to_vec();
        out.push(near_size(&cars));
        assert_eq!(state()[..], out[..5]);
        out
    }

    #[test]
    fn nearness_and_size_are_the_originals() {
        let _l = lock();
        let kept = Snapshot::of(&STORES);
        let mut vectors = Vectors::open("race.car_draw.nearness_and_size", Kind::Exact, 6);
        let mut r = Rng(6);
        at_each_control_word(|| {
            for _ in 0..20000 {
                fill(&mut r);
                let before = state();
                let got = plain();
                vectors.check(&got, |a, b| a == b, || format!("from {before:x?}"));
            }
        });
        kept.restore();
        vectors.finish();
    }
}
