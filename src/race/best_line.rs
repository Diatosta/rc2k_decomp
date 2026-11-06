//! `race::loadlevel_init`'s locacurrcourse, its "best line" (`sub_445772`, IDA
//! 2026-10-16): the checkpoint faces (first byte 0xFF, kind `g_lineFaceKind`) listed in
//! `g_checkpointFaces` (0x9E3738) and chained in order, `g_checkpointCount` of them
//! (0x623640, "numsorted"); then their angles, checks, distances, the map, the replay
//! cameras and the surface shares. Here the small steps.

use crate::errors::{BadInput, Refused};
use crate::math::x87;
use crate::race::checkpoints_store::CHECKPOINTS;
use crate::race::owner::race;
use crate::race::owner_course::Course;
use crate::race::trap::Trap;
use crate::scene::owner::scene;
use crate::session::Dword;

/// The entries `g_checkpointFaces` and the lists after it have room for.
pub const MOST_CHECKS: u32 = rc2k_core::race::checkpoints::MOST_CHECKS;

/// Field `offset` of course face `face`.
fn face_field<T: crate::race::course_places::Whole>(face: u32, offset: u32) -> Result<T, BadInput> {
    crate::race::course_places::face_field(face, offset, "a course face")
}

/// Checkpoint n's face (its number).
fn check_face(n: u32) -> u32 {
    crate::race::checkpoints_store::face(n as usize).get()
}

fn count() -> Result<u32, Refused> {
    let n: u32 = race().shared.vars.checkpoint_count.bits();
    if n > MOST_CHECKS {
        return Err(BadInput("more checkpoints than the lists hold").into());
    }
    Ok(n)
}

/// `race::set_line_face_kind` (0x44574A): `g_lineFaceKind` 0x3E with `g_stageDirection`
/// 0, 0x3D with 1, else 0.
pub fn set_line_face_kind() {
    let shared = &race().shared;
    // Converted: `rc2k_core::race::checkpoints::line_face_kind`.
    let kind = rc2k_core::race::checkpoints::line_face_kind(shared.work.stage_direction.bits());
    shared.vars.line_face_kind.set_bits(kind);
}

/// `race::add_surface_share` (0x445F7B): with `g_surfaceWeight` not 0, surface `surface`
/// & 0xF's share gets weight / 256 * 0.25 / `g_checkpointCount` * 100 more (on the x87,
/// the count as a whole number). The exact x87 version (for `race::surface_shares`, whose
/// whole numbers are rounded from the shares).
pub fn add_surface_share(course: &Course, surface: u32) {
    let shared = &race().shared;
    let weight = shared.vars.surface_weight.bits() as i32;
    if weight == 0 {
        return;
    }
    // g_surfaceShares: 16 floats
    let share = &shared.vars.surface_shares[(surface & 0xF) as usize];
    // 256, 0.25, 100
    let [a, b, c] = [
        course.numbers.k256.bits(),
        course.numbers.quarter.bits(),
        course.numbers.hundred.bits(),
    ];
    let count = shared.vars.checkpoint_count.bits() as i32;
    share.set_bits(x87::surface_share(weight, a, b, count, c, share.bits()));
}

/// `race::next_map_anchor` (0x445FEE): from checkpoint `from` on, the first whose face has
/// +0x2C (the map's x and y words) not 0: `g_mapAnchorCount` + 1, and it; None at
/// `g_checkpointCount` (the original's carry). Refuses more checkpoints than the lists hold.
pub fn next_map_anchor(from: u32) -> Result<Option<u32>, Refused> {
    let shared = &race().shared;
    // Converted: `rc2k_core::race::checkpoints::next_map_anchor`.
    let n = count()?;
    let trap = Trap::new();
    let anchor = rc2k_core::race::checkpoints::next_map_anchor(from, n, |c| {
        trap.keep_read(crate::race::course_places::face_number(
            check_face(c),
            "a checkpoint face",
        ))
        .map_place()
            != (0, 0)
    });
    trap.finish()?;
    if anchor.is_some() {
        shared
            .vars
            .map_anchor_count
            .set_bits(shared.vars.map_anchor_count.bits().wrapping_add(1));
    }
    Ok(anchor)
}

