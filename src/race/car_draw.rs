//! Drawing the cars: a car is a block (`Car`) whose `car_kind` is its model, `position` its place,
//! `parts_present` its parts' switches, and whose kept matrices (`part_matrices`, `body_matrix`,
//! `place_matrix`) are the four parts', the body's and the turn. The entry points are
//! `draw_car_entry`, `draw_car_shined` and `draw_car`.

use crate::errors::{BadInput, Refused};
use crate::race::car_draw_state::CarDrawState;
use crate::race::light_records_store::LightRecordsRef;
use crate::race::owner_course::CourseReads;
use crate::race::view_vars_store::ViewVars;
use crate::race::work_store::RaceWork;
use crate::scene::draw_described_object::{DescribedMatrix, Description};
use crate::scene::matrix::{Matrix, matrix_of, product};
use crate::scene::view_matrices::ViewMatrices;

/// The drawn car's turn (`placed_matrices`, the block's `place_matrix`), the view's own.
fn car_turn(cars: &CarDrawState) -> Matrix {
    cars.borrow().car_turn
}
/// The drawn car's four parts' matrices (`part_matrices` ..; the block's `part_matrices`), the
/// view's own.
pub fn part_matrices(cars: &CarDrawState) -> [Matrix; 4] {
    let values = cars.borrow();
    [0, 16, 32, 48].map(|k| std::array::from_fn(|n| values.part_matrices[k + n]))
}

/// Car part matrix: matrix `a` times the body's turn (`body_turn_matrix`) into `turned_matrix`,
/// times the car's turn (`placed_matrices`) into `lit_matrix`, times `view_matrix` into
/// `made_object_matrix`. There is one such function for each of the four parts.
fn car_part_matrix(cars: &CarDrawState, m: &ViewMatrices, a: &Matrix) {
    let mut kept = m.borrow_mut();
    let kept = &mut *kept;
    kept.turned_matrix = product(a, &kept.body_turn_matrix, kept.turned_matrix);
    kept.lit_matrix = product(&kept.turned_matrix, &car_turn(cars), kept.lit_matrix);
    // Times the view's matrix.
    kept.made_object_matrix = product(&kept.lit_matrix, &kept.view_matrix, kept.made_object_matrix);
}

/// Car body matrix: the turns about x and z by 0 (`set_fine_turn`)
/// multiplied into `lit_matrix`, then as a part's matrix on it.
fn car_body_matrix(course: CourseReads<'_>, cars: &CarDrawState, m: &ViewMatrices) {
    use crate::scene::view_matrix::{Axis, set_fine_turn};
    set_fine_turn(course, m, Axis::X, 0);
    set_fine_turn(course, m, Axis::Z, 0);
    let lit = {
        let mut kept = m.borrow_mut();
        kept.lit_matrix = product(&kept.turn_x_matrix, &kept.turn_z_matrix, kept.lit_matrix);
        kept.lit_matrix
    };
    car_part_matrix(cars, m, &lit);
}

/// The byte at `offset` of the number texture names. The table holds 32 names of 0x30 bytes; kinds
/// 32 and 33 read on into the fields after it, in their order: the drawn car's nearness and word,
/// the tens' textures, the parts' break limits, the still matrix. Past those is refused.
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

