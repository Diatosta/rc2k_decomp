//! A stage's best line: the checkpoint faces (first byte 0xFF, kind `line_face_kind`) chained in
//! order, then their angles, checks, distances, map places, the replay cameras and the surface
//! shares. `best_line` reads the loaded course and returns everything as a `BestLine`;
//! `store_best_line` writes it into the race's tables.

use crate::errors::{BadInput, Refused};
use crate::math::cell_x87::CellX87;
use crate::math::x87;
use crate::math::x87_float as bh;
use crate::memory::session::{PlainDword, PlainFloat};
use crate::race::best_line_store::{Along, BestLine, CameraSet, Leftovers, MapPlace};
use crate::race::owner::Race;
use crate::race::owner_shared::RaceShared;
use crate::race::trap::Trap;
use std::cell::RefCell;

/// The number of entries `checkpoint_faces` and the lists after it have room for.
pub const MOST_CHECKS: u32 = rc2k_core::race::checkpoints::MOST_CHECKS;

const MOST_FACES: u32 = 0x10_0000;

/// A step of the best line, announced before it runs (for the log and the load timings).
#[derive(Clone, Copy)]
pub enum Step {
    Sort,
    Angles,
    Checks,
    Speeds,
    Distances,
    Map,
    CamerasOne,
    CamerasTwo,
    SurfaceShares,
}

impl Step {
    /// The word logged before the step, if it has one.
    pub fn word(self) -> Option<&'static std::ffi::CStr> {
        match self {
            Step::Angles => Some(c"angles"),
            Step::Checks => Some(c"checks"),
            Step::Speeds => Some(c"speeds"),
            Step::Distances => Some(c"dists"),
            Step::Map => Some(c"map"),
            Step::CamerasOne => Some(c"replaycam1"),
            Step::CamerasTwo => Some(c"replaycam2"),
            Step::Sort | Step::SurfaceShares => None,
        }
    }

    /// The step's name in the load timings.
    pub fn mark(self) -> &'static str {
        match self {
            Step::Sort => "best_line sort_checkpoints",
            Step::Angles => "best_line angles",
            Step::Checks => "best_line count_checks",
            Step::Speeds => "best_line speeds",
            Step::Distances => "best_line distances",
            Step::Map => "best_line map",
            Step::CamerasOne => "best_line replay cameras 1",
            Step::CamerasTwo => "best_line replay cameras 2",
            Step::SurfaceShares => "best_line surface_shares",
        }
    }
}

/// A face's first point: its number (the face's point 0).
fn point_of(race: &Race, face: u32) -> Result<u32, BadInput> {
    crate::race::course_places::face_field(race.course.reads(), face, 4)
}

/// Point `n`'s x and z words.
fn point_xz(race: &Race, n: u32) -> Result<[u32; 2], BadInput> {
    let word = |k| {
        crate::race::scene_points::point_word(
            race.course.reads(),
            &crate::race::scene_points::no_own(),
            n,
            k,
        )
    };
    Ok([word(0)?, word(2)?])
}

/// The line's checkpoints chained: the faces listed (the chain first) and how many are chained
/// (None with fewer than two faces).
fn sort_checkpoints(race: &Race, kind: u8) -> Result<(Vec<u32>, Option<u32>), Refused> {
    use rc2k_core::race::checkpoints::chain;
    let course = &race.course;
    let n: u32 = course.course.borrow().course_face_count;
    if n == 0 || n >= MOST_FACES {
        return Err(BadInput("no faces, or 0x100000 or more").into());
    }
    let trap = Trap::new();
    let mut list = Vec::new();
    for face in 0..n {
        let [first, second] = trap.keep_read(crate::race::course_places::face_field::<[u8; 2]>(
            course.reads(),
            face,
            0,
        ));
        if first == 0xFF && second == kind {
            list.push(face);
        }
    }
    trap.finish()?;
    if list.len() as u32 > MOST_CHECKS {
        return Err(BadInput("more line faces than the checkpoint list holds").into());
    }
    if list.len() < 2 {
        return Ok((list, None));
    }
    let face = |f| trap.keep_read(crate::race::course_places::face_number(course.reads(), f));
    let chained = chain(&mut list, |f| face(f).check() == 1, |f| face(f).next_link());
    trap.finish()?;
    Ok((list, Some(chained.length)))
}