/// `race::count_checks` (0x445AAB): with two checkpoints or more, `g_checkCounts`[n] the
/// checkpoints up to n whose face's byte +0x26 is neither 0 nor 1. Refuses more
/// checkpoints than the lists hold.
pub fn count_checks() -> Result<(), Refused> {
    // Converted: `rc2k_core::race::checkpoints::check_counts`.
    let n = count()?;
    if n < 2 {
        return Ok(());
    }
    let trap = Trap::new();
    let counts = rc2k_core::race::checkpoints::check_counts(n, |k| {
        trap.keep_read(crate::race::course_places::face_number(
            check_face(k),
            "a checkpoint face",
        ))
        .check()
    });
    trap.finish()?;
    for (k, &c) in counts.iter().enumerate() {
        crate::race::checkpoints_store::cell_in(&CHECKPOINTS.check_counts, k).set(c);
    }
    Ok(())
}

const MOST_FACES: u32 = 0x10_0000;

/// A replay camera's set: the face kind, the list (0x400) of such faces, its count, and
/// each checkpoint's nearest.
#[derive(Clone, Copy)]
pub struct Cameras {
    pub kind: u8,
    pub list: fn() -> &'static Dword,
    pub count: fn() -> &'static Dword,
    pub nearest: fn() -> &'static Dword,
}

/// `sub_445BB2` ("replaycam1") and `sub_445CF1` ("replaycam2").
pub fn cameras() -> [Cameras; 2] {
    [
        Cameras {
            kind: 0x3C,
            list: || &CHECKPOINTS.replay_cameras_1[0],
            count: || &crate::race::owner::race().shared.vars.replay_camera_counts[0],
            nearest: || &CHECKPOINTS.nearest_camera_1[0],
        },
        Cameras {
            kind: 0x38,
            list: || &CHECKPOINTS.replay_cameras_2[0],
            count: || &crate::race::owner::race().shared.vars.replay_camera_counts[1],
            nearest: || &CHECKPOINTS.nearest_camera_2[0],
        },
    ]
}

/// The map work cell at `o` (the original's offsets, from 0xB8).
fn work(o: u32) -> &'static Dword {
    &race().shared.vars.map_work[((o - 0xB8) / 4) as usize]
}

fn read_n(table: &Dword, n: u32) -> u32 {
    crate::race::checkpoints_store::nth(table, n as usize).get()
}

fn write_n(table: &Dword, n: u32, value: u32) {
    crate::race::checkpoints_store::nth(table, n as usize).set(value);
}

/// A face's first point (+4): its number.
fn point_of(face: u32) -> Result<u32, BadInput> {
    face_field(face, 4)
}

fn face_count(course: &Course) -> Result<u32, Refused> {
    let n: u32 = course.course.course_face_count.bits();
    if n == 0 || n >= MOST_FACES {
        return Err(BadInput("no faces, or 0x100000 or more: the original counts round").into());
    }
    Ok(n)
}

