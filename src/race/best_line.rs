//! `race::loadlevel_init`'s locacurrcourse, its "best line" (`sub_445772`, IDA
//! 2026-10-16): the checkpoint faces (first byte 0xFF, kind `g_lineFaceKind`) listed in
//! `g_checkpointFaces` (0x9E3738) and chained in order, `g_checkpointCount` of them
//! (0x623640, "numsorted"); then their angles, checks, distances, the map, the replay
//! cameras and the surface shares. Here the small steps.

use crate::errors::{BadInput, Refused};
use crate::framework::{Regs, game_addr, game_read, game_write, rva};
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
pub const MOST_CHECKS: u32 = 0x1418;

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
/// 0, 0x3D with 1, else 0. Leaves ebx it.
pub unsafe fn set_line_face_kind(entry: &Regs) -> Regs {
    let kind = match unsafe { game_read::<u32>(DIRECTION_RVA) } {
        0 => 0x3E,
        1 => 0x3D,
        _ => 0,
    };
    unsafe { game_write(LINE_KIND_RVA, kind) };
    Regs {
        ebx: kind,
        ..*entry
    }
}

/// `race::add_surface_share` (0x445F7B): with `g_surfaceWeight` not 0, surface al & 0xF's
/// share gets weight / 256 * 0.25 / `g_checkpointCount` * 100 more (on the x87, the count
/// as a whole number). Leaves eax & 0xF then.
pub unsafe fn add_surface_share(entry: &Regs) -> Regs {
    let mut g = *entry;
    unsafe {
        let weight: i32 = game_read(WEIGHT_RVA);
        if weight == 0 {
            return g;
        }
        g.eax &= 0xF;
        let at = SHARES_RVA + 4 * g.eax as usize;
        let [a, b, c] = SHARE_SCALE.map(|r| game_read::<u32>(r));
        let sum = x87::surface_share(weight, a, b, game_read(CHECK_COUNT_RVA), c, game_read(at));
        game_write(at, sum);
    }
    g
}

/// `race::next_map_anchor` (0x445FEE): from checkpoint ecx on, the first whose face has
/// +0x2C (the map's x and y words) not 0: `g_mapAnchorCount` + 1, carry clear; carry set at
/// `g_checkpointCount`. Leaves ecx it, ebx and esi its face's number and address. Refuses
/// more checkpoints than the lists hold.
pub unsafe fn next_map_anchor(entry: &Regs) -> Result<(Regs, bool), Refused> {
    let mut g = *entry;
    let n = count()?;
    unsafe {
        loop {
            if g.ecx >= n {
                return Ok((g, true));
            }
            g.ebx = game_read(CHECK_FACES_RVA + 4 * g.ecx as usize);
            g.esi = check_face(g.ecx);
            if get(g.esi + 0x2C) != 0 {
                game_write(ANCHORS_RVA, game_read::<u32>(ANCHORS_RVA).wrapping_add(1));
                return Ok((g, false));
            }
            g.ecx += 1;
        }
    }
}