/// Each link's angle: the entries of the angles table to write.
fn checkpoint_angles(
    race: &Race,
    faces: &[u32],
    left: &RefCell<Leftovers>,
) -> Result<Vec<(u32, u32)>, Refused> {
    let numbers = race.course.numbers;
    let (half, scale) = (numbers.half.get(), numbers.k65536.get());
    let trap = Trap::new();
    let (writes, _) = rc2k_core::race::best_line::link_angles(
        faces.len() as u32,
        |b| {
            let point = |n: u32| {
                trap.keep_read(point_of(race, faces[n as usize]).and_then(|p| point_xz(race, p)))
            };
            let (p, q) = (point(b - 1), point(b));
            let dx = q[0].wrapping_sub(p[0]) as i32;
            let dz = q[1].wrapping_sub(p[1]) as i32;
            left.borrow_mut().scratch = Some(dz as u32);
            x87::angle_scaled3_to_int(dx, dz, half, scale, scale) as u32
        },
        || unreachable!("a line has two checkpoints or more"),
    );
    trap.finish()?;
    Ok(writes)
}

/// The distance along the line at each checkpoint.
fn checkpoint_distances(
    race: &Race,
    faces: &[u32],
    left: &RefCell<Leftovers>,
) -> Result<Vec<u32>, Refused> {
    let last = faces.len() as u32 - 1;
    if last == 0 || last >= MOST_CHECKS {
        return Err(BadInput("a last checkpoint of 0 or past the lists").into());
    }
    let trap = Trap::new();
    let sums = rc2k_core::race::best_line::distances(last, |k| {
        let point = |n: u32| {
            trap.keep_read(point_of(race, faces[n as usize]).and_then(|p| point_xz(race, p)))
        };
        let (a, b) = (point(k + 1), point(k));
        let d = [a[0].wrapping_sub(b[0]), a[1].wrapping_sub(b[1])];
        let length = x87::int_hypot(d[0] as i32, d[1] as i32) as u32;
        let mut left = left.borrow_mut();
        left.scratch = Some(length);
        length
    });
    trap.finish()?;
    Ok(sums)
}

/// A map segment: the checkpoints `from ..= to` placed on the map. The
/// scale and angles come from the two anchors' map places and first points.
fn map_segment(
    race: &Race,
    faces: &[u32],
    from: u32,
    to: u32,
    left: &RefCell<Leftovers>,
    map: &mut Vec<MapPlace>,
) -> Result<(), Refused> {
    use crate::math::rotate_point_about_y::rotate_point_about_y;
    if from >= MOST_CHECKS || to >= MOST_CHECKS {
        return Err(BadInput("map anchors past the lists").into());
    }
    let anchor = |n: u32| -> Result<([u32; 2], [u32; 2]), BadInput> {
        let face = faces[n as usize];
        let (x, y) =
            crate::race::course_places::face_number(race.course.reads(), face)?.map_place();
        let place = [(x as u32) << 14, ((y as u32) << 14).wrapping_neg()];
        Ok((place, point_xz(race, point_of(race, face)?)?))
    };
    let (place_a, point_a) = anchor(from)?;
    let (place_b, point_b) = anchor(to)?;
    let ground = [
        point_b[0].wrapping_sub(point_a[0]),
        point_b[1].wrapping_sub(point_a[1]),
    ];
    let on_map = [
        place_b[0].wrapping_sub(place_a[0]),
        place_b[1].wrapping_sub(place_a[1]),
    ];
    let scale = x87::hypot_ratio(
        on_map[0] as i32,
        on_map[1] as i32,
        ground[0] as i32,
        ground[1] as i32,
    );
    let numbers = race.course.numbers;
    let (half, unit) = (numbers.half.get(), numbers.k65536.get());
    let map_angle =
        (x87::angle_scaled5_to_int(on_map[0] as i32, on_map[1] as i32, half, unit) as u32) << 2;
    let ground_angle =
        (x87::angle_scaled5_to_int(ground[0] as i32, ground[1] as i32, half, unit) as u32) << 2;
    for e in from..=to {
        let number = point_of(race, faces[e as usize])?;
        let p = point_xz(race, number)?;
        let [x, _, z, _] = rotate_point_about_y(
            race.engine_tables(),
            p[0].wrapping_sub(point_a[0]) as i32,
            number as i32,
            p[1].wrapping_sub(point_a[1]) as i32,
            ground_angle.wrapping_neg(),
        );
        let x = x87::int_times_to_int(x as i32, scale);
        let z = x87::int_times_to_int(z as i32, scale);
        left.borrow_mut().scratch = Some(z as u32);
        let [x, _, z, _] =
            rotate_point_about_y(race.engine_tables(), x, number as i32, z, map_angle);
        map.push(MapPlace {
            checkpoint: e,
            x: x.wrapping_add(place_a[0]),
            y: z.wrapping_add(place_a[1]).wrapping_neg(),
        });
    }
    Ok(())
}

