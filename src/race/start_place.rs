//! Race init's first step (`sub_446463` -> `sub_4464AD`, IDA 2026-10-17): the current car
//! (the work copy `g_car`, 0x719FD8) set at its start on the best line. Here its steps:
//! the place on the line between two checkpoints (`race::place_car_on_line`), the nearest
//! checkpoint to the car (`race::nearest_checkpoint` and its two helpers), and the car's
//! matrix undone (`race::invert_car_matrix`).

use crate::errors::{BadInput, Refused};
use crate::math::x87;
use crate::physics::car_store::CAR;
use crate::physics::ground_store::GROUND_WORK;
use crate::physics::tables_store::PHYSICS;
use crate::race::best_line::MOST_CHECKS;
use crate::race::course_store::COURSE;
use crate::race::numbers_store::NUMBERS;
use crate::race::vars_store::RACE_VARS;
use crate::scene::draw_store::DRAW;
use crate::scene::light_store::LIGHT_STATE;

fn count() -> Result<u32, Refused> {
    let n: u32 = RACE_VARS.checkpoint_count.bits();
    if n > MOST_CHECKS {
        return Err(BadInput("more checkpoints than the lists hold").into());
    }
    Ok(n)
}

/// Checkpoint n's face (number, address) and its first point's (+4) number and address.
fn check_point(n: u32) -> Result<(u32, u32, u32, u32), BadInput> {
    let face: u32 = crate::race::checkpoints_store::face(n as usize).get();
    let address = (face << 6).wrapping_add(COURSE.course_faces.bits());
    let point: u32 =
        crate::race::course_places::read(address + 4, "a checkpoint face's first point")?;
    let place = (point.wrapping_mul(3) << 2).wrapping_add(COURSE.scene_points.bits());
    Ok((face, address, point, place))
}

/// A scene point's dword (`race::scene_points`).
fn get(at: u32) -> Result<u32, BadInput> {
    crate::race::scene_points::word(at)
}

/// `race::place_car_on_line` (0x44666F): checkpoint `dword_6365E4` (held below
/// `g_checkpointCount`) and the next (held so too); their faces' first points and their
/// headings (`dword_9ED808`) blended by `dword_6365EC` / 65536 into `dword_5CCDA0` /
/// `C0` (+ `dword_613FC4`) / `E0` and `dword_5CCD60`, then into the work car's place and
/// heading; the first face's number in `dword_5CCD28`. The registers it leaves are read
/// by none of its callers (conversion phase 3, 2026-10-24). Refuses no checkpoints (the
/// original reads entry -1) or more than the lists.
pub fn place_car_on_line() -> Result<(), Refused> {
    // Converted: `rc2k_core::race::checkpoints::{line_neighbours, lerp}`.
    use rc2k_core::race::checkpoints::{lerp, line_neighbours};
    let count = count()?;
    {
        let (c, n) = line_neighbours(LIGHT_STATE.line_place.bits(), count)?;
        let t: u32 = LIGHT_STATE.line_fraction.bits();
        let (face, _, _, a) = check_point(c)?;
        COURSE.place_face.set_bits(face);
        let (_, _, _, b) = check_point(n)?;
        let (x, _) = lerp(get(a)?, get(b)?, t);
        let (y, _) = lerp(get(a + 4)?, get(b + 4)?, t);
        let y = y.wrapping_add(PHYSICS.ground_drop.bits());
        let (z, _) = lerp(get(a + 8)?, get(b + 8)?, t);
        let heading_of =
            |k: u32| -> u32 { crate::race::checkpoints_store::angle(k as usize).get() };
        let (heading, _) = lerp(heading_of(c), heading_of(n), t);
        COURSE.place_x.set_bits(x);
        COURSE.place_y.set_bits(y);
        COURSE.place_z.set_bits(z);
        COURSE.place_heading.set_bits(heading);
        CAR.position[0].set_bits(x);
        CAR.position[2].set(z);
        CAR.car_heading.set_bits(heading);
        CAR.position[1].set(y);
        Ok(())
    }
}