/// `race::sort_checkpoints` (0x4458A7): `set_line_face_kind`; the faces of that kind
/// (first byte 0xFF) listed in `g_checkpointFaces`, `g_lineFaceTotal` of them; with two or
/// more, the first whose byte +0x26 is 1 put first (none: no checkpoints), then each next
/// the one the last one's +0xC names (searched over the whole list, swapped into place)
/// until none is found; `g_checkpointCount` and `g_lineFaceTotal` the chain's length,
/// `g_lastCheckpoint` it - 1. With fewer than two, both counts 0. Refuses no faces or
/// 0x100000 or more, and more of the kind than the list holds.
pub fn sort_checkpoints(course: &Course) -> Result<(), Refused> {
    let shared = &race().shared;
    // Converted: `rc2k_core::race::checkpoints::{line_face_kind, chain}`; the faces of the
    // kind listed here as the original lists them.
    use rc2k_core::race::checkpoints::{chain, line_face_kind};
    {
        let n = face_count(course)?;
        let kind = line_face_kind(shared.work.stage_direction.bits());
        let of_kind = |b: u32, kind: u8| -> Result<bool, BadInput> {
            Ok(face_field::<u8>(b, 0)? == 0xFF && face_field::<u8>(b, 1)? == kind)
        };
        let mut found = 0u32;
        for b in 0..n {
            if of_kind(b, kind as u8)? {
                found += 1;
            }
        }
        if found > MOST_CHECKS {
            return Err(BadInput("more line faces than the checkpoint list holds").into());
        }
        set_line_face_kind();
        let mut list = Vec::new();
        for b in 0..n {
            if of_kind(b, shared.vars.line_face_kind.bits() as u8)? {
                write_n(&CHECKPOINTS.checkpoint_faces[0], list.len() as u32, b);
                list.push(b);
            }
        }
        if list.len() < 2 {
            shared.vars.line_face_total.set_bits(0u32);
            shared.vars.checkpoint_count.set_bits(0u32);
            return Ok(());
        }
        let trap = Trap::new();
        let c = chain(
            &mut list,
            |f| {
                trap.keep_read(crate::race::course_places::face_number(
                    f,
                    "a checkpoint face",
                ))
                .check()
                    == 1
            },
            |f| {
                trap.keep_read(crate::race::course_places::face_number(
                    f,
                    "a checkpoint face",
                ))
                .next_link()
            },
        );
        trap.finish()?;
        for (k, &f) in list.iter().enumerate() {
            write_n(&CHECKPOINTS.checkpoint_faces[0], k as u32, f);
        }
        shared.vars.checkpoint_count.set_bits(c.length);
        shared.vars.line_face_total.set_bits(c.length);
        shared
            .vars
            .last_checkpoint
            .set_bits(c.length.wrapping_sub(1));
        Ok(())
    }
}

/// `race::checkpoint_angles` (0x4459DF): with two checkpoints or more, each link's angle
/// (x and z from the last checkpoint's first point to this one's: the x87's `fpatan` over
/// pi, times 0.5, 65536 and 65536) into `g_checkpointAngles` + 1 .. n, n + 1 the same as n; then 0
/// and 1 as 2.
pub fn checkpoint_angles(course: &Course) -> Result<(), Refused> {
    // Converted: `rc2k_core::race::best_line::link_angles`; each link's angle the x87's.
    let n = count()?;
    let trap = Trap::new();
    {
        let scales = (n >= 2).then(|| (course.numbers.half.bits(), course.numbers.k65536.bits()));
        let (writes, _) = rc2k_core::race::best_line::link_angles(
            n,
            |b| {
                let (half, scale) = scales.unwrap();
                let Ok(p) = point_of(check_face(b - 1)).inspect_err(|e| trap.keep(*e)) else {
                    return 0;
                };
                let Ok(q) = point_of(check_face(b)).inspect_err(|e| trap.keep(*e)) else {
                    return 0;
                };
                let w = |n: u32, k: u32| match crate::race::scene_points::point_word(n, k) {
                    Ok(v) => v,
                    Err(e) => {
                        trap.keep(e);
                        0
                    }
                };
                let dx = w(q, 0).wrapping_sub(w(p, 0)) as i32;
                let dz = w(q, 2).wrapping_sub(w(p, 2)) as i32;
                scene().drawing.scene.scratch.set_bits(dz as u32);
                x87::angle_scaled3_to_int(dx, dz, half, scale, scale) as u32
            },
            || read_n(&CHECKPOINTS.angles[0], 2),
        );
        for (k, angle) in writes {
            write_n(&CHECKPOINTS.angles[0], k, angle);
        }
    }
    trap.finish()?;
    Ok(())
}