/// The map: each stretch from checkpoint 0 to the next anchor placed. Gives the anchors' count and
/// the placed checkpoints.
fn map_checkpoints(
    race: &Race,
    faces: &[u32],
    left: &RefCell<Leftovers>,
) -> Result<(u32, Vec<MapPlace>), Refused> {
    let mut anchors = 0u32;
    let mut map = Vec::new();
    let n = faces.len() as u32;
    rc2k_core::race::best_line::map_stretches(
        |from| {
            let trap = Trap::new();
            let anchor = rc2k_core::race::checkpoints::next_map_anchor(from, n, |c| {
                trap.keep_read(crate::race::course_places::face_number(
                    race.course.reads(),
                    faces[c as usize],
                ))
                .map_place()
                    != (0, 0)
            });
            trap.finish()?;
            if anchor.is_some() {
                anchors = anchors.wrapping_add(1);
            }
            Ok::<_, Refused>(anchor)
        },
        |from, to| map_segment(race, faces, from, to, left, &mut map),
    )?;
    Ok((anchors, map))
}

/// Each checkpoint's count of checks.
fn count_checks(race: &Race, faces: &[u32]) -> Result<Vec<u32>, Refused> {
    let trap = Trap::new();
    let counts = rc2k_core::race::checkpoints::check_counts(faces.len() as u32, |k| {
        trap.keep_read(crate::race::course_places::face_number(
            race.course.reads(),
            faces[k as usize],
        ))
        .check()
    });
    trap.finish()?;
    Ok(counts)
}

/// The faces of one replay camera set's kind and each checkpoint's nearest of them.
fn replay_cameras(
    race: &Race,
    faces: &[u32],
    kind: u8,
    left: &RefCell<Leftovers>,
) -> Result<CameraSet, Refused> {
    let course = &race.course;
    let total: u32 = course.course.borrow().course_face_count;
    if total == 0 || total >= MOST_FACES {
        return Err(BadInput("no faces, or 0x100000 or more").into());
    }
    let trap = Trap::new();
    // The last face read whole proves every face before it lies inside the table; the scan then
    // reads each face's kind and object bytes alone.
    trap.keep_read(crate::race::course_places::face_number(
        course.reads(),
        total - 1,
    ));
    trap.finish()?;
    let (cameras, _) =
        rc2k_core::race::best_line::camera_faces(total, |b| {
            let [first, object] = trap.keep_read(
                crate::race::course_places::face_field::<[u8; 2]>(course.reads(), b, 0),
            );
            first == rc2k_core::race::faces::LIT_KIND && object == kind
        });
    trap.finish()?;
    let count = cameras.len() as u32;
    if count == 0 {
        return Ok(CameraSet::default());
    }
    // Each camera's first point, read once for all the checkpoints.
    let mut camera_points: Vec<[u32; 2]> = Vec::with_capacity(cameras.len());
    for &camera in &cameras {
        camera_points.push(point_xz(race, point_of(race, camera)?)?);
    }
    let mut nearest_cameras = Vec::with_capacity(faces.len());
    for &face in faces {
        let from = point_xz(race, point_of(race, face)?)?;
        let mut last_distance = 0;
        let nearest = rc2k_core::race::best_line::nearest_camera(count, |e| {
            let [x, z] = camera_points[e as usize];
            let dz = z.wrapping_sub(from[1]);
            let d = x87::int_hypot(x.wrapping_sub(from[0]) as i32, dz as i32) as u32;
            last_distance = d;
            d
        });
        let mut left = left.borrow_mut();
        left.scratch = Some(last_distance);
        nearest_cameras.push(nearest.1);
    }
    Ok(CameraSet {
        faces: cameras,
        nearest: nearest_cameras,
    })
}

