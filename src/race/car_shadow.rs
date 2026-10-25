//! The car's shadow (`sub_45754C`'s tree, IDA 2026-10-18; a reading: a dark polygon under
//! each car, the hull of its corners on the ground, cut by the course's faces). Its
//! leaves and shared code. The point records are 0x20 bytes at `unk_641AA4` (+0 x, +4 y,
//! +8 z as whole numbers, +0xC the angle, +0x10 the distance, +0x14 0), their count
//! `dword_64192C`, the hull's `dword_643ACC`.

use crate::errors::{BadInput, Refused};
use crate::framework::{game_addr, game_read, game_write, rva};

// unk_641AA4: the points, 0x20 bytes each
pub const COUNT_RVA: usize = rva(0x64192C); // dword_64192C
pub const HULL_COUNT_RVA: usize = rva(0x643ACC); // dword_643ACC
pub const FARTHEST_RVA: usize = rva(0x643ABC); // dword_643ABC: the farthest point's record
pub const FIRST_ANGLE_RVA: usize = rva(0x643AC0); // flt_643AC0
pub const GROUND_RVA: usize = rva(0x641AA0); // dword_641AA0: the ground's height
pub const PRODUCT_RVA: usize = rva(0x620460); // qword_620460
const SCRATCH_RVA: usize = rva(0x62400C); // dword_62400C

fn g(a: u32) -> u32 {
    game_addr(rva(a as usize)) as u32
}

unsafe fn get(a: u32) -> u32 {
    unsafe { (a as *const u32).read_unaligned() }
}

unsafe fn put(a: u32, v: u32) {
    unsafe { (a as *mut u32).write_unaligned(v) }
}

/// `fldpi; fadd st, st; fadd v; fstp`: 2 pi plus a float.
fn two_pi_plus(v: u32) -> u32 {
    use crate::math::x87::{Rounding, rounding};
    use crate::math::x87_float::{X, ld, st};
    // 2 x `fldpi`'s 64-bit pi plus v, rounded once: the f64 pi's double, plus what the
    // 64-bit pi has above it (565 units of 2^-62, one less rounding down or chop).
    let low = match rounding() {
        Rounding::Down | Rounding::Chop => 564.0,
        _ => 565.0,
    } * 2f64.powi(-62)
        * 2.0;
    let (hi, v) = (2.0 * std::f64::consts::PI, ld(v).0);
    let sum = hi + v;
    let back = sum - hi;
    let error = (hi - (sum - back)) + (v - back);
    st(X(sum) + X(error + low))
}

/// `race::shadow_angles` (0x4581F3): the farthest point (`dword_643ABC`) swapped into the
/// first record; each record's angle made relative to the first's (`flt_643AC0`; below 0
/// plus 2 pi), times 65536 as a whole number, +0x14 0; with two records or more sorted by
/// angle (`sub_4582AC`, `sorting::sort_records`). The registers it leaves are read by no
/// one (its one caller's next callee sets its own). Refuses (before anything) no points
/// (its loop tests at its end) or more than 0x40.
pub unsafe fn shadow_angles() -> Result<(), Refused> {
    use crate::math::x87::{mul_to_int, sub};
    unsafe {
        let count: u32 = game_read(COUNT_RVA);
        if count == 0 || count > 0x40 {
            return Err(BadInput("no shadow points or more than 0x40").into());
        }
        let records = g(0x641AA4);
        let farthest: u32 = game_read(FARTHEST_RVA);
        for k in 0..8 {
            let (a, b) = (get(records + 4 * k), get(farthest + 4 * k));
            put(records + 4 * k, b);
            put(farthest + 4 * k, a);
        }
        let first = get(records + 0xC);
        game_write(FIRST_ANGLE_RVA, first);
        let zero: u32 = game_read(rva(0x5C5D90));
        let unit: u32 = game_read(rva(0x5C5D98));
        for k in 0..count {
            let record = records + 0x20 * k;
            let mut v = sub(get(record + 0xC), first);
            if crate::math::x87_load::below_or_unordered(v, zero) {
                v = two_pi_plus(v);
            }
            game_write(SCRATCH_RVA, v);
            put(record + 0xC, mul_to_int(v, unit) as u32);
            put(record + 0x14, 0);
        }
        if count >= 2 {
            // Converted: `rc2k_core::race::quick_sort::quick_sort` (`sub_4582AC`).
            crate::race::sorting::sort_records::<8>(
                records,
                records + 0x20 * count - 0x20,
                3,
                rc2k_core::race::quick_sort::Order::Up,
            )?;
        }
        Ok(())
    }
}

/// `race::shadow_hull` (0x45836F): the first record copied after the last; then, walking
/// the points, each one where the turn from the one before to the one after is not to
/// the left (the 64-bit cross product of whole numbers, `qword_620460` its first half, not
/// negative) is taken out (the rest moved down, starting again), until none is;
/// `dword_643ACC` the points left. The registers it leaves are read by no one. Refuses
/// (before anything) fewer than two points or more than 0x40.
pub unsafe fn shadow_hull() -> Result<(), Refused> {
    // Converted: `rc2k_core::race::shadow::hull`.
    use rc2k_core::race::shadow::{Record, hull, hull_records};
    unsafe {
        let count: u32 = game_read(COUNT_RVA);
        if !(2..=0x40).contains(&count) {
            return Err(BadInput("fewer than two shadow points or more than 0x40").into());
        }
        let base = g(0x641AA4);
        let at = |n: usize| (base as usize + 0x20 * n) as *mut Record;
        let mut records: Vec<Record> = (0..hull_records(count))
            .map(|n| at(n).read_unaligned())
            .collect();
        let before = records.clone();
        let h = hull(&mut records, count)?;
        for (n, (now, was)) in records.iter().zip(&before).enumerate() {
            if now != was {
                at(n).write_unaligned(*now);
            }
        }
        game_write(PRODUCT_RVA, h.product as u64);
        game_write(HULL_COUNT_RVA, h.count);
        Ok(())
    }
}

/// The circle's texture place of corner `k` of `n` into the work entry `e` (+0x10, +0x14):
/// cos and sin of 2 pi k / n times `radius` plus `radius`, rounded to whole numbers, over
/// `unit`.
unsafe fn circle_place(e: u32, k: u32, n: u32, radius: u32, unit: u32) {
    use crate::math::x87_float::{int, ld, st, to_int};
    unsafe {
        let angle = (int(k as i32) / int(n as i32)).times_pi();
        let (sin, cos) = (angle + angle).sin_cos();
        let (radius, unit) = (ld(radius), ld(unit));
        let across = to_int(cos * radius + radius);
        let down = to_int(sin * radius + radius);
        put(e + 0x10, st(int(across) / unit));
        put(e + 0x14, st(int(down) / unit));
    }
}

