//! One car's state from its block (0xFE0 bytes) into its record (`car_records[n]`, read by the
//! drawing and the sounds) and back: the sound object's request flag and a countdown, two levels
//! from block +0x2B4 / +0x2B8 (`level_from`, `four`, held to 0 .. 7), the car file's light
//! record, the block handles at record +0x24 / +0x28, about eighty fields moved (`MOVES`, the
//! wheels' four each; four cleared by the block's switches, a part only when block +0xA34 is
//! set), and three arrays into the block (+0x700 with its 0x80000000 entries filled from the next
//! real one). Refuses (before anything) a car file number above 0x21 (past the light records).

use crate::errors::Refused;
use crate::math::x87;
use crate::math::x87::compare_full;
use crate::math::x87_float as bh;
use crate::physics::car_store::Car;
use crate::race::effects_settings_store::CarEffects;
use crate::race::mud::car_file_number;
use crate::race::owner::Race;
use crate::sound::store::SoundObject;
use rc2k_core::drive::sharing;

use crate::race::light_records_store::LightRecord;
use crate::session::{PlainDword, PlainFloat};

/// A whole-number field of the effects' car record.
type RecordWord = fn(&CarEffects) -> &PlainDword;
/// A float field of a car block.
type BlockFloat = fn(&Car) -> &PlainFloat;
/// A field of a light record.
type LightField = fn(&LightRecord) -> &PlainDword;

/// A field of a car block moved into the same kind of field of the effects' record: a whole
/// number (a surface kind, flags) or a float.
#[derive(Clone, Copy)]
enum Move {
    Word(RecordWord, fn(&Car) -> &PlainDword),
    Float(fn(&CarEffects) -> &PlainFloat, BlockFloat),
}

impl Move {
    /// The block's field into the record's.
    fn apply(self, o: &CarEffects, car: &Car) {
        match self {
            Move::Word(r, b) => r(o).set(b(car).get()),
            Move::Float(r, b) => r(o).set(b(car).get()),
        }
    }
}

/// The field `$field` of the effects' record number `$i` (the four wheels, then the body 4).
macro_rules! rec {
    ($i:expr, $field:ident) => {
        |o: &CarEffects| &o.records[$i].$field
    };
}

/// A car block's array field `$field` at `$i`.
macro_rules! blk {
    ($field:ident [$i:expr]) => {
        |c: &Car| &c.$field[$i]
    };
}

/// The light record's (record field, light record field) moves.
const LIGHT_MOVES: [(RecordWord, LightField); 4] = [
    (|o| &o.axles[2], |l| &l.axles[2]),
    (|o| &o.axles[3], |l| &l.axles[3]),
    (|o| &o.axles[0], |l| &l.axles[0]),
    (|o| &o.axles[1], |l| &l.axles[1]),
];

/// (record field, block field) moves after the light record: the four wheels' flags, second
/// flags, flags and second flags under the wheel, and chances.
const MOVES_A: [Move; 20] = [
    Move::Word(rec!(0, flags), blk!(wheel_cell_a_byte18[0])),
    Move::Word(rec!(1, flags), blk!(wheel_cell_a_byte18[1])),
    Move::Word(rec!(2, flags), blk!(wheel_cell_a_byte18[2])),
    Move::Word(rec!(3, flags), blk!(wheel_cell_a_byte18[3])),
    Move::Word(rec!(0, flags2), blk!(wheel_cell_a_byte14[0])),
    Move::Word(rec!(1, flags2), blk!(wheel_cell_a_byte14[1])),
    Move::Word(rec!(2, flags2), blk!(wheel_cell_a_byte14[2])),
    Move::Word(rec!(3, flags2), blk!(wheel_cell_a_byte14[3])),
    Move::Word(rec!(0, under_flags), blk!(wheel_cell_b_byte18[0])),
    Move::Word(rec!(1, under_flags), blk!(wheel_cell_b_byte18[1])),
    Move::Word(rec!(2, under_flags), blk!(wheel_cell_b_byte18[2])),
    Move::Word(rec!(3, under_flags), blk!(wheel_cell_b_byte18[3])),
    Move::Word(rec!(0, under_flags2), blk!(wheel_cell_b_byte14[0])),
    Move::Word(rec!(1, under_flags2), blk!(wheel_cell_b_byte14[1])),
    Move::Word(rec!(2, under_flags2), blk!(wheel_cell_b_byte14[2])),
    Move::Word(rec!(3, under_flags2), blk!(wheel_cell_b_byte14[3])),
    Move::Float(rec!(0, chance), blk!(wheel_surface_blend[0])),
    Move::Float(rec!(1, chance), blk!(wheel_surface_blend[1])),
    Move::Float(rec!(2, chance), blk!(wheel_surface_blend[2])),
    Move::Float(rec!(3, chance), blk!(wheel_surface_blend[3])),
];

/// The wheels' sums: (the two wheel records they go to, the block's driven speed that is
/// their first figure). Each record gets the speed, the local velocity's x, 0 and its z.
const QUADS: [([usize; 2], BlockFloat); 2] = [
    ([0, 1], |c| &c.drive_speed_a),
    ([2, 3], |c| &c.drive_speed_b),
];

