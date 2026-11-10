//! `drive::share_car_state` (0x45699C): one car's state from its block (esi, 0xFE0 bytes)
//! into its record (edi, `off_71B3D8[n]`, read by the drawing and the sounds) and back:
//! - a count at +0xFAC: when not 0, record +0x30 = 1 and the count less 1;
//! - record +0x2C = the sound object's (ebp) +0xF0, which is cleared (`xchg`);
//! - two levels (record +0x4C0, +0x4C4) from block +0x2B4, +0x2B8: 0 below
//!   `flt_63DB5C`, else (x - it) / `flt_5C5D44` `fistp`'d (through `dword_62400C`) held to
//!   0 .. 7 (the sign of the difference);
//! - the car file's light record (`g_lightRecords`, 0x134 each, by
//!   `race::car_file_number` of block +0x24) +0x2C .. +0x38 into record +0xC .. +0x18;
//! - the block named at record +0x24 and +0x28 (the original wrote the addresses of
//!   block +0x8C0 and +0x740 there; a block handle here: `CarEffects::whole`,
//!   `part_matrix`);
//! - about eighty fields moved as the original moves them (`MOVES`, the wheels' four
//!   each), four of them cleared by the block's switches (+0x600 .. and bits 5 .. 8 of
//!   +0x298), a part only when block +0xA34 is set;
//! - record +0x258 (6 dwords) into block +0x700, whose 0x80000000 entries are then
//!   filled from the next one that is not (round, at most six looked at); record +0x32C
//!   (8) into block +0xA3C, record +0x3F4 (0x11) into block +0xA5C.
//!
//! Takes: the block, the record, the sound object
//!
//! Caller: `drive::share_all_cars` (0x456484), for each of the eight cars, inside `pusha` / `popa`.
//! Refuses (before anything) a car file number above 0x21 (past `g_lightRecords`).

use crate::errors::{BadInput, Refused};
use crate::math::x87;
use crate::math::x87_run::{compare_full, sub_div_to_int};
use crate::physics::car_store::Car;
use crate::pointers::GamePtr;
use crate::race::effects_settings_store::CarEffects;
use crate::race::light_records_store::LIGHT_RECORDS;
use crate::race::mud::car_file_number;
use crate::race::owner::Race;
use crate::session::Dword;
use crate::sound::store::SoundObject;
use rc2k_core::drive::sharing;

use crate::race::light_records_store::LightRecord;
use crate::session::Cell32;

/// A field of a car block, picked out of the block.
type BlockField = fn(&Car) -> &dyn Cell32;
/// A field of the effects' car record.
type RecordField = fn(&CarEffects) -> &dyn Cell32;
/// A field of a light record.
type LightField = fn(&LightRecord) -> &Dword;

/// The field `$field` of the effects' record number `$i` (the four wheels, then the body 4).
macro_rules! rec {
    ($i:expr, $field:ident) => {
        (|o: &CarEffects| &o.records[$i].$field) as RecordField
    };
}

/// A car block's array field `$field` at `$i`.
macro_rules! blk {
    ($field:ident [$i:expr]) => {
        (|c: &Car| &c.$field[$i]) as BlockField
    };
}

/// The light record's (record field, light record field) moves.
const LIGHT_MOVES: [(RecordField, LightField); 4] = [
    (|o| &o.axles[2], |l| &l.axles[2]),
    (|o| &o.axles[3], |l| &l.axles[3]),
    (|o| &o.axles[0], |l| &l.axles[0]),
    (|o| &o.axles[1], |l| &l.axles[1]),
];

