//! The cars drawn (`race::draw_list_entry` place 5: `sub_455105` -> `sub_461669` ->
//! `sub_4616AA`, part R2, IDA 2026-10-18). A car is block esi (`g_carBlocks`, `CarBlock`):
//! +0x24 its model, +0x134 .. +0x13C its place, +0x298 its parts' switches, +0x6FC its
//! state bits, +0x740 .. +0x8BF six matrices (four parts', the body's, the turn), +0xF04 the
//! shown parts' mask.

use crate::errors::{BadInput, Refused};
use crate::physics::camera_store::CAMERA_WORK;
use crate::pointers::GamePtr;
use crate::race::course_store::COURSE;
use crate::race::frame_store::FRAME_WORK;
use crate::race::light_records_store::LIGHT_RECORDS;
use crate::race::matrices_store::MATRICES;
use crate::race::numbers_store::NUMBERS;
use crate::race::prefs_store::STAGE_PREFS;
use crate::race::sound_spots_store::RACE_SOUND_SPOTS;
use crate::race::switches_store::FRAME_SWITCHES;
use crate::race::trap::car_at;
use crate::race::vars_store::RACE_VARS;
use crate::race::view_store::RACE_VIEW;
use crate::race::view_vars_store::{VIEW_VARS, ViewVars};
use crate::race::work_store::RACE_WORK;
use crate::scene::draw_store::DRAW;
use crate::scene::light_store::LIGHT_STATE;
use crate::scene::light_work_store::LIGHT_WORK;
use crate::scene::matrix::multiply_matrices;
use crate::scene::object_store::OBJECT_WORK;
use crate::scene::object_tables_store::OBJECT_TABLES;
use crate::session::Float;

fn get(a: u32) -> Result<u32, BadInput> {
    GamePtr::new(a).read("a car block or model word")
}

fn view_matrix() -> &'static [Float] {
    &MATRICES.view_matrix // g_viewMatrix
}
fn turned() -> &'static [Float] {
    &MATRICES.turned_matrix // g_turnedMatrix
}
fn lit() -> &'static [Float] {
    &MATRICES.lit_matrix // g_litMatrix
}
fn made() -> &'static [Float] {
    &MATRICES.made_object_matrix // g_madeObjectMatrix
}
fn body_turn() -> &'static [Float] {
    &MATRICES.body_turn_matrix // flt_5EA08C: the car's +0x840
}
fn car_turn() -> &'static [Float] {
    &FRAME_WORK.placed_matrices[..16] // flt_641760: the car's +0x880
}
/// The four parts' matrices (`flt_6417E0` .., the car's +0x740 ..).
pub fn part_matrices() -> [&'static [Float]; 4] {
    [0, 16, 32, 48].map(|k| &FRAME_WORK.part_matrices[k..k + 16])
}

/// `race::car_part_matrix_1` .. `_4` (0x461F12, 0x461F58, 0x461F9E, 0x461FE4): matrix `a`
/// times the body's turn (`flt_5EA08C`) into `g_turnedMatrix`, times the car's turn
/// (`flt_641760`) into `g_litMatrix`, times `g_viewMatrix` into `g_madeObjectMatrix`. The
/// original leaves esi `g_litMatrix` (which `draw_car`'s drawing takes, see there), edi
/// and ebp the last product's other two.
pub fn car_part_matrix(a: &[Float]) {
    multiply_matrices(a, body_turn(), turned());
    multiply_matrices(turned(), car_turn(), lit());
    multiply_matrices(lit(), view_matrix(), made());
}

/// `race::car_body_matrix` (0x461EA1): the turns about x and z by 0 (`scene::set_fine_turn_x`,
/// `scene::set_turn_z`) multiplied into `g_litMatrix`, then as `car_part_matrix` on it.
pub fn car_body_matrix() {
    use crate::scene::view_matrix::{Axis, set_fine_turn};
    set_fine_turn(Axis::X, 0);
    set_fine_turn(Axis::Z, 0);
    multiply_matrices(&MATRICES.turn_x_matrix, &MATRICES.turn_z_matrix, lit());
    car_part_matrix(lit());
}

