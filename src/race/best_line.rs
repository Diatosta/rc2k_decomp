//! `race::loadlevel_init`'s locacurrcourse, its "best line" (`sub_445772`, IDA
//! 2026-10-16): the checkpoint faces (first byte 0xFF, kind `g_lineFaceKind`) listed in
//! `g_checkpointFaces` (0x9E3738) and chained in order, `g_checkpointCount` of them
//! (0x623640, "numsorted"); then their angles, checks, distances, the map, the replay
//! cameras and the surface shares. Here the small steps.

use crate::errors::{BadInput, Refused};
use crate::framework::{game_addr, game_read, game_write, rva};
use crate::math::x87;

pub const DIRECTION_RVA: usize = rva(0x71B400); // g_stageDirection ("dir")
pub const LINE_KIND_RVA: usize = rva(0x623654); // g_lineFaceKind
pub const CHECK_FACES_RVA: usize = rva(0x9E3738); // g_checkpointFaces
pub const CHECK_COUNT_RVA: usize = rva(0x623640); // g_checkpointCount
pub const CHECKS_RVA: usize = rva(0x9E87A0); // g_checkCounts
pub const WEIGHT_RVA: usize = rva(0x6236B0); // g_surfaceWeight
pub const SHARES_RVA: usize = rva(0x623670); // g_surfaceShares: 16 floats
pub const ANCHORS_RVA: usize = rva(0x6236B4); // g_mapAnchorCount
const SHARE_SCALE: [usize; 3] = [rva(0x5C5E2C), rva(0x5C5DA4), rva(0x5C5D14)]; // 256, 0.25, 100
pub const FACES_RVA: usize = rva(0x5CCCBC); // g_courseFaces
/// The entries `g_checkpointFaces` and the lists after it have room for.
pub const MOST_CHECKS: u32 = rc2k_core::race::checkpoints::MOST_CHECKS;

unsafe fn get(at: u32) -> u32 {
    unsafe { (at as *const u32).read_unaligned() }
}

/// Checkpoint n's face.
unsafe fn check_face(n: u32) -> u32 {
    unsafe {
        let face: u32 = game_read(CHECK_FACES_RVA + 4 * n as usize);
        (face << 6).wrapping_add(game_read(FACES_RVA))
    }
}

fn count() -> Result<u32, Refused> {
    let n: u32 = unsafe { game_read(CHECK_COUNT_RVA) };
    if n > MOST_CHECKS {
        return Err(BadInput("more checkpoints than the lists hold").into());
    }
    Ok(n)
}

/// `race::set_line_face_kind` (0x44574A): `g_lineFaceKind` 0x3E with `g_stageDirection`
/// 0, 0x3D with 1, else 0.
pub unsafe fn set_line_face_kind() {
    // Converted: `rc2k_core::race::checkpoints::line_face_kind`.
    let kind = rc2k_core::race::checkpoints::line_face_kind(unsafe { game_read(DIRECTION_RVA) });
    unsafe { game_write(LINE_KIND_RVA, kind) };
}

/// `race::add_surface_share` (0x445F7B): with `g_surfaceWeight` not 0, surface `surface`
/// & 0xF's share gets weight / 256 * 0.25 / `g_checkpointCount` * 100 more (on the x87,
/// the count as a whole number). The exact x87 version (for `race::surface_shares`, whose
/// whole numbers are rounded from the shares).
pub unsafe fn add_surface_share(surface: u32) {
    unsafe {
        let weight: i32 = game_read(WEIGHT_RVA);
        if weight == 0 {
            return;
        }
        let at = SHARES_RVA + 4 * (surface & 0xF) as usize;
        let [a, b, c] = SHARE_SCALE.map(|r| game_read::<u32>(r));
        let count = game_read(CHECK_COUNT_RVA);
        game_write(
            at,
            x87::surface_share(weight, a, b, count, c, game_read(at)),
        );
    }
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
            game_write(ANCHORS_RVA, game_read::<u32>(ANCHORS_RVA).wrapping_add(1));
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
            game_write(CHECKS_RVA + 4 * k, c);
        }
    }
    Ok(())
}

