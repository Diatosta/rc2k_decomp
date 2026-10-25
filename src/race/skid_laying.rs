//! The skidmarks laid (`sub_4585F3` -> `sub_4588DE` -> `sub_45860C`, part R3, IDA
//! 2026-10-18): each car's skidding wheels add a point to the ring of 0x400 skid points
//! (`g_skidPoints`, 0x18 bytes: the mark's left and right edges) linked in strips
//! (`g_skidLinks`, 8 bytes: +0 the next, +2 the previous, +4 the angle, +6 the car's bit).

use crate::errors::{BadInput, Refused};
use crate::framework::{game_addr, game_read, game_write, rva};
use crate::physics::car_store::Car;
use crate::race::frame_store::FRAME_WORK;
use crate::race::vars_store::RACE_VARS;
use crate::race::view_store::RACE_VIEW;
use crate::race::work_store::RACE_WORK;
use crate::scene::draw_store::DRAW;

fn k(a: u32) -> u32 {
    unsafe { game_read(rva(a as usize)) }
}

fn w(a: u32, v: u32) {
    unsafe { game_write(rva(a as usize), v) }
}

const POINTS_TABLE: u32 = 0x645B18; // g_skidPoints
const LINKS: u32 = 0x643AE8; // g_skidLinks
const NEXT: u32 = 0x645B10; // dword_645B10: the next point
const PLACE: u32 = 0x643AD8; // dword_643AD8 .. dword_643AE0: the wheel's place
const OFFSET_X: u32 = 0x645B08;
const OFFSET_Z: u32 = 0x645B0C;
const ANGLE: u32 = 0x645B00;

unsafe fn get(a: u32) -> u32 {
    unsafe { (a as *const u32).read_unaligned() }
}

unsafe fn put(a: u32, v: u32) {
    unsafe { (a as *mut u32).write_unaligned(v) }
}

/// The edges' offset for an angle (`math::turn_point_xz` of (0x1555, y, 0) by angle << 16).
unsafe fn offsets(angle: u32, y: u32) -> (u32, u32) {
    unsafe {
        let [x, _, z, _] = crate::math::turn_point_xz::turn_point_xz(
            rc2k_core::race::skids::HALF_WIDTH,
            y as i32,
            0,
            angle << 16,
        );
        (x, z)
    }
}

/// `race::add_skid_point` (0x45860C): a wheel's next skid point (`dword_645B10`, its slot
/// `slot` (esi) -> the wheel's last point, then the new one): across the wheel's place
/// (`dword_643AD8` ..) at the angle from the last point's middle (`scene::angle_between`;
/// 0 for none), the point's old links cut, linked after the last; the last point turned to
/// the new angle (its own when it starts the strip, else halfway between its previous one
/// and this); the ring's next moved on (0x400 points). Refuses (before anything) a point or a link outside the ring (the original reads and
/// writes past its tables).
pub unsafe fn add_skid_point(slot: u32) -> Result<(), Refused> {
    // Converted: `rc2k_core::race::skids::add_skid_point`.
    use rc2k_core::race::skids::{Link, POINTS, Point, Ring, add_skid_point, next_point};
    unsafe {
        let n = k(NEXT);
        let last = get(slot);
        let points = std::slice::from_raw_parts_mut(
            game_addr(rva(POINTS_TABLE as usize)) as *mut Point,
            POINTS as usize,
        );
        let links = std::slice::from_raw_parts_mut(
            game_addr(rva(LINKS as usize)) as *mut Link,
            POINTS as usize,
        );
        let place = [k(PLACE), k(PLACE + 4), k(PLACE + 8)];
        let a = add_skid_point(
            Ring { points, links },
            n,
            last,
            place,
            k(0x645B14) as u16,
            |dx, dz| crate::scene::angle_between::angle_of(dx, dz),
            |angle, y| offsets(angle, y),
        )?;
        put(slot, n);
        w(ANGLE, a.angle);
        w(OFFSET_X, a.offset.0);
        w(OFFSET_Z, a.offset.1);
        if let Some(m) = a.from {
            for (i, v) in m.iter().enumerate() {
                w(0x645AF4 + 4 * i as u32, *v);
            }
        }
        if let Some((turned, m)) = a.turned {
            w(0x645B04, turned);
            for (i, v) in m.iter().enumerate() {
                w(0x645AE8 + 4 * i as u32, *v);
            }
        }
        w(NEXT, next_point(k(NEXT)));
        Ok(())
    }
}