/// `race::count_checks` (0x445AAB): with two checkpoints or more, `g_checkCounts`[n] the
/// checkpoints up to n whose face's byte +0x26 is neither 0 nor 1. Leaves ebp the last
/// count, ebx the checkpoint count, esi the last face. Refuses more checkpoints than the
/// lists hold.
pub unsafe fn count_checks(entry: &Regs) -> Result<Regs, Refused> {
    let mut g = *entry;
    let n = count()?;
    if n < 2 {
        return Ok(g);
    }
    unsafe {
        g.ebp = 0;
        for k in 0..n {
            g.esi = check_face(k);
            if !matches!(*((g.esi + 0x26) as *const u8), 0 | 1) {
                g.ebp += 1;
            }
            game_write(CHECKS_RVA + 4 * k as usize, g.ebp);
        }
        g.ebx = n;
    }
    Ok(g)
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
pub const MAP_WORK_LEN: usize = 0x44;
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
/// `g_lastCheckpoint` it - 1. With fewer than two, both counts 0. Leaves eax the last - 1
/// (with a chain), ebx, ecx, edx, esi, ebp as the searches leave them. Refuses no faces or
/// 0x100000 or more, and more of the kind than the list holds.
pub unsafe fn sort_checkpoints(entry: &Regs) -> Result<Regs, Refused> {
    unsafe {
        let n = face_count()?;
        let kind: u32 = match game_read::<u32>(DIRECTION_RVA) {
            0 => 0x3E,
            1 => 0x3D,
            _ => 0,
        };
        let of_kind = |b: u32| {
            let f = face(b);
            *(f as *const u8) == 0xFF && *((f + 1) as *const u8) == kind as u8
        };
        if (0..n).filter(|&b| of_kind(b)).count() as u32 > MOST_CHECKS {
            return Err(BadInput("more line faces than the checkpoint list holds").into());
        }
        let mut g = set_line_face_kind(entry);
        g.ebp = 0;
        for b in 0..n {
            g.esi = face(b);
            if *(g.esi as *const u8) == 0xFF {
                g.eax = game_read(LINE_KIND_RVA);
                if *((g.esi + 1) as *const u8) == g.eax as u8 {
                    write_n(CHECK_FACES_RVA, g.ebp, b);
                    g.ebp += 1;
                }
            }
        }
        g.ebx = n;
        let found = g.ebp;
        game_write(FOUND_RVA, found);
        if found < 2 {
            game_write(FOUND_RVA, 0u32);
            game_write(CHECK_COUNT_RVA, 0u32);
            return Ok(g);
        }
        game_write(CHECK_COUNT_RVA, 0u32);
        let swap = |a: u32, b: u32| {
            let (x, y) = (read_n(CHECK_FACES_RVA, a), read_n(CHECK_FACES_RVA, b));
            write_n(CHECK_FACES_RVA, a, y);
            write_n(CHECK_FACES_RVA, b, x);
        };
        g.edx = 0;
        let mut start = false;
        loop {
            g.ebx = read_n(CHECK_FACES_RVA, g.edx);
            g.esi = face(g.ebx);
            if *((g.esi + 0x26) as *const u8) == 1 {
                start = true;
                break;
            }
            g.edx += 1;
            if g.edx == found {
                break;
            }
        }
        if start {
            swap(g.edx, 0);
            game_write(CHECK_COUNT_RVA, 1u32);
            loop {
                g.ecx = game_read(CHECK_COUNT_RVA);
                if g.ecx == found {
                    break;
                }
                g.ebx = read_n(CHECK_FACES_RVA, g.ecx - 1);
                g.esi = face(g.ebx);
                g.eax = get(g.esi + 0xC);
                g.edx = 0;
                let mut next = false;
                loop {
                    if g.eax == read_n(CHECK_FACES_RVA, g.edx) {
                        next = true;
                        break;
                    }
                    g.edx += 1;
                    if g.edx == found {
                        break;
                    }
                }
                if !next {
                    break;
                }
                g.ecx = game_read(CHECK_COUNT_RVA);
                swap(g.ecx, g.edx);
                game_write(CHECK_COUNT_RVA, g.ecx + 1);
            }
        }
        let chained: u32 = game_read(CHECK_COUNT_RVA);
        game_write(FOUND_RVA, chained);
        g.eax = chained.wrapping_sub(1);
        game_write(LAST_CHECK_RVA, g.eax);
        Ok(g)
    }
}

/// `race::checkpoint_angles` (0x4459DF): with two checkpoints or more, each link's angle
/// (x and z from the last checkpoint's first point to this one's: the x87's `fpatan` over
/// pi, times 0.5, 65536 and 65536) into `g_checkpointAngles` + 1 .. n, n + 1 the same as n; then 0
/// and 1 as 2. Leaves eax the angle at 2, ebx the count (with two or more).
pub unsafe fn checkpoint_angles(entry: &Regs) -> Result<Regs, Refused> {
    let mut g = *entry;
    let n = count()?;
    unsafe {
        if n >= 2 {
            let (half, scale) = (game_read::<u32>(HALF_RVA), game_read::<u32>(SCALE_RVA));
            for b in 1..n {
                let (_, p) = point_of(check_face(b - 1));
                let (_, q) = point_of(check_face(b));
                let dx = get(q).wrapping_sub(get(p)) as i32;
                let dz = get(q + 8).wrapping_sub(get(p + 8)) as i32;
                game_write(SCRATCH_RVA, dz);
                let angle = x87::angle_scaled3_to_int(dx, dz, half, scale, scale);
                write_n(ANGLES_RVA, b + 1, angle as u32);
            }
            g.ebx = n;
            g.eax = read_n(ANGLES_RVA, n);
            write_n(ANGLES_RVA, n + 1, g.eax);
        }
        g.eax = read_n(ANGLES_RVA, 2);
        write_n(ANGLES_RVA, 1, g.eax);
        write_n(ANGLES_RVA, 0, g.eax);
    }
    Ok(g)
}

/// `race::checkpoint_distances` (0x445AF1): with two checkpoints or more, the distance
/// along the line (x and z of their first points, each link's length rounded) at each of
/// 0 .. `g_lastCheckpoint`; the whole at `g_lastCheckpoint` and in `g_lineLength`. With
/// fewer, 0 at the caller's ecx (section 16) and in `g_lineLength`. Leaves edi the length,
/// ecx where it was put, eax, ebx and esi the last point's z, number and address. Refuses
/// a last checkpoint of 0 or past the lists (the original counts round), and the caller's
/// ecx past the list.
pub unsafe fn checkpoint_distances(entry: &Regs) -> Result<Regs, Refused> {
    let mut g = *entry;
    let n = count()?;
    unsafe {
        g.edi = 0;
        if n >= 2 {
            let last: u32 = game_read(LAST_CHECK_RVA);
            if last == 0 || last >= MOST_CHECKS {
                return Err(BadInput("a last checkpoint of 0 or past the lists").into());
            }
            g.ecx = 0;
            loop {
                let (_, a) = point_of(check_face(g.ecx + 1));
                let from = [get(a), get(a + 8)];
                (g.ebx, g.esi) = point_of(check_face(g.ecx));
                g.eax = get(g.esi + 8);
                let d = [
                    from[0].wrapping_sub(get(g.esi)),
                    from[1].wrapping_sub(g.eax),
                ];
                game_write(FROM_RVA, d);
                let length = x87::int_hypot(d[0] as i32, d[1] as i32) as u32;
                game_write(SCRATCH_RVA, length);
                write_n(DISTANCES_RVA, g.ecx, g.edi);
                g.edi = g.edi.wrapping_add(length);
                g.ecx += 1;
                if g.ecx == last {
                    break;
                }
            }
        } else if g.ecx >= MOST_CHECKS {
            return Err(BadInput("fewer than two checkpoints: the original writes at ecx").into());
        }
        write_n(DISTANCES_RVA, g.ecx, g.edi);
        game_write(LINE_LENGTH_RVA, g.edi);
    }
    Ok(g)
}

/// `race::replay_cameras` (0x445BB2, 0x445CF1): with two checkpoints or more, the faces
/// of the set's kind (first byte 0xFF) listed (0x400 at most); with any, each
/// checkpoint's nearest of them (x and z of their first points, the length rounded; the
/// last of the nearest when two are as near). Leaves eax the last nearest, ebx and esi
/// the last point's number and address, ecx the checkpoint count, edx the list's (with
/// none: ebx where the face search stopped, esi the last face). Refuses no faces or
/// 0x100000 or more.
pub unsafe fn replay_cameras(entry: &Regs, set: &Cameras) -> Result<Regs, Refused> {
    let mut g = *entry;
    let n = count()?;
    if n < 2 {
        return Ok(g);
    }
    let faces = face_count()?;
    unsafe {
        g.edx = 0;
        g.ebx = 0;
        loop {
            g.esi = face(g.ebx);
            if *(g.esi as *const u8) == 0xFF && *((g.esi + 1) as *const u8) == set.kind {
                write_n(set.list, g.edx, g.ebx);
                g.edx += 1;
                if g.edx == 0x400 {
                    break;
                }
            }
            g.ebx += 1;
            if g.ebx == faces {
                break;
            }
        }
        let m = g.edx;
        game_write(set.count, m);
        if m == 0 {
            return Ok(g);
        }
        for c in 0..n {
            g.ecx = c;
            (g.ebx, g.esi) = point_of(check_face(c));
            let from = [get(g.esi), get(g.esi + 8)];
            game_write(FROM_RVA, from);
            game_write(NEAREST_RVA, [u32::MAX; 2]);
            for e in 0..m {
                g.edx = e;
                (g.ebx, g.esi) = point_of(face(read_n(set.list, e)));
                let dz = get(g.esi + 8).wrapping_sub(from[1]);
                let d = x87::int_hypot(get(g.esi).wrapping_sub(from[0]) as i32, dz as i32) as u32;
                game_write(SCRATCH_RVA, d);
                g.eax = d;
                let [best, _]: [u32; 2] = game_read(NEAREST_RVA);
                if d <= best {
                    game_write(NEAREST_RVA, [d, e]);
                }
            }
            g.edx = m;
            g.eax = game_read(NEAREST_RVA + 4);
            write_n(set.nearest, c, g.eax);
        }
        g.ecx = n;
    }
    Ok(g)
}

/// `race::surface_shares` (0x445E30): with two checkpoints or more, `g_surfaceShares` 0;
/// for each checkpoint whose face's +0x3C names a face, that face's two textures (+1 and
/// +0x21, each 0x40 bytes of `g_textureInfos`) add their four surfaces (+0x10 .. +0x13,
/// `add_surface_share`): +0x21's weighted by the face's bytes +0x22 .. +0x25 (a quarter of
/// their sum; 0 without a +0x21, or with its kind (+0x2C, by `dword_63D9D0`) 4 or 5),
/// +1's by 0x100 less; then the shares made whole numbers. Leaves ecx 16, edi past the
/// shares, edx the count, eax, ebx, esi as the last face leaves them.
pub unsafe fn surface_shares(entry: &Regs) -> Result<Regs, Refused> {
    let mut g = *entry;
    let n = count()?;
    if n < 2 {
        return Ok(g);
    }
    unsafe {
        std::ptr::write_bytes(game_addr(SHARES_RVA) as *mut u8, 0, 0x40);
        g.edi = game_addr(SHARES_RVA) as u32 + 0x40;
        g.ecx = 0;
        g.eax = 0;
        let faces: u32 = game_read(FACE_COUNT_RVA);
        let infos = game_addr(TEXTURE_INFOS_RVA) as u32;
        let byte = |at: u32| *(at as *const u8) as u32;
        let four = |g: &mut Regs| {
            for k in 0..4 {
                g.eax = g.eax & !0xFF | byte(g.ebx + 0x10 + k);
                *g = add_surface_share(g);
            }
        };
        for e in 0..n {
            g.edx = e;
            g.ebx = get(check_face(e) + 0x3C);
            if g.ebx >= faces {
                continue;
            }
            g.esi = face(g.ebx);
            let mut weight = (0x22..0x26).map(|o| byte(g.esi + o)).sum::<u32>() >> 2;
            let second = byte(g.esi + 0x21);
            if second == 0 {
                weight = 0;
            }
            g.ebx = infos + second * 0x40;
            g.eax = game_read(SURFACE_KINDS_RVA + 4 * byte(g.ebx + 0x2C) as usize);
            if g.eax == 5 || g.eax == 4 {
                weight = 0;
            }
            g.eax = 0x100u32.wrapping_sub(weight);
            game_write(WEIGHT_RVA, g.eax);
            g.eax = byte(g.esi + 1) * 0x40;
            g.ebx = infos + g.eax;
            four(&mut g);
            g.eax = 0x100u32.wrapping_sub(game_read(WEIGHT_RVA));
            game_write(WEIGHT_RVA, g.eax);
            g.eax = second * 0x40;
            g.ebx = infos + g.eax;
            four(&mut g);
        }
        g.edx = n;
        for c in 0..16 {
            let at = SHARES_RVA + 4 * c;
            game_write(at, x87::to_int(game_read(at)) as u32);
        }
        g.ecx = 16;
    }
    Ok(g)
}

/// `race::map_segment` (0x446036): the checkpoints from `dword_6236C4` to `dword_6236C0`
/// (both included) placed on the map: the two anchors' map places (their faces' words +0x2C
/// and +0x2E, << 14, y negated) and first points; the stretch's scale (map length over
/// ground length, a float) and its two angles (`fpatan` / pi * 0.5^3 * 65536^2, << 2); each
/// checkpoint's point from the first anchor turned by minus the ground angle
/// (`math::turn_point_xz`), scaled, turned by the map angle, moved to the first anchor's
/// place: `g_mapX`, `g_mapY` (negated). Leaves ebx past the last, eax and ecx the last's
/// map x and minus its y, edx the turn's entry, esi its point. Refuses anchors past the
/// lists.
pub unsafe fn map_segment(entry: &Regs) -> Result<Regs, Refused> {
    use crate::math::turn_point_xz::turn_point_xz;
    let mut g = *entry;
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
            (g.ebx, g.esi) = point_of(check_face(e));
            let p = [get(g.esi), get(g.esi + 8)];
            put(w(0xCC), p[0]);
            put(w(0xD0), p[1]);
            g.eax = p[0].wrapping_sub(point_a[0]);
            g.ecx = p[1].wrapping_sub(point_a[1]);
            g.edx = ground_angle.wrapping_neg();
            let [x, _, z, entry] = turn_point_xz(g.eax as i32, g.ebx as i32, g.ecx as i32, g.edx);
            g.eax = x87::int_times_to_int(x as i32, scale) as u32;
            g.ecx = x87::int_times_to_int(z as i32, scale) as u32;
            game_write(SCRATCH_RVA, g.ecx);
            let _ = entry;
            g.edx = map_angle;
            let [x, _, z, entry] = turn_point_xz(g.eax as i32, g.ebx as i32, g.ecx as i32, g.edx);
            g.edx = entry;
            g.eax = x.wrapping_add(place_a[0]);
            g.ecx = z.wrapping_add(place_a[1]).wrapping_neg();
            write_n(MAP_X_RVA, e, g.eax);
            write_n(MAP_Y_RVA, e, g.ecx);
            e += 1;
            if e > b {
                break;
            }
        }
        g.ebx = e;
    }
    Ok(g)
}