/// Sets car `car`'s marks' textures into `marks` (`ObjectTables.car_mark_textures`). [1] and [4] /
/// [2] and [3] come from the car's `drive_speed_a` / `drive_speed_b` (1 when |x| is not below
/// `decal_limit`). [5] is the letter of its model's name (`number_texture_names` + 0x30 x model) at
/// `shown_engine_speed` (rounded, held to 0 .. 0x2EDF) / 250, as a digit or letter number. [6] and
/// [7] are its number `start_slot_value` as tens and units (999 for none) and [9] the units alone
/// below ten. [8] is the stage's tens' plate (`number_tens_textures`, held to 11 .. 0x50). While
/// `electrics_damage` is below `hundred` its 17 `part_switches` bytes go into the places
/// `car_decal_places` names (marks 0xA .. 0x1B; one past the marks is refused).
pub fn car_number_textures(
    reads: crate::scene::workers::DrawReads<'_>,
    car: usize,
    marks: &crate::scene::car_marks::CarMarks,
) -> Result<(), BadInput> {
    let cars = reads.frame_cars;
    let (part, numbers) = (reads.car_parts.car(car), reads.numbers);
    let (vars, work) = (reads.vars, reads.work);
    // `shown_engine_speed` is rounded as the x87 does, under the game's control word.
    use rc2k_core::race::car_marks::{
        decals_shown, lamp_side, letter_number, letter_place, number_digits, stage_plate,
    };
    let block = cars.car(car)?;
    let float = |c: &crate::memory::session::Float| c.get();
    {
        let limit = crate::race::frame_store::DECAL_LIMIT.get();
        let side = lamp_side(float(&block.drive_speed_a), limit);
        marks.mark(1).set(side);
        marks.mark(4).set(side);
        let side = lamp_side(float(&block.drive_speed_b), limit);
        marks.mark(2).set(side);
        marks.mark(3).set(side);
        let name = cars.car(car)?.car_kind.get().wrapping_mul(0x30);
        let n = crate::math::x87::to_int(part.shown_engine_speed);
        let (place, _) = letter_place(n);
        let letter = letter_number(number_name_byte(work, name.wrapping_add(place))?) as u32;
        marks.mark(5).set(letter);
        let d = number_digits(cars.car(car)?.start_slot_value.get());
        marks.mark(6).set(d.tens);
        marks.mark(7).set(d.units);
        marks.mark(9).set(d.alone);
        let (tens, _) = stage_plate(vars.stage_number());
        marks
            .mark(8)
            .set(work.number_tens_textures[tens as usize].get());
        if decals_shown(float(&block.electrics_damage), numbers.hundred.get()) {
            for k in 0..0x11u32 {
                let byte = u32::from(part.part_switches[k as usize]);
                let to = crate::race::frame_store::CAR_DECAL_PLACES[k as usize] as usize;
                if to >= crate::scene::car_marks::MARKS {
                    return Err(BadInput("a decal place past the car's marks"));
                }
                marks.mark(to).set(byte);
            }
        }
    }
    Ok(())
}

/// The group number at byte `offset` of model `model`'s groups.
fn model_field(lights: LightRecordsRef<'_>, model: u32, offset: u32) -> Result<u32, BadInput> {
    lights
        .read(|l| {
            l.records
                .get(model as usize)
                .and_then(|record| record.groups.get((offset / 4) as usize))
                .copied()
        })
        .ok_or(BadInput("a model's record"))
}

/// The model's bit at byte `offset` of its groups
/// (`light_records.records[0].groups[0]` ..): 1 << it, 0 for -1.
fn model_bit(lights: LightRecordsRef<'_>, model: u32, offset: u32) -> Result<u32, BadInput> {
    let c = model_field(lights, model, offset)?;
    Ok(if c == u32::MAX { 0 } else { 1u32 << (c & 31) })
}

fn below(a: u32, b: u32) -> bool {
    (a.wrapping_sub(b) as i32) < 0
}

/// Sets bit `bit` (mod 32) of `value`.
fn with_bit(value: u32, bit: u32) -> u32 {
    value | 1u32 << (bit & 31)
}

/// A mask slot of the view's group masks (`CarDrawState::group_masks`) and the index into the
/// model's `groups` whose bit it takes, in the order they are set.
const BODY_GROUPS: [(usize, usize); 5] = [(0, 0), (1, 1), (2, 2), (3, 3), (6, 5)];

/// The mask slot of the lamps and the model's group they come from.
const LAMPS: (usize, usize) = (9, 0xD);

/// Models whose lamp mask gets extra bits, and the bits.
const EXTRA_LAMP_BITS: [(&[u32], u32); 2] = [(&[4, 7], 0x600_0000), (&[0xD], 0x300_0000)];

