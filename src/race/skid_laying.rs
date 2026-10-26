//! The skidmarks laid (`sub_4585F3` -> `sub_4588DE` -> `sub_45860C`, part R3, IDA
//! 2026-10-18): each car's skidding wheels add a point to the ring of 0x400 skid points
//! (`g_skidPoints`, 0x18 bytes: the mark's left and right edges) linked in strips
//! (`g_skidLinks`, 8 bytes: +0 the next, +2 the previous, +4 the angle, +6 the car's bit).

use crate::errors::{BadInput, Refused};
use crate::framework::{Regs, game_addr, game_read, game_write, rva};

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
/// esi -> the wheel's last point, then the new one): across the wheel's place
/// (`dword_643AD8` ..) at the angle from the last point's middle (`scene::angle_between`;
/// 0 for none), the point's old links cut, linked after the last; the last point turned to
/// the new angle (its own when it starts the strip, else halfway between its previous one
/// and this); the ring's next moved on (0x400 points). Gives the registers it leaves.
/// Refuses (before anything) a point or a link outside the ring (the original reads and
/// writes past its tables).
pub unsafe fn add_skid_point(entry: &Regs) -> Result<Regs, Refused> {
    // Converted: `rc2k_core::race::skids::add_skid_point`.
    use rc2k_core::race::skids::{Link, POINTS, Point, Ring, add_skid_point, next_point};
    unsafe {
        let mut r = *entry;
        let slot = r.esi;
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
        let point = |i: u32| (game_addr(rva(POINTS_TABLE as usize)) as u32).wrapping_add(i * 0x18);
        r.edx = n;
        r.ebp = last;
        if a.last_moved {
            [r.eax, r.ebx, r.ecx] = a.right;
            r.esi = point(last);
        } else {
            r.eax = a.angle;
            r.ebx = place[1];
            r.ecx = a.next_read as u32;
            r.esi = point(n);
        }
        w(NEXT, next_point(k(NEXT)));
        Ok(r)
    }
}

/// What `lay_skidmarks` reads: the cars' blocks.
unsafe fn cars_refuse() -> Result<(), Refused> {
    for n in 0..k(0x63DB90).min(8) {
        if k(0x71B3B8 + 4 * n) == 0 {
            return Err(BadInput("a race car without its block").into());
        }
    }
    if k(0x63DB90) > 8 {
        return Err(BadInput("more than eight race cars").into());
    }
    Ok(())
}

/// `race::lay_skidmarks` (0x4588DE): for each race car (its bit in `dword_645B14`) and
/// each of its four wheels (slot +0xF48 ..): when the wheel skids (+0x600 ..), its axle's
/// grip (+0xF40 front, +0xF44 rear) is not below `flt_643AD0` and its mark is on (+0x298
/// bits 5 .. 8), its place (+0xF10 / +0xF20 / +0xF30 ..) plus (+-0x1555, 0, 0) turned by
/// the car's matrix (+0xB4; `scene::rotate_point`, rounded, the last in `dword_62400C`)
/// gets a point (part 1: `race::add_skid_point`, esi the slot); else the slot -1. Gives eax
/// the last car's bit and ecx the count. Refuses (before anything) a car without its
/// block.
pub unsafe fn lay_skidmarks(r: &mut Regs, part: crate::race::frame::Part) -> Result<(), Refused> {
    use crate::race::skidmarks::int_80;
    unsafe {
        cars_refuse()?;
        let count = k(0x63DB90);
        if count == 0 {
            return Ok(());
        }
        let grip: u32 = k(0x643AD0);
        let zero = int_80(0);
        for c in 0..count {
            r.ecx = c;
            r.eax = 1 << (c & 31);
            w(0x645B14, r.eax);
            let car = k(0x71B3B8 + 4 * c);
            for wheel in 0..4u32 {
                let slot = car + 0xF48 + 4 * wheel;
                let on = get(car + 0x600 + 4 * wheel) != 0
                    && !crate::math::x87_load::below_or_unordered(
                        get(car + 0xF40 + 4 * (wheel / 2)),
                        grip,
                    )
                    && get(car + 0x298) & (1 << (5 + wheel)) != 0;
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
                w(0x62400C, side);
                let t = crate::scene::rotate_point::rotate_point(
                    car + 0xB4,
                    [int_80(side), zero, zero],
                );
                for (i, v) in t.into_iter().enumerate() {
                    let n = crate::math::x87_load::tbyte_to_int(v);
                    w(0x62400C, n);
                    w(
                        PLACE + 4 * i as u32,
                        k(PLACE + 4 * i as u32).wrapping_add(n),
                    );
                }
                let mut p = Regs {
                    esi: slot,
                    ebp: car,
                    ecx: c,
                    eax: r.eax,
                    ..*r
                };
                part(1, &mut p)?;
            }
        }
        r.ecx = count;
        Ok(())
    }
}

/// `race::lay_skidmarks_if_on` (0x4585F3): `lay_skidmarks` (part 1) unless `dword_6235E8`
/// or `dword_6416BC` is set.
pub unsafe fn lay_skidmarks_if_on(
    r: &mut Regs,
    part: crate::race::frame::Part,
) -> Result<(), Refused> {
    if k(0x6235E8) != 0 || k(0x6416BC) != 0 {
        return Ok(());
    }
    part(1, r)
}

/// The parts in the game.
pub fn skid_point_part(_: u32, r: &mut Regs) -> Result<(), Refused> {
    unsafe { add_skid_point(r)? };
    Ok(())
}

pub fn lay_part(_: u32, r: &mut Regs) -> Result<(), Refused> {
    unsafe { lay_skidmarks(r, &mut skid_point_part) }
}