/// `race::checkpoint_distances` (0x445AF1): with two checkpoints or more, the distance
/// along the line (x and z of their first points, each link's length rounded) at each of
/// 0 .. `g_lastCheckpoint`; the whole at `g_lastCheckpoint` and in `g_lineLength`. Refuses
/// a last checkpoint of 0 or past the lists (the original counts round), and fewer than
/// two checkpoints: the original then writes 0 at its caller's ecx (section 16), which
/// its one caller, `build_best_line`, never reaches (it calls with two line faces or more,
/// and `sort_checkpoints` makes the checkpoint count the same).
pub fn checkpoint_distances() -> Result<(), Refused> {
    let shared = &race().shared;
    let n = count()?;
    if n < 2 {
        return Err(BadInput("fewer than two checkpoints: the original writes at ecx").into());
    }
    let trap = Trap::new();
    {
        let last: u32 = shared.vars.last_checkpoint.bits();
        if last == 0 || last >= MOST_CHECKS {
            return Err(BadInput("a last checkpoint of 0 or past the lists").into());
        }
        // Converted: `rc2k_core::race::best_line::distances`.
        let sums = rc2k_core::race::best_line::distances(last, |k| {
            let a = match point_of(check_face(k + 1)) {
                Ok(a) => a,
                Err(e) => {
                    trap.keep(e);
                    return 0;
                }
            };
            let b = match point_of(check_face(k)) {
                Ok(b) => b,
                Err(e) => {
                    trap.keep(e);
                    return 0;
                }
            };
            let w = |n: u32, k: u32| match crate::race::scene_points::point_word(n, k) {
                Ok(v) => v,
                Err(e) => {
                    trap.keep(e);
                    0
                }
            };
            let d = [w(a, 0).wrapping_sub(w(b, 0)), w(a, 2).wrapping_sub(w(b, 2))];
            crate::session::write_bits(&shared.vars.line_from, d);
            let length = x87::int_hypot(d[0] as i32, d[1] as i32) as u32;
            scene().drawing.scene.scratch.set_bits(length);
            length
        });
        for (k, &sum) in sums[..last as usize].iter().enumerate() {
            write_n(&CHECKPOINTS.distances[0], k as u32, sum);
        }
        write_n(&CHECKPOINTS.distances[0], last, sums[last as usize]);
        CHECKPOINTS.line_length.set(sums[last as usize]);
    }
    trap.finish()?;
    Ok(())
}

/// `race::replay_cameras` (0x445BB2, 0x445CF1): with two checkpoints or more, the faces
/// of the set's kind (first byte 0xFF) listed (0x400 at most); with any, each
/// checkpoint's nearest of them (x and z of their first points, the length rounded; the
/// last of the nearest when two are as near). Refuses no faces or 0x100000 or more.
pub fn replay_cameras(course: &Course, set: &Cameras) -> Result<(), Refused> {
    let shared = &race().shared;
    let n = count()?;
    if n < 2 {
        return Ok(());
    }
    let faces = face_count(course)?;
    let trap = Trap::new();
    {
        // Converted: `rc2k_core::race::best_line::{camera_faces, nearest_camera}`.
        let (cameras, _) = rc2k_core::race::best_line::camera_faces(faces, |b| {
            let f = trap.keep_read(crate::race::course_places::face_number(b, "a camera face"));
            f.kind() == rc2k_core::race::faces::LIT_KIND && f.object() == set.kind
        });
        trap.finish()?;
        for (e, &f) in cameras.iter().enumerate() {
            write_n((set.list)(), e as u32, f);
        }
        let m = cameras.len() as u32;
        (set.count)().set(m);
        if m == 0 {
            return Ok(());
        }
        for c in 0..n {
            let p = point_of(check_face(c))?;
            let from = [
                crate::race::scene_points::point_word(p, 0)?,
                crate::race::scene_points::point_word(p, 2)?,
            ];
            crate::session::write_bits(&shared.vars.line_from, from);
            let nearest = rc2k_core::race::best_line::nearest_camera(m, |e| {
                let q = match point_of(read_n((set.list)(), e)) {
                    Ok(q) => q,
                    Err(err) => {
                        trap.keep(err);
                        return 0;
                    }
                };
                let w = |n: u32, k: u32| match crate::race::scene_points::point_word(n, k) {
                    Ok(v) => v,
                    Err(e) => {
                        trap.keep(e);
                        0
                    }
                };
                let dz = w(q, 2).wrapping_sub(from[1]);
                let d = x87::int_hypot(w(q, 0).wrapping_sub(from[0]) as i32, dz as i32) as u32;
                scene().drawing.scene.scratch.set_bits(d);
                d
            });
            trap.finish()?;
            crate::session::write_bits(&shared.vars.line_nearest, [nearest.0, nearest.1]);
            write_n((set.nearest)(), c, nearest.1);
        }
    }
    Ok(())
}