pub const FOUND_RVA: usize = rva(0x62363C); // g_lineFaceTotal ("numblo")
pub const LAST_CHECK_RVA: usize = rva(0x623644); // g_lastCheckpoint ("numsortedm1")
pub const ANGLES_RVA: usize = rva(0x9ED800); // g_checkpointAngles (from +4: one a link)
pub const DISTANCES_RVA: usize = rva(0x9F78D8); // g_checkpointDistances
pub const LINE_LENGTH_RVA: usize = rva(0xA10AE0); // g_lineLength
pub const SCRATCH_RVA: usize = rva(0x62400C); // dword_62400C
const FROM_RVA: usize = rva(0x623658); // dword_623658, dword_62365C: a point's x, z
const NEAREST_RVA: usize = rva(0x623668); // dword_623668 the distance, dword_62366C the one
pub const FACE_COUNT_RVA: usize = rva(0x5CCD1C); // g_courseFaceCount
pub const POINTS_RVA: usize = rva(0x5CCCC0); // g_scenePoints
const HALF_RVA: usize = rva(0x5C5E40); // flt_5C5E40, 0.5
const SCALE_RVA: usize = rva(0x5C5D98); // flt_5C5D98, 65536.0
pub const TEXTURE_INFOS_RVA: usize = rva(0x979F30); // g_textureInfos, 0x40 each
const SURFACE_KINDS_RVA: usize = rva(0x63D9D0); // dword_63D9D0
/// The map: the stretch's two anchors and what is made of them (0x6236B8 .. 0x6236F8).
pub const MAP_WORK_RVA: usize = rva(0x6236B8);
pub const MAP_X_RVA: usize = rva(0x9FC940); // g_mapX
pub const MAP_Y_RVA: usize = rva(0xA019A8); // g_mapY
const MOST_FACES: u32 = 0x10_0000;

/// A replay camera's set: the face kind, the list (0x400) of such faces, its count, and
/// each checkpoint's nearest.
#[derive(Clone, Copy)]
pub struct Cameras {
    pub kind: u8,
    pub list: usize,
    pub count: usize,
    pub nearest: usize,
}

/// `sub_445BB2` ("replaycam1") and `sub_445CF1` ("replaycam2").
pub const CAMERAS: [Cameras; 2] = [
    Cameras {
        kind: 0x3C,
        list: rva(0xA10AE4),
        count: rva(0x623660),
        nearest: rva(0xA06A10),
    },
    Cameras {
        kind: 0x38,
        list: rva(0xA11AE4),
        count: rva(0x623664),
        nearest: rva(0xA0BA78),
    },
];

unsafe fn put(at: u32, value: u32) {
    unsafe { (at as *mut u32).write_unaligned(value) };
}

unsafe fn at_n(table: usize, n: u32) -> u32 {
    game_addr(table) as u32 + 4 * n
}

unsafe fn read_n(table: usize, n: u32) -> u32 {
    unsafe { get(at_n(table, n)) }
}

unsafe fn write_n(table: usize, n: u32, value: u32) {
    unsafe { put(at_n(table, n), value) };
}

unsafe fn face(n: u32) -> u32 {
    unsafe { (n << 6).wrapping_add(game_read(FACES_RVA)) }
}

/// A face's first point: its number (+4) and address.
unsafe fn point_of(face_address: u32) -> (u32, u32) {
    unsafe {
        let n = get(face_address + 4);
        (n, n.wrapping_mul(12).wrapping_add(game_read(POINTS_RVA)))
    }
}