/// Adds to surface `surface & 0xF`'s share: `weight` / 256 * 0.25 / `count` * 100 (nothing for a
/// weight of 0). Done with the exact x87 arithmetic because the shares stay floats while they add
/// and are made whole numbers last.
fn add_surface_share(
    race: &Race,
    shares: &[PlainFloat; 16],
    count: i32,
    weight: u32,
    surface: u32,
) {
    if weight == 0 {
        return;
    }
    let numbers = race.course.numbers;
    let [a, b, c] = [
        numbers.k256.get(),
        numbers.quarter.get(),
        numbers.hundred.get(),
    ];
    let share = &shares[(surface & 0xF) as usize];
    share.st(
        bh::int(weight as i32) / bh::ld(a) * bh::ld(b) / bh::int(count) * bh::ld(c) + share.ld(),
    );
}

/// The share of the line on each surface (whole numbers): for each checkpoint whose face links
/// another face, that face's two textures add their four surfaces, the second weighted by the
/// face's corner blend bytes and the first by what is left of 0x100.
fn surface_shares(race: &Race, faces: &[u32]) -> Result<[u32; 16], Refused> {
    let course = &race.course;
    let count = faces.len() as i32;
    let shares: [PlainFloat; 16] = std::array::from_fn(|_| PlainFloat::new(0.0));
    let face_count: u32 = course.course.borrow().course_face_count;
    let info_byte = |info: u32, offset: u32| -> Result<u32, BadInput> {
        Ok(crate::scene::polygon::texture_info_byte(race.polygon_tables(), info, offset)?.into())
    };
    let add_four = |info: u32, weight: u32| -> Result<(), BadInput> {
        for k in 0..4 {
            add_surface_share(race, &shares, count, weight, info_byte(info, 0x10 + k)?);
        }
        Ok(())
    };
    for &checkpoint in faces {
        let named: u32 = crate::race::course_places::face_field(course.reads(), checkpoint, 0x3C)?;
        if named >= face_count {
            continue;
        }
        let face_byte = |offset| -> Result<u32, BadInput> {
            Ok(u32::from(crate::race::course_places::face_field::<u8>(
                course.reads(),
                named,
                offset,
            )?))
        };
        let blend = [
            face_byte(0x22)?,
            face_byte(0x23)?,
            face_byte(0x24)?,
            face_byte(0x25)?,
        ]
        .map(|b| b as u8);
        let second = face_byte(0x21)?;
        // `surface_kinds` is indexed by the texture info's kind byte, 0 .. 3 in every record of
        // the shipped texture info files; a byte past them is refused.
        let kind: u32 = race
            .shared
            .view
            .get()
            .surface_kinds
            .get(info_byte(second, 0x2C)? as usize)
            .ok_or(BadInput("a texture's surface kind past the four kinds"))?
            .get();
        let second_weight = rc2k_core::race::best_line::second_weight(blend, second as u8, kind);
        let first_weight = rc2k_core::race::best_line::first_weight(second_weight);
        add_four(face_byte(1)?, first_weight)?;
        add_four(second, second_weight)?;
    }
    Ok(shares.each_ref().map(|share| x87::to_int(share) as u32))
}