/// `race::surface_shares` (0x445E30): with two checkpoints or more, `g_surfaceShares` 0;
/// for each checkpoint whose face's +0x3C names a face, that face's two textures (+1 and
/// +0x21, each 0x40 bytes of `g_textureInfos`) add their four surfaces (+0x10 .. +0x13,
/// `add_surface_share`): +0x21's weighted by the face's bytes +0x22 .. +0x25 (a quarter of
/// their sum; 0 without a +0x21, or with its kind (+0x2C, by `dword_63D9D0`) 4 or 5),
/// +1's by 0x100 less; then the shares made whole numbers.
pub fn surface_shares(course: &Course) -> Result<(), Refused> {
    let shared = &race().shared;
    let n = count()?;
    if n < 2 {
        return Ok(());
    }
    {
        for share in &shared.vars.surface_shares {
            share.set_bits(0u32);
        }
        let faces: u32 = course.course.course_face_count.bits();
        let byte = |info: u32, offset: u32| -> Result<u32, BadInput> {
            Ok(crate::scene::polygon::texture_info_byte(info, offset)?.into())
        };
        let four = |info: u32| -> Result<(), BadInput> {
            for k in 0..4 {
                add_surface_share(course, byte(info, 0x10 + k)?);
            }
            Ok(())
        };
        for e in 0..n {
            let named: u32 = face_field(check_face(e), 0x3C)?;
            if named >= faces {
                continue;
            }
            let f = named;
            let face_byte =
                |offset| -> Result<u32, BadInput> { Ok(u32::from(face_field::<u8>(f, offset)?)) };
            // Converted: `rc2k_core::race::best_line::{second_weight, first_weight}`.
            let blend = [
                face_byte(0x22)?,
                face_byte(0x23)?,
                face_byte(0x24)?,
                face_byte(0x25)?,
            ]
            .map(|b| b as u8);
            let second = face_byte(0x21)?;
            // dword_63D9D0: indexed by a byte of the texture info (not bounded by the
            // field's 4 dwords), so read by address.
            let kind: u32 = crate::session::read_on_in(
                &shared.view,
                &shared.view.surface_kinds,
                byte(second, 0x2C)? as usize,
                "the surface kinds table",
            )?;
            let weight = rc2k_core::race::best_line::second_weight(blend, second as u8, kind);
            shared
                .vars
                .surface_weight
                .set_bits(rc2k_core::race::best_line::first_weight(weight));
            four(face_byte(1)?)?;
            shared
                .vars
                .surface_weight
                .set_bits(rc2k_core::race::best_line::first_weight(
                    shared.vars.surface_weight.bits(),
                ));
            four(second)?;
        }
        for c in 0..16 {
            let share = &shared.vars.surface_shares[c];
            share.set_bits(x87::to_int(share.bits()) as u32);
        }
    }
    Ok(())
}