/// (record field, block field) moves in the original's order, after the light record: the
/// four wheels' flags, second flags, flags and second flags under the wheel, and chances.
const MOVES_A: [(RecordField, BlockField); 20] = [
    (rec!(0, flags), blk!(wheel_cell_a_byte18[0])),
    (rec!(1, flags), blk!(wheel_cell_a_byte18[1])),
    (rec!(2, flags), blk!(wheel_cell_a_byte18[2])),
    (rec!(3, flags), blk!(wheel_cell_a_byte18[3])),
    (rec!(0, flags2), blk!(wheel_cell_a_byte14[0])),
    (rec!(1, flags2), blk!(wheel_cell_a_byte14[1])),
    (rec!(2, flags2), blk!(wheel_cell_a_byte14[2])),
    (rec!(3, flags2), blk!(wheel_cell_a_byte14[3])),
    (rec!(0, under_flags), blk!(wheel_cell_b_byte18[0])),
    (rec!(1, under_flags), blk!(wheel_cell_b_byte18[1])),
    (rec!(2, under_flags), blk!(wheel_cell_b_byte18[2])),
    (rec!(3, under_flags), blk!(wheel_cell_b_byte18[3])),
    (rec!(0, under_flags2), blk!(wheel_cell_b_byte14[0])),
    (rec!(1, under_flags2), blk!(wheel_cell_b_byte14[1])),
    (rec!(2, under_flags2), blk!(wheel_cell_b_byte14[2])),
    (rec!(3, under_flags2), blk!(wheel_cell_b_byte14[3])),
    (rec!(0, chance), blk!(wheel_surface_blend[0])),
    (rec!(1, chance), blk!(wheel_surface_blend[1])),
    (rec!(2, chance), blk!(wheel_surface_blend[2])),
    (rec!(3, chance), blk!(wheel_surface_blend[3])),
];

/// The wheels' sums: (the two wheel records they go to, the block's driven speed that is
/// their first figure). Each record gets the speed, the local velocity's x, 0 and its z.
const QUADS: [([usize; 2], BlockField); 2] = [
    ([0, 1], |c| &c.drive_speed_a),
    ([2, 3], |c| &c.drive_speed_b),
];

/// The four wheel records' surface kind (from the block's `wheel_surface_a`), each cleared by
/// its switch.
const WHEELS: [RecordField; 4] = [rec!(0, kind), rec!(1, kind), rec!(2, kind), rec!(3, kind)];

/// With `contact_found` set: (body record field, block field) moves.
const MOVES_B: [(RecordField, BlockField); 9] = [
    (rec!(4, flags), blk!(wheel_cell_a_byte18[4])),
    (rec!(4, flags2), blk!(wheel_cell_a_byte14[4])),
    (rec!(4, under_flags), blk!(wheel_cell_b_byte18[4])),
    (rec!(4, under_flags2), blk!(wheel_cell_b_byte14[4])),
    (rec!(4, chance), blk!(wheel_surface_blend[4])),
    (rec!(4, kind), blk!(wheel_surface_a[4])),
    (rec!(4, under_kind), blk!(wheel_surface_b[4])),
    (rec!(4, local_velocity_x), blk!(velocity[0])),
    (rec!(4, local_velocity_z), blk!(velocity[2])),
];

/// The dword at `offset` of the block, for the offsets the core's tables give (bounded by the
/// block).
fn at(car: &Car, offset: u32) -> Result<u32, BadInput> {
    crate::pointers::load_value(car, offset as usize)
}

fn put(car: &Car, offset: u32, v: u32) -> Result<(), BadInput> {
    crate::pointers::store_value(car, offset as usize, v)
}

/// The light record's number for the block, or the refusal.
fn light_record(car: &Car) -> Result<u32, Refused> {
    let n = car_file_number(car.car_kind.get());
    sharing::check_light_record(n)?;
    Ok(n)
}

