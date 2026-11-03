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
//!   block +0x8C0 and +0x740 there; a block handle here: `MarkCar::whole_matrix`,
//!   `part_matrix`);
//! - about eighty fields moved as the original moves them (`MOVES`, the wheels' four
//!   each), four of them cleared by the block's switches (+0x600 .. and bits 5 .. 8 of
//!   +0x298), a part only when block +0xA34 is set;
//! - record +0x258 (6 dwords) into block +0x700, whose 0x80000000 entries are then
//!   filled from the next one that is not (round, at most six looked at); record +0x32C
//!   (8) into block +0xA3C, record +0x3F4 (0x11) into block +0xA5C.
//!
//! In:  esi the block, edi the record, ebp the sound object
//! Out: eax block +0x1A0 (with +0xA34 set) or +0x1BC; ebx +0x1A4, ecx +0x1AC, edx +0x92C;
//!      the rest kept (the copies are inside `pusha` / `popa`); no flags read
//!
//! Caller: `drive::share_all_cars` (0x456484), for each of the eight cars, inside `pusha` /
//! `popa`. Refuses (before anything) a car file number above 0x21 (past `g_lightRecords`).

use crate::errors::{BadInput, Refused};
use crate::math::x87;
use crate::math::x87_run::{compare_full, sub_div_to_int};
use crate::physics::car_store::Car;
use crate::pointers::GamePtr;
use crate::race::light_records_store::LIGHT_RECORDS;
use crate::race::marks_settings_store::MarkCar;
use crate::race::mud::car_file_number;
use crate::race::numbers_store::NUMBERS;
use crate::race::view_store::RACE_VIEW;
use crate::scene::draw_store::DRAW;
use crate::session::Dword;
use crate::sound::store::SoundObject;
use rc2k_core::drive::sharing;

/// The dword at `offset` of the block, for the offsets the core's tables give (bounded by the
/// block).
fn at(car: &Car, offset: u32) -> Result<u32, BadInput> {
    GamePtr::to(car).offset(offset).read("a car's block")
}

fn put(car: &Car, offset: u32, v: u32) -> Result<(), BadInput> {
    GamePtr::to(car).offset(offset).write(v, "a car's block")
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
    car: &'static Car,
    o: &MarkCar,
    p: &SoundObject,
    scratch: &Dword,
) -> Result<(), Refused> {
    let n = light_record(car)?;
    if let Some(count) = sharing::count_down(car.reset_marker.get()) {
        o.put(0x30, 1)?;
        car.reset_marker.set(count);
    }
    o.put(0x2C, p.gear_notice.get())?;
    p.gear_notice.set(0);
    let from = RACE_VIEW.level_from.bits();
    let step = NUMBERS.four.bits();
    for (out, field) in [(0x4C0, 0x2B4), (0x4C4, 0x2B8)] {
        o.put(out, 0)?;
        if compare_full(at(car, field)?, from) & 0x100 == 0 {
            let v = sub_div_to_int(at(car, field)?, from, step);
            scratch.set(v as u32);
            o.put(out, sharing::level(v))?;
        }
    }
    o.put(0x1C, car.light_colour.get())?;
    let light = LIGHT_RECORDS.record_addr(n);
    for (r, l) in sharing::LIGHT_MOVES {
        o.put(r, GamePtr::new(light + l).read("a light record")?)?;
    }
    o.set_block(crate::race::blocks_store::handle_of(
        GamePtr::to(car).addr(),
    ))?;
    o.put(8, x87::loaded(car.engine_speed.bits()))?;
    o.put(4, n)?;
    for (r, b) in sharing::MOVES_A {
        o.put(r, at(car, b)?)?;
    }
    for (offsets, first) in sharing::QUADS {
        let (a, bb, c) = (
            at(car, first)?,
            car.local_velocity[0].bits(),
            car.local_velocity[2].bits(),
        );
        for base in [offsets[0], offsets[0] + 0x48] {
            o.put(base, a)?;
            o.put(base + 4, bb)?;
            o.put(base + 8, 0)?;
            o.put(base + 0xC, c)?;
        }
    }
    for (k, r) in sharing::WHEELS.into_iter().enumerate() {
        o.put(r, car.wheel_surface_a[k].get())?;
    }
    for (k, r) in sharing::WHEELS.into_iter().enumerate() {
        if sharing::wheel_cleared(
            car.wheel_squeeze[k].bits(),
            car.parts_present.get(),
            k as u32,
        ) {
            o.put(r, 0)?;
        }
    }
    o.put(0x210, 0)?;
    o.put(0x21C, 0)?;
    let zero = x87::loaded(0);
    for r in [0x24C, 0x250, 0x254, 0x244] {
        o.put(r, zero)?;
    }
    if car.contact_found.get() != 0 {
        for (r, b) in sharing::MOVES_B {
            o.put(r, at(car, b)?)?;
        }
        for (r, v) in [
            (0x230, &car.contact_point_x),
            (0x234, &car.contact_point_ground),
            (0x238, &car.contact_point_z),
        ] {
            o.put(r, x87::from_int(v.get() as i32))?;
        }
        o.put(0x22C, car.contact_value.get())?;
    }
    for (k, mark) in car.impact_octant_marks.iter().enumerate() {
        o.put(0x2CC + 4 * k as u32, mark.bits())?;
    }
    let mut kept = [0u32; 6];
    for (k, v) in kept.iter_mut().enumerate() {
        *v = o.word(0x258 + 4 * k as u32)?;
    }
    sharing::fill_holes(&mut kept);
    // +0x700 .. (inside `unknown_6f4`), +0xA3C .. (eight dwords, past `part_damage_limits`'
    // seven) and +0xA5C .. (inside `unknown_a58`): by offset in the block.
    for (k, v) in kept.into_iter().enumerate() {
        put(car, 0x700 + 4 * k as u32, v)?;
    }
    for k in 0..8 {
        put(car, 0xA3C + 4 * k, o.word(0x32C + 4 * k)?)?;
    }
    for k in 0..0x11 {
        put(car, 0xA5C + 4 * k, o.word(0x3F4 + 4 * k)?)?;
    }
    Ok(())
}

/// The Rust version.
/// The Rust version: the car block `block` (esi), its record `record` (edi), its sound
/// object `sound` (ebp). The registers it leaves are read by no caller (liveness: only
/// DF after `share_all_cars`' call).
pub fn share_car_state(
    block: &'static Car,
    record: &MarkCar,
    sound: &SoundObject,
) -> Result<(), Refused> {
    share_car_state_at(block, record, sound, &DRAW.scratch /* dword_62400C */)
}