/// `race::map_checkpoints` (0x445FB4): `g_mapAnchorCount` 0; with two checkpoints or more,
/// from checkpoint 0, each stretch to the next anchor (`next_map_anchor`) mapped
/// (`map_segment`, inside `pusha`), until none is left. Leaves eax the last anchor, ecx the
/// count, ebx and esi as `next_map_anchor` leaves them. Refuses more checkpoints than the
/// lists hold.
pub unsafe fn map_checkpoints(entry: &Regs) -> Result<Regs, Refused> {
    let mut g = *entry;
    let n = count()?;
    unsafe {
        game_write(ANCHORS_RVA, 0u32);
        if n < 2 {
            return Ok(g);
        }
        g.ecx = 0;
        loop {
            g.eax = g.ecx;
            g.ecx = g.eax + 1;
            let (r, end) = next_map_anchor(&g)?;
            g = r;
            if end {
                return Ok(g);
            }
            game_write(MAP_WORK_RVA + 0xC4 - 0xB8, g.eax);
            game_write(MAP_WORK_RVA + 0xC0 - 0xB8, g.ecx);
            map_segment(&g)?;
        }
    }
}

/// `race::build_best_line`'s callees by place, as IDA shows them: the eight steps (and
/// `nullsub_7`, "speeds", at 3), then the log's four.
pub const BUILD_CALLEES: &[(usize, u32)] = &[
    (0x4458A7, 0),
    (0x4459DF, 1),
    (0x445AAB, 2),
    (0x445AF0, 3),
    (0x445AF1, 4),
    (0x445FB4, 5),
    (0x445BB2, 6),
    (0x445CF1, 7),
    (0x445E30, 8),
    (0x401427, 9),
    (0x40142B, 10),
    (0x401472, 11),
    (0x401476, 12),
];