/// The best line of the loaded course: its checkpoints, their angles, checks, distances, map
/// places, replay cameras and surface shares. Reads the course and the stage's direction and
/// writes nothing; `step` is told before each step runs. Refused: no faces, 0x100000 or more, more
/// of the line's faces than the lists hold, or a read past a table.
pub fn best_line(race: &Race, step: &mut dyn FnMut(Step)) -> Result<BestLine, Refused> {
    let left = RefCell::new(Leftovers::default());
    step(Step::Sort);
    let line_face_kind =
        rc2k_core::race::checkpoints::line_face_kind(race.shared.work.get().stage_direction.get());
    let (faces, chained) = sort_checkpoints(race, line_face_kind as u8)?;
    let count = chained.unwrap_or(0);
    if count < 2 {
        return Ok(BestLine {
            line_face_kind,
            faces,
            checkpoint_count: count,
            last_checkpoint: chained.map(|length| length.wrapping_sub(1)),
            along: None,
            leftovers: left.into_inner(),
        });
    }
    let chain = &faces[..count as usize];
    step(Step::Angles);
    let angles = checkpoint_angles(race, chain, &left)?;
    step(Step::Checks);
    let check_counts = count_checks(race, chain)?;
    step(Step::Speeds);
    step(Step::Distances);
    let distances = checkpoint_distances(race, chain, &left)?;
    step(Step::Map);
    let (map_anchors, map) = map_checkpoints(race, chain, &left)?;
    step(Step::CamerasOne);
    let cameras_one = replay_cameras(race, chain, 0x3C, &left)?;
    step(Step::CamerasTwo);
    let cameras_two = replay_cameras(race, chain, 0x38, &left)?;
    step(Step::SurfaceShares);
    let surface_shares = surface_shares(race, chain)?;
    Ok(BestLine {
        line_face_kind,
        checkpoint_count: count,
        last_checkpoint: Some(count - 1),
        along: Some(Along {
            angles,
            check_counts,
            distances,
            map_anchors,
            map,
            cameras: [cameras_one, cameras_two],
            surface_shares,
        }),
        faces,
        leftovers: left.into_inner(),
    })
}

/// Entry `n` of a checkpoint table set to `value`; refused past the table's end.
fn write_n(table: &[PlainDword], n: u32, value: u32) -> Result<(), BadInput> {
    table
        .get(n as usize)
        .ok_or(BadInput("a checkpoint number past its table"))?
        .set(value);
    Ok(())
}

/// Writes the best line into the race's tables.
fn store_best_line(
    race: &Race,
    scene: &crate::scene::owner::Scene,
    line: &BestLine,
) -> Result<(), Refused> {
    let vars = race.shared.vars.get();
    let tables = race.checkpoints();
    vars.line_face_kind.set(line.line_face_kind);
    for (k, &face) in line.faces.iter().enumerate() {
        write_n(&tables.checkpoint_faces, k as u32, face)?;
    }
    vars.line_face_total.set(line.checkpoint_count);
    vars.checkpoint_count.set(line.checkpoint_count);
    if let Some(last) = line.last_checkpoint {
        vars.last_checkpoint.set(last);
    }
    if let Some(along) = &line.along {
        for &(place, angle) in &along.angles {
            write_n(&tables.angles, place, angle)?;
        }
        for (k, &count) in along.check_counts.iter().enumerate() {
            write_n(&tables.check_counts, k as u32, count)?;
        }
        for (k, &distance) in along.distances.iter().enumerate() {
            write_n(&tables.distances, k as u32, distance)?;
        }
        tables
            .line_length
            .set(along.distances[along.distances.len() - 1]);
        vars.map_anchor_count.set(along.map_anchors);
        for place in &along.map {
            write_n(&tables.map_x, place.checkpoint, place.x)?;
            write_n(&tables.map_y, place.checkpoint, place.y)?;
        }
        for (set, cameras) in cameras().iter().zip(&along.cameras) {
            for (e, &face) in cameras.faces.iter().enumerate() {
                write_n((set.list)(race), e as u32, face)?;
            }
            (set.count)(&race.shared).set(cameras.faces.len() as u32);
            for (c, &nearest) in cameras.nearest.iter().enumerate() {
                write_n((set.nearest)(race), c as u32, nearest)?;
            }
        }
        for (cell, &share) in vars.surface_shares.iter().zip(&along.surface_shares) {
            cell.set(share);
        }
    }
    let left = &line.leftovers;
    if let Some(scratch) = left.scratch {
        scene.draw_work_mut().scratch = scratch;
    }
    Ok(())
}