/// The four wheel records' surface kind (from the block's `wheel_surface_a`), each cleared by
/// its switch.
const WHEELS: [RecordWord; 4] = [rec!(0, kind), rec!(1, kind), rec!(2, kind), rec!(3, kind)];

/// With `contact_found` set: (body record field, block field) moves.
const MOVES_B: [Move; 9] = [
    Move::Word(rec!(4, flags), blk!(wheel_cell_a_byte18[4])),
    Move::Word(rec!(4, flags2), blk!(wheel_cell_a_byte14[4])),
    Move::Word(rec!(4, under_flags), blk!(wheel_cell_b_byte18[4])),
    Move::Word(rec!(4, under_flags2), blk!(wheel_cell_b_byte14[4])),
    Move::Float(rec!(4, chance), blk!(wheel_surface_blend[4])),
    Move::Word(rec!(4, kind), blk!(wheel_surface_a[4])),
    Move::Word(rec!(4, under_kind), blk!(wheel_surface_b[4])),
    Move::Float(rec!(4, local_velocity_x), blk!(velocity[0])),
    Move::Float(rec!(4, local_velocity_z), blk!(velocity[2])),
];

/// The block's eight slot figures: the seven parts' damage figures and the first of the saved
/// point figures after them.
fn slot_cells(car: &Car) -> impl Iterator<Item = &PlainDword> {
    car.part_damage_limits
        .iter()
        .chain(&car.saved_point_figures[..1])
}

/// The block's seventeen kind figures: the saved point figures after the first.
fn kind_cells(car: &Car) -> &[PlainDword] {
    &car.saved_point_figures[1..18]
}

/// The light record's number for the block, or the refusal.
fn light_record(race: &Race, car: &Car) -> Result<u32, Refused> {
    let n = car_file_number(race.kind_tables(), car.car_kind.get());
    sharing::check_light_record(n)?;
    Ok(n)
}

/// The Rust core: block `car`, record `o` (the marks' car record), sound object `p`, the
/// scratch dword `scratch`.
pub fn share_car_state_at(
    race: &Race,
    car: &Car,
    o: &CarEffects,
    p: &SoundObject,
    scratch: &PlainDword,
) -> Result<(), Refused> {
    let n = light_record(race, car)?;
    if let Some(count) = sharing::count_down(car.reset_marker.get()) {
        o.reset_wait.set(1);
        car.reset_marker.set(count);
    }
    o.gear_notice.set(p.gear_notice.get());
    p.gear_notice.set(0);
    let from = race.shared.view.get().level_from.get();
    let step = race.course.numbers.get().four.get();
    for (level, gauge) in o.levels.iter().zip([&car.engine_heat, &car.engine_damage]) {
        level.set(0);
        if compare_full(gauge.bits(), from) & 0x100 == 0 {
            let v = bh::to_int((bh::ld(gauge.bits()) - bh::ld(from)) / bh::ld(step));
            scratch.set(v as u32);
            level.set(sharing::level(v));
        }
    }
    o.tag.set(car.light_colour.get());
    let light = &race.light_records().records[n as usize];
    for (r, l) in LIGHT_MOVES {
        r(o).set(l(light).get());
    }
    o.set_block(race.shared.car_blocks().handle_of(car));
    o.engine_speed.set(x87::loaded(&car.engine_speed));
    o.model.set(n);
    for m in MOVES_A {
        m.apply(o, car);
    }
    for (records, first) in QUADS {
        let (a, bb, c) = (
            first(car).get(),
            car.local_velocity[0].get(),
            car.local_velocity[2].get(),
        );
        for wheel in records {
            let w = &o.records[wheel];
            w.driven_speed.set(a);
            w.local_velocity_x.set(bb);
            w.local_velocity_y.set(0);
            w.local_velocity_z.set(c);
        }
    }
    for (k, r) in WHEELS.into_iter().enumerate() {
        r(o).set(car.wheel_surface_a[k].get());
    }
    for (k, r) in WHEELS.into_iter().enumerate() {
        if sharing::wheel_cleared(
            car.wheel_squeeze[k].bits(),
            car.parts_present.get(),
            k as u32,
        ) {
            r(o).set(0);
        }
    }
    o.records[4].kind.set(0);
    o.records[4].under_kind.set(0);
    let zero = x87::loaded(0);
    for cell in &o.morph_floats {
        cell.set(zero);
    }
    o.morph_start.set(zero.to_bits());
    if car.contact_found.get() != 0 {
        for m in MOVES_B {
            m.apply(o, car);
        }
        for (cell, v) in o.records[4].place.iter().zip([
            &car.contact_point_x,
            &car.contact_point_ground,
            &car.contact_point_z,
        ]) {
            cell.set(x87::from_int(v.get() as i32));
        }
        o.records[4]
            .size
            .set(f32::from_bits(car.contact_value.get()));
    }
    for (cell, mark) in o.factors.iter().zip(&car.impact_octant_marks) {
        cell.set(mark.get());
    }
    let kept_from = o.face_sums.each_ref().map(PlainDword::get);
    let mut kept = kept_from;
    sharing::fill_holes(&mut kept);
    for (cell, v) in car.part_wake_counts.iter().zip(kept) {
        cell.set(v);
    }
    for (cell, to) in o.slot_sums.iter().zip(slot_cells(car)) {
        to.set(cell.get());
    }
    for (cell, to) in o.kind_sums.iter().zip(kind_cells(car)) {
        to.set(cell.get());
    }
    Ok(())
}

