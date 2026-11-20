//! Race init's first step (`race::start_cars` -> `race::start_car`): sets the current car (the
//! work copy `car`) at its start on the best line. The steps here are the place on the line
//! between two checkpoints (`race::place_car_on_line`), the nearest checkpoint to the car
//! (`race::nearest_checkpoint` and its two helpers), and the car's matrix undone
//! (`race::invert_car_matrix`).

use crate::errors::{BadInput, Refused};
use crate::math::x87;
use crate::physics::car_store::Car;
use crate::physics::ground_store::GroundWork;
use crate::race::best_line::MOST_CHECKS;
use crate::race::owner_course::Course;
use crate::race::owner_physics::Physics;
use crate::race::owner_shared::RaceShared;
use crate::race::vars_store::RaceVars;

fn count(shared: &RaceShared) -> Result<u32, Refused> {
    let n: u32 = shared.vars.get().checkpoint_count.get();
    if n > MOST_CHECKS {
        return Err(BadInput("more checkpoints than the lists hold").into());
    }
    Ok(n)
}

/// Checkpoint `n`'s face and its first point (their numbers).
fn check_point(
    checkpoints: &crate::race::checkpoints_store::Checkpoints,
    course: &Course,
    n: u32,
) -> Result<(u32, u32), BadInput> {
    let face: u32 = crate::race::checkpoints_store::face(checkpoints, n as usize).get();
    let point: u32 = crate::race::course_places::face_field(course.reads(), face, 4)?;
    Ok((face, point))
}

/// Coordinate `k` of scene point `n`.
fn get(course: &Course, n: u32, k: u32) -> Result<u32, BadInput> {
    crate::race::scene_points::point_word(
        course.reads(),
        &crate::race::scene_points::no_own(),
        n,
        k,
    )
}

/// `race::place_car_on_line`: blends the first points of checkpoint `line_place` (held below
/// `checkpoint_count`) and the next one (held so too) and their headings
/// (`Buffers.checkpoints.angles`) by `line_fraction` / 65536. The result goes into `place_x`,
/// `place_y` (plus `ground_drop`), `place_z` and `place_heading`, then into the work car's place
/// and heading; the first face's number goes into `place_face`. Refuses no checkpoints or more
/// than the lists.
pub fn place_car_on_line(
    shared: &RaceShared,
    course: &Course,
    scene: &crate::scene::owner::Scene,
) -> Result<(), Refused> {
    let lights = &scene.lights;
    use rc2k_core::race::checkpoints::{lerp, line_neighbours};
    let count = count(shared)?;
    {
        let (c, n) = line_neighbours(lights.state.get().line_place.get(), count)?;
        let t: u32 = lights.state.get().line_fraction.get();
        let (face, a) = check_point(&shared.buffers.reads.get().checkpoints, course, c)?;
        course.course.get().place_face.set(face);
        let (_, b) = check_point(&shared.buffers.reads.get().checkpoints, course, n)?;
        let (x, _) = lerp(get(course, a, 0)?, get(course, b, 0)?, t);
        let (y, _) = lerp(get(course, a, 1)?, get(course, b, 1)?, t);
        let y = y.wrapping_add(shared.physics_tables.get().ground_drop.get());
        let (z, _) = lerp(get(course, a, 2)?, get(course, b, 2)?, t);
        let heading_of = |k: u32| -> u32 {
            crate::race::checkpoints_store::angle(
                &shared.buffers.reads.get().checkpoints,
                k as usize,
            )
            .get()
        };
        let (heading, _) = lerp(heading_of(c), heading_of(n), t);
        course.course.get().place_x.set(x);
        course.course.get().place_y.set(y);
        course.course.get().place_z.set(z);
        course.course.get().place_heading.set(heading);
        shared.car().position[0].set(x);
        shared.car().position[2].set(z);
        shared.car().car_heading.set(heading);
        shared.car().position[1].set(y);
        Ok(())
    }
}

/// `race::distance_to_checkpoint`: for `c` a checkpoint (0 .. count - 1, a signed test), sets
/// `scratch` to the distance in x and z (`x87::int_hypot`) from (`ground_x`, `ground_z`) to its
/// face's first point; else to -1. Refuses a checkpoint past the list.
fn distance_to_checkpoint(
    physics: &Physics,
    course: &Course,
    checkpoints: &crate::race::checkpoints_store::Checkpoints,
    vars: &RaceVars,
    c: u32,
    scene: &crate::scene::owner::Scene,
) -> Result<(), Refused> {
    let drawing = &scene.drawing;
    // The distance is the x87's.
    let count: u32 = vars.checkpoint_count.get();
    if !rc2k_core::race::checkpoints::on_line(c, count) {
        drawing.scene.get().scratch.set(u32::MAX);
        return Ok(());
    }
    if c >= MOST_CHECKS {
        return Err(BadInput("a checkpoint past the list").into());
    }
    {
        let (_, point) = check_point(checkpoints, course, c)?;
        let dx = get(course, point, 0)?.wrapping_sub(physics.ground().ground_x.get()) as i32;
        let dz = get(course, point, 2)?.wrapping_sub(physics.ground().ground_z.get()) as i32;
        drawing
            .scene
            .get()
            .scratch
            .set(x87::int_hypot(dx, dz) as u32);
    }
    Ok(())
}

/// The nearest of a stretch of checkpoints and its distance.
pub struct Nearest {
    pub checkpoint: u32,
    pub distance: u32,
}