/// Where a replay camera set's tables are.
#[derive(Clone, Copy)]
struct Cameras {
    list: for<'a> fn(&'a Race) -> &'a [PlainDword],
    count: for<'a> fn(&'a RaceShared) -> &'a PlainDword,
    nearest: for<'a> fn(&'a Race) -> &'a [PlainDword],
}

/// The two sets, "replaycam1" and "replaycam2" in the log.
fn cameras() -> [Cameras; 2] {
    [
        Cameras {
            list: |race| &race.checkpoints().replay_cameras_1,
            count: |shared| &shared.vars.get().replay_camera_counts[0],
            nearest: |race| &race.checkpoints().nearest_camera_1,
        },
        Cameras {
            list: |race| &race.checkpoints().replay_cameras_2,
            count: |shared| &shared.vars.get().replay_camera_counts[1],
            nearest: |race| &race.checkpoints().nearest_camera_2,
        },
    ]
}

const RULE: &std::ffi::CStr = c"---------------------------------";

/// Builds the stage's best line, stores it and keeps it for `store_loaded_best_line`, logging as
/// it goes: a rule, each step's word, "best line completed", the summary and a rule.
/// `line_place` and `line_fraction` end at 0.
pub(super) fn build_best_line(
    race: &Race,
    scene: &crate::scene::owner::Scene,
) -> Result<(), Refused> {
    use crate::logfile::log_int::log_int;
    let text =
        |at: &std::ffi::CStr, newline: bool| crate::logfile::log_write::log_nul(&at, newline, 1);
    text(RULE, true);
    let line = best_line(race, &mut |step| {
        crate::logfile::load_times::mark(step.mark());
        if let Some(word) = step.word() {
            text(word, false);
        }
    })?;
    crate::logfile::load_times::mark("best_line store");
    store_best_line(race, scene, &line)?;
    *race
        .course
        .stage
        .best_line
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(line);
    crate::logfile::load_times::mark("best_line summary");
    text(c"best line completed", true);
    let summary = [
        (c"dir", race.shared.work.get().stage_direction.get()),
        (c": numblo", race.shared.vars.get().line_face_total.get()),
        (c"numsorted", race.shared.vars.get().checkpoint_count.get()),
        (c"numsortedm1", race.shared.vars.get().last_checkpoint.get()),
    ];
    for (n, (at, value)) in summary.into_iter().enumerate() {
        text(at, false);
        log_int(value as i32, n == 3, 1);
    }
    text(RULE, true);
    reset_line_lights(scene);
    Ok(())
}

/// The race set-up's best line: the line the stage's load worked out is stored again (the course
/// has not changed since, so working it out again would give the same line), logging nothing. A
/// line not worked out yet is built as `build_best_line` does.
pub fn store_loaded_best_line(
    race: &Race,
    scene: &crate::scene::owner::Scene,
) -> Result<(), Refused> {
    let kept = race
        .course
        .stage
        .best_line
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .take();
    let Some(line) = kept else {
        return build_best_line(race, scene);
    };
    store_best_line(race, scene, &line)?;
    reset_line_lights(scene);
    Ok(())
}

/// `line_place` and `line_fraction` end at 0 after a line is built.
fn reset_line_lights(scene: &crate::scene::owner::Scene) {
    scene.line_and_shake_mut().line_place = 0;
    scene.line_and_shake_mut().line_fraction = 0;
}