/// The mask slots set after the lamps, as `BODY_GROUPS`.
const OTHER_GROUPS: [(usize, usize); 5] = [(10, 0xE), (7, 0xB), (8, 0xC), (11, 0xA), (12, 0xF)];

/// A part's switch bit (`parts_present`): when it is clear the part's group, in this mask slot, is
/// not shown.
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
fn set_group_masks(
    lights: LightRecordsRef<'_>,
    cars: &CarDrawState,
    model: u32,
) -> Result<(), BadInput> {
    for (to, group) in BODY_GROUPS {
        let bit = model_bit(lights, model, group as u32 * 4)?;
        cars.borrow_mut().group_masks[to] = bit;
    }
    let (slot, group) = LAMPS;
    let mut lamps = model_bit(lights, model, group as u32 * 4)?;
    if lamps != 0 {
        for (models, bits) in EXTRA_LAMP_BITS {
            if models.contains(&model) {
                lamps |= bits;
            }
        }
    }
    cars.borrow_mut().group_masks[slot] = lamps;
    for (to, group) in OTHER_GROUPS {
        let bit = model_bit(lights, model, group as u32 * 4)?;
        cars.borrow_mut().group_masks[to] = bit;
    }
    Ok(())
}

/// The groups of the parts still on the car: all but those of the parts whose switch is off.
fn shown_groups(masks: &[u32; 18], switches: u32) -> u32 {
    let mut shown = u32::MAX;
    for (bit, slot) in PART_SWITCHES {
        if switches & (1 << bit) == 0 {
            shown &= !masks[slot];
        }
    }
    shown
}