/// `race::shadow_polygon` (0x45787F): the hull's points as new scene points (after the
/// scene's, from `g_scenePointCount` + 0x10: x and z the record's, y `dword_641AA0`) and as
/// the polygon's work entries (+0 the point, +0x18 white, +0x21 `dword_609328` / 2, +0x10 /
/// +0x14 a circle's texture place: cos / sin of 2 pi k / n times `flt_641994` plus it,
/// rounded, over `flt_641998`). The registers it leaves are read by no one. Refuses
/// (before anything) a hull of no points (its loop tests at its end) or more than 0x40.
pub unsafe fn shadow_polygon() -> Result<(), Refused> {
    unsafe {
        let n: u32 = game_read(HULL_COUNT_RVA);
        if n == 0 || n > 0x40 {
            return Err(BadInput("a shadow hull of no points or more than 0x40").into());
        }
        let scene: u32 = game_read(rva(0x5CCD20));
        let points = scene
            .wrapping_mul(12)
            .wrapping_add(game_read(rva(0x5CCCC0)))
            + 0xC0;
        let records = g(0x641AA4);
        let entries = g(0x975510);
        for k in 0..n {
            let (record, point) = (records + 0x20 * k, points.wrapping_add(0xC * k));
            put(point, get(record));
            put(point + 8, get(record + 8));
            put(point + 4, game_read(GROUND_RVA));
            let e = entries + 0x44 * k;
            put(e, scene.wrapping_add(0x10).wrapping_add(k));
            put(e + 0x18, 0x00FF_FFFF);
            ((e + 0x21) as *mut u8).write((game_read::<u32>(rva(0x609328)) >> 1) as u8);
            game_write(SCRATCH_RVA, k);
            circle_place(e, k, n, game_read(rva(0x641994)), game_read(rva(0x641998)));
        }
        Ok(())
    }
}

/// A scene point's address (`g_scenePoints` + 12 x n).
unsafe fn scene_point(n: u32) -> u32 {
    unsafe { n.wrapping_mul(12).wrapping_add(game_read(rva(0x5CCCC0))) }
}

/// A polygon work entry's address (`dword_975510` + 0x44 x n).
fn work_entry(n: u32) -> u32 {
    g(0x975510).wrapping_add(n.wrapping_mul(0x44))
}

/// `sub_457BE8`: whether scene point `n` is on the inner side of the edge from
/// (`dword_641948`, `dword_64194C`) to (`dword_641950`, `dword_641954`): the cross product
/// of whole numbers on the x87 (the original's instructions) above 0, or not a number
/// (`fldz; fcompp`, C0). The point's x and z into `dword_639824` / `dword_63982C`.
unsafe fn inside_edge(n: u32) -> bool {
    unsafe {
        let p = scene_point(n);
        game_write(rva(0x639824), get(p));
        game_write(rva(0x63982C), get(p + 8));
        use crate::math::x87_float::{X, compare, int};
        let word = |a: usize| int(game_read::<u32>(rva(a)) as i32);
        let first = (word(0x639824) - word(0x641948)) * (word(0x641954) - word(0x64194C));
        let second = (word(0x63982C) - word(0x64194C)) * (word(0x641950) - word(0x641948));
        // `fldz; fcompp`: above 0, or not a number (C0)
        compare(X(0.0), first - second) & 0x0100 != 0
    }
}

/// `sub_457CA8`'s arithmetic: the share of the edge's crossing (-(`dword_641978` x
/// `dword_641984` - `dword_641980` x `dword_64197C`) over the difference of the two cross
/// products, rounded to a float into `flt_639810` and used as read back), the crossing's
/// place (x and z whole numbers into `dword_623F88` / `dword_623F90`) and the entry's
/// texture floats (`flt_623FA8`, `flt_623FAC`) and two bytes (`dword_62419C`, `dword_6241AC`)
/// shared the same way between the scene points `pa`, `pb` and the entries `ea`, `eb`;
/// `dword_62400C` is left with the share's bits.
unsafe fn crossing_block(pa: u32, pb: u32, ea: u32, eb: u32) {
    use crate::math::x87_float::{int, ld, st, to_int};
    unsafe {
        let word = |a: usize| int(game_read::<u32>(rva(a)) as i32);
        let first = word(0x641984) * word(0x641978) - word(0x641980) * word(0x64197C);
        let second = word(0x64198C) * word(0x641978) - word(0x641988) * word(0x64197C);
        let share = st(-(first / (second - first)));
        game_write(rva(0x639810), share);
        let share = ld(share);
        // a + (b - a) x share
        let whole = |a: u32, b: u32| {
            to_int(int(get(a) as i32) + (int(get(b) as i32) - int(get(a) as i32)) * share)
        };
        game_write(rva(0x623F88), whole(pa, pb) as u32);
        game_write(rva(0x623F90), whole(pa + 8, pb + 8) as u32);
        let float = |a: u32, b: u32| st(ld(get(a)) + (ld(get(b)) - ld(get(a))) * share);
        game_write(rva(0x623FA8), float(ea + 0x10, eb + 0x10));
        game_write(rva(0x623FAC), float(ea + 0x14, eb + 0x14));
        let byte = |a: u32, b: u32| {
            let (a, b) = (
                int(((a) as *const u8).read().into()),
                int(((b) as *const u8).read().into()),
            );
            to_int(a + (b - a) * share) as u32
        };
        game_write(rva(0x62419C), byte(ea + 0x21, eb + 0x21));
        game_write(rva(0x6241AC), byte(ea + 0x20, eb + 0x20));
        game_write(rva(0x62400C), st(share));
    }
}