fn face_count() -> Result<u32, Refused> {
    let n: u32 = unsafe { game_read(FACE_COUNT_RVA) };
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
        let kind = line_face_kind(game_read(DIRECTION_RVA));
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
                && *((f + 1) as *const u8) == game_read::<u32>(LINE_KIND_RVA) as u8
            {
                write_n(CHECK_FACES_RVA, list.len() as u32, b);
                list.push(b);
            }
        }
        if list.len() < 2 {
            game_write(FOUND_RVA, 0u32);
            game_write(CHECK_COUNT_RVA, 0u32);
            return Ok(());
        }
        let c = chain(
            &mut list,
            |f| *((face(f) + 0x26) as *const u8) == 1,
            |f| get(face(f) + 0xC),
        );
        for (k, &f) in list.iter().enumerate() {
            write_n(CHECK_FACES_RVA, k as u32, f);
        }
        game_write(CHECK_COUNT_RVA, c.length);
        game_write(FOUND_RVA, c.length);
        game_write(LAST_CHECK_RVA, c.length.wrapping_sub(1));
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
        let scales = (n >= 2).then(|| (game_read::<u32>(HALF_RVA), game_read::<u32>(SCALE_RVA)));
        let (writes, _) = rc2k_core::race::best_line::link_angles(
            n,
            |b| {
                let (half, scale) = scales.unwrap();
                let (_, p) = point_of(check_face(b - 1));
                let (_, q) = point_of(check_face(b));
                let dx = get(q).wrapping_sub(get(p)) as i32;
                let dz = get(q + 8).wrapping_sub(get(p + 8)) as i32;
                game_write(SCRATCH_RVA, dz);
                x87::angle_scaled3_to_int(dx, dz, half, scale, scale) as u32
            },
            || read_n(ANGLES_RVA, 2),
        );
        for (k, angle) in writes {
            write_n(ANGLES_RVA, k, angle);
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
        let last: u32 = game_read(LAST_CHECK_RVA);
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
            game_write(FROM_RVA, d);
            let length = x87::int_hypot(d[0] as i32, d[1] as i32) as u32;
            game_write(SCRATCH_RVA, length);
            length
        });
        for (k, &sum) in sums[..last as usize].iter().enumerate() {
            write_n(DISTANCES_RVA, k as u32, sum);
        }
        write_n(DISTANCES_RVA, last, sums[last as usize]);
        game_write(LINE_LENGTH_RVA, sums[last as usize]);
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
            write_n(set.list, e as u32, f);
        }
        let m = cameras.len() as u32;
        game_write(set.count, m);
        if m == 0 {
            return Ok(());
        }
        for c in 0..n {
            let (_, p) = point_of(check_face(c));
            let from = [get(p), get(p + 8)];
            game_write(FROM_RVA, from);
            let nearest = rc2k_core::race::best_line::nearest_camera(m, |e| {
                let (_, q) = point_of(face(read_n(set.list, e)));
                let dz = get(q + 8).wrapping_sub(from[1]);
                let d = x87::int_hypot(get(q).wrapping_sub(from[0]) as i32, dz as i32) as u32;
                game_write(SCRATCH_RVA, d);
                d
            });
            game_write(NEAREST_RVA, [nearest.0, nearest.1]);
            write_n(set.nearest, c, nearest.1);
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
        std::ptr::write_bytes(game_addr(SHARES_RVA) as *mut u8, 0, 0x40);
        let faces: u32 = game_read(FACE_COUNT_RVA);
        let infos = game_addr(TEXTURE_INFOS_RVA) as u32;
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
            let kind: u32 =
                game_read(SURFACE_KINDS_RVA + 4 * byte(infos + second * 0x40 + 0x2C) as usize);
            let weight = rc2k_core::race::best_line::second_weight(blend, second as u8, kind);
            game_write(WEIGHT_RVA, rc2k_core::race::best_line::first_weight(weight));
            four(infos + byte(f + 1) * 0x40);
            game_write(
                WEIGHT_RVA,
                rc2k_core::race::best_line::first_weight(game_read(WEIGHT_RVA)),
            );
            four(infos + second * 0x40);
        }
        for c in 0..16 {
            let at = SHARES_RVA + 4 * c;
            game_write(at, x87::to_int(game_read(at)) as u32);
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
        let w = |o: u32| game_addr(MAP_WORK_RVA) as u32 + o - 0xB8;
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
        let (half, unit) = (game_read::<u32>(HALF_RVA), game_read::<u32>(SCALE_RVA));
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
            game_write(SCRATCH_RVA, z as u32);
            let [x, _, z, _] = turn_point_xz(x, number as i32, z, map_angle);
            write_n(MAP_X_RVA, e, x.wrapping_add(place_a[0]));
            write_n(MAP_Y_RVA, e, z.wrapping_add(place_a[1]).wrapping_neg());
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
        game_write(ANCHORS_RVA, 0u32);
        if n < 2 {
            return Ok(());
        }
        rc2k_core::race::best_line::map_stretches(
            |start| next_map_anchor(start),
            |from, to| {
                game_write(MAP_WORK_RVA + 0xC4 - 0xB8, from);
                game_write(MAP_WORK_RVA + 0xC0 - 0xB8, to);
                map_segment()
            },
        )
    }
}