/// `race::clear_car_marks` (0x46202A): the 0x40 dwords at `dword_5CD688` (the car's
/// marks: number, digits, plates) 0. (The original leaves eax 0, ecx 0, edi past them;
/// its one caller reads none.)
pub fn clear_car_marks() {
    for mark in &OBJECT_TABLES.car_mark_textures[..0x40] {
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
pub fn car_number_textures(car: u32) -> Result<(), BadInput> {
    // Converted: `rc2k_core::race::car_marks` (the rounding of +0x718 the x87's, under the
    // game's control word).
    use rc2k_core::race::car_marks::{
        decals_shown, lamp_side, letter_number, letter_place, number_digits, stage_plate,
    };
    let block = car_at(car)?;
    let float = |c: &crate::session::Float| f32::from_bits(c.bits());
    {
        let limit = f32::from_bits(FRAME_WORK.decal_limit.bits());
        let side = lamp_side(float(&block.drive_speed_a), limit);
        OBJECT_TABLES.car_mark_textures[1].set(side);
        OBJECT_TABLES.car_mark_textures[4].set(side);
        let side = lamp_side(float(&block.drive_speed_b), limit);
        OBJECT_TABLES.car_mark_textures[2].set(side);
        OBJECT_TABLES.car_mark_textures[3].set(side);
        let name = (RACE_WORK.number_texture_names[0].addr())
            .wrapping_add(car_at(car)?.car_kind.bits().wrapping_mul(0x30));
        let n = crate::math::x87::to_int(car_at(car)?.shown_engine_speed.bits());
        DRAW.scratch.set(n as u32);
        let (place, _) = letter_place(n);
        let letter =
            letter_number(GamePtr::new(name + place).read::<u8>("a car's number texture name")?)
                as u32;
        OBJECT_TABLES.car_mark_textures[5].set(letter);
        OBJECT_TABLES.car_letter_b.set(letter);
        let d = number_digits(car_at(car)?.start_slot_value.bits());
        OBJECT_TABLES.car_mark_textures[6].set(d.tens);
        OBJECT_TABLES.car_mark_textures[7].set(d.units);
        OBJECT_TABLES.car_mark_textures[9].set(d.alone);
        let (tens, _) = stage_plate(RACE_VARS.stage_number.bits());
        OBJECT_TABLES.car_mark_textures[8]
            .set(RACE_WORK.number_tens_textures[tens as usize].bits());
        if decals_shown(
            float(&block.electrics_damage),
            f32::from_bits(NUMBERS.hundred.bits()),
        ) {
            for k in 0..0x11u32 {
                let byte = block.part_switches.get(k as usize) as u32;
                let to: u32 = FRAME_WORK.car_decal_places[k as usize].bits();
                crate::scene::texture_places_store::cell_at(
                    &OBJECT_TABLES.car_mark_textures[0],
                    to,
                )
                .set(byte);
            }
        }
    }
    Ok(())
}

/// The dword `offset` bytes from the first model's groups (the original does not bound it).
fn model_field(offset: u32) -> Result<u32, BadInput> {
    GamePtr::to(&LIGHT_RECORDS.records[0].groups)
        .offset(offset)
        .read("a model's record")
}

/// The model's bit at +`offset` of its 0x134-byte record (`dword_5C90C4` ..): 1 << it, 0 for
/// -1.
fn model_bit(record: u32, offset: u32) -> Result<u32, BadInput> {
    let c = model_field(record + offset)?;
    Ok(if c == u32::MAX { 0 } else { 1u32 << (c & 31) })
}

fn below(a: u32, b: u32) -> bool {
    (a.wrapping_sub(b) as i32) < 0
}

/// `bts value, bit` (bit mod 32).
fn with_bit(value: u32, bit: u32) -> u32 {
    value | 1u32 << (bit & 31)
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
pub fn draw_car(car: u32) -> Result<(), Refused> {
    let n: u32 = VIEW_VARS.drawn_car.get();
    if NUMBERS.in_race.bits() != 0 && n < 8 && RACE_WORK.car_block(n as usize) == 0 {
        return Err(BadInput("the drawn car without its block").into());
    }
    car_number_textures(car)?;
    let [p0, p1, p2, p3] = part_matrices();
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
    .zip([p0, p1, p2, p3, body_turn(), car_turn()])
    {
        crate::math::cells::copy_cells(from, to, 16);
    }
    RACE_SOUND_SPOTS
        .groups_which
        .set(car_at(car)?.mode_on.bits());
    let model = car_at(car)?.car_kind.bits();
    MATRICES.known_object.set(model);
    let rec = model.wrapping_mul(0x134);
    for (to, offset) in [(0usize, 0u32), (1, 4), (2, 8), (3, 0xC), (6, 0x14)] {
        COURSE.group_masks[to].set(model_bit(rec, offset)?);
    }
    let mut lamps = model_bit(rec, 0x34)?;
    if lamps != 0 {
        if model == 4 || model == 7 {
            lamps |= 0x600_0000;
        }
        if model == 0xD {
            lamps |= 0x300_0000;
        }
    }
    COURSE.group_masks[9].set(lamps);
    for (to, offset) in [
        (10usize, 0x38u32),
        (7, 0x2C),
        (8, 0x30),
        (11, 0x28),
        (12, 0x3C),
    ] {
        COURSE.group_masks[to].set(model_bit(rec, offset)?);
    }
    let switches = car_at(car)?.parts_present.bits();
    let mut shown = u32::MAX;
    for (bit, var) in [
        (9u32, 6usize),
        (12, 11),
        (13, 9),
        (14, 10),
        (15, 12),
        (16, 7),
        (17, 8),
    ] {
        if switches & (1 << bit) == 0 {
            shown &= !COURSE.group_masks[var].bits();
        }
    }
    FRAME_SWITCHES.group_mask.set(shown);
    let field = |offset: u32| model_field(rec + offset);
    CAMERA_WORK.faded_bits.set(field(0x10)?);
    COURSE.group_masks[4].set(
        u32::MAX
            .wrapping_sub(COURSE.group_masks[0].bits())
            .wrapping_sub(COURSE.group_masks[1].bits())
            .wrapping_sub(COURSE.group_masks[2].bits())
            .wrapping_sub(COURSE.group_masks[3].bits()),
    );
    CAMERA_WORK.car_draw_kept.set(0x210_0000);
    for (to, offset) in [
        (13usize, 0x18u32),
        (14, 0x1C),
        (15, 0x20),
        (16, 0x24),
        (17, 0x14),
    ] {
        COURSE.group_masks[to].set(field(offset)?);
    }
    let state = car_at(car)?.unknown_6f4[2].bits();
    let bit = COURSE.group_masks[17].bits();
    VIEW_VARS.group_mask.set(u32::MAX);
    if state & 2 != 0 {
        VIEW_VARS.group_mask.set(with_bit(field(0x48)?, bit));
    }
    if state & 4 != 0 {
        VIEW_VARS.group_mask.set(with_bit(field(0x44)?, bit));
    }
    RACE_WORK.car_nearness.set_bits(NUMBERS.zero.bits());
    let mut stage = VIEW_VARS.view_car_stage.bits();
    if (stage as i32) < 0 {
        stage = 0;
    }
    if !below(stage, 0x33) {
        stage = 0x33;
    }
    let record = ViewVars::record_of(stage as usize)?;
    if get(record + 0x50)? != 0 {
        FRAME_WORK.car_depth.set(0x5_0000);
    }
    let and = |mask: u32| VIEW_VARS.group_mask.set(VIEW_VARS.group_mask.bits() & mask);
    if state & 0x10 == 0 {
        nearness();
        let depth = FRAME_WORK.car_depth.bits();
        if !below(depth, 0x14_0000) {
            and(with_bit(field(0x40)?, bit));
            for var in [13usize, 14, 15, 16] {
                and(!(1u32 << (COURSE.group_masks[var].bits() & 31)));
            }
            and(!COURSE.group_masks[7].bits());
            and(!COURSE.group_masks[8].bits());
        }
        if !below(depth, 0x28_0000) {
            and(!COURSE.group_masks[9].bits());
            and(!COURSE.group_masks[10].bits());
            and(!COURSE.group_masks[6].bits());
        }
        if !below(depth, 0x3C_0000) {
            and(!COURSE.group_masks[12].bits());
            and(!COURSE.group_masks[11].bits());
        }
    }
    let mut ecx = 0x60u32.wrapping_add(near_size());
    RACE_WORK.car_draw_word.set(ecx);
    if state & 0x100 != 0 {
        if state & 6 == 0 {
            and(with_bit(field(0x40)?, bit));
        }
        ecx = 0xF0;
    }
    VIEW_VARS.fade_alpha.set(ecx);
    car_at(car)?.group_mask.set(VIEW_VARS.group_mask.bits());
    // `view_cars[drawn_view]`: past the two, read on by address as the original does.
    let view_car = crate::session::read_on(
        &RACE_VIEW.view_cars,
        RACE_VIEW.drawn_view.bits() as usize,
        "the view cars",
    )?;
    if view_car == VIEW_VARS.drawn_car.bits() && VIEW_VARS.view_car_flag_a.bits() != 0 {
        VIEW_VARS.group_mask.set(u32::MAX);
    }
    OBJECT_WORK.light_lowest.set(0);
    OBJECT_WORK.light_highest.set(0xFF);
    OBJECT_WORK.light_level.set(STAGE_PREFS.light_level.bits());
    OBJECT_WORK
        .light_strength
        .set(STAGE_PREFS.light_strength.bits());
    OBJECT_WORK.light_turns[0].set(STAGE_PREFS.sun_turn.bits().wrapping_neg());
    OBJECT_WORK.light_turns[1].set(STAGE_PREFS.sun_height.bits());
    LIGHT_STATE.view_kind.set(1);
    LIGHT_WORK.light_kind.set(3);
    let this_car = car_at(car)?;
    let place = || {
        OBJECT_WORK.place[0].set(this_car.position[0].bits());
        OBJECT_WORK.place[1].set(this_car.position[1].bits());
        OBJECT_WORK.place[2].set(this_car.position[2].bits());
    };
    place();
    OBJECT_WORK.size.set(0x10);
    crate::scene::matrix::set_light_direction();
    place();
    OBJECT_WORK.car_switches.set(switches);
    CAMERA_WORK.drawing_car.set(1);
    OBJECT_WORK.other_light.set(0);
    LIGHT_WORK.light_kind.set(3);
    LIGHT_WORK.light_kind_extra.set(0);
    FRAME_SWITCHES.object_number.set(
        VIEW_VARS
            .drawn_car
            .bits()
            .wrapping_mul(0x14)
            .wrapping_add(0x50),
    );
    car_body_matrix();
    crate::race::frame_draw::start_kept_reflections()?;
    crate::scene::draw_scene_object::draw_scene_object_in_game(lit()[0].addr())?;
    crate::race::frame_draw::end_kept_reflections();
    LIGHT_WORK.light_kind_extra.set(1);
    if below(FRAME_WORK.car_depth.bits(), 0x3C_0000) {
        for (bit, matrix) in [0x20u32, 0x40, 0x80, 0x100]
            .into_iter()
            .zip(part_matrices())
        {
            FRAME_SWITCHES
                .object_number
                .set(FRAME_SWITCHES.object_number.bits().wrapping_add(1));
            if OBJECT_WORK.car_switches.bits() & bit != 0 {
                car_part_matrix(matrix);
                crate::scene::draw_scene_object::draw_scene_object_in_game(lit()[0].addr())?;
            }
        }
    }
    CAMERA_WORK.drawing_car.set(0);
    CAMERA_WORK.faded_bits.set(u32::MAX);
    clear_car_marks();
    Ok(())
}

/// `sub_4616AA`'s nearness: `dword_71BA08` = (`dword_6415A4` - 0xA0000) / 0x140000 held to 0 ..
/// 1 (`flt_5C5D90`, `flt_5C5D38`), `g_shine` (and `dword_5CCE78`) times 1 less it (`dword_62400C`
/// left 0x140000).
fn nearness() {
    use crate::math::x87_float::{ONE, int, ld, st, to_int};
    use crate::math::x87_load::below_or_unordered;
    {
        DRAW.scratch.set(0xA_0000u32);
        let depth = int(FRAME_WORK.car_depth.get() as i32);
        DRAW.scratch.set(0x14_0000u32);
        RACE_WORK
            .car_nearness
            .set_bits(st((depth - int(0xA_0000)) / int(0x14_0000)));
        let near: u32 = RACE_WORK.car_nearness.bits();
        // `fld zero; fld near; fcompp`: near below 0 (or no number) -> 0.
        if below_or_unordered(near, NUMBERS.zero.bits()) {
            RACE_WORK.car_nearness.set_bits(NUMBERS.zero.bits());
        }
        let near: u32 = RACE_WORK.car_nearness.bits();
        if !below_or_unordered(near, NUMBERS.one.bits()) {
            RACE_WORK.car_nearness.set_bits(NUMBERS.one.bits());
        }
        let shine = (ONE - ld(RACE_WORK.car_nearness.bits())) * int(COURSE.shine.get() as i32);
        let whole = to_int(shine);
        COURSE.shine_whole.set(whole as u32);
        COURSE.shine.set(whole as u32);
    }
}

/// `sub_4616AA`'s size: 0x90 (into `dword_62400C`) times `dword_71BA08`, rounded (left in
/// `dword_62400C`).
fn near_size() -> u32 {
    use crate::math::x87_float::{int, ld, to_int};
    {
        let size = to_int(int(0x90) * ld(RACE_WORK.car_nearness.bits()));
        DRAW.scratch.set(size as u32);
        size as u32
    }
}

/// `race::draw_car_shined` (0x461669): `g_shine` for the car (0x3FCE0 less `dword_623D80`,
/// held to 0 .. 0x40000, times `dword_60AD0C`, >> 18), the car `car` (esi, its block)
/// drawn (`race::draw_car`), `g_shine` put back. Of its callers' registers only esi is
/// read (by `draw_car`).
pub fn draw_car_shined(car: u32) -> Result<(), Refused> {
    let shine: u32 = COURSE.shine.get();
    let mut eax = 0x3_FCE0u32.wrapping_sub(RACE_VARS.mud_thrown_counts[0].bits());
    if (eax as i32) < 0 {
        eax = 0;
    }
    if !below(eax, 0x4_0000) {
        eax = 0x4_0000;
    }
    let product = eax as u64 * STAGE_PREFS.shine_scale.get() as u64;
    COURSE.shine.set((product as u32) >> 18);
    let done = draw_car(car);
    COURSE.shine.set(shine);
    done
}

/// `race::draw_car_entry` (0x455105): car `entry` (ebx) & 7 (into `dword_60627C`; `dword_62DAD4` its
/// +0x240): its place turned by `g_viewMatrix` (`scene::rotate_point`) into `dword_64159C` ..
/// `dword_6415A4`; drawn (`race::draw_car_shined`, counted in `dword_641594`) unless
/// behind (depth + 0x40000 below 0) or aside (|x| - 0x80000 not below the depth), counted in
/// `dword_641598`. Refuses (before anything) a car without its block.
pub fn draw_car_entry(entry: u32) -> Result<(), Refused> {
    use crate::math::x87_load::tbyte_to_int;
    use crate::race::skidmarks::int_80;
    let n = entry & 7;
    let car: u32 = RACE_WORK.car_block(n as usize);
    if car == 0 {
        return Err(BadInput("a car without its block").into());
    }
    VIEW_VARS.drawn_car.set(n);
    LIGHT_WORK
        .view_light_base
        .set(car_at(car)?.light_colour.bits());
    let t = crate::scene::rotate_point::rotate_point(
        view_matrix(),
        [
            int_80(car_at(car)?.position[0].bits()),
            int_80(car_at(car)?.position[1].bits()),
            int_80(car_at(car)?.position[2].bits()),
        ],
    );
    let place = t.map(tbyte_to_int);
    FRAME_WORK.car_kept[0].set_bits(place[0]);
    FRAME_WORK.car_kept[1].set_bits(place[1]);
    FRAME_WORK.car_depth.set_bits(place[2]);
    let depth = place[2];
    let mut x = place[0];
    if below(depth, 0xFFFC_0000) {
        FRAME_WORK
            .cars_culled
            .set(FRAME_WORK.cars_culled.get().wrapping_add(1));
        return Ok(());
    }
    if (x as i32) < 0 {
        x = x.wrapping_neg();
    }
    x = x.wrapping_sub(0x8_0000);
    if !below(x, depth) {
        FRAME_WORK
            .cars_culled
            .set(FRAME_WORK.cars_culled.get().wrapping_add(1));
        return Ok(());
    }
    FRAME_WORK
        .cars_drawn
        .set(FRAME_WORK.cars_drawn.get().wrapping_add(1));
    draw_car_shined(car)
}

#[cfg(all(test, target_arch = "x86"))]
mod asm_reference {
    use super::*;
    use crate::framework::{game_read, rva};
    use crate::race::asm_test::{Rng, Saved, at_each_control_word, lock};

    // The original blocks, verbatim (with the clamps between them).
    pub(super) unsafe fn nearness_asm() {
        use crate::math::x87_load::below_or_unordered;
        unsafe {
            DRAW.scratch.set(0xA_0000u32);
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
            let near: u32 = RACE_WORK.car_nearness.bits();
            // `fld zero; fld near; fcompp`: near below 0 (or no number) -> 0.
            if below_or_unordered(near, NUMBERS.zero.bits()) {
                RACE_WORK.car_nearness.set_bits(NUMBERS.zero.bits());
            }
            let near: u32 = RACE_WORK.car_nearness.bits();
            if !below_or_unordered(near, NUMBERS.one.bits()) {
                RACE_WORK.car_nearness.set_bits(NUMBERS.one.bits());
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

    /// `sub_4616AA`'s size: 0x90 (into `dword_62400C`) times `dword_71BA08`, rounded (left in
    /// `dword_62400C`).
    pub(super) unsafe fn near_size_asm() -> u32 {
        unsafe {
            DRAW.scratch.set(0x90u32);
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
            DRAW.scratch.bits()
        }
    }

    const PLACES: [usize; 8] = [
        0x62400C, 0x6415A4, 0x71BA08, 0x5CCE78, 0x5CCE74, 0x5C5D90, 0x5C5D38, 0x6415A4,
    ];

    fn state() -> Vec<u32> {
        PLACES.iter().map(|&a| game_read(rva(a))).collect()
    }

    fn fill(r: &mut Rng) {
        DRAW.scratch.set(0xAAAA_AAAAu32);
        // the depth is a whole number around the view's reach
        FRAME_WORK.car_depth.set((r.int() as u32) & 0x3F_FFFF);
        RACE_WORK.car_nearness.set_bits(r.float());
        COURSE.shine_whole.set(0xAAAA_AAAAu32);
        COURSE.shine.set(r.int() as u32);
        NUMBERS
            .zero
            .set_bits(if r.below(4) == 0 { r.float() } else { 0 });
        NUMBERS.one.set_bits(if r.below(4) == 0 {
            r.float()
        } else {
            0x3F80_0000
        });
    }

    #[test]
    fn nearness_and_size_match() {
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
                nearness();
                assert_eq!(state(), want, "nearness from {before:x?}");
                let size_got = near_size();
                assert_eq!(size_got, size_want, "size from {before:x?}");
                assert_eq!(state(), want_size_place);
            }
        });
    }
}