/// `sub_457CA8`: the edge's crossing between work entries `ea` and `eb` as a new scene
/// point (`dword_641990`, then counted on: x and z from the share `flt_639810`, y 0) and
/// the new work entry `ed` (its point, texture places +0x10 / +0x14 and the two bytes
/// +0x20 / +0x21 shared the same way); the original's work dwords (0x641958 .. 0x64198C,
/// `flt_623F88` ..) kept as it leaves them.
unsafe fn edge_crossing(ea: u32, eb: u32, ed: u32) {
    unsafe {
        let pa = scene_point(get(ea));
        let pb = scene_point(get(eb));
        let w = |a: u32, v: u32| game_write(rva(a as usize), v);
        let rd = |a: u32| game_read::<u32>(rva(a as usize));
        w(0x64195C, get(pb));
        w(0x64196C, get(pb + 8));
        w(0x641958, get(pa));
        w(0x641968, get(pa + 8));
        w(0x641960, rd(0x641948));
        w(0x641964, rd(0x641950));
        w(0x641970, rd(0x64194C));
        w(0x641974, rd(0x641954));
        w(0x641978, rd(0x641964).wrapping_sub(rd(0x641960)));
        w(0x64197C, rd(0x641974).wrapping_sub(rd(0x641970)));
        w(0x641980, rd(0x641958).wrapping_sub(rd(0x641960)));
        w(0x641984, rd(0x641968).wrapping_sub(rd(0x641970)));
        w(0x641988, rd(0x64195C).wrapping_sub(rd(0x641960)));
        w(0x64198C, rd(0x64196C).wrapping_sub(rd(0x641970)));
        crossing_block(pa, pb, ea, eb);
        let n = rd(0x641990);
        let p = scene_point(n);
        put(p, rd(0x623F88));
        put(p + 4, 0);
        put(p + 8, rd(0x623F90));
        put(ed, n);
        w(0x641990, n.wrapping_add(1));
        put(ed + 0x10, rd(0x623FA8));
        put(ed + 0x14, rd(0x623FAC));
        ((ed + 0x21) as *mut u8).write(rd(0x62419C) as u8);
        ((ed + 0x20) as *mut u8).write(rd(0x6241AC) as u8);
    }
}

const LIST_RVA: usize = rva(0x977710); // g_polygonList
const OLD_LIST_RVA: usize = rva(0x977810); // g_polygonOldList
const VERTEX_COUNT_RVA: usize = rva(0x6240F8); // g_polygonVertexCount
const NEXT_ENTRY_RVA: usize = rva(0x624114); // g_polygonNextEntry

/// `sub_457A85`: the polygon cut by the edge from scene point `from` (eax) to scene point
/// `to` (ebx) (kept: those on the inner side, and the crossings), with three vertices or
/// more. The registers it leaves are read by no one.
unsafe fn clip_to_edge(from: u32, to: u32) {
    unsafe {
        let count: u32 = game_read(VERTEX_COUNT_RVA);
        if count < 3 {
            return;
        }
        let (a, b) = (scene_point(from), scene_point(to));
        game_write(rva(0x641950), get(b));
        game_write(rva(0x641954), get(b + 8));
        game_write(rva(0x641948), get(a));
        game_write(rva(0x64194C), get(a + 8));
        let (list, old) = (game_addr(LIST_RVA) as u32, game_addr(OLD_LIST_RVA) as u32);
        std::ptr::copy(list as *const u8, old as *mut u8, 4 * count as usize);
        put(old + 4 * count, get(list));
        // Converted: `rc2k_core::race::shadow::clip_by_edge`.
        let mut added = 0u32;
        rc2k_core::race::shadow::clip_by_edge(
            count,
            |k| get(old + 4 * k),
            |e| inside_edge(get(work_entry(e))),
            |a, b| {
                let next: u32 = game_read(NEXT_ENTRY_RVA);
                edge_crossing(work_entry(a), work_entry(b), work_entry(next));
                game_write(NEXT_ENTRY_RVA, next.wrapping_add(1));
                next
            },
            |e| {
                put(list + 4 * added, e);
                added += 1;
            },
        );
        game_write(VERTEX_COUNT_RVA, added);
    }
}

/// `race::clip_shadow_to_face` (0x457960): the shadow's polygon cut to the course face
/// whose corners are `dword_641938` .. `dword_641940` (scene points): none (count 0) when
/// a corner is more than 0x100000 off the work car in x before one within 0x100000 in z
/// is met (or none is); else cut by its three edges in turn while three vertices are left
/// (`sub_457A85`, with `sub_457BE8` the side test and `sub_457CA8` the crossing). The
/// registers it leaves are read by no one.
pub unsafe fn clip_shadow_to_face() {
    unsafe {
        let corners = [
            game_read::<u32>(rva(0x641938)),
            game_read(rva(0x64193C)),
            game_read(rva(0x641940)),
        ];
        // Converted: `rc2k_core::race::shadow::face_near`.
        let car = g(0x719FD8);
        let found = rc2k_core::race::shadow::face_near((get(car + 0x134), get(car + 0x13C)), |n| {
            let p = scene_point(corners[n]);
            (get(p), get(p + 8))
        });
        if !found.near {
            game_write(VERTEX_COUNT_RVA, 0u32);
            return;
        }
        for (a, b) in [(0usize, 1usize), (1, 2), (2, 0)] {
            if game_read::<u32>(VERTEX_COUNT_RVA) < 3 {
                return;
            }
            clip_to_edge(corners[a], corners[b]);
        }
    }
}

/// `sub_457B86`: work entry eax's scene point raised onto the face (`dword_641938` ..):
/// the face's corners (into `dword_639890` ..) and the point (into `dword_639824` /
/// `dword_63982C`) through `physics::face_shares` and `physics::plane_height`, its y the
/// height (`dword_639828`) plus `dword_641928`. Every register kept.
unsafe fn raise_onto_face(index: u32) {
    use crate::physics::ground::{Ground, face_shares_at};
    unsafe {
        let p = scene_point(get(work_entry(index)));
        for k in 0..3 {
            game_write(
                rva(0x639890 + 4 * k),
                game_read::<u32>(rva(0x641938 + 4 * k)),
            );
        }
        game_write(rva(0x639824), get(p));
        game_write(rva(0x63982C), get(p + 8));
        face_shares_at(&Ground::game());
        crate::physics::plane_height::plane_height();
        put(
            p + 4,
            game_read::<u32>(rva(0x639828)).wrapping_add(game_read(rva(0x641928))),
        );
    }
}

