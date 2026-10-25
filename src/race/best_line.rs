//! `race::loadlevel_init`'s locacurrcourse, its "best line" (`sub_445772`, IDA
//! 2026-10-16): the checkpoint faces (first byte 0xFF, kind `g_lineFaceKind`) listed in
//! `g_checkpointFaces` (0x9E3738) and chained in order, `g_checkpointCount` of them
//! (0x623640, "numsorted"); then their angles, checks, distances, the map, the replay
//! cameras and the surface shares. Here the small steps.

use crate::errors::{BadInput, Refused};
use crate::math::x87;
use crate::race::course_store::COURSE;
use crate::race::numbers_store::NUMBERS;
use crate::race::vars_store::RACE_VARS;
use crate::race::view_store::RACE_VIEW;
use crate::race::work_store::RACE_WORK;
use crate::scene::draw_store::DRAW;
use crate::scene::light_store::LIGHT_STATE;
use crate::session::Dword;

/// The entries `g_checkpointFaces` and the lists after it have room for.
pub const MOST_CHECKS: u32 = rc2k_core::race::checkpoints::MOST_CHECKS;

unsafe fn get(at: u32) -> u32 {
    unsafe { (at as *const u32).read_unaligned() }
}

/// Checkpoint n's face.
unsafe fn check_face(n: u32) -> u32 {
    {
        let face: u32 = crate::race::checkpoints_store::face(n as usize).get();
        (face << 6).wrapping_add(COURSE.course_faces.bits())
    }
}

fn count() -> Result<u32, Refused> {
    let n: u32 = RACE_VARS.checkpoint_count.bits();
    if n > MOST_CHECKS {
        return Err(BadInput("more checkpoints than the lists hold").into());
    }
    Ok(n)
}

/// `race::set_line_face_kind` (0x44574A): `g_lineFaceKind` 0x3E with `g_stageDirection`
/// 0, 0x3D with 1, else 0.
pub unsafe fn set_line_face_kind() {
    // Converted: `rc2k_core::race::checkpoints::line_face_kind`.
    let kind = rc2k_core::race::checkpoints::line_face_kind(RACE_WORK.stage_direction.bits());
    RACE_VARS.line_face_kind.set_bits(kind);
}

/// `race::add_surface_share` (0x445F7B): with `g_surfaceWeight` not 0, surface `surface`
/// & 0xF's share gets weight / 256 * 0.25 / `g_checkpointCount` * 100 more (on the x87,
/// the count as a whole number). The exact x87 version (for `race::surface_shares`, whose
/// whole numbers are rounded from the shares).
pub unsafe fn add_surface_share(surface: u32) {
    let weight = RACE_VARS.surface_weight.bits() as i32;
    if weight == 0 {
        return;
    }
    // g_surfaceShares: 16 floats
    let share = &RACE_VARS.surface_shares[(surface & 0xF) as usize];
    // 256, 0.25, 100
    let [a, b, c] = [
        NUMBERS.k256.bits(),
        NUMBERS.quarter.bits(),
        NUMBERS.hundred.bits(),
    ];
    let count = RACE_VARS.checkpoint_count.bits() as i32;
    share.set_bits(x87::surface_share(weight, a, b, count, c, share.bits()));
}

/// `race::next_map_anchor` (0x445FEE): from checkpoint `from` on, the first whose face has
/// +0x2C (the map's x and y words) not 0: `g_mapAnchorCount` + 1, and it; None at
/// `g_checkpointCount` (the original's carry). Refuses more checkpoints than the lists hold.
pub unsafe fn next_map_anchor(from: u32) -> Result<Option<u32>, Refused> {
    // Converted: `rc2k_core::race::checkpoints::next_map_anchor`.
    let n = count()?;
    unsafe {
        let anchor = rc2k_core::race::checkpoints::next_map_anchor(from, n, |c| {
            get(check_face(c) + 0x2C) != 0
        });
        if anchor.is_some() {
            RACE_VARS
                .map_anchor_count
                .set_bits(RACE_VARS.map_anchor_count.bits().wrapping_add(1));
        }
        Ok(anchor)
    }
}