/// Draws car `car`: its marks' textures (`car_number_textures`); its six kept matrices into
/// `part_matrices`, `body_turn_matrix` and `placed_matrices`; the course's group masks from its
/// model's part groups (the shown mask by `parts_present`, the state bits' mask by the view's car
/// state, the view's depth `car_depth` and the stage's fog, `car_nearness` (0 .. 1) dimming
/// `shine`) and `fade_alpha_moved`; all parts for the drawn view's own car when
/// `ViewAndRecords.view_vars.view_car_flag_a`; then the light and place,
/// `scene::set_light_direction`, the body (`object_number` 0x50 + 0x14 x car, `car_body_matrix`,
/// `scene::draw_scene_object` between the kept reflections) and, nearer than 0x3C0000, its four
/// parts by `parts_present` bits 5 .. 8 (each `car_part_matrix`, `scene::draw_scene_object`). The
/// marks are the drawing's own (`scene::car_marks`). Each drawing gets `lit_matrix`, as the
/// matrices leave it. Refused (before anything): what `scene::draw_scene_object` refuses for the
/// car's objects, and in a race (`in_race`) car `drawn_car` (below eight) without its block.
pub fn draw_car(ctx: &crate::race::view_scratch::DrawCtx<'_>, car: usize) -> Result<(), Refused> {
    let race = ctx.race;
    let n: u32 = ctx.view.state.borrow().drawn_car;
    if ctx.reads.number_vars.in_race() && n < 8 && !ctx.reads.work.has_car_block(n as usize) {
        return Err(BadInput("the drawn car without its block").into());
    }
    // The car's marks, its own while it is drawn.
    let marks = crate::scene::car_marks::CarMarks::new();
    car_number_textures(ctx.reads, car, &marks)?;
    let ctx = &crate::race::view_scratch::DrawCtx {
        marks: &marks,
        ..*ctx
    };
    let c = ctx.reads.car_parts.car(car);
    {
        let mut cars = ctx.view.cars.borrow_mut();
        cars.part_matrices = c.part_matrices;
        cars.car_turn = matrix_of(&c.place_matrix);
    }
    ctx.view.matrices.borrow_mut().body_turn_matrix = matrix_of(&c.body_matrix);
    ctx.view.cars.borrow_mut().groups_which = c.mode_on;
    let model = ctx.reads.frame_cars.car(car)?.car_kind.get();
    ctx.view.matrices.borrow_mut().known_object = model;
    set_group_masks(ctx.reads.light_records, &ctx.view.cars, model)?;
    let switches = ctx.reads.frame_cars.car(car)?.parts_present.get();
    let shown = shown_groups(&ctx.view.cars.borrow().group_masks, switches);
    ctx.view.cars.borrow_mut().shown_groups = shown;
    let field = |offset: u32| model_field(ctx.reads.light_records, model, offset);
    ctx.view.face.borrow_mut().faded_bits = field(0x10)?;
    let masks = ctx.view.cars.borrow().group_masks;
    ctx.view.cars.borrow_mut().group_masks[4] = u32::MAX
        .wrapping_sub(masks[0])
        .wrapping_sub(masks[1])
        .wrapping_sub(masks[2])
        .wrapping_sub(masks[3]);
    ctx.view.cars.borrow_mut().car_draw_kept = 0x210_0000;
    for (to, offset) in [
        (13usize, 0x18u32),
        (14, 0x1C),
        (15, 0x20),
        (16, 0x24),
        (17, 0x14),
    ] {
        ctx.view.cars.borrow_mut().group_masks[to] = field(offset)?;
    }
    let state = ctx.view.cars.borrow().state_of(car)?;
    let bit = ctx.view.cars.borrow().group_masks[17];
    ctx.view.cars.borrow_mut().group_mask = u32::MAX;
    if state & 2 != 0 {
        ctx.view.cars.borrow_mut().group_mask = with_bit(field(0x48)?, bit);
    }
    if state & 4 != 0 {
        ctx.view.cars.borrow_mut().group_mask = with_bit(field(0x44)?, bit);
    }
    ctx.view.cars.borrow_mut().car_nearness = ctx.reads.numbers.zero.get();
    let mut stage = ctx.view.state.borrow().view_car_stage;
    if (stage as i32) < 0 {
        stage = 0;
    }
    if !below(stage, 0x33) {
        stage = 0x33;
    }
    let record = ViewVars::record_of(ctx.reads.view_and_records, stage as usize)?;
    if record.fixed_zoom.get() != 0 {
        ctx.view.cars.borrow_mut().car_depth = 0x5_0000;
    }
    let and = |mask: u32| {
        let view = &ctx.view.cars;
        let kept = view.borrow().group_mask;
        view.borrow_mut().group_mask = kept & mask;
    };
    if state & 0x10 == 0 {
        nearness(ctx.reads.numbers, &ctx.view.cars, &ctx.view.course);
        let depth = ctx.view.cars.borrow().car_depth;
        // The masks are read into a `let` first: a borrow in the argument would still be held
        // when `and` writes the same record.
        let masks = ctx.view.cars.borrow().group_masks;
        if !below(depth, 0x14_0000) {
            and(with_bit(field(0x40)?, bit));
            for var in [13usize, 14, 15, 16] {
                and(!(1u32 << (masks[var] & 31)));
            }
            and(!masks[7]);
            and(!masks[8]);
        }
        if !below(depth, 0x28_0000) {
            and(!masks[9]);
            and(!masks[10]);
            and(!masks[6]);
        }
        if !below(depth, 0x3C_0000) {
            and(!masks[12]);
            and(!masks[11]);
        }
    }
    let mut fade_alpha = 0x60u32.wrapping_add(near_size(&ctx.view.cars));
    ctx.view.cars.borrow_mut().car_draw_word = fade_alpha;
    if state & 0x100 != 0 {
        if state & 6 == 0 {
            and(with_bit(field(0x40)?, bit));
        }
        fade_alpha = 0xF0;
    }
    ctx.view.face.borrow_mut().fade_alpha = fade_alpha;
    // The car's block does not keep the group mask: nothing reads it.
    let view_car = ctx.reads.view.drawn_view_car(ctx.view.drawn_view(race))?;
    let state = *ctx.view.state.borrow();
    if view_car == state.drawn_car && state.view_car_flag_a_bits != 0 {
        ctx.view.cars.borrow_mut().group_mask = u32::MAX;
    }
    ctx.view.light.borrow_mut().light_lowest = 0;
    ctx.view.light.borrow_mut().light_highest = 0xFF;
    ctx.view.light.borrow_mut().light_level = ctx.reads.stage_prefs.read(|p| p.light_level);
    ctx.view.object.borrow_mut().light_strength = ctx.reads.stage_prefs.read(|p| p.light_strength);
    ctx.view.light.borrow_mut().light_turns[0] =
        ctx.reads.stage_prefs.read(|p| p.sun_turn).wrapping_neg();
    ctx.view.light.borrow_mut().light_turns[1] = ctx.reads.stage_prefs.read(|p| p.sun_height);
    ctx.view.light.borrow_mut().view_kind = 1;
    ctx.view.light.borrow_mut().light_kind = 3;
    let this_car = ctx.reads.frame_cars.car(car)?;
    let place = || {
        ctx.view.object.borrow_mut().place[0] = this_car.position[0].get();
        ctx.view.object.borrow_mut().place[1] = this_car.position[1].get();
        ctx.view.object.borrow_mut().place[2] = this_car.position[2].get();
    };
    place();
    ctx.view.object.borrow_mut().size = 0x10;
    {
        let mut kept = ctx.view.matrices.borrow_mut();
        let view_turn = kept.view_turn_matrix;
        crate::scene::matrix::set_light_direction(
            ctx.reads.engine_tables,
            &mut kept,
            &view_turn,
            &ctx.view.light,
        );
    }
    place();
    ctx.view.object.borrow_mut().car_switches = switches;
    ctx.view.light.borrow_mut().other_light = 0;
    ctx.view.light.borrow_mut().light_kind = 3;
    ctx.view.light.borrow_mut().light_kind_extra = 0;
    let object_number = ctx
        .view
        .state
        .borrow()
        .drawn_car
        .wrapping_mul(0x14)
        .wrapping_add(0x50);
    ctx.view.cars.borrow_mut().object_number = object_number;
    car_body_matrix(ctx.reads.course(), &ctx.view.cars, &ctx.view.matrices);
    crate::race::frame_draw::start_kept_reflections(ctx.view);
    crate::scene::draw_scene_object::draw_scene_object_in_game(
        ctx,
        Some(Description::Floats(
            &ctx.view.matrices,
            DescribedMatrix::Lit,
        )),
    )?;
    crate::race::frame_draw::end_kept_reflections(ctx.view);
    ctx.view.light.borrow_mut().light_kind_extra = 1;
    if below(ctx.view.cars.borrow().car_depth, 0x3C_0000) {
        for (bit, matrix) in [0x20u32, 0x40, 0x80, 0x100]
            .into_iter()
            .zip(part_matrices(&ctx.view.cars))
        {
            let number = ctx.view.cars.borrow().object_number;
            ctx.view.cars.borrow_mut().object_number = number.wrapping_add(1);
            if ctx.view.object.borrow().car_switches & bit != 0 {
                car_part_matrix(&ctx.view.cars, &ctx.view.matrices, &matrix);
                crate::scene::draw_scene_object::draw_scene_object_in_game(
                    ctx,
                    Some(Description::Floats(
                        &ctx.view.matrices,
                        DescribedMatrix::Lit,
                    )),
                )?;
            }
        }
    }
    ctx.view.face.borrow_mut().faded_bits = u32::MAX;
    Ok(())
}