/// `race::shadow_on_face` (0x45773C): the shadow's polygon (the hull's list at
/// `dword_64199C`, its count `dword_641A9C`) cut to the face `dword_641938` .. and drawn on
/// it: new points from `g_scenePointCount` + 0x30 (`dword_641990`), entries from 0x20, the
/// 0x40 point records after the scene's marked unmade; `clip_shadow_to_face`; with three
/// vertices or more each raised onto the face, projected (`scene::project_polygon`,
/// `scene::fog_polygon`), cut by depth (unless all or none are outside) and to the screen
/// and drawn (`scene::draw_polygon_in_game`) in mode 5 with texture `dword_641930`. The
/// registers it leaves are not read (its caller calls it inside `pusha`). Refuses what the projection and the
/// clips refuse (after the work is set; not met with a hull's few vertices) and a hull
/// list past 0x40.
pub unsafe fn shadow_on_face() -> Result<(), Refused> {
    unsafe {
        let count: u32 = game_read(rva(0x641A9C));
        if count > 0x40 {
            return Err(BadInput("a shadow list past 0x40").into());
        }
        game_write(rva(0x624134), 0u32);
        let list = game_addr(LIST_RVA) as u32;
        std::ptr::copy(
            g(0x64199C) as *const u8,
            list as *mut u8,
            4 * count as usize,
        );
        game_write(VERTEX_COUNT_RVA, count);
        put(list + 4 * count, game_read(rva(0x64199C)));
        game_write(NEXT_ENTRY_RVA, 0x20u32);
        let scene: u32 = game_read(rva(0x5CCD20));
        game_write(rva(0x641990), scene.wrapping_add(0x30));
        game_write(rva(0x6240E0), 0u32);
        game_write(rva(0x6240E4), 1u32);
        let records: u32 = game_read(rva(0x5C6FB8));
        let mut at = (scene & 0xFFF) * 0x20;
        for _ in 0..0x40 {
            put(records.wrapping_add(at), u32::MAX);
            at = (at + 0x20) & 0x1FFE0;
        }
        clip_shadow_to_face();
        if game_read::<u32>(VERTEX_COUNT_RVA) < 3 {
            return Ok(());
        }
        let n: u32 = game_read(VERTEX_COUNT_RVA);
        for k in 0..n {
            raise_onto_face(get(list + 4 * k));
        }
        crate::scene::project_polygon::project_polygon()?;
        crate::scene::fog_polygon::fog_polygon()?;
        game_write(rva(0x6240A4), g(0x977F10));
        let outside: u32 = game_read(rva(0x6240F4));
        if outside != 0 {
            if outside == game_read::<u32>(VERTEX_COUNT_RVA) {
                return Ok(());
            }
            crate::scene::clip_polygon_by_depth::clip_polygon_by_depth()?;
            if game_read::<u32>(VERTEX_COUNT_RVA) < 3 {
                return Ok(());
            }
        }
        crate::scene::clip_polygon_to_screen::clip_polygon_to_screen()?;
        if game_read::<u32>(VERTEX_COUNT_RVA) < 3 {
            return Ok(());
        }
        game_write(rva(0x6240BC), u32::MAX);
        game_write(rva(0x6240C0), game_read::<u32>(rva(0x641930)));
        game_write(rva(0x6240E0), 0u32);
        game_write(rva(0x6240E4), 1u32);
        game_write(rva(0x63DAB4), 5u32);
        crate::scene::draw_polygon::draw_polygon_in_game()?;
        game_write(rva(0x624134), 1u32);
        Ok(())
    }
}

/// `sub_4576C9`: each face of course record esi (+8 .. +0xC) with a kind below 0x10 and
/// without bit 7 of its +2 word gets the shadow (`shadow_on_face`) on its two triangles
/// (corners +4, +8, +0xC and +0x10, +0xC, +8); `g_currentCourseFace` each face.
unsafe fn shadow_on_record(record: u32) -> Result<(), Refused> {
    // Converted: `rc2k_core::race::shadow::{takes_shadow, SHADOW_TRIANGLES}`,
    // `rc2k_core::race::view::record_faces`.
    use rc2k_core::race::shadow::{SHADOW_TRIANGLES, takes_shadow};
    unsafe {
        let Some((first, last)) =
            rc2k_core::race::view::record_faces(get(record + 8), get(record + 0xC))
        else {
            return Ok(());
        };
        for face in first..=last {
            game_write(rva(0x5CCCD0), face);
            let at = (face << 6).wrapping_add(game_read(rva(0x5CCCBC)));
            let kind = (at as *const u8).read();
            if takes_shadow(kind, || ((at + 2) as *const u16).read_unaligned()) {
                for corners in SHADOW_TRIANGLES {
                    for (k, &o) in corners.iter().enumerate() {
                        game_write(rva(0x641938 + 4 * k), get(at + o));
                    }
                    shadow_on_face()?;
                }
            }
        }
        Ok(())
    }
}

/// `race::shadow_faces` (0x457618): the shadow (`shadow_on_face`) on the last course
/// record's faces, then on every record in the 3 x 3 cells round the work car
/// (`sub_4576C9` each). The original keeps every register but ebx (the record count),
/// which its one caller does not read.
pub unsafe fn shadow_faces() -> Result<(), Refused> {
    unsafe {
        let count: u32 = game_read(rva(0x5CCD18));
        let records: u32 = game_read(rva(0x5CCCB8));
        shadow_on_record((count.wrapping_sub(1) << 5).wrapping_add(records))?;
        if count == 0 {
            return Ok(());
        }
        // Converted: `rc2k_core::race::shadow::{record_cell, cell_near_car}`.
        use rc2k_core::race::shadow::{cell_near_car, record_cell};
        let car = g(0x719FD8);
        for n in 0..count {
            let at = (n << 5).wrapping_add(records);
            let Some((x, z)) = record_cell(get(at)) else {
                continue;
            };
            game_write(rva(0x5CCBF0), x);
            game_write(rva(0x5CCBF4), z);
            if cell_near_car((get(car + 0x134), get(car + 0x13C)), (x, z)) {
                shadow_on_record(at)?;
            }
        }
        Ok(())
    }
}

/// `fld tbyte; fstp dword`: 80 bits as a float.
fn tbyte_to_float(t: [u8; 10]) -> u32 {
    crate::math::x87_float::st(crate::math::x87_float::from_f80(t))
}

/// The sun's slant from `sun` and `turn` (the angles as 128ths of a half turn): `flt_643AC4`
/// = cos(turn) x cot(sun), `flt_643AC8` = sin(turn) x cot(sun); `dword_62400C` is left 0x80.
unsafe fn sun_slant(sun: u32, turn: u32) {
    use crate::math::x87_float::{int, st};
    unsafe {
        let cot = {
            let (sin, cos) = (int(sun as i32) / int(0x80)).times_pi().sin_cos();
            cos / sin
        };
        let (sin, cos) = (int(turn as i32) / int(0x80)).times_pi().sin_cos();
        game_write(rva(0x643AC4), st(cos * cot));
        game_write(rva(0x643AC8), st(cot * sin));
        game_write(rva(0x62400C), 0x80u32);
    }
}

/// `fld x; fld z; fpatan; fstp`: the angle of the corner at `at` (x at +0, z at +8).
unsafe fn angle_of(at: u32) -> u32 {
    use crate::math::x87_float::{ld, st};
    unsafe { st(ld(get(at)).atan2(ld(get(at + 8)))) }
}