/// `race::count_checks` (0x445AAB): with two checkpoints or more, `g_checkCounts`[n] the
/// checkpoints up to n whose face's byte +0x26 is neither 0 nor 1. Refuses more
/// checkpoints than the lists hold.
pub unsafe fn count_checks() -> Result<(), Refused> {
    // Converted: `rc2k_core::race::checkpoints::check_counts`.
    let n = count()?;
    if n < 2 {
        return Ok(());
    }
    unsafe {
        let counts = rc2k_core::race::checkpoints::check_counts(n, |k| {
            *((check_face(k) + 0x26) as *const u8)
        });
        for (k, &c) in counts.iter().enumerate() {
            crate::race::checkpoints_store::nth(
                &crate::race::checkpoints_store::CHECKPOINTS.check_counts[0],
                k,
            )
            .set(c);
        }
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
            list: || &crate::race::checkpoints_store::CHECKPOINTS.replay_cameras_1[0],
            count: || &RACE_VARS.replay_camera_counts[0],
            nearest: || &crate::race::checkpoints_store::CHECKPOINTS.nearest_camera_1[0],
        },
        Cameras {
            kind: 0x38,
            list: || &crate::race::checkpoints_store::CHECKPOINTS.replay_cameras_2[0],
            count: || &RACE_VARS.replay_camera_counts[1],
            nearest: || &crate::race::checkpoints_store::CHECKPOINTS.nearest_camera_2[0],
        },
    ]
}

unsafe fn put(at: u32, value: u32) {
    unsafe { (at as *mut u32).write_unaligned(value) };
}

unsafe fn at_n(table: &Dword, n: u32) -> u32 {
    table.addr() as u32 + 4 * n
}

unsafe fn read_n(table: &Dword, n: u32) -> u32 {
    unsafe { get(at_n(table, n)) }
}

unsafe fn write_n(table: &Dword, n: u32, value: u32) {
    unsafe { put(at_n(table, n), value) };
}

unsafe fn face(n: u32) -> u32 {
    (n << 6).wrapping_add(COURSE.course_faces.bits())
}

/// A face's first point: its number (+4) and address.
unsafe fn point_of(face_address: u32) -> (u32, u32) {
    unsafe {
        let n = get(face_address + 4);
        (
            n,
            n.wrapping_mul(12).wrapping_add(COURSE.scene_points.bits()),
        )
    }
}

