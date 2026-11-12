//! The skidmarks laid (`race::lay_skidmarks_if_on` -> `race::lay_skidmarks` ->
//! `race::add_skid_point`): each car's skidding wheels add a point to the
//! ring of 0x400 skid points (`g_skidPoints`, 0x18 bytes: the mark's left and right edges) linked
//! in strips (`g_skidLinks`, 8 bytes: +0 the next, +2 the previous, +4 the angle, +6 the car's
//! bit).

use crate::errors::{BadInput, Refused};
use crate::race::owner::Race;
use crate::race::owner_shared::RaceShared;
use crate::session::Dword;

/// The edges' offset for an angle (`math::rotate_point_about_y` of (0x1555, y, 0) by angle << 16).
fn offsets(
    engine_tables: &crate::scene::engine_tables_store::EngineTables,
    angle: u32,
    y: u32,
) -> (u32, u32) {
    let [x, _, z, _] = crate::math::rotate_point_about_y::rotate_point_about_y(
        engine_tables,
        rc2k_core::race::skids::HALF_WIDTH,
        y as i32,
        0,
        angle << 16,
    );
    (x, z)
}

/// `race::add_skid_point` (0x45860C): a wheel's next skid point: across the wheel's place
/// (`skid_place` ..) at the angle from the last point's middle (`scene::angle_between`; 0 for
/// none), the point's old links cut, linked after the last; the last point turned to the new angle
/// (its own when it starts the strip, else halfway between its previous one and this); the ring's
/// next moved on (0x400 points). Refuses (before anything) a point or a link outside the ring (the
/// original reads and writes past its tables).
pub fn add_skid_point(race: &Race, slot: &Dword) -> Result<(), Refused> {
    // Converted: `rc2k_core::race::skids::add_skid_point`.
    use rc2k_core::race::skids::{Link, POINTS, Point, Ring, add_skid_point, next_point};
    let skids = race.frame.shadow_skids.get();
    let n = skids.skid_next.get();
    let last = slot.get();
    // The ring read out of its tables (and written back after, what changed).
    let word = |b: &[u8], k: usize| u32::from_le_bytes([b[k], b[k + 1], b[k + 2], b[k + 3]]);
    let half = |b: &[u8], k: usize| u16::from_le_bytes([b[k], b[k + 1]]);
    let point_bytes = skids.skid_points.read();
    let link_bytes = skids.skid_links.read();
    let mut points: Vec<Point> = point_bytes
        .as_chunks::<0x18>()
        .0
        .iter()
        .take(POINTS as usize)
        .map(|b| std::array::from_fn(|i| word(b, 4 * i)))
        .collect();
    let mut links: Vec<Link> = link_bytes
        .as_chunks::<8>()
        .0
        .iter()
        .take(POINTS as usize)
        .map(|b| Link {
            next: half(b, 0),
            previous: half(b, 2),
            angle: half(b, 4),
            car: half(b, 6),
        })
        .collect();
    let (points_before, links_before) = (points.clone(), links.clone());
    let place = crate::session::read_bits(&skids.skid_place);
    let a = add_skid_point(
        Ring {
            points: &mut points,
            links: &mut links,
        },
        n,
        last,
        place,
        skids.skid_cars.get() as u16,
        |first, second| crate::scene::angle_between::angle_of(&race.course, first, second),
        |angle, y| offsets(race.engine_tables(), angle, y),
    )?;
    for (k, (now, was)) in points.iter().zip(&points_before).enumerate() {
        if now != was {
            for (i, v) in now.iter().enumerate() {
                skids.skid_points.set_u32(0x18 * k + 4 * i, *v);
            }
        }
    }
    for (k, (now, was)) in links.iter().zip(&links_before).enumerate() {
        if now != was {
            for (i, v) in [now.next, now.previous, now.angle, now.car]
                .iter()
                .enumerate()
            {
                for (j, b) in v.to_le_bytes().into_iter().enumerate() {
                    skids.skid_links.set(8 * k + 2 * i + j, b);
                }
            }
        }
    }
    slot.set(n);
    skids.skid_angle.set(a.angle);
    skids.skid_offset_x.set(a.offset.0);
    skids.skid_offset_z.set(a.offset.1);
    if let Some(m) = a.from {
        for (i, v) in m.iter().enumerate() {
            skids.skid_from[i].set(*v);
        }
    }
    if let Some((turned, m)) = a.turned {
        skids.skid_turned_angle.set(turned);
        for (i, v) in m.iter().enumerate() {
            skids.skid_turned[i].set(*v);
        }
    }
    skids.skid_next.set(next_point(skids.skid_next.get()));
    Ok(())
}