/// The car's nearness: `car_nearness` = (`car_depth` - 0xA0000) / 0x140000 held to 0 .. 1 (`zero`,
/// `one`); `shine` is multiplied by 1 less it.
fn nearness(
    numbers: &crate::race::numbers_store::Numbers,
    cars: &CarDrawState,
    mine: &crate::race::view_course::ViewCourse,
) {
    use crate::math::x87::below_or_unordered;
    use crate::math::x87_float::{ONE, int, ld, to_int};
    {
        let depth = int(cars.borrow().car_depth as i32);
        let nearness = crate::math::x87_float::st((depth - int(0xA_0000)) / int(0x14_0000));
        cars.borrow_mut().car_nearness = f32::from_bits(nearness);
        let near = cars.borrow().car_nearness;
        // Near below 0 (or no number) gives 0.
        if below_or_unordered(near, numbers.zero.get()) {
            cars.borrow_mut().car_nearness = numbers.zero.get();
        }
        let near = cars.borrow().car_nearness;
        if !below_or_unordered(near, numbers.one.get()) {
            cars.borrow_mut().car_nearness = numbers.one.get();
        }
        let near = cars.borrow().car_nearness;
        let shine = (ONE - ld(near)) * int(mine.borrow().shine as i32);
        let whole = to_int(shine);
        mine.borrow_mut().shine = whole as u32;
    }
}