/// `race::map_segment` (0x446036): the checkpoints from `dword_6236C4` to `dword_6236C0`
/// (both included) placed on the map: the two anchors' map places (their faces' words +0x2C
/// and +0x2E, << 14, y negated) and first points; the stretch's scale (map length over
/// ground length, a float) and its two angles (`fpatan` / pi * 0.5^3 * 65536^2, << 2); each
/// checkpoint's point from the first anchor turned by minus the ground angle
/// (`math::turn_point_xz`), scaled, turned by the map angle, moved to the first anchor's
/// place: `g_mapX`, `g_mapY` (negated). Leaves ebx past the last, eax and ecx the last's
/// map x and minus its y. Refuses anchors past the lists.
pub fn map_segment(course: &Course) -> Result<(), Refused> {
    use crate::math::turn_point_xz::turn_point_xz;
    {
        let (a, b) = (work(0xC4).get(), work(0xC0).get());
        if a >= MOST_CHECKS || b >= MOST_CHECKS {
            return Err(BadInput("map anchors past the lists").into());
        }
        let anchor = |n: u32| {
            let f = check_face(n);
            let (x, y) =
                crate::race::course_places::face_number(f, "a checkpoint face")?.map_place();
            let place = [(x as u32) << 14, ((y as u32) << 14).wrapping_neg()];
            let p = point_of(f)?;
            Ok::<_, BadInput>((
                place,
                [
                    crate::race::scene_points::point_word(p, 0)?,
                    crate::race::scene_points::point_word(p, 2)?,
                ],
            ))
        };
        let (place_a, point_a) = anchor(a)?;
        work(0xEC).set(place_a[0]);
        work(0xF0).set(place_a[1]);
        work(0xD4).set(point_a[0]);
        work(0xD8).set(point_a[1]);
        let (place_b, point_b) = anchor(b)?;
        let ground = [
            point_b[0].wrapping_sub(point_a[0]),
            point_b[1].wrapping_sub(point_a[1]),
        ];
        let map = [
            place_b[0].wrapping_sub(place_a[0]),
            place_b[1].wrapping_sub(place_a[1]),
        ];
        work(0xF4).set(map[0]);
        work(0xF8).set(map[1]);
        work(0xDC).set(ground[0]);
        work(0xE0).set(ground[1]);
        let scale = x87::hypot_ratio(
            map[0] as i32,
            map[1] as i32,
            ground[0] as i32,
            ground[1] as i32,
        );
        work(0xC8).set(scale);
        let (half, unit) = (course.numbers.half.bits(), course.numbers.k65536.bits());
        let map_angle =
            (x87::angle_scaled5_to_int(map[0] as i32, map[1] as i32, half, unit) as u32) << 2;
        work(0xB8).set(map_angle);
        let ground_angle =
            (x87::angle_scaled5_to_int(ground[0] as i32, ground[1] as i32, half, unit) as u32) << 2;
        work(0xBC).set(ground_angle);
        let mut e = a;
        loop {
            // The point's number rides along as `turn_point_xz`'s y (ebx in the original).
            let number = point_of(check_face(e))?;
            let p = [
                crate::race::scene_points::point_word(number, 0)?,
                crate::race::scene_points::point_word(number, 2)?,
            ];
            work(0xCC).set(p[0]);
            work(0xD0).set(p[1]);
            let [x, _, z, _] = {
                turn_point_xz(
                    p[0].wrapping_sub(point_a[0]) as i32,
                    number as i32,
                    p[1].wrapping_sub(point_a[1]) as i32,
                    ground_angle.wrapping_neg(),
                )
            };
            let x = x87::int_times_to_int(x as i32, scale);
            let z = x87::int_times_to_int(z as i32, scale);
            scene().drawing.scene.scratch.set_bits(z as u32);
            let [x, _, z, _] = turn_point_xz(x, number as i32, z, map_angle);
            write_n(&CHECKPOINTS.map_x[0], e, x.wrapping_add(place_a[0]));
            write_n(
                &CHECKPOINTS.map_y[0],
                e,
                z.wrapping_add(place_a[1]).wrapping_neg(),
            );
            e += 1;
            if e > b {
                break;
            }
        }
    }
    Ok(())
}

