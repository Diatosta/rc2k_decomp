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
    struct Tables<'a>(&'a mut crate::race::shadow_skids_store::ShadowSkids);
    impl Ring for Tables<'_> {
        fn capacity(&self) -> usize {
            POINTS as usize
        }
        fn point(&self, index: usize) -> Point {
            std::array::from_fn(|i| self.0.point_word(0x18 * index + 4 * i))
        }
        fn set_point(&mut self, index: usize, point: Point) {
            for (i, v) in point.iter().enumerate() {
                self.0.set_point_word(0x18 * index + 4 * i, *v);
            }
        }
        fn link(&self, index: usize) -> Link {
            let half = |at: usize| {
                u16::from_le_bytes([
                    self.0.skid_links[8 * index + at],
                    self.0.skid_links[8 * index + at + 1],
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
                    self.0.skid_links[8 * index + 2 * i + j] = b;
                }
            }
        }
    }
    let mut skids = race.frame.shadow_skids.borrow_mut();
    let n = skids.skid_next;
    let last = slot.get();
    let place = skids.skid_place;
    let cars = skids.skid_cars as u16;
    add_skid_point(
        Tables(&mut skids),
        n,
        last,
        place,
        cars,
        |first, second| crate::scene::angle_between::angle_of(&race.course, first, second),
        |angle, y| offsets(race.engine_tables(), angle, y),
    )?;
    slot.set(n);
    skids.skid_next = next_point(skids.skid_next);
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
    let skids = &race.frame.shadow_skids;
    let shared = &race.shared;
    use crate::race::skidmarks::int_80;
    cars_refuse(&race.shared)?;
    let count = shared.view.get().race_cars.get();
    if count == 0 {
        return Ok(());
    }
    let grip = skids.borrow().skid_grip;
    let zero = int_80(0);
    for c in 0..count {
        skids.borrow_mut().skid_cars = 1 << (c & 31);
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
                skids.borrow_mut().skid_place[i] = run[wheel].get();
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
                let mut skids = skids.borrow_mut();
                skids.skid_place[i] = skids.skid_place[i].wrapping_add(n);
            }
            // The wheel has not moved since its last point: that point stands for the place, and
            // the strip goes on from it (see `rc2k_core::race::skids::is_repeat`).
            let last = slot.get();
            if last < rc2k_core::race::skids::POINTS {
                let at = 0x18 * last as usize;
                let (last_point, place) = {
                    let skids = skids.borrow();
                    (
                        std::array::from_fn(|k| skids.point_word(at + 4 * k)),
                        skids.skid_place,
                    )
                };
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
        || frame.frame_work.borrow().replay_still_on != 0
    {
        return Ok(());
    }
    lay_skidmarks(race)
}