/// The Rust core: block `car`, record `o` (the marks' car record), sound object `p`, the
/// scratch dword `scratch`.
// Converted: `rc2k_core::drive::sharing` (the levels' x87 sums stay).
pub fn share_car_state_at(
    race: &Race,
    car: &'static Car,
    o: &CarEffects,
    p: &SoundObject,
    scratch: &Dword,
) -> Result<(), Refused> {
    let n = light_record(car)?;
    if let Some(count) = sharing::count_down(car.reset_marker.get()) {
        o.reset_wait.set(1);
        car.reset_marker.set(count);
    }
    o.gear_notice.set(p.gear_notice.get());
    p.gear_notice.set(0);
    let from = race.shared.view.level_from.bits();
    let step = race.course.numbers.four.bits();
    for (level, field) in o.levels.iter().zip([0x2B4, 0x2B8]) {
        level.set(0);
        if compare_full(at(car, field)?, from) & 0x100 == 0 {
            let v = sub_div_to_int(at(car, field)?, from, step);
            scratch.set(v as u32);
            level.set(sharing::level(v));
        }
    }
    o.tag.set(car.light_colour.get());
    let light = &LIGHT_RECORDS.records[n as usize];
    for (r, l) in LIGHT_MOVES {
        r(o).set_bits(l(light).get());
    }
    o.set_block(crate::race::blocks_store::handle_of(
        GamePtr::to(car).addr(),
    ));
    o.engine_speed
        .set_bits(x87::loaded(car.engine_speed.bits()));
    o.model.set(n);
    for (r, b) in MOVES_A {
        r(o).set_bits(b(car).bits());
    }
    for (records, first) in QUADS {
        let (a, bb, c) = (
            first(car).bits(),
            car.local_velocity[0].bits(),
            car.local_velocity[2].bits(),
        );
        for wheel in records {
            let w = &o.records[wheel];
            w.driven_speed.set_bits(a);
            w.local_velocity_x.set_bits(bb);
            w.local_velocity_y.set(0);
            w.local_velocity_z.set_bits(c);
        }
    }
    for (k, r) in WHEELS.into_iter().enumerate() {
        r(o).set_bits(car.wheel_surface_a[k].get());
    }
    for (k, r) in WHEELS.into_iter().enumerate() {
        if sharing::wheel_cleared(
            car.wheel_squeeze[k].bits(),
            car.parts_present.get(),
            k as u32,
        ) {
            r(o).set_bits(0);
        }
    }
    o.records[4].kind.set(0);
    o.records[4].under_kind.set(0);
    let zero = x87::loaded(0);
    for cell in &o.morph_floats {
        cell.set_bits(zero);
    }
    o.morph_start.set(zero);
    if car.contact_found.get() != 0 {
        for (r, b) in MOVES_B {
            r(o).set_bits(b(car).bits());
        }
        for (cell, v) in o.records[4].place.iter().zip([
            &car.contact_point_x,
            &car.contact_point_ground,
            &car.contact_point_z,
        ]) {
            cell.set_bits(x87::from_int(v.get() as i32));
        }
        o.records[4].size.set_bits(car.contact_value.get());
    }
    for (cell, mark) in o.factors.iter().zip(&car.impact_octant_marks) {
        cell.set_bits(mark.bits());
    }
    let kept_from = o.face_sums.each_ref().map(Dword::get);
    let mut kept = kept_from;
    sharing::fill_holes(&mut kept);
    for (cell, v) in car.part_wake_counts.iter().zip(kept) {
        cell.set(v);
    }
    // +0xA3C .. (eight dwords, past `part_damage_limits`' seven) and +0xA5C .. (inside
    // `saved_point_figures`): by offset in the block.
    for (k, cell) in o.slot_sums.iter().enumerate() {
        put(car, 0xA3C + 4 * k as u32, cell.get())?;
    }
    for (k, cell) in o.kind_sums.iter().enumerate() {
        put(car, 0xA5C + 4 * k as u32, cell.get())?;
    }
    Ok(())
}