/// `race::distance_to_checkpoint` (0x44141D): with `c` a checkpoint (0 .. count - 1, by
/// sign tests), `dword_62400C` the distance in x and z (`x87::int_hypot`) from
/// (`dword_639824`, `dword_63982C`) to its face's first point; else -1. Refuses a
/// checkpoint past the list.
pub fn distance_to_checkpoint(c: u32) -> Result<(), Refused> {
    // Converted: `rc2k_core::race::checkpoints::on_line`; the distance the x87's.
    let count: u32 = RACE_VARS.checkpoint_count.bits();
    if !rc2k_core::race::checkpoints::on_line(c, count) {
        DRAW.scratch.set_bits(u32::MAX);
        return Ok(());
    }
    if c >= MOST_CHECKS {
        return Err(BadInput("a checkpoint past the list").into());
    }
    {
        let (_, _, _, place) = check_point(c)?;
        let dx = crate::race::scene_points::word(place)?.wrapping_sub(GROUND_WORK.ground_x.bits())
            as i32;
        let dz = crate::race::scene_points::word(place + 8)?
            .wrapping_sub(GROUND_WORK.ground_z.bits()) as i32;
        DRAW.scratch.set_bits(x87::int_hypot(dx, dz) as u32);
    }
    Ok(())
}

/// The nearest of a stretch of checkpoints and its distance (ebp and edi in the original).
pub struct Nearest {
    pub checkpoint: u32,
    pub distance: u32,
}

/// `race::nearest_checkpoint_in` (0x4413F8): checkpoints `from` up to `to` (once at
/// least, `jb`) measured by `distance_to_checkpoint`; the nearest (unsigned, the later of
/// equals) and its distance (-1 and 0 for none). Refuses more than 0x1418 measures or
/// checkpoints, before anything.
pub fn nearest_checkpoint_in(from: u32, to: u32) -> Result<Nearest, Refused> {
    count()?;
    let steps = if from < to { to - from } else { 1 };
    if steps > MOST_CHECKS {
        return Err(BadInput("more measures than checkpoints").into());
    }
    // Converted: `rc2k_core::race::checkpoints::Nearest`.
    let mut nearest = rc2k_core::race::checkpoints::Nearest::default();
    let mut c = from;
    loop {
        distance_to_checkpoint(c)?;
        nearest.consider(c, DRAW.scratch.bits());
        c = c.wrapping_add(1);
        if c >= to {
            return Ok(Nearest {
                checkpoint: nearest.checkpoint,
                distance: nearest.distance,
            });
        }
    }
}

/// `race::nearest_checkpoint` (0x44134F): the work car's x and z into `dword_639824` /
/// `dword_63982C`; with checkpoints, the nearest of them all (`nearest_checkpoint_in` 0 ..
/// count), its distance in `dword_62400C` and / 65536 in `flt_614C68`; else 0. The other
/// registers it leaves are read by none of its callers. Refuses more checkpoints than the
/// lists, before anything.
pub fn nearest_checkpoint() -> Result<u32, Refused> {
    let count = count()?;
    {
        GROUND_WORK.ground_x.set_bits(CAR.position[0].bits());
        GROUND_WORK.ground_z.set_bits(CAR.position[2].get());
        if count == 0 {
            return Ok(0);
        }
        let near = nearest_checkpoint_in(0, count)?;
        DRAW.scratch.set_bits(near.distance);
        // Converted: `rc2k_core::race::checkpoints::distance_units`.
        let units = rc2k_core::race::checkpoints::distance_units(
            near.distance,
            f32::from_bits(NUMBERS.k65536.bits()),
        );
        PHYSICS
            .nearest_checkpoint_distance
            .set_bits(units.to_bits());
        Ok(near.checkpoint)
    }
}

/// `race::invert_car_matrix` (0x43F16C): `scene::invert_turn` of the work car's matrix
/// (`g_crashMatrix`, +0xB4) into its inverse (+0x9CC, 0x71A9A4), every register kept.
pub fn invert_car_matrix() {
    crate::scene::view_matrix::invert_turn(
        CAR.crash_matrix[0].addr() as usize,
        CAR.car_inverse_matrix[0].addr() as usize,
    );
}
