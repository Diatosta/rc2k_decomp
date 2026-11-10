//! Race init's first step (`race::start_cars` -> `race::start_car`, IDA 2026-10-17): the current
//! car (the work copy `g_car`, 0x719FD8) set at its start on the best line. Here its steps:
//! the place on the line between two checkpoints (`race::place_car_on_line`), the nearest
//! checkpoint to the car (`race::nearest_checkpoint` and its two helpers), and the car's
//! matrix undone (`race::invert_car_matrix`).

use crate::errors::{BadInput, Refused};
use crate::math::x87;
use crate::race::best_line::MOST_CHECKS;
use crate::race::owner::Race;
use crate::race::owner_course::Course;
use crate::race::owner_shared::Shared;

fn count(shared: &Shared) -> Result<u32, Refused> {
    let n: u32 = shared.vars.checkpoint_count.bits();
    if n > MOST_CHECKS {
        return Err(BadInput("more checkpoints than the lists hold").into());
    }
    Ok(n)
}

/// Checkpoint n's face (number, address) and its first point's (+4) number and address.
/// Checkpoint `n`'s face and its first point (their numbers).
fn check_point(n: u32) -> Result<(u32, u32), BadInput> {
    let face: u32 = crate::race::checkpoints_store::face(n as usize).get();
    let point: u32 = crate::race::course_places::face_field(face, 4, "a checkpoint face")?;
    Ok((face, point))
}

/// Coordinate `k` of scene point `n`.
fn get(n: u32, k: u32) -> Result<u32, BadInput> {
    crate::race::scene_points::point_word(n, k)
}

/// `race::place_car_on_line` (0x44666F): checkpoint `dword_6365E4` (held below `g_checkpointCount`)
/// and the next (held so too); their faces' first points and their headings (`dword_9ED808`)
/// blended by `dword_6365EC` / 65536 into `dword_5CCDA0` / `C0` (+ `dword_613FC4`) / `E0` and
/// `dword_5CCD60`, then into the work car's place and heading; the first face's number in
/// `dword_5CCD28`. Refuses no checkpoints (the original reads entry -1) or more than the lists.
pub fn place_car_on_line(
    race: &Race,
    course: &Course,
    scene: &crate::scene::owner::Scene,
) -> Result<(), Refused> {
    let shared = &race.shared;
    let lights = &scene.lights;
    // Converted: `rc2k_core::race::checkpoints::{line_neighbours, lerp}`.
    use rc2k_core::race::checkpoints::{lerp, line_neighbours};
    let count = count(shared)?;
    {
        let (c, n) = line_neighbours(lights.state.line_place.bits(), count)?;
        let t: u32 = lights.state.line_fraction.bits();
        let (face, a) = check_point(c)?;
        course.course.place_face.set_bits(face);
        let (_, b) = check_point(n)?;
        let (x, _) = lerp(get(a, 0)?, get(b, 0)?, t);
        let (y, _) = lerp(get(a, 1)?, get(b, 1)?, t);
        let y = y.wrapping_add(shared.physics_tables.ground_drop.bits());
        let (z, _) = lerp(get(a, 2)?, get(b, 2)?, t);
        let heading_of =
            |k: u32| -> u32 { crate::race::checkpoints_store::angle(k as usize).get() };
        let (heading, _) = lerp(heading_of(c), heading_of(n), t);
        course.course.place_x.set_bits(x);
        course.course.place_y.set_bits(y);
        course.course.place_z.set_bits(z);
        course.course.place_heading.set_bits(heading);
        shared.car.position[0].set_bits(x);
        shared.car.position[2].set(z);
        shared.car.car_heading.set_bits(heading);
        shared.car.position[1].set(y);
        Ok(())
    }
}