/// `race::shadow_points` (0x458078): the shadow's points: the ground under the work car
/// (`physics::find_ground`, its height to `dword_641AA0`); the sun's slant from
/// `dword_60933C` (at least 0x10) and -`dword_609338` - 0x40 (`flt_643AC4`, `flt_643AC8`);
/// each of the car's `dword_64192C` corners (`g_car` +0xAF0 .. +0xCF0 by axis) turned by
/// its matrix (+0x840), moved along the slant by its height plus 0.7, its angle (`fpatan`)
/// and distance, the farthest kept (`flt_643AB8`, `dword_643ABC`), as whole numbers (x
/// 65536) plus the car's place. Refuses what `find_ground` refuses (before anything), and
/// no corners or more than 0x40.
pub unsafe fn shadow_points() -> Result<(), Refused> {
    use crate::math::x87::mul_to_int;
    use crate::math::x87_load::{below_or_unordered, to_tbyte};
    unsafe {
        let count: u32 = game_read(COUNT_RVA);
        if count == 0 || count > 0x40 {
            return Err(BadInput("no shadow corners or more than 0x40").into());
        }
        let car = g(0x719FD8);
        let inputs = [rva(0x639824), rva(0x63982C), rva(0x5CCC78)];
        let was: Vec<u32> = inputs.iter().map(|&a| game_read::<u32>(a)).collect();
        game_write(inputs[0], get(car + 0x134));
        game_write(inputs[1], get(car + 0x13C));
        game_write(inputs[2], get(car + 0xE00));
        if let Err(e) = crate::physics::find_ground::find_ground() {
            // Its inputs put back: a refusal has written nothing.
            for (&a, &v) in inputs.iter().zip(&was) {
                game_write(a, v);
            }
            return Err(e);
        }
        game_write(rva(0x643AB8), (-1.0f32).to_bits());
        game_write(GROUND_RVA, game_read::<u32>(rva(0x639828)));
        let sun = game_read::<u32>(rva(0x60933C)).max(0x10);
        let turn = game_read::<u32>(rva(0x609338))
            .wrapping_neg()
            .wrapping_sub(0x40);
        sun_slant(sun, turn);
        let (sx, sz) = (
            game_read::<u32>(rva(0x643AC4)),
            game_read::<u32>(rva(0x643AC8)),
        );
        let lift: u32 = game_read(rva(0x613FC0));
        let unit: u32 = game_read(rva(0x5C5D98));
        let matrix = car + 0x840;
        let mut at = g(0x641AA4);
        for k in 0..count {
            let p = [
                to_tbyte(get(car + 0xAF0 + 4 * k)),
                to_tbyte(get(car + 0xBF0 + 4 * k)),
                to_tbyte(get(car + 0xCF0 + 4 * k)),
            ];
            let t = crate::scene::rotate_point::rotate_point(matrix, p);
            let [x, y, z] = t.map(tbyte_to_float);
            put(at + 4, y);
            let x = lifted(y, lift, sx, x);
            put(at, x);
            let z = lifted(y, lift, sz, z);
            put(at + 8, z);
            let angle = angle_of(at);
            put(at + 0xC, angle);
            let length = crate::math::x87_load::float_hypot(x, z);
            put(at + 0x10, length);
            if !below_or_unordered(length, game_read(rva(0x643AB8))) {
                game_write(rva(0x643AB8), length);
                game_write(FARTHEST_RVA, at);
            }
            put(
                at,
                (mul_to_int(x, unit) as u32).wrapping_add(get(car + 0x134)),
            );
            put(
                at + 8,
                (mul_to_int(z, unit) as u32).wrapping_add(get(car + 0x13C)),
            );
            put(at + 4, game_read(GROUND_RVA));
            at += 0x20;
        }
        Ok(())
    }
}

/// `fld y; fadd lift; fmul slant; fadd base; fstp`: a corner moved along the sun's slant,
/// rounded once.
fn lifted(y: u32, lift: u32, slant: u32, base: u32) -> u32 {
    use crate::math::x87_float::{ld, st};
    st((ld(y) + ld(lift)) * ld(slant) + ld(base))
}

/// `race::cast_car_shadow` (0x4575C1): with three corners or more, the shadow's points, their
/// order, hull and polygon, the hull's list (`dword_64199C`, `dword_641A9C`), texture 0xC3
/// (`dword_641930`), and the shadow on the faces round the car (`shadow_faces`). The
/// registers it leaves are read by no one (its caller calls it inside `pusha`); it reads
/// none at entry (each part sets what it uses). Refuses what its parts refuse (before
/// anything for the points and the counts).
pub unsafe fn cast_car_shadow() -> Result<(), Refused> {
    // Converted: `rc2k_core::race::shadow::{casts, hull_drawn, hull_list,
    // SHADOW_TEXTURE}`.
    use rc2k_core::race::shadow::{SHADOW_TEXTURE, casts, hull_drawn, hull_list};
    unsafe {
        if !casts(game_read(COUNT_RVA)) {
            return Ok(());
        }
        shadow_points()?;
        shadow_angles()?;
        shadow_hull()?;
        if !hull_drawn(game_read(HULL_COUNT_RVA)) {
            return Ok(());
        }
        shadow_polygon()?;
        let n: u32 = game_read(HULL_COUNT_RVA);
        game_write(rva(0x641A9C), n);
        for k in hull_list(n) {
            put(g(0x64199C) + 4 * k, k);
        }
        game_write(rva(0x641930), SHADOW_TEXTURE);
        shadow_faces()
    }
}

/// `race::cast_car_shadows` (0x45754C): the depth test off, `dword_641928` 0x1999, then
/// for each race car with +0x6FC not 0: loaded into the work copy (`drive::load_car`), its
/// corner count from `g_car` +0x260, `cast_car_shadow` once with `dword_6238B0` set, then
/// once more with `dword_60933C` 0x40 (put back after), inside `pusha`; the depth test on.
/// Its one caller, `race::draw_view_scene`, reads no register it leaves. Refuses (before anything) no cars or more than eight, or a null block.
pub unsafe fn cast_car_shadows() -> Result<(), Refused> {
    // Converted: `rc2k_core::race::shadow::{check_shadow_cars, casts_shadow, first_pass,
    // SHADOW_STRENGTH, SUN_PASS}`.
    use rc2k_core::race::shadow::{
        SHADOW_STRENGTH, SUN_PASS, casts_shadow, check_shadow_cars, first_pass,
    };
    unsafe {
        let cars: u32 = game_read(rva(0x63DB90));
        let block = |n: u32| game_read::<u32>(rva(0x71B3B8 + 4 * n as usize));
        check_shadow_cars(cars, block)?;
        crate::scene::depth_test_off::depth_test_off();
        game_write(rva(0x641928), SHADOW_STRENGTH);
        for n in 0..cars {
            let b = block(n);
            if !casts_shadow(get(b + 0x6FC)) {
                continue;
            }
            game_write(rva(0x71B3F8), n);
            crate::drive::load_car::load_car()?;
            game_write(COUNT_RVA, get(g(0x719FD8) + 0x260));
            if first_pass(game_read(rva(0x6238B0))) {
                cast_car_shadow()?;
            }
            let sun: u32 = game_read(rva(0x60933C));
            game_write(rva(0x60933C), SUN_PASS);
            cast_car_shadow()?;
            game_write(rva(0x60933C), sun);
        }
        crate::scene::depth_test_on::depth_test_on();
        Ok(())
    }
}

