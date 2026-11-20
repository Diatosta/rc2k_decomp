//! The skidmarks laid (`race::lay_skidmarks_if_on` -> `race::lay_skidmarks` ->
//! `race::add_skid_point`): each car's skidding wheels add a point to the ring of 0x400 skid
//! points (`skid_points`, 0x18 bytes: the mark's left and right edges) linked in strips
//! (`skid_links`, 8 bytes: +0 the next, +2 the previous, +4 the angle, +6 the car's bit).

use crate::errors::{BadInput, Refused};
use crate::memory::session::PlainDword;
use crate::race::owner::Race;
use crate::race::owner_shared::RaceShared;

/// The edges' offset for an angle (`math::rotate_point_about_y` of (0x1555, y, 0) by angle << 16).
fn offsets(
    engine_tables: &crate::math::engine_tables::EngineTables,
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

/// `race::add_skid_point`: lays a wheel's next skid point across the wheel's place (`skid_place`),
/// at the angle from the last point's middle (`scene::angle_between`; 0 for none). The point's old
/// links are cut and it is linked after the last point, which is turned to the new angle (its
/// own when it starts the strip, else halfway between its previous one and this). The ring's next
/// point moves on (0x400 points). Refuses (before anything) a point or link outside the ring.
pub fn add_skid_point(race: &Race, slot: &PlainDword) -> Result<(), Refused> {
    use rc2k_core::race::skids::{Link, POINTS, Point, Ring, add_skid_point, next_point};
    /// The ring over the race's two tables: an entry is read or written alone.
    struct Tables<'a>(&'a crate::race::shadow_skids_store::ShadowSkids);
    impl Ring for Tables<'_> {
        fn capacity(&self) -> usize {
            POINTS as usize
        }
        fn point(&self, index: usize) -> Point {
            std::array::from_fn(|i| self.0.skid_points.get_u32(0x18 * index + 4 * i))
        }
        fn set_point(&mut self, index: usize, point: Point) {
            for (i, v) in point.iter().enumerate() {
                self.0.skid_points.set_u32(0x18 * index + 4 * i, *v);
            }
        }
        fn link(&self, index: usize) -> Link {
            let half = |at: usize| {
                u16::from_le_bytes([
                    self.0.skid_links.get(8 * index + at),
                    self.0.skid_links.get(8 * index + at + 1),
                ])
            };
            Link {
                next: half(0),
                previous: half(2),
                angle: half(4),
                car: half(6),
            }
        }
        fn set_link(&mut self, index: usize, link: Link) {
            for (i, v) in [link.next, link.previous, link.angle, link.car]
                .iter()
                .enumerate()
            {
                for (j, b) in v.to_le_bytes().into_iter().enumerate() {
                    self.0.skid_links.set(8 * index + 2 * i + j, b);
                }
            }
        }
    }
    let skids = race.frame.shadow_skids.get();
    let n = skids.skid_next.get();
    let last = slot.get();
    let place = crate::memory::session::read_bits(&skids.skid_place);
    let a = add_skid_point(
        Tables(skids),
        n,
        last,
        place,
        skids.skid_cars.get() as u16,
        |first, second| crate::scene::angle_between::angle_of(&race.course, first, second),
        |angle, y| offsets(race.engine_tables(), angle, y),
    )?;
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
    for n in 0..shared.view.get().race_cars.get().min(8) {
        if !shared.work.get().has_car_block(n as usize) {
            return Err(BadInput("a race car without its block").into());
        }
    }
    if shared.view.get().race_cars.get() > 8 {
        return Err(BadInput("more than eight race cars").into());
    }
    Ok(())
}

/// `race::lay_skidmarks`: for each race car (its bit in `skid_cars`) and each of its four wheels,
/// a point is laid when the wheel skids, its axle's grip is not below `skid_grip` and its mark is
/// on (`parts_present` bits 5 .. 8). The point is the wheel's ground place plus (+-0x1555, 0, 0)
/// turned by the car's crash matrix (`scene::rotate_point`, rounded). Otherwise the wheel's skid
/// slot is set to -1. Refuses (before anything) a car without its block.
pub fn lay_skidmarks(race: &Race) -> Result<(), Refused> {
    let skids = race.frame.shadow_skids.get();
    let shared = &race.shared;
    use crate::race::skidmarks::int_80;
    cars_refuse(&race.shared)?;
    let count = shared.view.get().race_cars.get();
    if count == 0 {
        return Ok(());
    }
    let grip = skids.skid_grip.get();
    let zero = int_80(0);
    for c in 0..count {
        skids.skid_cars.set(1 << (c & 31));
        // The skid slots are the drawing's own (in the block); the wheels, the physics'.
        let slots = &shared.race_car(c as usize)?.skid_slots;
        let car = race
            .frame
            .cars
            .car(shared.work.get().block_number(c as usize)?)?;
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
            // The wheel has not moved since its last point: that point stands for the place, and
            // the strip goes on from it (see `rc2k_core::race::skids::is_repeat`).
            let last = slot.get();
            if last < rc2k_core::race::skids::POINTS {
                let at = 0x18 * last as usize;
                let last_point = std::array::from_fn(|k| skids.skid_points.get_u32(at + 4 * k));
                let place = crate::memory::session::read_bits(&skids.skid_place);
                if rc2k_core::race::skids::is_repeat(&last_point, place) {
                    continue;
                }
            }
            add_skid_point(race, slot)?;
        }
    }
    Ok(())
}

/// `race::lay_skidmarks_if_on`: `lay_skidmarks` unless `scene_switch` or `replay_still_on`
/// is set.
pub(super) fn lay_skidmarks_if_on(race: &Race) -> Result<(), Refused> {
    let frame = &race.frame;
    if race.shared.vars.get().scene_switch.get() != 0
        || frame.frame_work.get().replay_still_on.get() != 0
    {
        return Ok(());
    }
    lay_skidmarks(race)
}
