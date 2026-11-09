//! The cars drawn (`race::draw_list_entry` place 5: `race::draw_car_entry` ->
//! `race::draw_car_shined` ->
//! `race::draw_car`, part R2, IDA 2026-10-18). A car is block esi (`g_carBlocks`, `CarBlock`):
//! +0x24 its model, +0x134 .. +0x13C its place, +0x298 its parts' switches, +0x6FC its
//! state bits, +0x740 .. +0x8BF six matrices (four parts', the body's, the turn), +0xF04 the
//! shown parts' mask.

use crate::errors::{BadInput, Refused};
use crate::pointers::GamePtr;
use crate::race::frame_view::car_at as car_view;
use crate::race::light_records_store::LIGHT_RECORDS;
use crate::race::owner::Race;
use crate::race::owner_course::Course;
use crate::race::owner_frame::Frame;
use crate::race::owner_shared::Shared;
use crate::race::trap::car_at;
use crate::race::view_vars_store::ViewVars;
use crate::scene::matrix::multiply_matrices;
use crate::session::Float;

fn view_matrix(frame: &Frame) -> &'_ [Float] {
    &frame.matrices.view_matrix // g_viewMatrix
}
fn turned(frame: &Frame) -> &'_ [Float] {
    &frame.matrices.turned_matrix // g_turnedMatrix
}
fn lit(frame: &Frame) -> &'_ [Float] {
    &frame.matrices.lit_matrix // g_litMatrix
}
/// `g_litMatrix` as the esi the drawing gets (a description read there runs past the
/// matrix: bounded by the whole store).
fn lit_place(frame: &Frame) -> crate::pointers::GamePtr {
    crate::pointers::GamePtr::to(&frame.matrices)
        .offset(
            core::mem::offset_of!(crate::race::matrices_store::SceneMatrices, lit_matrix) as u32,
        )
}
fn made(frame: &Frame) -> &'_ [Float] {
    &frame.matrices.made_object_matrix // g_madeObjectMatrix
}
fn body_turn(frame: &Frame) -> &'_ [Float] {
    &frame.matrices.body_turn_matrix // flt_5EA08C: the car's +0x840
}
fn car_turn(frame: &Frame) -> &'_ [Float] {
    &frame.frame_work.placed_matrices[..16] // flt_641760: the car's +0x880
}
/// The four parts' matrices (`flt_6417E0` .., the car's +0x740 ..).
pub fn part_matrices(frame: &Frame) -> [&'_ [Float]; 4] {
    [0, 16, 32, 48].map(|k| &frame.frame_work.part_matrices[k..k + 16])
}

/// `race::car_part_matrix_1` .. `_4` (0x461F12, 0x461F58, 0x461F9E, 0x461FE4): matrix `a`
/// times the body's turn (`flt_5EA08C`) into `g_turnedMatrix`, times the car's turn
/// (`flt_641760`) into `g_litMatrix`, times `g_viewMatrix` into `g_madeObjectMatrix`. The
/// original leaves esi `g_litMatrix` (which `draw_car`'s drawing takes, see there), edi
/// and ebp the last product's other two.
pub fn car_part_matrix(frame: &Frame, a: &[Float]) {
    multiply_matrices(a, body_turn(frame), turned(frame));
    multiply_matrices(turned(frame), car_turn(frame), lit(frame));
    multiply_matrices(lit(frame), view_matrix(frame), made(frame));
}

/// `race::car_body_matrix` (0x461EA1): the turns about x and z by 0 (`scene::set_fine_turn_x`,
/// `scene::set_turn_z`) multiplied into `g_litMatrix`, then as `car_part_matrix` on it.
pub fn car_body_matrix(course: &Course, frame: &Frame) {
    use crate::scene::view_matrix::{Axis, set_fine_turn};
    set_fine_turn(course, &frame.matrices, Axis::X, 0);
    set_fine_turn(course, &frame.matrices, Axis::Z, 0);
    multiply_matrices(
        &frame.matrices.turn_x_matrix,
        &frame.matrices.turn_z_matrix,
        lit(frame),
    );
    car_part_matrix(frame, lit(frame));
}