/// The car's size: 0x90 times `car_nearness`, rounded.
fn near_size(cars: &CarDrawState) -> u32 {
    use crate::math::x87_float::{int, ld, to_int};
    {
        let size = to_int(int(0x90) * ld(cars.borrow().car_nearness));
        size as u32
    }
}

/// Sets `shine` for the car (0x3FCE0 less `mud_thrown_counts`, held to 0 .. 0x40000, times
/// `shine_scale`, >> 18), draws the car `car` (`draw_car`) and puts `shine` back.
pub(super) fn draw_car_shined(
    ctx: &crate::race::view_scratch::DrawCtx<'_>,
    car: usize,
) -> Result<(), Refused> {
    let shine: u32 = ctx.view.course.borrow().shine;
    let mut shine_level = 0x3_FCE0u32.wrapping_sub(ctx.reads.vars.mud_thrown_count());
    if (shine_level as i32) < 0 {
        shine_level = 0;
    }
    if !below(shine_level, 0x4_0000) {
        shine_level = 0x4_0000;
    }
    let product = shine_level as u64 * ctx.reads.stage_prefs.read(|p| p.shine_scale) as u64;
    ctx.view.course.borrow_mut().shine = (product as u32) >> 18;
    let done = draw_car(ctx, car);
    ctx.view.course.borrow_mut().shine = shine;
    done
}

/// Draws car `entry` & 7 (into `drawn_car`; `car_light` gets its `light_colour`): its place is
/// turned by `view_matrix` (`scene::rotate_point`) into `car_kept` .. `car_depth`. It is drawn
/// (`draw_car_shined`, counted in `cars_drawn`) unless behind (depth + 0x40000 below 0) or aside
/// (|x| - 0x80000 not below the depth), and those are not counted. Refused (before anything): a car
/// without its block.
pub fn draw_car_entry(
    ctx: &crate::race::view_scratch::DrawCtx<'_>,
    entry: u32,
) -> Result<(), Refused> {
    use crate::math::x87::tbyte_to_int;
    use crate::race::skidmarks::int_80;
    let n = entry & 7;
    if !ctx.reads.work.has_car_block(n as usize) {
        return Err(BadInput("a car without its block").into());
    }
    let car = ctx.reads.work.block_number(n as usize)?;
    ctx.view.vars.drawn_car.set(n);
    ctx.view.state.borrow_mut().drawn_car = n;
    ctx.view.light.borrow_mut().car_light = ctx.reads.frame_cars.car(car)?.light_colour.get();
    let t = crate::scene::rotate_point::rotate_point_by(
        &ctx.view.matrices.borrow().view_matrix,
        [
            int_80(ctx.reads.frame_cars.car(car)?.position[0].get()),
            int_80(ctx.reads.frame_cars.car(car)?.position[1].get()),
            int_80(ctx.reads.frame_cars.car(car)?.position[2].get()),
        ],
    );
    let place = t.map(tbyte_to_int);
    ctx.view.cars.borrow_mut().car_kept[0] = place[0];
    ctx.view.cars.borrow_mut().car_kept[1] = place[1];
    ctx.view.cars.borrow_mut().car_depth = place[2];
    let depth = place[2];
    let mut x = place[0];
    if below(depth, 0xFFFC_0000) {
        return Ok(());
    }
    if (x as i32) < 0 {
        x = x.wrapping_neg();
    }
    x = x.wrapping_sub(0x8_0000);
    if !below(x, depth) {
        return Ok(());
    }
    let drawn = ctx.view.cars.borrow().cars_drawn;
    ctx.view.cars.borrow_mut().cars_drawn = drawn.wrapping_add(1);
    draw_car_shined(ctx, car)
}