const RULE_RVA: [usize; 2] = [rva(0x4DC4A8), rva(0x4DC536)]; // "-----..."
/// "angles", "checks", "speeds", "dists", "map", "replaycam1", "replaycam2", with the
/// step each comes before.
const STEP_WORDS: [(usize, u32); 7] = [
    (rva(0x4DC4CA), 1),
    (rva(0x4DC4D1), 2),
    (rva(0x4DC4D8), 3),
    (rva(0x4DC4DF), 4),
    (rva(0x4DC4E5), 5),
    (rva(0x4DC4E9), 6),
    (rva(0x4DC4F4), 7),
];
const COMPLETED_RVA: usize = rva(0x4DC4FF); // "best line completed"
/// "dir" `g_stageDirection`, ": numblo" `g_lineFaceTotal`, "numsorted"
/// `g_checkpointCount`, "numsortedm1" `g_lastCheckpoint` (the last with the line's end).
const SUMMARY: [(usize, usize); 4] = [
    (rva(0x4DC513), DIRECTION_RVA),
    (rva(0x4DC517), FOUND_RVA),
    (rva(0x4DC520), CHECK_COUNT_RVA),
    (rva(0x4DC52A), LAST_CHECK_RVA),
];
const CLEARED_RVA: [usize; 2] = [rva(0x6365E4), rva(0x6365EC)]; // dword_6365E4, dword_6365EC

/// `race::build_best_line` (0x445772): a rule in the log; `sort_checkpoints`; with two
/// line faces or more, each step with its word in the log (angles, checks, "speeds" =
/// `nullsub_7`, distances, the map, the two replay cameras, then the surface shares);
/// "best line completed", the summary ("dir", ": numblo", "numsorted", "numsortedm1"),
/// a rule; `dword_6365E4` and `dword_6365EC` 0. The registers it leaves are read by no
/// caller (conversion phase 3, 2026-10-24).
pub unsafe fn build_best_line() -> Result<(), Refused> {
    use crate::logfile::{log_int::log_int, log_write::log_write};
    let text =
        |at: usize, newline: bool| unsafe { log_write(game_addr(at) as *const u8, newline, 1) };
    unsafe {
        text(RULE_RVA[0], true);
        sort_checkpoints()?;
        if game_read::<u32>(FOUND_RVA) >= 2 {
            for (word, step) in STEP_WORDS {
                text(word, false);
                match step {
                    1 => checkpoint_angles()?,
                    2 => count_checks()?,
                    3 => {} // `nullsub_7`
                    4 => checkpoint_distances()?,
                    5 => map_checkpoints()?,
                    6 => replay_cameras(&CAMERAS[0])?,
                    _ => replay_cameras(&CAMERAS[1])?,
                }
            }
            surface_shares()?;
        }
        text(COMPLETED_RVA, true);
        for (n, (at, value)) in SUMMARY.into_iter().enumerate() {
            text(at, false);
            log_int(game_read::<u32>(value) as i32, n == 3, 1);
        }
        text(RULE_RVA[1], true);
        for at in CLEARED_RVA {
            game_write(at, 0u32);
        }
    }
    Ok(())
}

// ---- for the self-tests ----