/// The Rust version: the car block `block`, its record `record`, its sound object `sound`.
pub fn share_car_state(
    race: &Race,
    block: &'static Car,
    record: &CarEffects,
    sound: &SoundObject,
) -> Result<(), Refused> {
    share_car_state_at(
        race,
        block,
        record,
        sound,
        // The original's spill to `g_scratch` (0x62400C), which nothing read: a local.
        &Dword::new(0),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::race::blocks_store::own_car;
    use crate::race::effects_settings_store::EFFECTS_SETTINGS;

    fn rec_off(o: &CarEffects, f: RecordField) -> usize {
        core::ptr::from_ref(f(o)).cast::<u8>().addr() - core::ptr::from_ref(o).addr()
    }

    fn blk_off(c: &Car, f: BlockField) -> usize {
        core::ptr::from_ref(f(c)).cast::<u8>().addr() - core::ptr::from_ref(c).addr()
    }

    #[test]
    fn the_tables_sit_where_the_original_moved_them() {
        let o = &EFFECTS_SETTINGS.cars[0];
        let car = own_car();
        let light = &LIGHT_RECORDS.records[0];
        let lbase = core::ptr::from_ref(light).addr();
        let lights = [(0x14, 0x34), (0x18, 0x38), (0xC, 0x2C), (0x10, 0x30)];
        for ((r, l), (ro, lo)) in LIGHT_MOVES.into_iter().zip(lights) {
            assert_eq!(rec_off(o, r), ro);
            assert_eq!(core::ptr::from_ref(l(light)).addr() - lbase, lo);
        }
        let moves_a = [
            (0xF4, 0x95C),
            (0x13C, 0x960),
            (0x184, 0x964),
            (0x1CC, 0x968),
            (0xF8, 0x984),
            (0x140, 0x988),
            (0x188, 0x98C),
            (0x1D0, 0x990),
            (0x100, 0x970),
            (0x148, 0x974),
            (0x190, 0x978),
            (0x1D8, 0x97C),
            (0x104, 0x998),
            (0x14C, 0x99C),
            (0x194, 0x9A0),
            (0x1DC, 0x9A4),
            (0x108, 0x9AC),
            (0x150, 0x9B0),
            (0x198, 0x9B4),
            (0x1E0, 0x9B8),
        ];
        for ((r, b), (ro, bo)) in MOVES_A.into_iter().zip(moves_a) {
            assert_eq!((rec_off(o, r), blk_off(car, b)), (ro, bo));
        }
        let quads = [
            ([0xE0, 0xE4, 0xE8, 0xEC], 0x1B8),
            ([0x170, 0x174, 0x178, 0x17C], 0x1BC),
        ];
        for ((records, first), (offsets, bo)) in QUADS.into_iter().zip(quads) {
            assert_eq!(blk_off(car, first), bo);
            for (wheel, base) in records.into_iter().zip([offsets[0], offsets[0] + 0x48]) {
                let w = &o.records[wheel];
                let at = |f: &Dword| core::ptr::from_ref(f).addr() - core::ptr::from_ref(o).addr();
                let atf = |f: &crate::session::Float| {
                    core::ptr::from_ref(f).addr() - core::ptr::from_ref(o).addr()
                };
                assert_eq!(atf(&w.driven_speed), base);
                assert_eq!(atf(&w.local_velocity_x), base + 4);
                assert_eq!(at(&w.local_velocity_y), base + 8);
                assert_eq!(atf(&w.local_velocity_z), base + 0xC);
            }
        }
        for (r, wo) in WHEELS.into_iter().zip([0xF0, 0x138, 0x180, 0x1C8]) {
            assert_eq!(rec_off(o, r), wo);
        }
        let moves_b = [
            (0x214, 0x96C),
            (0x218, 0x994),
            (0x220, 0x980),
            (0x224, 0x9A8),
            (0x228, 0x9BC),
            (0x210, 0x930),
            (0x21C, 0x944),
            (0x204, 0x198),
            (0x20C, 0x1A0),
        ];
        for ((r, b), (ro, bo)) in MOVES_B.into_iter().zip(moves_b) {
            assert_eq!((rec_off(o, r), blk_off(car, b)), (ro, bo));
        }
        assert_eq!(rec_off(o, rec!(4, kind)), 0x210);
        assert_eq!(rec_off(o, rec!(4, under_kind)), 0x21C);
    }
}