/// The car block `block`, its record `record`, its sound object `sound`.
pub fn share_car_state(
    race: &Race,
    block: &Car,
    record: &CarEffects,
    sound: &SoundObject,
) -> Result<(), Refused> {
    share_car_state_at(
        race,
        block,
        record,
        sound,
        // A local for a scratch dword nothing reads.
        &PlainDword::new(0),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::physics::car_offsets_1999::{
        bytes_1999, car_with_1999_bytes, load_1999, offset_1999,
    };
    use crate::race::effects_settings_store::EFFECTS_SETTINGS;
    use crate::race::light_records_store::LIGHT_RECORDS;

    fn rec_off(o: &CarEffects, f: RecordWord) -> usize {
        core::ptr::from_ref(f(o)).addr() - core::ptr::from_ref(o).addr()
    }

    fn blk_off(c: &Car, f: BlockFloat) -> usize {
        offset_1999(c, core::ptr::from_ref(f(c)).addr())
    }

    fn move_offs(o: &CarEffects, c: &Car, m: Move) -> (usize, usize) {
        let (r, b) = match m {
            Move::Word(r, b) => (
                core::ptr::from_ref(r(o)).addr(),
                core::ptr::from_ref(b(c)).addr(),
            ),
            Move::Float(r, b) => (
                core::ptr::from_ref(r(o)).addr(),
                core::ptr::from_ref(b(c)).addr(),
            ),
        };
        (r - core::ptr::from_ref(o).addr(), offset_1999(c, b))
    }

    fn at(car: &Car, offset: usize) -> u32 {
        load_1999(car, offset)
    }

    #[test]
    fn the_gauges_and_figures_are_the_offsets_they_replace() {
        let pattern = |n: usize| (n * 7 + 3) as u8;
        let car = car_with_1999_bytes(pattern);
        assert_eq!(car.engine_heat.bits(), at(&car, 0x2B4));
        assert_eq!(car.engine_damage.bits(), at(&car, 0x2B8));
        let slots: Vec<u32> = slot_cells(&car).map(PlainDword::get).collect();
        assert_eq!(slots.len(), 8);
        for (k, v) in slots.into_iter().enumerate() {
            assert_eq!(v, at(&car, 0xA3C + 4 * k));
        }
        for (k, c) in kind_cells(&car).iter().enumerate() {
            assert_eq!(c.get(), at(&car, 0xA5C + 4 * k));
        }
        // The writes land on the same bytes.
        let mut expected = bytes_1999(&car);
        for (k, c) in slot_cells(&car).enumerate() {
            c.set(0x1111_0000 + k as u32);
            expected[0xA3C + 4 * k..][..4].copy_from_slice(&(0x1111_0000 + k as u32).to_le_bytes());
        }
        for (k, c) in kind_cells(&car).iter().enumerate() {
            c.set(0x2222_0000 + k as u32);
            expected[0xA5C + 4 * k..][..4].copy_from_slice(&(0x2222_0000 + k as u32).to_le_bytes());
        }
        assert_eq!(bytes_1999(&car), expected);
    }

    #[test]
    fn the_tables_sit_where_the_original_moved_them() {
        let o = &EFFECTS_SETTINGS.cars[0];
        let car = &crate::race::owner::race().car_blocks().cars[0];
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
        for (m, offs) in MOVES_A.into_iter().zip(moves_a) {
            assert_eq!(move_offs(o, car, m), offs);
        }
        let quads = [
            ([0xE0, 0xE4, 0xE8, 0xEC], 0x1B8),
            ([0x170, 0x174, 0x178, 0x17C], 0x1BC),
        ];
        for ((records, first), (offsets, bo)) in QUADS.into_iter().zip(quads) {
            assert_eq!(blk_off(car, first), bo);
            for (wheel, base) in records.into_iter().zip([offsets[0], offsets[0] + 0x48]) {
                let w = &o.records[wheel];
                let at =
                    |f: &PlainDword| core::ptr::from_ref(f).addr() - core::ptr::from_ref(o).addr();
                let atf =
                    |f: &PlainFloat| core::ptr::from_ref(f).addr() - core::ptr::from_ref(o).addr();
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
        for (m, offs) in MOVES_B.into_iter().zip(moves_b) {
            assert_eq!(move_offs(o, car, m), offs);
        }
        assert_eq!(rec_off(o, rec!(4, kind)), 0x210);
        assert_eq!(rec_off(o, rec!(4, under_kind)), 0x21C);
    }
}