pub type Part<'a> = &'a mut dyn FnMut(u32, &mut Regs) -> Result<(), Refused>;

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

/// A log call inside `pusha`: place 9 .. 12, eax the text or the number.
fn log(place: u32, eax: u32, g: &Regs, part: Part) -> Result<(), Refused> {
    let mut r = Regs { eax, ..*g };
    part(place, &mut r)
}

/// `race::build_best_line` (0x445772): a rule in the log; `sort_checkpoints`; with two
/// line faces or more, each step with its word in the log (angles, checks, "speeds" =
/// `nullsub_7`, distances, the map, the two replay cameras, then the surface shares);
/// "best line completed", the summary ("dir", ": numblo", "numsorted", "numsortedm1"),
/// a rule; `dword_6365E4` and `dword_6365EC` 0. Leaves the registers as the last step
/// does (each step has the last one's).
pub unsafe fn build_best_line(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut g = *entry;
    unsafe {
        log(10, game_addr(RULE_RVA[0]) as u32, &g, part)?;
        part(0, &mut g)?;
        if game_read::<u32>(FOUND_RVA) >= 2 {
            for (word, step) in STEP_WORDS {
                log(9, game_addr(word) as u32, &g, part)?;
                part(step, &mut g)?;
            }
            part(8, &mut g)?;
        }
        log(10, game_addr(COMPLETED_RVA) as u32, &g, part)?;
        for (n, (text, value)) in SUMMARY.into_iter().enumerate() {
            log(9, game_addr(text) as u32, &g, part)?;
            log(if n == 3 { 12 } else { 11 }, game_read(value), &g, part)?;
        }
        log(10, game_addr(RULE_RVA[1]) as u32, &g, part)?;
        for at in CLEARED_RVA {
            game_write(at, 0u32);
        }
    }
    Ok(g)
}

