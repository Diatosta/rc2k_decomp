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

const POINTS: u32 = 0x645B18; // g_skidPoints
const LINKS: u32 = 0x643AE8; // g_skidLinks
const NEXT: u32 = 0x645B10; // dword_645B10: the next point
const PLACE: u32 = 0x643AD8; // dword_643AD8 .. dword_643AE0: the wheel's place
const OFFSET_X: u32 = 0x645B08;
const OFFSET_Z: u32 = 0x645B0C;
const ANGLE: u32 = 0x645B00;
const HALF_WIDTH: i32 = 0x1555;

fn point(n: u32) -> u32 {
    (game_addr(rva(POINTS as usize)) as u32).wrapping_add(n.wrapping_mul(0x18))
}

/// The link's word `o` (0 next, 2 previous, 4 angle, 6 the car's bit) of point `n`.
fn link(n: u32, o: u32) -> *mut u16 {
    (game_addr(rva(LINKS as usize)) as u32)
        .wrapping_add(n.wrapping_mul(8))
        .wrapping_add(o) as *mut u16
}

unsafe fn get(a: u32) -> u32 {
    unsafe { (a as *const u32).read_unaligned() }
}

unsafe fn put(a: u32, v: u32) {
    unsafe { (a as *mut u32).write_unaligned(v) }
}

/// A point's middle, (left - right) >> 1 + right for x, y, z.
unsafe fn middle(p: u32) -> [u32; 3] {
    unsafe {
        std::array::from_fn(|i| {
            let (a, b) = (get(p + 4 * i as u32), get(p + 0xC + 4 * i as u32));
            ((a.wrapping_sub(b) as i32 >> 1) as u32).wrapping_add(b)
        })
    }
}

/// The edges' offset for an angle (`math::turn_point_xz` of (0x1555, y, 0) by angle << 16)
/// into `dword_645B08` / `dword_645B0C`.
unsafe fn offsets(angle: u32, y: u32) {
    unsafe {
        let [x, _, z, _] =
            crate::math::turn_point_xz::turn_point_xz(HALF_WIDTH, y as i32, 0, angle << 16);
        w(OFFSET_X, x);
        w(OFFSET_Z, z);
    }
}

/// Point `p`'s edges: (centre - offset, centre + offset) in x and z, y the centre's.
unsafe fn set_edges(p: u32, centre: [u32; 3]) -> Regs {
    unsafe {
        let (ox, oz) = (k(OFFSET_X), k(OFFSET_Z));
        put(p, centre[0].wrapping_sub(ox));
        put(p + 4, centre[1]);
        put(p + 8, centre[2].wrapping_sub(oz));
        let r = Regs {
            eax: centre[0].wrapping_add(ox),
            ebx: centre[1],
            ecx: centre[2].wrapping_add(oz),
            ..Regs::default()
        };
        put(p + 0xC, r.eax);
        put(p + 0x10, r.ebx);
        put(p + 0x14, r.ecx);
        r
    }
}

/// `race::add_skid_point` (0x45860C): a wheel's next skid point (`dword_645B10`, its slot
/// esi -> the wheel's last point, then the new one): across the wheel's place
/// (`dword_643AD8` ..) at the angle from the last point's middle (`scene::angle_between`;
/// 0 for none), the point's old links cut, linked after the last; the last point turned to
/// the new angle (its own when it starts the strip, else halfway between its previous one
/// and this); the ring's next moved on (0x400 points). Gives the registers it leaves.
pub unsafe fn add_skid_point(entry: &Regs) -> Regs {
    unsafe {
        let mut r = *entry;
        let slot = r.esi;
        r.edx = k(NEXT);
        r.ebp = get(slot);
        put(slot, r.edx);
        w(OFFSET_X, 0);
        w(OFFSET_Z, 0);
        w(ANGLE, 0);
        let n = r.edx;
        let last = r.ebp;
        if last as u16 != 0xFFFF {
            let m = middle(point(last));
            for (i, v) in m.iter().enumerate() {
                w(0x645AF4 + 4 * i as u32, *v);
            }
            let angle = crate::scene::angle_between::angle_of(
                k(PLACE).wrapping_sub(m[0]),
                k(PLACE + 8).wrapping_sub(m[2]),
            );
            w(ANGLE, angle);
            offsets(angle & 0xFFFF, m[2]);
        }
        r.esi = k(0x645B14);
        link(n, 6).write_unaligned(r.esi as u16);
        r.esi = point(n);
        set_edges(r.esi, [k(PLACE), k(PLACE + 4), k(PLACE + 8)]);
        r.ebx = k(PLACE + 4);
        r.ecx = link(n, 2).read_unaligned() as u32;
        if r.ecx as u16 != 0xFFFF {
            link(r.ecx, 0).write_unaligned(0xFFFF);
        }
        r.ecx = link(n, 0).read_unaligned() as u32;
        if r.ecx as u16 != 0xFFFF {
            link(r.ecx, 2).write_unaligned(0xFFFF);
        }
        if last as u16 != 0xFFFF {
            link(last, 0).write_unaligned(n as u16);
        }
        link(n, 2).write_unaligned(last as u16);
        link(n, 0).write_unaligned(0xFFFF);
        r.eax = k(ANGLE);
        link(n, 4).write_unaligned(r.eax as u16);
        if last != u32::MAX {
            if link(last, 2).read_unaligned() == 0xFFFF {
                link(last, 4).write_unaligned(k(ANGLE) as u16);
                r.esi = point(last);
                let p = r.esi;
                let centre = [get(p), get(p + 4), get(p + 8)];
                let right = [get(p + 0xC), get(p + 0x10), get(p + 0x14)];
                let (ox, oz) = (k(OFFSET_X), k(OFFSET_Z));
                put(p, centre[0].wrapping_sub(ox));
                put(p + 4, centre[1]);
                put(p + 8, centre[2].wrapping_sub(oz));
                r.eax = right[0].wrapping_add(ox);
                r.ebx = right[1];
                r.ecx = right[2].wrapping_add(oz);
                put(p + 0xC, r.eax);
                put(p + 0x10, r.ebx);
                put(p + 0x14, r.ecx);
            } else {
                let a = k(ANGLE);
                let b = link(last, 4).read_unaligned();
                let half = ((a as u16).wrapping_sub(b) as i16 >> 1) as u16;
                let half = half.wrapping_add(b);
                let between = (a & 0xFFFF_0000) | half as u32;
                w(0x645B04, between);
                let m = middle(point(last));
                for (i, v) in m.iter().enumerate() {
                    w(0x645AE8 + 4 * i as u32, *v);
                }
                offsets(between, m[1]);
                r.esi = point(last);
                let e = set_edges(r.esi, m);
                (r.eax, r.ebx, r.ecx) = (e.eax, e.ebx, e.ecx);
            }
        }
        let next = k(NEXT).wrapping_add(1);
        w(NEXT, if next == 0x400 { 0 } else { next });
        r
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
    unsafe { add_skid_point(r) };
    Ok(())
}

pub fn lay_part(_: u32, r: &mut Regs) -> Result<(), Refused> {
    unsafe { lay_skidmarks(r, &mut skid_point_part) }
}