/// `race::map_checkpoints` (0x445FB4): `g_mapAnchorCount` 0; with two checkpoints or more,
/// from checkpoint 0, each stretch to the next anchor (`next_map_anchor`) mapped
/// (`map_segment`), until none is left. Refuses more checkpoints than the lists hold.
pub fn map_checkpoints(course: &Course) -> Result<(), Refused> {
    // Converted: `rc2k_core::race::best_line::map_stretches`.
    let n = count()?;
    {
        crate::race::owner::RACE
            .shared
            .vars
            .map_anchor_count
            .set_bits(0u32);
        if n < 2 {
            return Ok(());
        }
        rc2k_core::race::best_line::map_stretches(next_map_anchor, |from, to| {
            // The stretch's two anchors (0x6236C4, 0x6236C0 of the map work).
            crate::race::owner::RACE.shared.vars.map_work[3].set(from);
            crate::race::owner::RACE.shared.vars.map_work[2].set(to);
            map_segment(course)
        })
    }
}

const RULE_RVA: [&std::ffi::CStr; 2] = [
    c"---------------------------------",
    c"---------------------------------",
]; // "-----..."
/// "angles", "checks", "speeds", "dists", "map", "replaycam1", "replaycam2", with the
/// step each comes before.
const STEP_WORDS: [(&std::ffi::CStr, u32); 7] = [
    (c"angles", 1),
    (c"checks", 2),
    (c"speeds", 3),
    (c"dists", 4),
    (c"map", 5),
    (c"replaycam1", 6),
    (c"replaycam2", 7),
];
const COMPLETED_RVA: &std::ffi::CStr = c"best line completed"; // "best line completed"
/// "dir" `g_stageDirection`, ": numblo" `g_lineFaceTotal`, "numsorted"
/// `g_checkpointCount`, "numsortedm1" `g_lastCheckpoint` (the last with the line's end).
type Summary = [(&'static std::ffi::CStr, fn() -> u32); 4];
const SUMMARY: Summary = [
    (c"dir", || {
        crate::race::owner::RACE.shared.work.stage_direction.get()
    }),
    (c": numblo", || {
        crate::race::owner::race().shared.vars.line_face_total.get()
    }),
    (c"numsorted", || {
        crate::race::owner::race()
            .shared
            .vars
            .checkpoint_count
            .get()
    }),
    (c"numsortedm1", || {
        crate::race::owner::race().shared.vars.last_checkpoint.get()
    }),
];

/// `race::build_best_line` (0x445772): a rule in the log; `sort_checkpoints`; with two
/// line faces or more, each step with its word in the log (angles, checks, "speeds" =
/// `nullsub_7`, distances, the map, the two replay cameras, then the surface shares);
/// "best line completed", the summary ("dir", ": numblo", "numsorted", "numsortedm1"),
/// a rule; `dword_6365E4` and `dword_6365EC` 0. The registers it leaves are read by no
/// caller (conversion phase 3, 2026-10-24).
pub fn build_best_line() -> Result<(), Refused> {
    let lights = &scene().lights;
    let course = &race().course;
    use crate::logfile::log_int::log_int;
    let text =
        |at: &std::ffi::CStr, newline: bool| crate::logfile::log_write::log_nul(&at, newline, 1);
    {
        text(RULE_RVA[0], true);
        sort_checkpoints(course)?;
        if crate::race::owner::race()
            .shared
            .vars
            .line_face_total
            .bits()
            >= 2
        {
            for (word, step) in STEP_WORDS {
                text(word, false);
                match step {
                    1 => checkpoint_angles(course)?,
                    2 => count_checks()?,
                    3 => {} // `nullsub_7`
                    4 => checkpoint_distances()?,
                    5 => map_checkpoints(course)?,
                    6 => replay_cameras(course, &cameras()[0])?,
                    _ => replay_cameras(course, &cameras()[1])?,
                }
            }
            surface_shares(course)?;
        }
        text(COMPLETED_RVA, true);
        for (n, (at, value)) in SUMMARY.into_iter().enumerate() {
            text(at, false);
            log_int(value() as i32, n == 3, 1);
        }
        text(RULE_RVA[1], true);
        // dword_6365E4, dword_6365EC
        lights.state.line_place.set(0);
        lights.state.line_fraction.set(0);
    }
    Ok(())
}