/// `race::distance_to_checkpoint` (0x44141D): with `c` a checkpoint (0 .. count - 1, by sign
/// tests), `dword_62400C` the distance in x and z (`x87::int_hypot`) from (`dword_639824`,
/// `dword_63982C`) to its face's first point; else -1. Refuses a checkpoint past the list.
pub fn distance_to_checkpoint(
    race: &Race,
    c: u32,
    scene: &crate::scene::owner::Scene,
) -> Result<(), Refused> {
    let physics = &race.physics;
    let drawing = &scene.drawing;
    // Converted: `rc2k_core::race::checkpoints::on_line`; the distance the x87's.
    let count: u32 = race.shared.vars.checkpoint_count.bits();
    if !rc2k_core::race::checkpoints::on_line(c, count) {
        drawing.scene.scratch.set_bits(u32::MAX);
        return Ok(());
    }
    if c >= MOST_CHECKS {
        return Err(BadInput("a checkpoint past the list").into());
    }
    {
        let (_, point) = check_point(c)?;
        let dx = get(point, 0)?.wrapping_sub(physics.ground.ground_x.bits()) as i32;
        let dz = get(point, 2)?.wrapping_sub(physics.ground.ground_z.bits()) as i32;
        drawing
            .scene
            .scratch
            .set_bits(x87::int_hypot(dx, dz) as u32);
    }
    Ok(())
}

/// The nearest of a stretch of checkpoints and its distance (ebp and edi in the original).
pub struct Nearest {
    pub checkpoint: u32,
    pub distance: u32,
}

/// `race::nearest_checkpoint_in` (0x4413F8): checkpoints `from` up to `to` (once at least, `jb`)
/// measured by `distance_to_checkpoint`; the nearest (unsigned, the later of equals) and its
/// distance (-1 and 0 for none). Refuses more than 0x1418 measures or checkpoints, before anything.
pub fn nearest_checkpoint_in(
    race: &Race,
    from: u32,
    to: u32,
    scene: &crate::scene::owner::Scene,
) -> Result<Nearest, Refused> {
    count(&race.shared)?;
    let steps = if from < to { to - from } else { 1 };
    if steps > MOST_CHECKS {
        return Err(BadInput("more measures than checkpoints").into());
    }
    // Converted: `rc2k_core::race::checkpoints::Nearest`.
    let mut nearest = rc2k_core::race::checkpoints::Nearest::default();
    let mut c = from;
    loop {
        // Every checkpoint, every physics tick: read straight from the stores while nothing
        // needs looking at more closely, by `distance_to_checkpoint` otherwise.
        let measured = measure_held(race, &mut c, to, &mut nearest, scene);
        if measured.is_err() || c < to {
            distance_to_checkpoint(race, c, scene)?;
            nearest.consider(c, scene.drawing.scene.scratch.bits());
            c = c.wrapping_add(1);
        }
        if c >= to {
            return Ok(Nearest {
                checkpoint: nearest.checkpoint,
                distance: nearest.distance,
            });
        }
    }
}

/// Checkpoints from `c` up to `to` measured as `distance_to_checkpoint` does, until one is
/// off the line or its reads leave the stores (`c` is then that one).
/// The checkpoints' places as last read: for each, its face and its first point's x and z.
/// Kept while the faces' first points and the course's scene points are unchanged
/// (`course_places::first_points_changes`) and the scene point count is the same; each
/// entry is used only for the face it was read for, and only points below the count are
/// kept (the ones past it change while drawing).
struct Places {
    changes: u64,
    point_count: u32,
    places: Vec<Option<(u32, u32, u32)>>,
}

thread_local! {
    static PLACES: std::cell::RefCell<Places> = const {
        std::cell::RefCell::new(Places { changes: u64::MAX, point_count: 0, places: Vec::new() })
    };
}