/// The blocks against the recorded outputs, recorded (`test_support::race_vectors`).
#[cfg(test)]
mod vector_tests {
    use super::*;
    use crate::race::asm_test::{Rng, at_each_control_word};
    use crate::test_support::own::race;
    use crate::test_support::own::scene;
    use crate::test_support::race_vectors::{Kind, Vectors};
    use crate::test_support::state::Snapshot;

    thread_local! {
        /// The shine the blocks start from and leave (a view's own in the game).
        static SHINE: std::cell::Cell<u32> = const { std::cell::Cell::new(0) };
        /// The bits of the constants `zero` and `one` the blocks read (the test's own).
        static ZERO_AND_ONE: std::cell::Cell<[u32; 2]> = const { std::cell::Cell::new([0; 2]) };
    }

    /// The stores the test writes.
    const STORES: [&str; 3] = [
        "owners.scene.draw_work",
        "owners.race.frame.frame_work",
        "owners.race.shared.work",
    ];

    /// The values the blocks read and write, without `scratch` (the assembly's spill dword) and
    /// `shine_whole` (the shine as a whole number, read by nothing): neither kept by the plain
    /// versions.
    fn state() -> [u32; 5] {
        let race = race();
        [
            race.frame.frame_work.borrow().car_depth,
            race.shared.work.get().car_nearness.get().to_bits(),
            SHINE.get(),
            ZERO_AND_ONE.get()[0],
            ZERO_AND_ONE.get()[1],
        ]
    }

    fn fill(r: &mut Rng) {
        let race = race();
        scene().draw_work_mut().scratch = 0xAAAA_AAAAu32;
        // the depth is a whole number around the view's reach
        race.frame.frame_work.borrow_mut().car_depth = (r.int() as u32) & 0x3F_FFFF;
        race.shared
            .work
            .get()
            .car_nearness
            .set(f32::from_bits(r.float()));
        SHINE.set(r.int() as u32);
        let zero = if r.below(4) == 0 {
            f32::from_bits(r.float())
        } else {
            0.0
        };
        let one = if r.below(4) == 0 {
            f32::from_bits(r.float())
        } else {
            1.0
        };
        ZERO_AND_ONE.set([zero.to_bits(), one.to_bits()]);
    }

    /// The state after the nearness, then the size.
    fn plain() -> Vec<u32> {
        let race = race();
        let cars = CarDrawState::new(crate::race::car_draw_state::CarDrawValues::new());
        cars.borrow_mut().car_depth = race.frame.frame_work.borrow().car_depth;
        let mine = crate::race::view_course::ViewCourse::new(
            crate::race::view_course::CourseValues::new(),
        );
        mine.borrow_mut().shine = SHINE.get();
        let [zero, one] = ZERO_AND_ONE.get();
        let numbers = crate::race::numbers_store::Numbers {
            zero: crate::memory::session::ConstFloat::from_bits(zero),
            one: crate::memory::session::ConstFloat::from_bits(one),
            ..race.course.numbers
        };
        nearness(&numbers, &cars, &mine);
        SHINE.set(mine.borrow().shine);
        race.shared
            .work
            .get()
            .car_nearness
            .set(cars.borrow().car_nearness);
        let mut out = state().to_vec();
        out.push(near_size(&cars));
        assert_eq!(state()[..], out[..5]);
        out
    }

    #[test]
    fn nearness_and_size_are_the_originals() {
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