/// What `lay_skidmarks` reads: the cars' blocks.
unsafe fn cars_refuse() -> Result<(), Refused> {
    for n in 0..k(0x63DB90).min(8) {
        if RACE_WORK.car_block(n as usize) == 0 {
            return Err(BadInput("a race car without its block").into());
        }
    }
    if RACE_VIEW.race_cars.get() > 8 {
        return Err(BadInput("more than eight race cars").into());
    }
    Ok(())
}

/// `race::lay_skidmarks` (0x4588DE): for each race car (its bit in `dword_645B14`) and
/// each of its four wheels (slot +0xF48 ..): when the wheel skids (+0x600 ..), its axle's
/// grip (+0xF40 front, +0xF44 rear) is not below `flt_643AD0` and its mark is on (+0x298
/// bits 5 .. 8), its place (+0xF10 / +0xF20 / +0xF30 ..) plus (+-0x1555, 0, 0) turned by
/// the car's matrix (+0xB4; `scene::rotate_point`, rounded, the last in `dword_62400C`)
/// gets a point (`race::add_skid_point`, esi the slot); else the slot -1. Refuses (before anything) a car without its
/// block.
pub unsafe fn lay_skidmarks() -> Result<(), Refused> {
    use crate::race::skidmarks::int_80;
    unsafe {
        cars_refuse()?;
        let count = RACE_VIEW.race_cars.get();
        if count == 0 {
            return Ok(());
        }
        let grip: u32 = k(0x643AD0);
        let zero = int_80(0);
        for c in 0..count {
            w(0x645B14, 1 << (c & 31));
            let car = RACE_WORK.car_block(c as usize);
            for wheel in 0..4u32 {
                let slot = car + 0xF48 + 4 * wheel;
                let on = get(car + 0x600 + 4 * wheel) != 0
                    && !crate::math::x87_load::below_or_unordered(
                        get(car + 0xF40 + 4 * (wheel / 2)),
                        grip,
                    )
                    && Car::at(car).parts_present.bits() & (1 << (5 + wheel)) != 0;
                if !on {
                    put(slot, u32::MAX);
                    continue;
                }
                for i in 0..3 {
                    w(PLACE + 4 * i, get(car + 0xF10 + 0x10 * i + 4 * wheel));
                }
                let side = if wheel % 2 == 0 {
                    0x1555u32
                } else {
                    0xFFFF_EAAB
                };
                DRAW.scratch.set(side);
                let t = crate::scene::rotate_point::rotate_point(
                    car + 0xB4,
                    [int_80(side), zero, zero],
                );
                for (i, v) in t.into_iter().enumerate() {
                    let n = crate::math::x87_load::tbyte_to_int(v);
                    DRAW.scratch.set(n);
                    w(
                        PLACE + 4 * i as u32,
                        k(PLACE + 4 * i as u32).wrapping_add(n),
                    );
                }
                add_skid_point(slot)?;
            }
        }
        Ok(())
    }
}

/// `race::lay_skidmarks_if_on` (0x4585F3): `lay_skidmarks` unless `dword_6235E8` or
/// `dword_6416BC` is set. Its one caller, `race::draw_race_scene`, reads no register it
/// leaves.
pub unsafe fn lay_skidmarks_if_on() -> Result<(), Refused> {
    if RACE_VARS.scene_switch.get() != 0 || FRAME_WORK.still_2.get() != 0 {
        return Ok(());
    }
    unsafe { lay_skidmarks() }
}