/// What `lay_skidmarks` reads: the cars' blocks.
fn cars_refuse(shared: &RaceShared) -> Result<(), Refused> {
    for n in 0..shared.view.race_cars.get().min(8) {
        if !shared.work.has_car_block(n as usize) {
            return Err(BadInput("a race car without its block").into());
        }
    }
    if shared.view.race_cars.get() > 8 {
        return Err(BadInput("more than eight race cars").into());
    }
    Ok(())
}

/// `race::lay_skidmarks` (0x4588DE): for each race car (its bit in `skid_cars`) and each of its
/// four wheels (slot +0xF48 ..): when the wheel skids (+0x600 ..), its axle's grip (+0xF40 front,
/// +0xF44 rear) is not below `skid_grip` and its mark is on (+0x298 bits 5 .. 8), its place
/// (+0xF10 / +0xF20 / +0xF30 ..) plus (+-0x1555, 0, 0) turned by the car's matrix (+0xB4;
/// `scene::rotate_point`, rounded; the original's stores to `scratch` are not kept) gets a
/// point; else the slot -1. Refuses (before anything) a car without its block.
pub fn lay_skidmarks(race: &Race) -> Result<(), Refused> {
    let skids = race.frame.shadow_skids.get();
    let shared = &race.shared;
    use crate::race::skidmarks::int_80;
    cars_refuse(&race.shared)?;
    let count = shared.view.race_cars.get();
    if count == 0 {
        return Ok(());
    }
    let grip = skids.skid_grip.get();
    let zero = int_80(0);
    for c in 0..count {
        skids.skid_cars.set(1 << (c & 31));
        // The skid slots are the drawing's own (in the block); the wheels, the physics'.
        let slots = &shared.race_car(c as usize)?.skid_slots;
        let car = crate::race::frame_view::car(shared.work.block_number(c as usize)?)?;
        for wheel in 0..4usize {
            let slot = &slots[wheel];
            let on = car.wheel_squeeze[wheel].bits() != 0
                && !crate::math::x87::below_or_unordered(&car.skid_grip[wheel / 2], grip)
                && car.parts_present.get() & (1 << (5 + wheel)) != 0;
            if !on {
                slot.set(u32::MAX);
                continue;
            }
            let place = [
                &car.wheel_ground_x,
                &car.wheel_ground_height,
                &car.wheel_ground_z,
            ];
            for (i, run) in place.into_iter().enumerate() {
                skids.skid_place[i].set(run[wheel].get());
            }
            let side = if wheel % 2 == 0 {
                0x1555u32
            } else {
                0xFFFF_EAAB
            };
            let t = crate::scene::rotate_point::rotate_point(
                &car.crash_matrix,
                [int_80(side), zero, zero],
            );
            for (i, v) in t.into_iter().enumerate() {
                let n = crate::math::x87::tbyte_to_int(v);
                skids.skid_place[i].set(skids.skid_place[i].get().wrapping_add(n));
            }
            add_skid_point(race, slot)?;
        }
    }
    Ok(())
}

/// `race::lay_skidmarks_if_on` (0x4585F3): `lay_skidmarks` unless `scene_switch` or `replay_still_on`
/// is set.
pub fn lay_skidmarks_if_on(race: &Race) -> Result<(), Refused> {
    let frame = &race.frame;
    if race.shared.vars.scene_switch.get() != 0 || frame.frame_work.replay_still_on.get() != 0 {
        return Ok(());
    }
    lay_skidmarks(race)
}