#[cfg(test)]
mod asm_reference {
    use super::*;

    // The original blocks, verbatim.
    pub(super) fn two_pi_plus_asm(v: u32) -> u32 {
        let (x, mut out) = ([v], 0u32);
        unsafe {
            core::arch::asm!(
                "fldpi",
                ".byte 0xD8, 0xC0",
                "fadd dword ptr [{x}]",
                "fstp dword ptr [{o}]",
                x = in(reg) x.as_ptr(),
                o = in(reg) &raw mut out,
                out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
                out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
                options(nostack),
            );
        }
        out
    }

    pub(super) fn tbyte_to_float_asm(t: [u8; 10]) -> u32 {
        let mut out = 0u32;
        unsafe {
            core::arch::asm!(
                "fld tbyte ptr [{t}]",
                "fstp dword ptr [{o}]",
                t = in(reg) t.as_ptr(),
                o = in(reg) &raw mut out,
                out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
                out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
                options(nostack),
            );
        }
        out
    }

    pub(super) fn lifted_asm(y: u32, lift: u32, slant: u32, base: u32) -> u32 {
        let (v, mut out) = ([y, lift, slant, base], 0u32);
        unsafe {
            core::arch::asm!(
                "fld dword ptr [{v}]",
                "fadd dword ptr [{v} + 4]",
                "fmul dword ptr [{v} + 8]",
                "fadd dword ptr [{v} + 12]",
                "fstp dword ptr [{o}]",
                v = in(reg) v.as_ptr(),
                o = in(reg) &raw mut out,
                out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
                out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
                options(nostack),
            );
        }
        out
    }

    pub(super) unsafe fn circle_place_asm(e: u32, k: u32, n: u32, radius: u32, unit: u32) {
        let v = [k, n, radius, unit];
        unsafe {
            core::arch::asm!(
                "fild dword ptr [{v}]",
                "fild dword ptr [{v} + 4]",
                ".byte 0xDE, 0xF9",
                "fldpi",
                ".byte 0xDE, 0xC9",
                ".byte 0xD8, 0xC0",
                "fsincos",
                "fmul dword ptr [{v} + 8]",
                "fadd dword ptr [{v} + 8]",
                "fistp dword ptr [{e} + 0x10]",
                "fmul dword ptr [{v} + 8]",
                "fadd dword ptr [{v} + 8]",
                "fistp dword ptr [{e} + 0x14]",
                "fild dword ptr [{e} + 0x10]",
                "fdiv dword ptr [{v} + 12]",
                "fstp dword ptr [{e} + 0x10]",
                "fild dword ptr [{e} + 0x14]",
                "fdiv dword ptr [{v} + 12]",
                "fstp dword ptr [{e} + 0x14]",
                v = in(reg) v.as_ptr(),
                e = in(reg) e,
                out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
                out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
                options(nostack),
            );
        }
    }

    pub(super) unsafe fn inside_edge_asm(n: u32) -> bool {
        unsafe {
            let p = scene_point(n);
            game_write(rva(0x639824), get(p));
            game_write(rva(0x63982C), get(p + 8));
            let status: u16;
            core::arch::asm!(
                "fild dword ptr [{rc} + 0x239824]",
                "fild dword ptr [{rc} + 0x241948]",
                ".byte 0xDE, 0xE9",
                "fild dword ptr [{rc} + 0x241954]",
                "fild dword ptr [{rc} + 0x24194C]",
                ".byte 0xDE, 0xE9",
                ".byte 0xDE, 0xC9",
                "fild dword ptr [{rc} + 0x23982C]",
                "fild dword ptr [{rc} + 0x24194C]",
                ".byte 0xDE, 0xE9",
                "fild dword ptr [{rc} + 0x241950]",
                "fild dword ptr [{rc} + 0x241948]",
                ".byte 0xDE, 0xE9",
                ".byte 0xDE, 0xC9",
                ".byte 0xDE, 0xE9",
                ".byte 0xD9, 0xEE",
                ".byte 0xDE, 0xD9",
                "fnstsw ax",
                rc = in(reg) crate::race::store::asm_base(),
                out("ax") status,
                out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
                out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
                options(nostack),
            );
            status & 0x0100 != 0
        }
    }