/// `build_best_line`'s callees in the game: the Rust versions.
pub fn build_part(place: u32, r: &mut Regs) -> Result<(), Refused> {
    use crate::logfile::{log_int::log_int, log_write::log_write};
    unsafe {
        match place {
            0 => *r = sort_checkpoints(r)?,
            1 => *r = checkpoint_angles(r)?,
            2 => *r = count_checks(r)?,
            3 => {}
            4 => *r = checkpoint_distances(r)?,
            5 => *r = map_checkpoints(r)?,
            6 => *r = replay_cameras(r, &CAMERAS[0])?,
            7 => *r = replay_cameras(r, &CAMERAS[1])?,
            8 => *r = surface_shares(r)?,
            9 | 10 => log_write(r.eax as *const u8, place == 10, 1),
            _ => log_int(r.eax as i32, place == 12, 1),
        }
    }
    Ok(())
}

// ---- for the self-tests ----

/// A course of the test's for the line's steps: `faces` faces (filled by `fill`), the
/// first `checks` of `g_checkpointFaces` naming them (by `order`), `g_checkpointCount`
/// `checks`; the game's put back when dropped.
pub struct TestLine {
    pub faces: crate::race::course_data::TestTable,
    pub points: Vec<i32>,
    vars: Vec<crate::service::harness::Region>,
    saved: Vec<u8>,
}