fn measure_held(
    race: &Race,
    c: &mut u32,
    to: u32,
    nearest: &mut rc2k_core::race::checkpoints::Nearest,
    scene: &crate::scene::owner::Scene,
) -> Result<(), BadInput> {
    use crate::race::course_places::{faces_base, points_base, with_faces_and_points};
    if race.course.course.course_faces.bits() != faces_base()
        || race.course.course.scene_points.bits() != points_base()
    {
        return Ok(());
    }
    let count: u32 = race.shared.vars.checkpoint_count.bits();
    let (x, z) = (
        race.physics.ground.ground_x.bits(),
        race.physics.ground.ground_z.bits(),
    );
    // With rounding to nearest the distance is `round(sqrt(dx² + dz²))` in plain doubles,
    // rising with the sum: a checkpoint whose sum is at least (nearest + 1)² cannot be kept,
    // and its root is not taken (the bound is exact while the nearest is below 2^26).
    let plain = crate::math::x87::rounds_to_nearest();
    let mut last = None;
    let point_count = race.course.course.scene_point_count.get();
    let changes = crate::race::course_places::first_points_changes();
    let measured = PLACES.with_borrow_mut(|kept| {
        if kept.changes != changes || kept.point_count != point_count {
            kept.changes = changes;
            kept.point_count = point_count;
            kept.places.clear();
        }
        with_faces_and_points(|faces, points| {
            while *c < to && *c < MOST_CHECKS && rc2k_core::race::checkpoints::on_line(*c, count) {
                let face = crate::race::checkpoints_store::face(*c as usize).get();
                let slot = *c as usize;
                let (px, pz) = match kept.places.get(slot).copied().flatten() {
                    Some((f, px, pz)) if f == face => (px, pz),
                    _ => {
                        // As the original's u32 arithmetic on the places.
                        let Some(point) = faces((face << 6).wrapping_add(4) as usize) else {
                            return;
                        };
                        let at = point.wrapping_mul(12);
                        let (Some(px), Some(pz)) =
                            (points(at as usize), points(at.wrapping_add(8) as usize))
                        else {
                            return;
                        };
                        if point < point_count {
                            if kept.places.len() <= slot {
                                kept.places.resize(slot + 1, None);
                            }
                            kept.places[slot] = Some((face, px, pz));
                        }
                        (px, pz)
                    }
                };
                let dx = px.wrapping_sub(x) as i32;
                let dz = pz.wrapping_sub(z) as i32;
                let (a, b) = (f64::from(dx), f64::from(dz));
                let beyond = plain && nearest.distance < 1 << 26 && {
                    let bound = f64::from(nearest.distance + 1);
                    a * a + b * b >= bound * bound
                };
                if !beyond {
                    nearest.consider(*c, x87::int_hypot(dx, dz) as u32);
                }
                last = Some((dx, dz));
                *c += 1;
            }
        })
    });
    // The last one measured leaves its distance, as `distance_to_checkpoint` does.
    if let Some((dx, dz)) = last {
        scene
            .drawing
            .scene
            .scratch
            .set_bits(x87::int_hypot(dx, dz) as u32);
    }
    measured
}

/// `race::nearest_checkpoint` (0x44134F): the work car's x and z into `dword_639824` /
/// `dword_63982C`; with checkpoints, the nearest of them all (`nearest_checkpoint_in` 0 .. count),
/// its distance in `dword_62400C` and / 65536 in `flt_614C68`; else 0. Refuses more checkpoints
/// than the lists, before anything.
pub fn nearest_checkpoint(race: &Race, scene: &crate::scene::owner::Scene) -> Result<u32, Refused> {
    let count = count(&race.shared)?;
    {
        race.physics
            .ground
            .ground_x
            .set_bits(race.shared.car.position[0].bits());
        race.physics
            .ground
            .ground_z
            .set_bits(race.shared.car.position[2].get());
        if count == 0 {
            return Ok(0);
        }
        let near = nearest_checkpoint_in(race, 0, count, scene)?;
        scene.drawing.scene.scratch.set_bits(near.distance);
        // Converted: `rc2k_core::race::checkpoints::distance_units`.
        let units = rc2k_core::race::checkpoints::distance_units(
            near.distance,
            f32::from_bits(race.course.numbers.k65536.bits()),
        );
        race.shared
            .physics_tables
            .nearest_checkpoint_distance
            .set_bits(units.to_bits());
        Ok(near.checkpoint)
    }
}

pub fn invert_car_matrix(race: &Race) {
    let shared = &race.shared;
    let length = crate::scene::view_matrix::invert_turn(
        &race.course,
        &shared.car.crash_matrix,
        &shared.car.car_inverse_matrix,
    );
    // `scene::square_matrix` left its last length in `g_scratch`.
    crate::scene::owner::scene()
        .drawing
        .scene
        .scratch
        .set_bits(length);
}