    pub(super) unsafe fn crossing_block_asm(pa: u32, pb: u32, ea: u32, eb: u32) {
        unsafe {
            core::arch::asm!(
                "fild dword ptr [{rc} + 0x241978]",
                "fild dword ptr [{rc} + 0x24197C]",
                "fild dword ptr [{rc} + 0x241980]",
                "fild dword ptr [{rc} + 0x241984]",
                ".byte 0xD8, 0xCB",
                ".byte 0xD9, 0xC9",
                ".byte 0xD8, 0xCA",
                ".byte 0xDE, 0xE9",
                "fild dword ptr [{rc} + 0x241988]",
                "fild dword ptr [{rc} + 0x24198C]",
                ".byte 0xD8, 0xCC",
                ".byte 0xD9, 0xC9",
                ".byte 0xD8, 0xCB",
                ".byte 0xDE, 0xE9",
                ".byte 0xD8, 0xE1",
                ".byte 0xDE, 0xF9",
                ".byte 0xD9, 0xE0",
                "fstp dword ptr [{rc} + 0x239810]",
                "fstp dword ptr [{rc} + 0x22400C]",
                "fstp dword ptr [{rc} + 0x22400C]",
                "fld dword ptr [{rc} + 0x239810]",
                "fild dword ptr [{pa}]",
                "fild dword ptr [{pb}]",
                ".byte 0xD8, 0xE1",
                ".byte 0xD8, 0xCA",
                ".byte 0xDE, 0xC1",
                "fistp dword ptr [{rc} + 0x223F88]",
                "fild dword ptr [{pa} + 8]",
                "fild dword ptr [{pb} + 8]",
                ".byte 0xD8, 0xE1",
                ".byte 0xD8, 0xCA",
                ".byte 0xDE, 0xC1",
                "fistp dword ptr [{rc} + 0x223F90]",
                "fld dword ptr [{ea} + 0x10]",
                "fld dword ptr [{eb} + 0x10]",
                ".byte 0xD8, 0xE1",
                ".byte 0xD8, 0xCA",
                ".byte 0xDE, 0xC1",
                "fstp dword ptr [{rc} + 0x223FA8]",
                "fld dword ptr [{ea} + 0x14]",
                "fld dword ptr [{eb} + 0x14]",
                ".byte 0xD8, 0xE1",
                ".byte 0xD8, 0xCA",
                ".byte 0xDE, 0xC1",
                "fstp dword ptr [{rc} + 0x223FAC]",
                "xor eax, eax",
                "mov al, byte ptr [{ea} + 0x21]",
                "mov dword ptr [{rc} + 0x22400C], eax",
                "fild dword ptr [{rc} + 0x22400C]",
                "mov al, byte ptr [{eb} + 0x21]",
                "mov dword ptr [{rc} + 0x22400C], eax",
                "fild dword ptr [{rc} + 0x22400C]",
                ".byte 0xD8, 0xE1",
                ".byte 0xD8, 0xCA",
                ".byte 0xDE, 0xC1",
                "fistp dword ptr [{rc} + 0x22419C]",
                "xor eax, eax",
                "mov al, byte ptr [{ea} + 0x20]",
                "mov dword ptr [{rc} + 0x22400C], eax",
                "fild dword ptr [{rc} + 0x22400C]",
                "mov al, byte ptr [{eb} + 0x20]",
                "mov dword ptr [{rc} + 0x22400C], eax",
                "fild dword ptr [{rc} + 0x22400C]",
                ".byte 0xD8, 0xE1",
                ".byte 0xD8, 0xCA",
                ".byte 0xDE, 0xC1",
                "fistp dword ptr [{rc} + 0x2241AC]",
                "fstp dword ptr [{rc} + 0x22400C]",
                rc = in(reg) crate::race::store::asm_base(),
                pa = in(reg) pa,
                pb = in(reg) pb,
                ea = in(reg) ea,
                eb = in(reg) eb,
                out("eax") _,
                out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
                out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
                options(nostack),
            );
        }
    }

    pub(super) unsafe fn sun_slant_asm(sun: u32, turn: u32) {
        unsafe {
            let v = [sun, 0x80u32, turn];
            core::arch::asm!(
                "mov eax, dword ptr [{v}]",
                "mov dword ptr [{rc} + 0x22400C], eax",
                "fild dword ptr [{rc} + 0x22400C]",
                "mov dword ptr [{rc} + 0x22400C], 0x80",
                "fild dword ptr [{rc} + 0x22400C]",
                ".byte 0xDE, 0xF9",
                "fldpi",
                ".byte 0xDE, 0xC9",
                "fsincos",
                ".byte 0xDE, 0xF1",
                "mov eax, dword ptr [{v} + 8]",
                "mov dword ptr [{rc} + 0x22400C], eax",
                "fild dword ptr [{rc} + 0x22400C]",
                "mov dword ptr [{rc} + 0x22400C], 0x80",
                "fild dword ptr [{rc} + 0x22400C]",
                ".byte 0xDE, 0xF9",
                "fldpi",
                ".byte 0xDE, 0xC9",
                "fsincos",
                ".byte 0xD8, 0xCA",
                "fstp dword ptr [{rc} + 0x243AC4]",
                ".byte 0xDE, 0xC9",
                "fstp dword ptr [{rc} + 0x243AC8]",
                rc = in(reg) crate::race::store::asm_base(),
                v = in(reg) v.as_ptr(),
                out("eax") _,
                out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
                out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
                options(nostack),
            );
        }
    }

    pub(super) unsafe fn angle_asm(at: u32) -> u32 {
        unsafe {
            let mut angle = 0u32;
            core::arch::asm!(
                "fld dword ptr [{a}]",
                "fld dword ptr [{a} + 8]",
                "fpatan",
                "fstp dword ptr [{o}]",
                a = in(reg) at,
                o = in(reg) &raw mut angle,
                out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
                out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
                options(nostack),
            );
            angle
        }
    }

    use crate::race::asm_test::{Rng, Saved, at_each_control_word, lock, same, within_ulps};

    #[test]
    fn two_pi_plus_matches() {
        let _l = lock();
        let mut r = Rng(21);
        at_each_control_word(|| {
            for n in 0..40000 {
                let v = if n % 2 == 0 {
                    // the angles it is used for: below 0, above -2 pi
                    (-(f64::from(r.u32()) / 4294967296.0 * std::f64::consts::TAU) as f32).to_bits()
                } else {
                    r.float()
                };
                let (want, got) = (two_pi_plus_asm(v), two_pi_plus(v));
                assert!(same(want, got), "{v:#x}: {want:#x} {got:#x}");
            }
        });
    }

    #[test]
    fn float_blocks_match() {
        let _l = lock();
        let mut r = Rng(22);
        at_each_control_word(|| {
            for _ in 0..20000 {
                let t = crate::math::x87_float::f80(crate::math::x87_float::X(
                    f64::from(f32::from_bits(r.float())) * f64::from(f32::from_bits(r.float())),
                ));
                let (want, got) = (tbyte_to_float_asm(t), tbyte_to_float(t));
                assert!(same(want, got), "{t:x?}: {want:#x} {got:#x}");
                let (y, l, sl, b) = (r.float(), r.float(), r.float(), r.float());
                let (want, got) = (lifted_asm(y, l, sl, b), lifted(y, l, sl, b));
                assert!(
                    same(want, got),
                    "{y:#x} {l:#x} {sl:#x} {b:#x}: {want:#x} {got:#x}"
                );
            }
        });
    }

    #[test]
    fn circle_place_matches() {
        let _l = lock();
        let mut r = Rng(23);
        let mut worst = 0;
        at_each_control_word(|| {
            for _ in 0..20000 {
                let n = 3 + r.below(0x3E);
                let k = r.below(n);
                let radius = r.float();
                let unit = r.float();
                let (mut a, mut b) = ([0xAAAA_AAAAu32; 8], [0xAAAA_AAAAu32; 8]);
                unsafe {
                    circle_place_asm(a.as_mut_ptr() as u32, k, n, radius, unit);
                    circle_place(b.as_mut_ptr() as u32, k, n, radius, unit);
                }
                for w in [4, 5] {
                    assert!(
                        same(a[w], b[w]),
                        "{k}/{n} {radius:#x} {unit:#x}: {:#x} {:#x}",
                        a[w],
                        b[w]
                    );
                    worst = worst.max(a[w].abs_diff(b[w]));
                }
                assert_eq!(a[..4], b[..4]);
                assert_eq!(a[6..], b[6..]);
            }
        });
        eprintln!("circle_place: worst difference {worst} ulps");
    }