/// Runs `check` with a synthetic course held: `faces` and `points` (x, y, z words), the stage's
/// counts and layout set, everything put back after.
#[cfg(test)]
pub(super) fn with_course<R>(
    faces: &[[u8; 64]],
    points: &[[i32; 3]],
    check: impl FnOnce(&Race) -> R,
) -> R {
    let _lock = crate::test_support::test_lock();
    let race = crate::test_support::own::race();
    let face_bytes: Vec<u8> = faces.concat();
    let point_bytes: Vec<u8> = points
        .iter()
        .flatten()
        .flat_map(|v| v.to_le_bytes())
        .collect();
    let data = &race.course.stage.course;
    crate::race::course_places::take(data, &face_bytes, &[]).unwrap();
    crate::race::scene_points::take(data, &point_bytes).unwrap();
    let before = {
        let counts = race.course.course.borrow();
        (counts.course_face_count, counts.scene_point_count)
    };
    {
        let mut counts = race.course.course.borrow_mut();
        counts.course_face_count = faces.len() as u32;
        counts.scene_point_count = points.len() as u32;
    }
    let result = check(race);
    {
        let mut counts = race.course.course.borrow_mut();
        counts.course_face_count = before.0;
        counts.scene_point_count = before.1;
    }
    crate::race::course_places::give_back(data);
    crate::race::scene_points::give_back(data);
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A line face of the forward direction: its check byte, next face and first point.
    fn line_face(check: u8, next: u32, point: u32) -> [u8; 64] {
        let mut face = [0u8; 64];
        face[0] = 0xFF;
        face[1] = 0x3E;
        face[4..8].copy_from_slice(&point.to_le_bytes());
        face[0xC..0x10].copy_from_slice(&next.to_le_bytes());
        face[0x26] = check;
        face
    }

    #[test]
    fn three_checkpoints_are_chained_and_measured() {
        // Faces 1 (the start), 2 and 0 chained in that order, 100 apart along x, then z.
        let faces = [
            line_face(0, u32::MAX, 2),
            line_face(1, 2, 0),
            line_face(0, 0, 1),
        ];
        let points = [[0, 0, 0], [100, 0, 0], [100, 0, 100]];
        let line = with_course(&faces, &points, |race| {
            race.shared.work.get().stage_direction.set(0u32);
            best_line(race, &mut |_| {}).unwrap()
        });
        assert_eq!(line.line_face_kind, 0x3E);
        assert_eq!(line.faces, [1, 2, 0]);
        assert_eq!((line.checkpoint_count, line.last_checkpoint), (3, Some(2)));
        let along = line.along.unwrap();
        assert_eq!(along.distances, [0, 100, 200]);
        assert_eq!(along.check_counts, [0, 0, 0]);
        let angles: Vec<u32> = along.angles.iter().map(|&(place, _)| place).collect();
        assert_eq!(angles, [2, 3, 4, 1, 0]);
        assert_eq!(along.angles[3].1, along.angles[0].1);
        assert_eq!(along.angles[4].1, along.angles[0].1);
        assert_eq!(along.angles[2].1, along.angles[1].1);
        assert_eq!((along.map_anchors, along.map.len()), (0, 0));
        assert!(along.cameras.iter().all(|set| set.faces.is_empty()));
        // Every checkpoint's face names face 0, whose textures have the surface 0.
        assert_eq!(along.surface_shares[0], 100);
        assert_eq!(line.leftovers.scratch, Some(100));
    }

    #[test]
    fn fewer_than_two_line_faces_make_no_line() {
        let faces = [line_face(1, 0, 0), [0u8; 64]];
        let line = with_course(&faces, &[[0; 3]], |race| {
            race.shared.work.get().stage_direction.set(0u32);
            best_line(race, &mut |step| assert!(matches!(step, Step::Sort))).unwrap()
        });
        assert_eq!(line.faces, [0]);
        assert_eq!((line.checkpoint_count, line.last_checkpoint), (0, None));
        assert!(line.along.is_none());
    }
}