fn face_count() -> Result<u32, Refused> {
    let n: u32 = COURSE.course_face_count.bits();
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
pub unsafe fn sort_checkpoints() -> Result<(), Refused> {
    // Converted: `rc2k_core::race::checkpoints::{line_face_kind, chain}`; the faces of the
    // kind listed here as the original lists them.
    use rc2k_core::race::checkpoints::{chain, line_face_kind};
    unsafe {
        let n = face_count()?;
        let kind = line_face_kind(RACE_WORK.stage_direction.bits());
        let of_kind = |b: u32| {
            let f = face(b);
            *(f as *const u8) == 0xFF && *((f + 1) as *const u8) == kind as u8
        };
        if (0..n).filter(|&b| of_kind(b)).count() as u32 > MOST_CHECKS {
            return Err(BadInput("more line faces than the checkpoint list holds").into());
        }
        set_line_face_kind();
        let mut list = Vec::new();
        for b in 0..n {
            let f = face(b);
            if *(f as *const u8) == 0xFF
                && *((f + 1) as *const u8) == RACE_VARS.line_face_kind.bits() as u8
            {
                write_n(
                    &crate::race::checkpoints_store::CHECKPOINTS.checkpoint_faces[0],
                    list.len() as u32,
                    b,
                );
                list.push(b);
            }
        }
        if list.len() < 2 {
            RACE_VARS.line_face_total.set_bits(0u32);
            RACE_VARS.checkpoint_count.set_bits(0u32);
            return Ok(());
        }
        let c = chain(
            &mut list,
            |f| *((face(f) + 0x26) as *const u8) == 1,
            |f| get(face(f) + 0xC),
        );
        for (k, &f) in list.iter().enumerate() {
            write_n(
                &crate::race::checkpoints_store::CHECKPOINTS.checkpoint_faces[0],
                k as u32,
                f,
            );
        }
        RACE_VARS.checkpoint_count.set_bits(c.length);
        RACE_VARS.line_face_total.set_bits(c.length);
        RACE_VARS.last_checkpoint.set_bits(c.length.wrapping_sub(1));
        Ok(())
    }
}

/// `race::checkpoint_angles` (0x4459DF): with two checkpoints or more, each link's angle
/// (x and z from the last checkpoint's first point to this one's: the x87's `fpatan` over
/// pi, times 0.5, 65536 and 65536) into `g_checkpointAngles` + 1 .. n, n + 1 the same as n; then 0
/// and 1 as 2.
pub unsafe fn checkpoint_angles() -> Result<(), Refused> {
    // Converted: `rc2k_core::race::best_line::link_angles`; each link's angle the x87's.
    let n = count()?;
    unsafe {
        let scales = (n >= 2).then(|| (NUMBERS.half.bits(), NUMBERS.k65536.bits()));
        let (writes, _) = rc2k_core::race::best_line::link_angles(
            n,
            |b| {
                let (half, scale) = scales.unwrap();
                let (_, p) = point_of(check_face(b - 1));
                let (_, q) = point_of(check_face(b));
                let dx = get(q).wrapping_sub(get(p)) as i32;
                let dz = get(q + 8).wrapping_sub(get(p + 8)) as i32;
                DRAW.scratch.set_bits(dz as u32);
                x87::angle_scaled3_to_int(dx, dz, half, scale, scale) as u32
            },
            || read_n(&crate::race::checkpoints_store::CHECKPOINTS.angles[0], 2),
        );
        for (k, angle) in writes {
            write_n(
                &crate::race::checkpoints_store::CHECKPOINTS.angles[0],
                k,
                angle,
            );
        }
    }
    Ok(())
}

/// `race::checkpoint_distances` (0x445AF1): with two checkpoints or more, the distance
/// along the line (x and z of their first points, each link's length rounded) at each of
/// 0 .. `g_lastCheckpoint`; the whole at `g_lastCheckpoint` and in `g_lineLength`. Refuses
/// a last checkpoint of 0 or past the lists (the original counts round), and fewer than
/// two checkpoints: the original then writes 0 at its caller's ecx (section 16), which
/// its one caller, `build_best_line`, never reaches (it calls with two line faces or more,
/// and `sort_checkpoints` makes the checkpoint count the same).
pub unsafe fn checkpoint_distances() -> Result<(), Refused> {
    let n = count()?;
    if n < 2 {
        return Err(BadInput("fewer than two checkpoints: the original writes at ecx").into());
    }
    unsafe {
        let last: u32 = RACE_VARS.last_checkpoint.bits();
        if last == 0 || last >= MOST_CHECKS {
            return Err(BadInput("a last checkpoint of 0 or past the lists").into());
        }
        // Converted: `rc2k_core::race::best_line::distances`.
        let sums = rc2k_core::race::best_line::distances(last, |k| {
            let (_, a) = point_of(check_face(k + 1));
            let (_, b) = point_of(check_face(k));
            let d = [
                get(a).wrapping_sub(get(b)),
                get(a + 8).wrapping_sub(get(b + 8)),
            ];
            crate::session::write_bits(&RACE_VARS.line_from, d);
            let length = x87::int_hypot(d[0] as i32, d[1] as i32) as u32;
            DRAW.scratch.set_bits(length);
            length
        });
        for (k, &sum) in sums[..last as usize].iter().enumerate() {
            write_n(
                &crate::race::checkpoints_store::CHECKPOINTS.distances[0],
                k as u32,
                sum,
            );
        }
        write_n(
            &crate::race::checkpoints_store::CHECKPOINTS.distances[0],
            last,
            sums[last as usize],
        );
        crate::race::checkpoints_store::CHECKPOINTS
            .line_length
            .set(sums[last as usize]);
    }
    Ok(())
}

/// `race::replay_cameras` (0x445BB2, 0x445CF1): with two checkpoints or more, the faces
/// of the set's kind (first byte 0xFF) listed (0x400 at most); with any, each
/// checkpoint's nearest of them (x and z of their first points, the length rounded; the
/// last of the nearest when two are as near). Refuses no faces or 0x100000 or more.
pub unsafe fn replay_cameras(set: &Cameras) -> Result<(), Refused> {
    let n = count()?;
    if n < 2 {
        return Ok(());
    }
    let faces = face_count()?;
    unsafe {
        // Converted: `rc2k_core::race::best_line::{camera_faces, nearest_camera}`.
        let (cameras, _) = rc2k_core::race::best_line::camera_faces(faces, |b| {
            let f = face(b);
            *(f as *const u8) == 0xFF && *((f + 1) as *const u8) == set.kind
        });
        for (e, &f) in cameras.iter().enumerate() {
            write_n((set.list)(), e as u32, f);
        }
        let m = cameras.len() as u32;
        (set.count)().set(m);
        if m == 0 {
            return Ok(());
        }
        for c in 0..n {
            let (_, p) = point_of(check_face(c));
            let from = [get(p), get(p + 8)];
            crate::session::write_bits(&RACE_VARS.line_from, from);
            let nearest = rc2k_core::race::best_line::nearest_camera(m, |e| {
                let (_, q) = point_of(face(read_n((set.list)(), e)));
                let dz = get(q + 8).wrapping_sub(from[1]);
                let d = x87::int_hypot(get(q).wrapping_sub(from[0]) as i32, dz as i32) as u32;
                DRAW.scratch.set_bits(d);
                d
            });
            crate::session::write_bits(&RACE_VARS.line_nearest, [nearest.0, nearest.1]);
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
pub unsafe fn surface_shares() -> Result<(), Refused> {
    let n = count()?;
    if n < 2 {
        return Ok(());
    }
    unsafe {
        std::ptr::write_bytes(RACE_VARS.surface_shares[0].addr() as *mut u8, 0, 0x40);
        let faces: u32 = COURSE.course_face_count.bits();
        let infos = crate::scene::polygon::texture_infos_addr() as u32;
        let byte = |at: u32| *(at as *const u8) as u32;
        let four = |info: u32| {
            for k in 0..4 {
                add_surface_share(byte(info + 0x10 + k));
            }
        };
        for e in 0..n {
            let named = get(check_face(e) + 0x3C);
            if named >= faces {
                continue;
            }
            let f = face(named);
            // Converted: `rc2k_core::race::best_line::{second_weight, first_weight}`.
            let blend = [0x22, 0x23, 0x24, 0x25].map(|o| byte(f + o) as u8);
            let second = byte(f + 0x21);
            // dword_63D9D0: indexed by a byte of the texture info (not bounded by the
            // field's 4 dwords), so read by address.
            let kind: u32 = ((RACE_VIEW.surface_kinds[0].addr()
                + 4 * byte(infos + second * 0x40 + 0x2C) as usize)
                as *const u32)
                .read_unaligned();
            let weight = rc2k_core::race::best_line::second_weight(blend, second as u8, kind);
            RACE_VARS
                .surface_weight
                .set_bits(rc2k_core::race::best_line::first_weight(weight));
            four(infos + byte(f + 1) * 0x40);
            RACE_VARS
                .surface_weight
                .set_bits(rc2k_core::race::best_line::first_weight(
                    RACE_VARS.surface_weight.bits(),
                ));
            four(infos + second * 0x40);
        }
        for c in 0..16 {
            let share = &RACE_VARS.surface_shares[c];
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
pub unsafe fn map_segment() -> Result<(), Refused> {
    use crate::math::turn_point_xz::turn_point_xz;
    unsafe {
        let w = |o: u32| RACE_VARS.map_work[0].addr() as u32 + o - 0xB8;
        let (a, b) = (get(w(0xC4)), get(w(0xC0)));
        if a >= MOST_CHECKS || b >= MOST_CHECKS {
            return Err(BadInput("map anchors past the lists").into());
        }
        let anchor = |n: u32| {
            let f = check_face(n);
            let place = [
                (*((f + 0x2C) as *const u16) as u32) << 14,
                ((*((f + 0x2E) as *const u16) as u32) << 14).wrapping_neg(),
            ];
            let (_, p) = point_of(f);
            (place, [get(p), get(p + 8)])
        };
        let (place_a, point_a) = anchor(a);
        put(w(0xEC), place_a[0]);
        put(w(0xF0), place_a[1]);
        put(w(0xD4), point_a[0]);
        put(w(0xD8), point_a[1]);
        let (place_b, point_b) = anchor(b);
        let ground = [
            point_b[0].wrapping_sub(point_a[0]),
            point_b[1].wrapping_sub(point_a[1]),
        ];
        let map = [
            place_b[0].wrapping_sub(place_a[0]),
            place_b[1].wrapping_sub(place_a[1]),
        ];
        put(w(0xF4), map[0]);
        put(w(0xF8), map[1]);
        put(w(0xDC), ground[0]);
        put(w(0xE0), ground[1]);
        let scale = x87::hypot_ratio(
            map[0] as i32,
            map[1] as i32,
            ground[0] as i32,
            ground[1] as i32,
        );
        put(w(0xC8), scale);
        let (half, unit) = (NUMBERS.half.bits(), NUMBERS.k65536.bits());
        let map_angle =
            (x87::angle_scaled5_to_int(map[0] as i32, map[1] as i32, half, unit) as u32) << 2;
        put(w(0xB8), map_angle);
        let ground_angle =
            (x87::angle_scaled5_to_int(ground[0] as i32, ground[1] as i32, half, unit) as u32) << 2;
        put(w(0xBC), ground_angle);
        let mut e = a;
        loop {
            // The point's number rides along as `turn_point_xz`'s y (ebx in the original).
            let (number, at) = point_of(check_face(e));
            let p = [get(at), get(at + 8)];
            put(w(0xCC), p[0]);
            put(w(0xD0), p[1]);
            let [x, _, z, _] = turn_point_xz(
                p[0].wrapping_sub(point_a[0]) as i32,
                number as i32,
                p[1].wrapping_sub(point_a[1]) as i32,
                ground_angle.wrapping_neg(),
            );
            let x = x87::int_times_to_int(x as i32, scale);
            let z = x87::int_times_to_int(z as i32, scale);
            DRAW.scratch.set_bits(z as u32);
            let [x, _, z, _] = turn_point_xz(x, number as i32, z, map_angle);
            write_n(
                &crate::race::checkpoints_store::CHECKPOINTS.map_x[0],
                e,
                x.wrapping_add(place_a[0]),
            );
            write_n(
                &crate::race::checkpoints_store::CHECKPOINTS.map_y[0],
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
pub unsafe fn map_checkpoints() -> Result<(), Refused> {
    // Converted: `rc2k_core::race::best_line::map_stretches`.
    let n = count()?;
    unsafe {
        RACE_VARS.map_anchor_count.set_bits(0u32);
        if n < 2 {
            return Ok(());
        }
        rc2k_core::race::best_line::map_stretches(
            |start| next_map_anchor(start),
            |from, to| {
                // The stretch's two anchors (0x6236C4, 0x6236C0 of the map work).
                RACE_VARS.map_work[3].set(from);
                RACE_VARS.map_work[2].set(to);
                map_segment()
            },
        )
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
    (c"dir", || RACE_WORK.stage_direction.get()),
    (c": numblo", || RACE_VARS.line_face_total.get()),
    (c"numsorted", || RACE_VARS.checkpoint_count.get()),
    (c"numsortedm1", || RACE_VARS.last_checkpoint.get()),
];

/// `race::build_best_line` (0x445772): a rule in the log; `sort_checkpoints`; with two
/// line faces or more, each step with its word in the log (angles, checks, "speeds" =
/// `nullsub_7`, distances, the map, the two replay cameras, then the surface shares);
/// "best line completed", the summary ("dir", ": numblo", "numsorted", "numsortedm1"),
/// a rule; `dword_6365E4` and `dword_6365EC` 0. The registers it leaves are read by no
/// caller (conversion phase 3, 2026-10-24).
pub unsafe fn build_best_line() -> Result<(), Refused> {
    use crate::logfile::{log_int::log_int, log_write::log_write};
    let text =
        |at: &std::ffi::CStr, newline: bool| unsafe { log_write(at.as_ptr().cast(), newline, 1) };
    unsafe {
        text(RULE_RVA[0], true);
        sort_checkpoints()?;
        if RACE_VARS.line_face_total.bits() >= 2 {
            for (word, step) in STEP_WORDS {
                text(word, false);
                match step {
                    1 => checkpoint_angles()?,
                    2 => count_checks()?,
                    3 => {} // `nullsub_7`
                    4 => checkpoint_distances()?,
                    5 => map_checkpoints()?,
                    6 => replay_cameras(&cameras()[0])?,
                    _ => replay_cameras(&cameras()[1])?,
                }
            }
            surface_shares()?;
        }
        text(COMPLETED_RVA, true);
        for (n, (at, value)) in SUMMARY.into_iter().enumerate() {
            text(at, false);
            log_int(value() as i32, n == 3, 1);
        }
        text(RULE_RVA[1], true);
        // dword_6365E4, dword_6365EC
        LIGHT_STATE.line_place.set(0);
        LIGHT_STATE.line_fraction.set(0);
    }
    Ok(())
}