/// `race::nearest_checkpoint_in`: measures checkpoints `from` up to `to` (once at least, whatever
/// `to`) with `distance_to_checkpoint` and returns the nearest (unsigned, the later of equals)
/// and its distance (-1 and 0 for none). Refuses more than 0x1418 measures or checkpoints, before
/// anything.
pub fn nearest_checkpoint_in(
    physics: &Physics,
    course: &Course,
    shared: &RaceShared,
    from: u32,
    to: u32,
    scene: &crate::scene::owner::Scene,
) -> Result<Nearest, Refused> {
    count(shared)?;
    let steps = if from < to { to - from } else { 1 };
    if steps > MOST_CHECKS {
        return Err(BadInput("more measures than checkpoints").into());
    }
    let mut nearest = rc2k_core::race::checkpoints::Nearest::default();
    let mut c = from;
    loop {
        // Every checkpoint, every physics tick: read straight from the stores while nothing
        // needs looking at more closely, by `distance_to_checkpoint` otherwise.
        let measured = measure_held(
            physics.ground(),
            course,
            shared,
            &mut c,
            to,
            &mut nearest,
            scene,
        );
        if measured.is_err() || c < to {
            distance_to_checkpoint(
                physics,
                course,
                &shared.buffers.reads.get().checkpoints,
                shared.vars.get(),
                c,
                scene,
            )?;
            nearest.consider(c, scene.drawing.scene.get().scratch.get());
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
    #[cfg_attr(
        target_os = "android",
        allow(
            clippy::missing_const_for_thread_local,
            reason = "clippy reports an initializer that is already const when building for Android"
        )
    )]
    static PLACES: std::cell::RefCell<Places> = const {
        std::cell::RefCell::new(Places { changes: u64::MAX, point_count: 0, places: Vec::new() })
    };
}

/// Measures checkpoints from `c` up to `to` as `distance_to_checkpoint` does, until one is off the
/// line or its reads leave the stores (`c` is then that one).
fn measure_held(
    ground: &GroundWork,
    course: &Course,
    shared: &RaceShared,
    c: &mut u32,
    to: u32,
    nearest: &mut rc2k_core::race::checkpoints::Nearest,
    scene: &crate::scene::owner::Scene,
) -> Result<(), BadInput> {
    use crate::race::course_places::with_faces_and_points;
    let (course_work, data) = (course.course.get(), &course.stage.course);
    if !crate::race::course_places::held(data) || !crate::race::scene_points::held(data) {
        return Ok(());
    }
    let (checkpoints, vars) = (&shared.buffers.reads.get().checkpoints, shared.vars.get());
    let count: u32 = vars.checkpoint_count.get();
    let (x, z) = (ground.ground_x.get(), ground.ground_z.get());
    // With rounding to nearest the distance is `round(sqrt(dx^2 + dz^2))` in plain doubles,
    // rising with the sum: a checkpoint whose sum is at least (nearest + 1)^2 cannot be kept,
    // and its root is not taken (the bound is exact while the nearest is below 2^26).
    let plain = crate::math::x87::rounds_to_nearest();
    let mut last = None;
    let point_count = course_work.scene_point_count.get();
    let changes = crate::race::course_places::first_points_changes(data);
    let measured = PLACES.with_borrow_mut(|kept| {
        if kept.changes != changes || kept.point_count != point_count {
            kept.changes = changes;
            kept.point_count = point_count;
            kept.places.clear();
        }
        with_faces_and_points(data, |faces, points| {
            while *c < to && *c < MOST_CHECKS && rc2k_core::race::checkpoints::on_line(*c, count) {
                let face = crate::race::checkpoints_store::face(checkpoints, *c as usize).get();
                let slot = *c as usize;
                let (px, pz) = match kept.places.get(slot).copied().flatten() {
                    Some((f, px, pz)) if f == face => (px, pz),
                    _ => {
                        // The place arithmetic wraps as u32.
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
    // The last one measured leaves its distance in `scratch`, as `distance_to_checkpoint` does.
    if let Some((dx, dz)) = last {
        scene
            .drawing
            .scene
            .get()
            .scratch
            .set(x87::int_hypot(dx, dz) as u32);
    }
    measured
}

/// `race::nearest_checkpoint`: copies the work car's x and z into `ground_x` / `ground_z`. With
/// checkpoints it returns the nearest of them all (`nearest_checkpoint_in` 0 .. count), its
/// distance going into `scratch` and / 65536 into `nearest_checkpoint_distance`; else 0. Refuses
/// more checkpoints than the lists, before anything.
pub fn nearest_checkpoint(
    physics: &Physics,
    course: &Course,
    shared: &RaceShared,
    scene: &crate::scene::owner::Scene,
) -> Result<u32, Refused> {
    let count = count(shared)?;
    {
        physics
            .ground()
            .ground_x
            .set(shared.car().position[0].get());
        physics
            .ground()
            .ground_z
            .set(shared.car().position[2].get());
        if count == 0 {
            return Ok(0);
        }
        let near = nearest_checkpoint_in(physics, course, shared, 0, count, scene)?;
        scene.drawing.scene.get().scratch.set(near.distance);
        let units = rc2k_core::race::checkpoints::distance_units(
            near.distance,
            course.numbers.get().k65536.get(),
        );
        shared
            .physics_tables
            .get()
            .nearest_checkpoint_distance
            .set(units);
        Ok(near.checkpoint)
    }
}

/// `race::invert_car_matrix`: the car's inverse matrix from its crash matrix.
pub fn invert_car_matrix(course: &Course, car: &Car, scene: &crate::scene::owner::Scene) {
    let length = crate::scene::view_matrix::invert_turn(
        course.reads(),
        &car.crash_matrix,
        &car.car_inverse_matrix,
    );
    // The matrix's last length is left in `scratch`.
    scene.drawing.scene.get().scratch.set(length);
}