    #[test]
    fn inside_edge_matches() {
        let _l = lock();
        let places = [
            0x639824, 0x63982C, 0x641948, 0x64194C, 0x641950, 0x641954, 0x5CCCC0,
        ];
        let _saved = Saved::new(&places);
        let mut r = Rng(24);
        let mut points = [0u32; 12];
        let (mut yes, mut no) = (0, 0);
        at_each_control_word(|| {
            for _ in 0..20000 {
                for q in points.iter_mut() {
                    *q = r.int() as u32;
                }
                unsafe {
                    game_write(rva(0x5CCCC0), points.as_ptr() as u32);
                    for a in &places[2..6] {
                        game_write(rva(*a), r.int() as u32);
                    }
                    game_write(rva(0x639824), 0xAAAA_AAAAu32);
                    game_write(rva(0x63982C), 0xAAAA_AAAAu32);
                }
                let n = r.below(4);
                let want = unsafe { inside_edge_asm(n) };
                let seen = unsafe {
                    (
                        game_read::<u32>(rva(0x639824)),
                        game_read::<u32>(rva(0x63982C)),
                    )
                };
                unsafe {
                    game_write(rva(0x639824), 0xAAAA_AAAAu32);
                    game_write(rva(0x63982C), 0xAAAA_AAAAu32);
                }
                let got = unsafe { inside_edge(n) };
                assert_eq!(want, got, "{points:x?} {n}");
                assert_eq!(seen, unsafe {
                    (
                        game_read::<u32>(rva(0x639824)),
                        game_read::<u32>(rva(0x63982C)),
                    )
                });
                if want { yes += 1 } else { no += 1 }
            }
        });
        assert!(yes > 1000 && no > 1000, "{yes} {no}");
    }

    #[test]
    fn crossing_block_matches() {
        let _l = lock();
        let places = [
            0x62400C, 0x641978, 0x64197C, 0x641980, 0x641984, 0x641988, 0x64198C, 0x639810,
            0x623F88, 0x623F90, 0x623FA8, 0x623FAC, 0x62419C, 0x6241AC,
        ];
        let _saved = Saved::new(&places);
        let mut r = Rng(25);
        at_each_control_word(|| {
            for _ in 0..20000 {
                let small = r.below(2) == 0;
                for a in &places[1..7] {
                    let v = if small {
                        r.below(200) as i32 - 100
                    } else {
                        r.int()
                    };
                    unsafe { game_write(rva(*a), v as u32) };
                }
                let point = |r: &mut Rng| [r.int() as u32, r.u32(), r.int() as u32];
                let (pa, pb) = (point(&mut r), point(&mut r));
                let entry = |r: &mut Rng| {
                    let mut e = [0u32; 10];
                    for (k, q) in e.iter_mut().enumerate() {
                        *q = if (4..6).contains(&k) {
                            r.float()
                        } else {
                            r.u32()
                        };
                    }
                    e
                };
                let (ea, eb) = (entry(&mut r), entry(&mut r));
                let args = (
                    pa.as_ptr() as u32,
                    pb.as_ptr() as u32,
                    ea.as_ptr() as u32,
                    eb.as_ptr() as u32,
                );
                let read = || places.map(|a| unsafe { game_read::<u32>(rva(a)) });
                for a in &places[7..] {
                    unsafe { game_write(rva(*a), 0xAAAA_AAAAu32) };
                }
                unsafe { crossing_block_asm(args.0, args.1, args.2, args.3) };
                let want = read();
                for a in &places[7..] {
                    unsafe { game_write(rva(*a), 0xAAAA_AAAAu32) };
                }
                unsafe { game_write(rva(0x62400C), 0xAAAA_AAAAu32) };
                unsafe { crossing_block(args.0, args.1, args.2, args.3) };
                let got = read();
                for k in 0..places.len() {
                    assert!(
                        same(want[k], got[k]),
                        "place {:#x}: {:#x} {:#x} (inputs {:x?})",
                        places[k],
                        want[k],
                        got[k],
                        &want[1..7]
                    );
                }
            }
        });
    }

    #[test]
    fn sun_slant_and_angle_match() {
        let _l = lock();
        let _saved = Saved::new(&[0x62400C, 0x643AC4, 0x643AC8]);
        let mut r = Rng(26);
        let mut worst = 0;
        at_each_control_word(|| {
            for _ in 0..20000 {
                let sun = 0x10 + r.below(0x400);
                let turn = (r.below(0x800) as i32 - 0x400) as u32;
                // At a multiple of a quarter turn (sun 0x40, ...) the sine or the cosine is
                // about 1e-16: the x87 reduces by a 66-bit pi, libm exactly, so those few
                // digits differ (flt_643AC4 and flt_643AC8 are then about 1e-17 or 1e16 in
                // the original, whatever the processor). Left out here, reported.
                if sun.is_multiple_of(64) || turn.is_multiple_of(64) {
                    continue;
                }
                unsafe { sun_slant_asm(sun, turn) };
                let want =
                    unsafe { [0x62400C, 0x643AC4, 0x643AC8].map(|a| game_read::<u32>(rva(a))) };
                unsafe {
                    for a in [0x62400C, 0x643AC4, 0x643AC8] {
                        game_write(rva(a), 0xAAAA_AAAAu32);
                    }
                    sun_slant(sun, turn);
                }
                let got =
                    unsafe { [0x62400C, 0x643AC4, 0x643AC8].map(|a| game_read::<u32>(rva(a))) };
                assert_eq!(want[0], got[0]);
                for k in 1..3 {
                    assert!(
                        within_ulps(want[k], got[k], 1),
                        "{sun:#x} {turn:#x}: {:#x} {:#x}",
                        want[k],
                        got[k]
                    );
                    worst = worst.max(want[k].abs_diff(got[k]));
                }
                // the angle
                let at = [r.float(), r.float(), r.float()];
                let (want, got) =
                    unsafe { (angle_asm(at.as_ptr() as u32), angle_of(at.as_ptr() as u32)) };
                assert!(within_ulps(want, got, 1), "{at:x?}: {want:#x} {got:#x}");
                worst = worst.max(want.abs_diff(got));
            }
        });
        eprintln!("sun_slant, angle_of: worst difference {worst} ulps");
    }
}