/// `race::clear_car_marks` (0x46202A): the 0x40 dwords at `dword_5CD688` (the car's
/// marks: number, digits, plates) 0. (The original leaves eax 0, ecx 0, edi past them;
/// its one caller reads none.)
pub fn clear_car_marks(scene: &crate::scene::owner::Scene) {
    for mark in &scene.objects.tables.car_mark_textures[..0x40] {
        mark.set(0);
    }
}

/// `race::car_number_textures` (0x46203D): car `car`'s (esi) marks' textures
/// (`dword_5CD688` ..): +0x1B8 / +0x1BC (|x| not below `flt_641740`: 1) into
/// `dword_5CD68C`, `dword_5CD698` / `dword_5CD690`, `dword_5CD694`; the letter of its
/// model's name (`unk_71B408` + 0x30 x model) at +0x718 (rounded, held to 0 .. 0x2EDF) /
/// 250, as a digit or letter number, into `dword_5CD69C`, `dword_5CDA88`; its number
/// +0xAA8 as tens and units (`dword_5CD6A0`, `dword_5CD6A4`, 999 for none; `dword_5CD6AC`
/// the units alone below ten); the stage's (held to 11 .. 0x50) tens' plate
/// `dword_71BA10[]` into `dword_5CD6A8`; and while +0x310 is below `flt_5C5D14` its 17
/// bytes +0xAC0 .. into the places `dword_6416F8[]` names. The registers it leaves are
/// read by no one (`draw_car` reads only esi, which it keeps).
pub fn car_number_textures(
    race: &Race,
    car: u32,
    scene: &crate::scene::owner::Scene,
) -> Result<(), BadInput> {
    // Converted: `rc2k_core::race::car_marks` (the rounding of +0x718 the x87's, under the
    // game's control word).
    use rc2k_core::race::car_marks::{
        decals_shown, lamp_side, letter_number, letter_place, number_digits, stage_plate,
    };
    let block = car_view(car)?;
    let float = |c: &crate::session::Float| f32::from_bits(c.bits());
    {
        let limit = f32::from_bits(race.frame.frame_work.decal_limit.bits());
        let side = lamp_side(float(&block.drive_speed_a), limit);
        scene.objects.tables.car_mark_textures[1].set(side);
        scene.objects.tables.car_mark_textures[4].set(side);
        let side = lamp_side(float(&block.drive_speed_b), limit);
        scene.objects.tables.car_mark_textures[2].set(side);
        scene.objects.tables.car_mark_textures[3].set(side);
        // Unbounded: 34 kinds of 0x30 bytes, the table holds 32.
        let name = GamePtr::to(&race.shared.work.number_texture_names)
            .unbounded()
            .offset(car_view(car)?.car_kind.bits().wrapping_mul(0x30));
        let n = crate::math::x87::to_int(car_at(car)?.shown_engine_speed.bits());
        let (place, _) = letter_place(n);
        let letter = letter_number(
            name.offset(place)
                .read::<u8>("a car's number texture name")?,
        ) as u32;
        scene.objects.tables.car_mark_textures[5].set(letter);
        scene.objects.tables.car_letter_b.set(letter);
        let d = number_digits(car_view(car)?.start_slot_value.bits());
        scene.objects.tables.car_mark_textures[6].set(d.tens);
        scene.objects.tables.car_mark_textures[7].set(d.units);
        scene.objects.tables.car_mark_textures[9].set(d.alone);
        let (tens, _) = stage_plate(race.shared.vars.stage_number.bits());
        scene.objects.tables.car_mark_textures[8]
            .set(race.shared.work.number_tens_textures[tens as usize].bits());
        if decals_shown(
            float(&block.electrics_damage),
            f32::from_bits(race.course.numbers.hundred.bits()),
        ) {
            for k in 0..0x11u32 {
                let byte = car_at(car)?.part_switches.get(k as usize) as u32;
                let to: u32 = race.frame.frame_work.car_decal_places[k as usize].bits();
                crate::scene::texture_places_store::cell_at(
                    &scene.objects.tables.car_mark_textures[0],
                    to,
                )
                .set(byte);
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

/// The model's bit at +`offset` of its groups (`dword_5C90C4` ..): 1 << it, 0 for -1.
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

/// A mask slot of `race().course.course.group_masks` and the index into the model's `groups` whose bit it
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
fn set_group_masks(course: &Course, model: u32) -> Result<(), BadInput> {
    for (to, group) in BODY_GROUPS {
        course.course.group_masks[to].set(model_bit(model, group as u32 * 4)?);
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
    course.course.group_masks[slot].set(lamps);
    for (to, group) in OTHER_GROUPS {
        course.course.group_masks[to].set(model_bit(model, group as u32 * 4)?);
    }
    Ok(())
}

/// The groups of the parts still on the car: all but those of the parts whose switch is off.
fn shown_groups(course: &Course, switches: u32) -> u32 {
    let mut shown = u32::MAX;
    for (bit, slot) in PART_SWITCHES {
        if switches & (1 << bit) == 0 {
            shown &= !course.course.group_masks[slot].bits();
        }
    }
    shown
}

/// `race::draw_car` (0x4616AA): car `car` (esi, its block) drawn: its marks' textures
/// (`race::car_number_textures`); its six matrices (+0x740 ..) to `flt_6417E0` .. /
/// `flt_5EA08C` / `flt_641760`; its model's parts' bits (`dword_5CCE1C` ..; the shown mask
/// `dword_62150C` by +0x298, `dword_606280` by +0x6FC, the view's depth `dword_6415A4` and
/// the stage's fog, `dword_71BA08` the nearness (0 .. 1) dimming `g_shine`), `dword_606284`,
/// +0xF04; all parts for the drawn view's own car when `dword_60626C`; the light and place;
/// `scene::set_light_direction`; the body (`g_objectNumber` 0x50 + 0x14 x car,
/// `race::car_body_matrix`, `scene::draw_scene_object` between the kept reflections), and
/// nearer than 0x3C0000 its four parts by +0x298's bits 5 .. 8 (each `car_part_matrix`,
/// `scene::draw_scene_object`); then `race::clear_car_marks`. Each drawing gets esi
/// `g_litMatrix`, as the matrices leave it (the object's description, read only on the
/// described way). Refuses (before anything) what
/// `scene::draw_scene_object` would for the car's objects: in a race (`g_inRace`) car
/// `dword_60627C` (below eight) without its block.
pub fn draw_car(ctx: &crate::race::view_scratch::DrawCtx<'_>, car: u32) -> Result<(), Refused> {
    let (race, scene) = (ctx.race, ctx.scene);
    let n: u32 = race.frame.view_and_records.view_vars.drawn_car.get();
    if race.course.numbers.in_race.bits() != 0
        && n < 8
        && race.shared.work.car_block(n as usize) == 0
    {
        return Err(BadInput("the drawn car without its block").into());
    }
    car_number_textures(race, car, scene)?;
    let [p0, p1, p2, p3] = part_matrices(&race.frame);
    let c = car_at(car)?;
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
        body_turn(&race.frame),
        car_turn(&race.frame),
    ]) {
        crate::math::cells::copy_cells(from, to, 16);
    }
    race.frame
        .sound_spots
        .groups_which
        .set(car_at(car)?.mode_on.bits());
    let model = car_view(car)?.car_kind.bits();
    race.frame.matrices.known_object.set(model);
    set_group_masks(&race.course, model)?;
    let switches = car_view(car)?.parts_present.bits();
    let shown = shown_groups(&race.course, switches);
    race.shared.switches.group_mask.set(shown);
    let field = |offset: u32| model_field(model, offset);
    race.physics.camera.faded_bits.set(field(0x10)?);
    race.course.course.group_masks[4].set(
        u32::MAX
            .wrapping_sub(race.course.course.group_masks[0].bits())
            .wrapping_sub(race.course.course.group_masks[1].bits())
            .wrapping_sub(race.course.course.group_masks[2].bits())
            .wrapping_sub(race.course.course.group_masks[3].bits()),
    );
    race.physics.camera.car_draw_kept.set(0x210_0000);
    for (to, offset) in [
        (13usize, 0x18u32),
        (14, 0x1C),
        (15, 0x20),
        (16, 0x24),
        (17, 0x14),
    ] {
        race.course.course.group_masks[to].set(field(offset)?);
    }
    let state = car_at(car)?.state_bits.bits();
    let bit = race.course.course.group_masks[17].bits();
    race.frame
        .view_and_records
        .view_vars
        .group_mask
        .set(u32::MAX);
    if state & 2 != 0 {
        race.frame
            .view_and_records
            .view_vars
            .group_mask
            .set(with_bit(field(0x48)?, bit));
    }
    if state & 4 != 0 {
        race.frame
            .view_and_records
            .view_vars
            .group_mask
            .set(with_bit(field(0x44)?, bit));
    }
    race.shared
        .work
        .car_nearness
        .set_bits(race.course.numbers.zero.bits());
    let mut stage = race.frame.view_and_records.view_vars.view_car_stage.bits();
    if (stage as i32) < 0 {
        stage = 0;
    }
    if !below(stage, 0x33) {
        stage = 0x33;
    }
    let record = ViewVars::record_of(stage as usize)?;
    if record.fixed_zoom.get() != 0 {
        race.frame.frame_work.car_depth.set(0x5_0000);
    }
    let and = |mask: u32| {
        let view = &race.frame.view_and_records.view_vars;
        view.group_mask.set(view.group_mask.bits() & mask);
    };
    if state & 0x10 == 0 {
        nearness(race);
        let depth = race.frame.frame_work.car_depth.bits();
        if !below(depth, 0x14_0000) {
            and(with_bit(field(0x40)?, bit));
            for var in [13usize, 14, 15, 16] {
                and(!(1u32 << (race.course.course.group_masks[var].bits() & 31)));
            }
            and(!race.course.course.group_masks[7].bits());
            and(!race.course.course.group_masks[8].bits());
        }
        if !below(depth, 0x28_0000) {
            and(!race.course.course.group_masks[9].bits());
            and(!race.course.course.group_masks[10].bits());
            and(!race.course.course.group_masks[6].bits());
        }
        if !below(depth, 0x3C_0000) {
            and(!race.course.course.group_masks[12].bits());
            and(!race.course.course.group_masks[11].bits());
        }
    }
    let mut ecx = 0x60u32.wrapping_add(near_size(&race.shared));
    race.shared.work.car_draw_word.set(ecx);
    if state & 0x100 != 0 {
        if state & 6 == 0 {
            and(with_bit(field(0x40)?, bit));
        }
        ecx = 0xF0;
    }
    race.frame.fade_alpha.set(ecx);
    car_at(car)?
        .group_mask
        .set(race.frame.view_and_records.view_vars.group_mask.bits());
    // `view_cars[drawn_view]`: past the two, read on by address as the original does.
    let view_car = crate::session::read_on_in(
        &*race.shared.view,
        &race.shared.view.view_cars,
        race.shared.view.drawn_view.bits() as usize,
        "the view cars",
    )?;
    if view_car == race.frame.view_and_records.view_vars.drawn_car.bits()
        && race.frame.view_and_records.view_vars.view_car_flag_a.bits() != 0
    {
        race.frame
            .view_and_records
            .view_vars
            .group_mask
            .set(u32::MAX);
    }
    scene.drawing.object_work.light_lowest.set(0);
    scene.drawing.object_work.light_highest.set(0xFF);
    scene
        .drawing
        .object_work
        .light_level
        .set(race.course.stage_prefs.light_level.bits());
    scene
        .drawing
        .object_work
        .light_strength
        .set(race.course.stage_prefs.light_strength.bits());
    scene.drawing.object_work.light_turns[0]
        .set(race.course.stage_prefs.sun_turn.bits().wrapping_neg());
    scene.drawing.object_work.light_turns[1].set(race.course.stage_prefs.sun_height.bits());
    scene.lights.state.view_kind.set(1);
    scene.lights.work.light_kind.set(3);
    let this_car = car_view(car)?;
    let place = || {
        scene.drawing.object_work.place[0].set(this_car.position[0].bits());
        scene.drawing.object_work.place[1].set(this_car.position[1].bits());
        scene.drawing.object_work.place[2].set(this_car.position[2].bits());
    };
    place();
    scene.drawing.object_work.size.set(0x10);
    crate::scene::matrix::set_light_direction(&race.frame.matrices, scene);
    place();
    scene.drawing.object_work.car_switches.set(switches);
    race.physics.camera.drawing_car.set(1);
    scene.drawing.object_work.other_light.set(0);
    scene.lights.work.light_kind.set(3);
    scene.lights.work.light_kind_extra.set(0);
    race.shared.switches.object_number.set(
        race.frame
            .view_and_records
            .view_vars
            .drawn_car
            .bits()
            .wrapping_mul(0x14)
            .wrapping_add(0x50),
    );
    car_body_matrix(&race.course, &race.frame);
    crate::race::frame_draw::start_kept_reflections(race, &race.physics)?;
    crate::scene::draw_scene_object::draw_scene_object_in_game(
        ctx,
        Some(lit_place(&race.frame).into()),
    )?;
    crate::race::frame_draw::end_kept_reflections(&race.shared);
    scene.lights.work.light_kind_extra.set(1);
    if below(race.frame.frame_work.car_depth.bits(), 0x3C_0000) {
        for (bit, matrix) in [0x20u32, 0x40, 0x80, 0x100]
            .into_iter()
            .zip(part_matrices(&race.frame))
        {
            race.shared
                .switches
                .object_number
                .set(race.shared.switches.object_number.bits().wrapping_add(1));
            if scene.drawing.object_work.car_switches.bits() & bit != 0 {
                car_part_matrix(&race.frame, matrix);
                crate::scene::draw_scene_object::draw_scene_object_in_game(
                    ctx,
                    Some(lit_place(&race.frame).into()),
                )?;
            }
        }
    }
    race.physics.camera.drawing_car.set(0);
    race.physics.camera.faded_bits.set(u32::MAX);
    clear_car_marks(scene);
    Ok(())
}

/// `race::draw_car`'s nearness: `dword_71BA08` = (`dword_6415A4` - 0xA0000) / 0x140000 held to 0 ..
/// 1 (`flt_5C5D90`, `flt_5C5D38`), `g_shine` (and `dword_5CCE78`) times 1 less it (the
/// original's stores to `dword_62400C` are not kept).
fn nearness(race: &Race) {
    use crate::math::x87_float::{ONE, int, ld, st, to_int};
    use crate::math::x87_load::below_or_unordered;
    {
        let depth = int(race.frame.frame_work.car_depth.get() as i32);
        race.shared
            .work
            .car_nearness
            .set_bits(st((depth - int(0xA_0000)) / int(0x14_0000)));
        let near: u32 = race.shared.work.car_nearness.bits();
        // `fld zero; fld near; fcompp`: near below 0 (or no number) -> 0.
        if below_or_unordered(near, race.course.numbers.zero.bits()) {
            race.shared
                .work
                .car_nearness
                .set_bits(race.course.numbers.zero.bits());
        }
        let near: u32 = race.shared.work.car_nearness.bits();
        if !below_or_unordered(near, race.course.numbers.one.bits()) {
            race.shared
                .work
                .car_nearness
                .set_bits(race.course.numbers.one.bits());
        }
        let shine = (ONE - ld(race.shared.work.car_nearness.bits()))
            * int(race.course.course.shine.get() as i32);
        let whole = to_int(shine);
        race.course.course.shine_whole.set(whole as u32);
        race.course.course.shine.set(whole as u32);
    }
}

/// `race::draw_car`'s size: 0x90 times `dword_71BA08`, rounded (the original's stores to
/// `dword_62400C` are not kept).
fn near_size(shared: &Shared) -> u32 {
    use crate::math::x87_float::{int, ld, to_int};
    {
        let size = to_int(int(0x90) * ld(shared.work.car_nearness.bits()));
        size as u32
    }
}

/// `race::draw_car_shined` (0x461669): `g_shine` for the car (0x3FCE0 less `dword_623D80`,
/// held to 0 .. 0x40000, times `dword_60AD0C`, >> 18), the car `car` (esi, its block)
/// drawn (`race::draw_car`), `g_shine` put back. Of its callers' registers only esi is
/// read (by `draw_car`).
pub fn draw_car_shined(
    ctx: &crate::race::view_scratch::DrawCtx<'_>,
    car: u32,
) -> Result<(), Refused> {
    let race = ctx.race;
    let course = &race.course;
    let shine: u32 = course.course.shine.get();
    let mut eax = 0x3_FCE0u32.wrapping_sub(race.shared.vars.mud_thrown_counts[0].bits());
    if (eax as i32) < 0 {
        eax = 0;
    }
    if !below(eax, 0x4_0000) {
        eax = 0x4_0000;
    }
    let product = eax as u64 * course.stage_prefs.shine_scale.get() as u64;
    course.course.shine.set((product as u32) >> 18);
    let done = draw_car(ctx, car);
    course.course.shine.set(shine);
    done
}

/// `race::draw_car_entry` (0x455105): car `entry` (ebx) & 7 (into `dword_60627C`; `dword_62DAD4` its
/// +0x240): its place turned by `g_viewMatrix` (`scene::rotate_point`) into `dword_64159C` ..
/// `dword_6415A4`; drawn (`race::draw_car_shined`, counted in `dword_641594`) unless
/// behind (depth + 0x40000 below 0) or aside (|x| - 0x80000 not below the depth), counted in
/// `dword_641598`. Refuses (before anything) a car without its block.
pub fn draw_car_entry(
    ctx: &crate::race::view_scratch::DrawCtx<'_>,
    entry: u32,
) -> Result<(), Refused> {
    let (race, scene) = (ctx.race, ctx.scene);
    let frame = &race.frame;
    use crate::math::x87_load::tbyte_to_int;
    use crate::race::skidmarks::int_80;
    let n = entry & 7;
    let car: u32 = race.shared.work.car_block(n as usize);
    if car == 0 {
        return Err(BadInput("a car without its block").into());
    }
    frame.view_and_records.view_vars.drawn_car.set(n);
    scene
        .lights
        .work
        .view_light_base
        .set(car_view(car)?.light_colour.bits());
    let t = crate::scene::rotate_point::rotate_point(
        view_matrix(&race.frame),
        [
            int_80(car_view(car)?.position[0].bits()),
            int_80(car_view(car)?.position[1].bits()),
            int_80(car_view(car)?.position[2].bits()),
        ],
    );
    let place = t.map(tbyte_to_int);
    frame.frame_work.car_kept[0].set_bits(place[0]);
    frame.frame_work.car_kept[1].set_bits(place[1]);
    frame.frame_work.car_depth.set_bits(place[2]);
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
    frame
        .frame_work
        .cars_drawn
        .set(frame.frame_work.cars_drawn.get().wrapping_add(1));
    draw_car_shined(ctx, car)
}

#[cfg(all(test, target_arch = "x86"))]
mod asm_reference {
    use super::*;
    use crate::framework::{game_read, rva};
    use crate::race::asm_test::{Rng, Saved, at_each_control_word, lock};
    use crate::race::owner::race;
    use crate::scene::owner::scene;

    // The original blocks, verbatim (with the clamps between them).
    pub(super) unsafe fn nearness_asm() {
        let shared = &race().shared;
        let course = &race().course;
        use crate::math::x87_load::below_or_unordered;
        unsafe {
            scene().drawing.scene.scratch.set(0xA_0000u32);
            let old = crate::race::store::OldPlaces::lay(&[0x62400c, 0x6415a4, 0x71ba08]);
            core::arch::asm!(
                "fild dword ptr [{rc} + 0x2415A4]",
                "fisub dword ptr [{rc} + 0x22400C]",
                "mov dword ptr [{rc} + 0x22400C], 0x140000",
                "fidiv dword ptr [{rc} + 0x22400C]",
                "fstp dword ptr [{rc} + 0x31BA08]",
                rc = in(reg) crate::race::store::asm_base(),
                out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
                out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
                options(nostack),
            );
            drop(old);
            let near: u32 = shared.work.car_nearness.bits();
            // `fld zero; fld near; fcompp`: near below 0 (or no number) -> 0.
            if below_or_unordered(near, course.numbers.zero.bits()) {
                shared
                    .work
                    .car_nearness
                    .set_bits(course.numbers.zero.bits());
            }
            let near: u32 = shared.work.car_nearness.bits();
            if !below_or_unordered(near, course.numbers.one.bits()) {
                shared.work.car_nearness.set_bits(course.numbers.one.bits());
            }
            let old = crate::race::store::OldPlaces::lay(&[0x5cce74, 0x5cce78, 0x71ba08]);
            core::arch::asm!(
                ".byte 0xD9, 0xE8",
                "fsub dword ptr [{rc} + 0x31BA08]",
                "fimul dword ptr [{rc} + 0x1CCE74]",
                "fist dword ptr [{rc} + 0x1CCE78]",
                "fistp dword ptr [{rc} + 0x1CCE74]",
                rc = in(reg) crate::race::store::asm_base(),
                out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
                out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
                options(nostack),
            );
            drop(old);
        }
    }

    /// `race::draw_car`'s size: 0x90 (into `dword_62400C`) times `dword_71BA08`, rounded (left in
    /// `dword_62400C`).
    pub(super) unsafe fn near_size_asm() -> u32 {
        let drawing = &scene().drawing;
        unsafe {
            drawing.scene.scratch.set(0x90u32);
            let old = crate::race::store::OldPlaces::lay(&[0x62400c, 0x71ba08]);
            core::arch::asm!(
                "fild dword ptr [{rc} + 0x22400C]",
                "fmul dword ptr [{rc} + 0x31BA08]",
                "fistp dword ptr [{rc} + 0x22400C]",
                rc = in(reg) crate::race::store::asm_base(),
                out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
                out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
                options(nostack),
            );
            drop(old);
            drawing.scene.scratch.bits()
        }
    }

    const PLACES: [usize; 8] = [
        0x62400C, 0x6415A4, 0x71BA08, 0x5CCE78, 0x5CCE74, 0x5C5D90, 0x5C5D38, 0x6415A4,
    ];

    /// The places' values, without `dword_62400C` (the assembly's spill dword, not kept by
    /// the Rust versions).
    fn state() -> Vec<u32> {
        let mut v: Vec<u32> = PLACES.iter().map(|&a| game_read(rva(a))).collect();
        v[0] = 0;
        v
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
        race.course.course.shine_whole.set(0xAAAA_AAAAu32);
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

    #[test]
    fn nearness_and_size_match() {
        let race = race();
        let _l = lock();
        let _saved = Saved::new(&PLACES);
        let mut r = Rng(6);
        at_each_control_word(|| {
            for _ in 0..20000 {
                let seed = r.0;
                fill(&mut r);
                let before = state();
                unsafe { nearness_asm() };
                let want = state();
                let size_want = unsafe { near_size_asm() };
                let want_size_place = state();
                // again with the plain versions
                r.0 = seed;
                fill(&mut r);
                assert_eq!(state(), before);
                nearness(race);
                assert_eq!(state(), want, "nearness from {before:x?}");
                let size_got = near_size(&race.shared);
                assert_eq!(size_got, size_want, "size from {before:x?}");
                assert_eq!(state(), want_size_place);
            }
        });
    }
}