impl TestLine {
    /// Each face's first point (+4) is its own number; the points lie on a wavy line.
    pub fn new(
        faces: u32,
        checks: u32,
        order: &dyn Fn(u32) -> u32,
        fill: &dyn Fn(usize, &mut [u8]),
    ) -> Self {
        use crate::service::harness::{at, capture};
        let vars = vec![
            (game_addr(CHECK_FACES_RVA) as u32, 4 * checks as usize + 4),
            at(CHECK_COUNT_RVA),
            at(POINTS_RVA),
        ];
        let saved = capture(&vars);
        let points: Vec<i32> = (0..faces as i32)
            .flat_map(|n| [n * 0x3_1000 - 0x40_0000, n * 77, (n * n * 0x900) ^ 0x1_5555])
            .collect();
        let table = crate::race::course_data::TestTable::new(
            FACES_RVA,
            crate::race::course_data::FACE_COUNT_RVA,
            0x40,
            faces,
            faces as usize,
            &|n, f| {
                fill(n, f);
                f[4..8].copy_from_slice(&(n as u32).to_le_bytes());
            },
        );
        unsafe {
            game_write(POINTS_RVA, points.as_ptr() as u32);
            for k in 0..checks {
                game_write(CHECK_FACES_RVA + 4 * k as usize, order(k));
            }
            game_write(CHECK_COUNT_RVA, checks);
        }
        Self {
            faces: table,
            points,
            vars,
            saved,
        }
    }
}

impl Drop for TestLine {
    fn drop(&mut self) {
        crate::service::harness::restore(&self.vars, &self.saved);
    }
}
