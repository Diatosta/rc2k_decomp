//! The car's shadow (`sub_45754C`'s tree, IDA 2026-10-18; a reading: a dark polygon under
//! each car, the hull of its corners on the ground, cut by the course's faces). Its
//! leaves and shared code. The point records are 0x20 bytes at `unk_641AA4` (+0 x, +4 y,
//! +8 z as whole numbers, +0xC the angle, +0x10 the distance, +0x14 0), their count
//! `dword_64192C`, the hull's `dword_643ACC`.

use crate::errors::{BadInput, Refused};
use crate::physics::car_store::CAR;
use crate::physics::ground_probe_store::GROUND_PROBE;
use crate::physics::ground_store::GROUND_WORK;
use crate::physics::tables_store::PHYSICS;
use crate::pointers::GamePtr;
use crate::race::course_store::COURSE;
use crate::race::frame_store::FRAME_WORK;
use crate::race::numbers_store::NUMBERS;
use crate::race::prefs_store::STAGE_PREFS;
use crate::race::shadow_skids_store::SHADOW_SKIDS;
use crate::race::trap::Trap;
use crate::race::vars_store::RACE_VARS;
use crate::race::view_store::RACE_VIEW;
use crate::race::work_store::RACE_WORK;
use crate::scene::draw_store::DRAW;
use crate::scene::polygon_store::{POLYGON_WORK, PolygonEntry};
use crate::session::{Byte, Dword, copy_on, read_on, write_on};

// unk_641AA4: the points, 0x20 bytes each

fn get(a: u32) -> Result<u32, BadInput> {
    crate::race::course_places::read(a, "a shadow point or entry")
}

fn put(a: u32, v: u32) -> Result<(), BadInput> {
    crate::race::course_places::write(a, v, "a shadow point or entry")
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
pub fn shadow_angles() -> Result<(), Refused> {
    use crate::math::x87::{mul_to_int, sub};
    {
        let count: u32 = FRAME_WORK.shadow_point_count.bits();
        if count == 0 || count > 0x40 {
            return Err(BadInput("no shadow points or more than 0x40").into());
        }
        let records = SHADOW_SKIDS.shadow_points.addr();
        let farthest: u32 = SHADOW_SKIDS.farthest.get();
        for k in 0..8 {
            let (a, b) = (get(records + 4 * k)?, get(farthest + 4 * k)?);
            put(records + 4 * k, b)?;
            put(farthest + 4 * k, a)?;
        }
        let first = get(records + 0xC)?;
        SHADOW_SKIDS.first_angle.set_bits(first);
        let zero: u32 = NUMBERS.zero.bits();
        let unit: u32 = NUMBERS.k65536.bits();
        for k in 0..count {
            let record = records + 0x20 * k;
            let mut v = sub(get(record + 0xC)?, first);
            if crate::math::x87_load::below_or_unordered(v, zero) {
                v = two_pi_plus(v);
            }
            DRAW.scratch.set_bits(v);
            put(record + 0xC, mul_to_int(v, unit) as u32)?;
            put(record + 0x14, 0)?;
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
pub fn shadow_hull() -> Result<(), Refused> {
    // Converted: `rc2k_core::race::shadow::hull`.
    use rc2k_core::race::shadow::{Record, hull, hull_records};
    {
        let count: u32 = FRAME_WORK.shadow_point_count.bits();
        if !(2..=0x40).contains(&count) {
            return Err(BadInput("fewer than two shadow points or more than 0x40").into());
        }
        let at = |n: usize| GamePtr::to(&SHADOW_SKIDS.shadow_points).offset(0x20 * n as u32);
        let mut records: Vec<Record> = Vec::new();
        for n in 0..hull_records(count) {
            records.push(at(n).read::<Record>("a shadow hull record")?);
        }
        let before = records.clone();
        let h = hull(&mut records, count)?;
        for (n, (now, was)) in records.iter().zip(&before).enumerate() {
            if now != was {
                at(n).write(*now, "a shadow hull record")?;
            }
        }
        GROUND_PROBE.product[0].set(h.product as u32);
        GROUND_PROBE.product[1].set((h.product as u64 >> 32) as u32);
        SHADOW_SKIDS.hull_count.set(h.count);
        Ok(())
    }
}

/// The circle's texture place of corner `k` of `n` into the work entry `e` (+0x10, +0x14):
/// cos and sin of 2 pi k / n times `radius` plus `radius`, rounded to whole numbers, over
/// `unit`.
fn circle_place(e: &PolygonEntry, k: u32, n: u32, radius: u32, unit: u32) {
    use crate::math::x87_float::{int, ld, st, to_int};
    {
        let angle = (int(k as i32) / int(n as i32)).times_pi();
        let (sin, cos) = (angle + angle).sin_cos();
        let (radius, unit) = (ld(radius), ld(unit));
        let across = to_int(cos * radius + radius);
        let down = to_int(sin * radius + radius);
        e.u2.set(st(int(across) / unit));
        e.v2.set(st(int(down) / unit));
    }
}

/// `race::shadow_polygon` (0x45787F): the hull's points as new scene points (after the
/// scene's, from `g_scenePointCount` + 0x10: x and z the record's, y `dword_641AA0`) and as
/// the polygon's work entries (+0 the point, +0x18 white, +0x21 `dword_609328` / 2, +0x10 /
/// +0x14 a circle's texture place: cos / sin of 2 pi k / n times `flt_641994` plus it,
/// rounded, over `flt_641998`). The registers it leaves are read by no one. Refuses
/// (before anything) a hull of no points (its loop tests at its end) or more than 0x40.
pub fn shadow_polygon() -> Result<(), Refused> {
    {
        let n: u32 = SHADOW_SKIDS.hull_count.get();
        if n == 0 || n > 0x40 {
            return Err(BadInput("a shadow hull of no points or more than 0x40").into());
        }
        let scene: u32 = COURSE.scene_point_count.get();
        let points = scene
            .wrapping_mul(12)
            .wrapping_add(COURSE.scene_points.bits())
            + 0xC0;
        let records = GamePtr::to(&SHADOW_SKIDS.shadow_points);
        for k in 0..n {
            let (record, point) = (records.offset(0x20 * k), points.wrapping_add(0xC * k));
            let field = |offset| record.offset(offset).read::<u32>("a shadow point");
            crate::race::scene_points::put(point, field(0)?)?;
            crate::race::scene_points::put(point + 8, field(8)?)?;
            crate::race::scene_points::put(point + 4, FRAME_WORK.shadow_ground.bits())?;
            let e = &POLYGON_WORK.entries[k as usize];
            e.point.set(scene.wrapping_add(0x10).wrapping_add(k));
            e.light.set(0x00FF_FFFFu32);
            e.amount.set((STAGE_PREFS.shadow_alpha.get() >> 1) as u8);
            DRAW.scratch.set_bits(k);
            circle_place(
                e,
                k,
                n,
                FRAME_WORK.shadow_circle[0].bits(),
                FRAME_WORK.shadow_circle[1].bits(),
            );
        }
        Ok(())
    }
}

/// A scene point's address (`g_scenePoints` + 12 x n).
fn scene_point(n: u32) -> u32 {
    n.wrapping_mul(12).wrapping_add(COURSE.scene_points.bits())
}

/// A polygon work entry's address (`dword_975510` + 0x44 x n).
fn work_entry(n: u32) -> Result<&'static PolygonEntry, BadInput> {
    POLYGON_WORK
        .entries
        .get(n as usize)
        .ok_or(BadInput("a polygon work entry past 0x80"))
}

/// `sub_457BE8`: whether scene point `n` is on the inner side of the edge from
/// (`dword_641948`, `dword_64194C`) to (`dword_641950`, `dword_641954`): the cross product
/// of whole numbers on the x87 (the original's instructions) above 0, or not a number
/// (`fldz; fcompp`, C0). The point's x and z into `dword_639824` / `dword_63982C`.
fn inside_edge(n: u32) -> Result<bool, BadInput> {
    {
        let p = scene_point(n);
        GROUND_WORK
            .ground_x
            .set(crate::race::scene_points::word(p)?);
        GROUND_WORK
            .ground_z
            .set(crate::race::scene_points::word(p + 8)?);
        use crate::math::x87_float::{X, compare, int};
        let word = |d: &crate::session::Dword| int(d.bits() as i32);
        let (x, z) = (&GROUND_WORK.ground_x, &GROUND_WORK.ground_z);
        let c = &FRAME_WORK.shadow_corners;
        let first = (word(x) - word(&c[0])) * (word(&c[3]) - word(&c[1]));
        let second = (word(z) - word(&c[1])) * (word(&c[2]) - word(&c[0]));
        // `fldz; fcompp`: above 0, or not a number (C0)
        Ok(compare(X(0.0), first - second) & 0x0100 != 0)
    }
}

/// `sub_457CA8`'s arithmetic: the share of the edge's crossing (-(`dword_641978` x
/// `dword_641984` - `dword_641980` x `dword_64197C`) over the difference of the two cross
/// products, rounded to a float into `flt_639810` and used as read back), the crossing's
/// place (x and z whole numbers into `dword_623F88` / `dword_623F90`) and the entry's
/// texture floats (`flt_623FA8`, `flt_623FAC`) and two bytes (`dword_62419C`, `dword_6241AC`)
/// shared the same way between the scene points `pa`, `pb` and the entries `ea`, `eb`;
/// `dword_62400C` is left with the share's bits.
fn crossing_block(pa: u32, pb: u32, ea: &PolygonEntry, eb: &PolygonEntry) -> Result<(), BadInput> {
    use crate::math::x87_float::{int, ld, st, to_int};
    {
        let word = |n: usize| int(FRAME_WORK.shadow_work[n].bits() as i32);
        let first = word(11) * word(8) - word(10) * word(9);
        let second = word(13) * word(8) - word(12) * word(9);
        let share = st(-(first / (second - first)));
        GROUND_WORK.plane_f.set_bits(share);
        let share = ld(share);
        // a + (b - a) x share
        let whole = |a: u32, b: u32| -> Result<i32, BadInput> {
            Ok(to_int(
                int(crate::race::scene_points::word(a)? as i32)
                    + (int(crate::race::scene_points::word(b)? as i32)
                        - int(crate::race::scene_points::word(a)? as i32))
                        * share,
            ))
        };
        DRAW.clip_work[2].set_bits(whole(pa, pb)? as u32);
        DRAW.clip_work[4].set_bits(whole(pa + 8, pb + 8)? as u32);
        let float = |a: &Dword, b: &Dword| st(ld(a.get()) + (ld(b.get()) - ld(a.get())) * share);
        DRAW.clip_work[10].set_bits(float(&ea.u2, &eb.u2));
        DRAW.clip_work[11].set_bits(float(&ea.v2, &eb.v2));
        let byte = |a: &Byte, b: &Byte| {
            let (a, b) = (int(a.get().into()), int(b.get().into()));
            to_int(a + (b - a) * share) as u32
        };
        DRAW.vertex_byte_21.set(byte(&ea.amount, &eb.amount));
        DRAW.clip_work_2[0].set(byte(&ea.fog, &eb.fog));
        DRAW.scratch.set(st(share));
    }
    Ok(())
}

/// `sub_457CA8`: the edge's crossing between work entries `ea` and `eb` as a new scene
/// point (`dword_641990`, then counted on: x and z from the share `flt_639810`, y 0) and
/// the new work entry `ed` (its point, texture places +0x10 / +0x14 and the two bytes
/// +0x20 / +0x21 shared the same way); the original's work dwords (0x641958 .. 0x64198C,
/// `flt_623F88` ..) kept as it leaves them.
fn edge_crossing(ea: &PolygonEntry, eb: &PolygonEntry, ed: &PolygonEntry) -> Result<(), BadInput> {
    {
        let pa = scene_point(ea.point.get());
        let pb = scene_point(eb.point.get());
        let c = &FRAME_WORK.shadow_corners;
        let w = &FRAME_WORK.shadow_work;
        let rd = |d: &crate::session::Dword| d.bits();
        w[1].set(crate::race::scene_points::word(pb)?);
        w[5].set(crate::race::scene_points::word(pb + 8)?);
        w[0].set(crate::race::scene_points::word(pa)?);
        w[4].set(crate::race::scene_points::word(pa + 8)?);
        w[2].set(rd(&c[0]));
        w[3].set(rd(&c[2]));
        w[6].set(rd(&c[1]));
        w[7].set(rd(&c[3]));
        w[8].set(rd(&w[3]).wrapping_sub(rd(&w[2])));
        w[9].set(rd(&w[7]).wrapping_sub(rd(&w[6])));
        w[10].set(rd(&w[0]).wrapping_sub(rd(&w[2])));
        w[11].set(rd(&w[4]).wrapping_sub(rd(&w[6])));
        w[12].set(rd(&w[1]).wrapping_sub(rd(&w[2])));
        w[13].set(rd(&w[5]).wrapping_sub(rd(&w[6])));
        crossing_block(pa, pb, ea, eb)?;
        let n = rd(&FRAME_WORK.shadow_scene);
        let p = scene_point(n);
        crate::race::scene_points::put(p, DRAW.clip_work[2].bits())?;
        crate::race::scene_points::put(p + 4, 0)?;
        crate::race::scene_points::put(p + 8, DRAW.clip_work[4].bits())?;
        ed.point.set(n);
        FRAME_WORK.shadow_scene.set(n.wrapping_add(1));
        ed.u2.set(DRAW.clip_work[10].bits());
        ed.v2.set(DRAW.clip_work[11].bits());
        ed.amount.set(rd(&DRAW.vertex_byte_21) as u8);
        ed.fog.set(rd(&DRAW.clip_work_2[0]) as u8);
    }
    Ok(())
}

/// `sub_457A85`: the polygon cut by the edge from scene point `from` (eax) to scene point
/// `to` (ebx) (kept: those on the inner side, and the crossings), with three vertices or
/// more. The registers it leaves are read by no one.
fn clip_to_edge(from: u32, to: u32) -> Result<(), BadInput> {
    {
        let count: u32 = DRAW.polygon_vertex_count.bits();
        if count < 3 {
            return Ok(());
        }
        let (a, b) = (scene_point(from), scene_point(to));
        FRAME_WORK.shadow_corners[2].set(crate::race::scene_points::word(b)?);
        FRAME_WORK.shadow_corners[3].set(crate::race::scene_points::word(b + 8)?);
        FRAME_WORK.shadow_corners[0].set(crate::race::scene_points::word(a)?);
        FRAME_WORK.shadow_corners[1].set(crate::race::scene_points::word(a + 8)?);
        let (list, old) = (&POLYGON_WORK.list, &POLYGON_WORK.old_list);
        copy_on(list, old, count as usize, "the polygon list")?;
        write_on(old, count as usize, list[0].get(), "the old polygon list")?;
        // Converted: `rc2k_core::race::shadow::clip_by_edge`.
        let mut added = 0u32;
        let trap = Trap::new();
        rc2k_core::race::shadow::clip_by_edge(
            count,
            |k| {
                read_on(old, k as usize, "the old polygon list").unwrap_or_else(|err| {
                    trap.keep(err);
                    0
                })
            },
            |e| {
                let entry = match work_entry(e) {
                    Ok(x) => x.point.get(),
                    Err(err) => {
                        trap.keep(err);
                        0
                    }
                };
                inside_edge(entry).unwrap_or_else(|err| {
                    trap.keep(err);
                    false
                })
            },
            |a, b| {
                let next: u32 = DRAW.polygon_next_entry.bits();
                let crossing =
                    (|| edge_crossing(work_entry(a)?, work_entry(b)?, work_entry(next)?))();
                if let Err(err) = crossing {
                    trap.keep(err);
                }
                DRAW.polygon_next_entry.set_bits(next.wrapping_add(1));
                next
            },
            |e| {
                if let Err(err) = write_on(list, added as usize, e, "the polygon list") {
                    trap.keep(err);
                }
                added += 1;
            },
        );
        trap.finish()?;
        DRAW.polygon_vertex_count.set_bits(added);
    }
    Ok(())
}

/// `race::clip_shadow_to_face` (0x457960): the shadow's polygon cut to the course face
/// whose corners are `dword_641938` .. `dword_641940` (scene points): none (count 0) when
/// a corner is more than 0x100000 off the work car in x before one within 0x100000 in z
/// is met (or none is); else cut by its three edges in turn while three vertices are left
/// (`sub_457A85`, with `sub_457BE8` the side test and `sub_457CA8` the crossing). The
/// registers it leaves are read by no one.
pub fn clip_shadow_to_face() -> Result<(), BadInput> {
    {
        let corners = [
            FRAME_WORK.shadow_light[0].get(),
            FRAME_WORK.shadow_light[1].bits(),
            FRAME_WORK.shadow_light[2].bits(),
        ];
        // Converted: `rc2k_core::race::shadow::face_near`.
        let trap = Trap::new();
        let found = rc2k_core::race::shadow::face_near(
            (CAR.position[0].bits(), CAR.position[2].bits()),
            |n| {
                let p = scene_point(corners[n]);
                let w = |a: u32| match crate::race::scene_points::word(a) {
                    Ok(v) => v,
                    Err(e) => {
                        trap.keep(e);
                        0
                    }
                };
                (w(p), w(p + 8))
            },
        );
        trap.finish()?;
        if !found.near {
            DRAW.polygon_vertex_count.set_bits(0u32);
            return Ok(());
        }
        for (a, b) in [(0usize, 1usize), (1, 2), (2, 0)] {
            if DRAW.polygon_vertex_count.bits() < 3 {
                return Ok(());
            }
            clip_to_edge(corners[a], corners[b])?;
        }
    }
    Ok(())
}

/// `sub_457B86`: work entry eax's scene point raised onto the face (`dword_641938` ..):
/// the face's corners (into `dword_639890` ..) and the point (into `dword_639824` /
/// `dword_63982C`) through `physics::face_shares` and `physics::plane_height`, its y the
/// height (`dword_639828`) plus `dword_641928`. Every register kept.
fn raise_onto_face(index: u32) -> Result<(), Refused> {
    use crate::physics::ground::face_shares_at;
    let p = scene_point(work_entry(index)?.point.get());
    for k in 0..3 {
        GROUND_WORK.face_corner_numbers[k].set_bits(FRAME_WORK.shadow_light[k].bits());
    }
    GROUND_WORK
        .ground_x
        .set(crate::race::scene_points::word(p)?);
    GROUND_WORK
        .ground_z
        .set(crate::race::scene_points::word(p + 8)?);
    face_shares_at()?;
    crate::physics::plane_height::plane_height();
    crate::race::scene_points::put(
        p + 4,
        GROUND_WORK
            .ground_height
            .get()
            .wrapping_add(FRAME_WORK.shadow_strength.bits()),
    )?;
    Ok(())
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
pub fn shadow_on_face() -> Result<(), Refused> {
    let count: u32 = FRAME_WORK.shadow_list_count.get();
    if count > 0x40 {
        return Err(BadInput("a shadow list past 0x40").into());
    }
    DRAW.polygon_drawn.set(0u32);
    let list = &POLYGON_WORK.list;
    copy_on(
        &FRAME_WORK.shadow_list,
        list,
        count as usize,
        "the shadow list",
    )?;
    DRAW.polygon_vertex_count.set_bits(count);
    write_on(
        list,
        count as usize,
        FRAME_WORK.shadow_list[0].get(),
        "the polygon list",
    )?;
    DRAW.polygon_next_entry.set_bits(0x20u32);
    let scene: u32 = COURSE.scene_point_count.get();
    FRAME_WORK.shadow_scene.set(scene.wrapping_add(0x30));
    DRAW.polygon_sides[0].set(0u32);
    DRAW.polygon_sides[1].set(1u32);
    let mut at = (scene & 0xFFF) * 0x20;
    for _ in 0..0x40 {
        crate::scene::point_records::put_field(
            crate::scene::point_records::at_offset(at),
            0,
            u32::MAX,
        )?;
        at = (at + 0x20) & 0x1FFE0;
    }
    clip_shadow_to_face()?;
    if DRAW.polygon_vertex_count.bits() < 3 {
        return Ok(());
    }
    let n: u32 = DRAW.polygon_vertex_count.bits();
    for k in 0..n {
        raise_onto_face(read_on(list, k as usize, "the polygon list")?)?;
    }
    crate::scene::project_polygon::project_polygon()?;
    crate::scene::fog_polygon::fog_polygon()?;
    DRAW.polygon_next_record
        .set(crate::scene::polygon::pool_start());
    let outside: u32 = DRAW.polygon_outside_count.get();
    if outside != 0 {
        if outside == DRAW.polygon_vertex_count.bits() {
            return Ok(());
        }
        crate::scene::clip_polygon_by_depth::clip_polygon_by_depth()?;
        if DRAW.polygon_vertex_count.bits() < 3 {
            return Ok(());
        }
    }
    crate::scene::clip_polygon_to_screen::clip_polygon_to_screen()?;
    if DRAW.polygon_vertex_count.bits() < 3 {
        return Ok(());
    }
    DRAW.polygon_textures[0].set(u32::MAX);
    DRAW.polygon_textures[1].set(FRAME_WORK.shadow_texture.get());
    DRAW.polygon_sides[0].set(0u32);
    DRAW.polygon_sides[1].set(1u32);
    RACE_VIEW.polygon_mode.set(5u32);
    crate::scene::draw_polygon::draw_polygon_in_game()?;
    DRAW.polygon_drawn.set(1u32);
    Ok(())
}

/// `sub_4576C9`: each face of course record esi (+8 .. +0xC) with a kind below 0x10 and
/// without bit 7 of its +2 word gets the shadow (`shadow_on_face`) on its two triangles
/// (corners +4, +8, +0xC and +0x10, +0xC, +8); `g_currentCourseFace` each face.
fn shadow_on_record(record: u32) -> Result<(), Refused> {
    // Converted: `rc2k_core::race::shadow::{takes_shadow, SHADOW_TRIANGLES}`,
    // `rc2k_core::race::view::record_faces`.
    use rc2k_core::race::shadow::{SHADOW_TRIANGLES, takes_shadow};
    let Some((first, last)) =
        rc2k_core::race::view::record_faces(get(record + 8)?, get(record + 0xC)?)
    else {
        return Ok(());
    };
    for face in first..=last {
        COURSE.current_course_face.set(face);
        let at = (face << 6).wrapping_add(COURSE.course_faces.bits());
        let f = crate::race::course_places::face(at, "a course face")?;
        let takes = takes_shadow(f.kind(), || f.switches());
        if takes {
            for corners in SHADOW_TRIANGLES {
                for (k, &o) in corners.iter().enumerate() {
                    FRAME_WORK.shadow_light[k].set_bits(get(at + o)?);
                }
                shadow_on_face()?;
            }
        }
    }
    Ok(())
}

/// `race::shadow_faces` (0x457618): the shadow (`shadow_on_face`) on the last course
/// record's faces, then on every record in the 3 x 3 cells round the work car
/// (`sub_4576C9` each). The original keeps every register but ebx (the record count),
/// which its one caller does not read.
pub fn shadow_faces() -> Result<(), Refused> {
    let count: u32 = COURSE.course_record_count.get();
    let records: u32 = COURSE.course_records.get();
    shadow_on_record((count.wrapping_sub(1) << 5).wrapping_add(records))?;
    if count == 0 {
        return Ok(());
    }
    // Converted: `rc2k_core::race::shadow::{record_cell, cell_near_car}`.
    use rc2k_core::race::shadow::{cell_near_car, record_cell};
    // Every record of the course is looked at each frame: the run is resolved once.
    let all =
        crate::race::course_places::bytes(records, (count as usize) << 5, "the course records")?;
    for n in 0..count {
        let at = (n << 5).wrapping_add(records);
        let o = (n as usize) << 5;
        let Some((x, z)) = record_cell(u32::from_le_bytes([
            all[o],
            all[o + 1],
            all[o + 2],
            all[o + 3],
        ])) else {
            continue;
        };
        COURSE.record_place[0].set(x);
        COURSE.record_place[1].set(z);
        if cell_near_car((CAR.position[0].bits(), CAR.position[2].bits()), (x, z)) {
            shadow_on_record(at)?;
        }
    }
    Ok(())
}

/// `fld tbyte; fstp dword`: 80 bits as a float.
fn tbyte_to_float(t: [u8; 10]) -> u32 {
    crate::math::x87_float::st(crate::math::x87_float::from_f80(t))
}

/// The sun's slant from `sun` and `turn` (the angles as 128ths of a half turn): `flt_643AC4`
/// = cos(turn) x cot(sun), `flt_643AC8` = sin(turn) x cot(sun); `dword_62400C` is left 0x80.
fn sun_slant(sun: u32, turn: u32) {
    use crate::math::x87_float::{int, st};
    let cot = {
        let (sin, cos) = (int(sun as i32) / int(0x80)).times_pi().sin_cos();
        cos / sin
    };
    let (sin, cos) = (int(turn as i32) / int(0x80)).times_pi().sin_cos();
    SHADOW_SKIDS.slant_x.set_bits(st(cos * cot));
    SHADOW_SKIDS.slant_z.set_bits(st(cot * sin));
    DRAW.scratch.set(0x80u32);
}

/// `fld x; fld z; fpatan; fstp`: the angle of the corner at `at` (x at +0, z at +8).
fn angle_of(at: u32) -> Result<u32, BadInput> {
    use crate::math::x87_float::{ld, st};
    Ok(st(ld(get(at)?).atan2(ld(get(at + 8)?))))
}

/// `race::shadow_points` (0x458078): the shadow's points: the ground under the work car
/// (`physics::find_ground`, its height to `dword_641AA0`); the sun's slant from
/// `dword_60933C` (at least 0x10) and -`dword_609338` - 0x40 (`flt_643AC4`, `flt_643AC8`);
/// each of the car's `dword_64192C` corners (`g_car` +0xAF0 .. +0xCF0 by axis) turned by
/// its matrix (+0x840), moved along the slant by its height plus 0.7, its angle (`fpatan`)
/// and distance, the farthest kept (`flt_643AB8`, `dword_643ABC`), as whole numbers (x
/// 65536) plus the car's place. Refuses what `find_ground` refuses (before anything), and
/// no corners or more than 0x40.
pub fn shadow_points() -> Result<(), Refused> {
    use crate::math::x87::mul_to_int;
    use crate::math::x87_load::{below_or_unordered, to_tbyte};
    let count: u32 = FRAME_WORK.shadow_point_count.bits();
    if count == 0 || count > 0x40 {
        return Err(BadInput("no shadow corners or more than 0x40").into());
    }
    let inputs = [
        &GROUND_WORK.ground_x,
        &GROUND_WORK.ground_z,
        &COURSE.ground_face_last,
    ];
    let was: Vec<u32> = inputs.iter().map(|d| d.bits()).collect();
    inputs[0].set_bits(CAR.position[0].bits());
    inputs[1].set_bits(CAR.position[2].bits());
    inputs[2].set_bits(CAR.probe_point_face[4].bits());
    if let Err(e) = crate::physics::find_ground::find_ground() {
        // Its inputs put back: a refusal has written nothing.
        for (d, &v) in inputs.iter().zip(&was) {
            d.set_bits(v);
        }
        return Err(e);
    }
    SHADOW_SKIDS.farthest_length.set(-1.0f32);
    FRAME_WORK
        .shadow_ground
        .set_bits(GROUND_WORK.ground_height.get());
    let sun = STAGE_PREFS.sun_height.get().max(0x10);
    let turn = STAGE_PREFS.sun_turn.get().wrapping_neg().wrapping_sub(0x40);
    sun_slant(sun, turn);
    let (sx, sz) = (SHADOW_SKIDS.slant_x.bits(), SHADOW_SKIDS.slant_z.bits());
    let lift: u32 = PHYSICS.camera_drop.bits();
    let unit: u32 = NUMBERS.k65536.bits();
    let mut at = SHADOW_SKIDS.shadow_points.addr();
    for k in 0..count as usize {
        let p = [
            to_tbyte(CAR.probe_point_x[k].bits()),
            to_tbyte(CAR.probe_point_y[k].bits()),
            to_tbyte(CAR.probe_point_z[k].bits()),
        ];
        let t = crate::scene::rotate_point::rotate_point(&CAR.body_matrix, p);
        let [x, y, z] = t.map(tbyte_to_float);
        put(at + 4, y)?;
        let x = lifted(y, lift, sx, x);
        put(at, x)?;
        let z = lifted(y, lift, sz, z);
        put(at + 8, z)?;
        let angle = angle_of(at)?;
        put(at + 0xC, angle)?;
        let length = crate::math::x87_load::float_hypot(x, z);
        put(at + 0x10, length)?;
        if !below_or_unordered(length, SHADOW_SKIDS.farthest_length.bits()) {
            SHADOW_SKIDS.farthest_length.set_bits(length);
            SHADOW_SKIDS.farthest.set(at);
        }
        put(
            at,
            (mul_to_int(x, unit) as u32).wrapping_add(CAR.position[0].bits()),
        )?;
        put(
            at + 8,
            (mul_to_int(z, unit) as u32).wrapping_add(CAR.position[2].bits()),
        )?;
        put(at + 4, FRAME_WORK.shadow_ground.bits())?;
        at += 0x20;
    }
    Ok(())
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
pub fn cast_car_shadow() -> Result<(), Refused> {
    // Converted: `rc2k_core::race::shadow::{casts, hull_drawn, hull_list,
    // SHADOW_TEXTURE}`.
    use rc2k_core::race::shadow::{SHADOW_TEXTURE, casts, hull_drawn, hull_list};
    if !casts(FRAME_WORK.shadow_point_count.bits()) {
        return Ok(());
    }
    shadow_points()?;
    shadow_angles()?;
    shadow_hull()?;
    if !hull_drawn(SHADOW_SKIDS.hull_count.get()) {
        return Ok(());
    }
    shadow_polygon()?;
    let n: u32 = SHADOW_SKIDS.hull_count.get();
    FRAME_WORK.shadow_list_count.set(n);
    for k in hull_list(n) {
        crate::session::write_on_in(
            &FRAME_WORK,
            &FRAME_WORK.shadow_list,
            k as usize,
            k,
            "the shadow list",
        )?;
    }
    FRAME_WORK.shadow_texture.set(SHADOW_TEXTURE);
    shadow_faces()
}

/// `race::cast_car_shadows` (0x45754C): the depth test off, `dword_641928` 0x1999, then
/// for each race car with +0x6FC not 0: loaded into the work copy (`drive::load_car`), its
/// corner count from `g_car` +0x260, `cast_car_shadow` once with `dword_6238B0` set, then
/// once more with `dword_60933C` 0x40 (put back after), inside `pusha`; the depth test on.
/// Its one caller, `race::draw_view_scene`, reads no register it leaves. Refuses (before anything) no cars or more than eight, or a null block.
pub fn cast_car_shadows() -> Result<(), Refused> {
    // Converted: `rc2k_core::race::shadow::{check_shadow_cars, casts_shadow, first_pass,
    // SHADOW_STRENGTH, SUN_PASS}`.
    use rc2k_core::race::shadow::{
        SHADOW_STRENGTH, SUN_PASS, casts_shadow, check_shadow_cars, first_pass,
    };
    let cars: u32 = RACE_VIEW.race_cars.get();
    let block = |n: u32| RACE_WORK.car_block(n as usize);
    check_shadow_cars(cars, block)?;
    crate::scene::depth_test_off::depth_test_off();
    FRAME_WORK.shadow_strength.set(SHADOW_STRENGTH);
    for n in 0..cars {
        let b = block(n);
        if !casts_shadow(get(b + 0x6FC)?) {
            continue;
        }
        RACE_WORK.current_car.set(n);
        crate::drive::load_car::load_car()?;
        FRAME_WORK
            .shadow_point_count
            .set_bits(get(crate::physics::car_store::car_addr() + 0x260)?);
        if first_pass(RACE_VARS.settings_words[2].bits()) {
            cast_car_shadow()?;
        }
        let sun: u32 = STAGE_PREFS.sun_height.get();
        STAGE_PREFS.sun_height.set(SUN_PASS);
        cast_car_shadow()?;
        STAGE_PREFS.sun_height.set(sun);
    }
    crate::scene::depth_test_on::depth_test_on();
    Ok(())
}

#[cfg(all(test, target_arch = "x86"))]
mod asm_reference {
    use super::*;

    // The reference blocks read and write the test's own memory plainly.
    fn get(a: u32) -> u32 {
        super::get(a).expect("the test's memory is registered")
    }

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
            GROUND_WORK.ground_x.set(get(p));
            GROUND_WORK.ground_z.set(get(p + 8));
            let status: u16;
            let old = crate::race::store::OldPlaces::lay(&[
                0x639824, 0x63982c, 0x641948, 0x64194c, 0x641950, 0x641954,
            ]);
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
            drop(old);
            status & 0x0100 != 0
        }
    }

    pub(super) unsafe fn crossing_block_asm(pa: u32, pb: u32, ea: u32, eb: u32) {
        unsafe {
            let old = crate::race::store::OldPlaces::lay(&[
                0x623f88, 0x623f90, 0x623fa8, 0x623fac, 0x62400c, 0x62419c, 0x6241ac, 0x639810,
                0x641978, 0x64197c, 0x641980, 0x641984, 0x641988, 0x64198c,
            ]);
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
            drop(old);
        }
    }

    pub(super) unsafe fn sun_slant_asm(sun: u32, turn: u32) {
        unsafe {
            let v = [sun, 0x80u32, turn];
            let old = crate::race::store::OldPlaces::lay(&[0x62400c, 0x643ac4, 0x643ac8]);
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
            drop(old);
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

    use crate::framework::{game_read, game_write, rva};
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
                let mut a = [0xAAAA_AAAAu32; 17];
                let cells: [Dword; 17] = std::array::from_fn(|_| Dword::new(0xAAAA_AAAA));
                let _r = crate::race::trap::test_region(&cells);
                let entry = GamePtr::new(crate::pointers::game_address(cells.as_ptr()))
                    .record::<PolygonEntry>("the test's entry")
                    .expect("the test's memory is registered");
                unsafe {
                    circle_place_asm(
                        crate::pointers::game_address(a.as_mut_ptr()),
                        k,
                        n,
                        radius,
                        unit,
                    );
                }
                circle_place(entry, k, n, radius, unit);
                let b: [u32; 17] = std::array::from_fn(|w| cells[w].get());
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
        let _region = crate::race::trap::test_region(&points);
        let (mut yes, mut no) = (0, 0);
        at_each_control_word(|| {
            for _ in 0..20000 {
                for q in points.iter_mut() {
                    *q = r.int() as u32;
                }
                COURSE
                    .scene_points
                    .set(crate::pointers::game_address(points.as_ptr()));
                for a in &places[2..6] {
                    game_write(rva(*a), r.int() as u32);
                }
                GROUND_WORK.ground_x.set(0xAAAA_AAAAu32);
                GROUND_WORK.ground_z.set(0xAAAA_AAAAu32);
                let n = r.below(4);
                let want = unsafe { inside_edge_asm(n) };
                let seen = (GROUND_WORK.ground_x.get(), GROUND_WORK.ground_z.get());
                {
                    GROUND_WORK.ground_x.set(0xAAAA_AAAAu32);
                    GROUND_WORK.ground_z.set(0xAAAA_AAAAu32);
                }
                let got = crate::race::scene_points::with_test_points(&points, || inside_edge(n))
                    .expect("the test's memory is registered");
                assert_eq!(want, got, "{points:x?} {n}");
                assert_eq!(
                    seen,
                    (GROUND_WORK.ground_x.get(), GROUND_WORK.ground_z.get())
                );
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
                    game_write(rva(*a), v as u32);
                }
                let point = |r: &mut Rng| [r.int() as u32, r.u32(), r.int() as u32];
                let (pa, pb) = (point(&mut r), point(&mut r));
                let entry = |r: &mut Rng| {
                    let mut e = [0u32; 17];
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
                let (_r1, _r2, _r3, _r4) = (
                    crate::race::trap::test_region(&pa),
                    crate::race::trap::test_region(&pb),
                    crate::race::trap::test_region(&ea),
                    crate::race::trap::test_region(&eb),
                );
                let args = (
                    crate::pointers::game_address(pa.as_ptr()),
                    crate::pointers::game_address(pb.as_ptr()),
                    crate::pointers::game_address(ea.as_ptr()),
                    crate::pointers::game_address(eb.as_ptr()),
                );
                let read = || places.map(|a| game_read::<u32>(rva(a)));
                for a in &places[7..] {
                    game_write(rva(*a), 0xAAAA_AAAAu32);
                }
                unsafe { crossing_block_asm(args.0, args.1, args.2, args.3) };
                let want = read();
                for a in &places[7..] {
                    game_write(rva(*a), 0xAAAA_AAAAu32);
                }
                DRAW.scratch.set_bits(0xAAAA_AAAAu32);
                let both = [pa, pb].concat();
                crate::race::scene_points::with_test_points(&both, || {
                    let base = crate::race::scene_points::base();
                    let entry = |a: u32| {
                        GamePtr::new(a)
                            .record::<PolygonEntry>("the test's entry")
                            .expect("the test's memory is registered")
                    };
                    crossing_block(base, base + 12, entry(args.2), entry(args.3))
                })
                .expect("the test's memory is registered");
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
                let want = [0x62400C, 0x643AC4, 0x643AC8].map(|a| game_read::<u32>(rva(a)));
                for a in [0x62400C, 0x643AC4, 0x643AC8] {
                    game_write(rva(a), 0xAAAA_AAAAu32);
                }
                sun_slant(sun, turn);
                let got = [0x62400C, 0x643AC4, 0x643AC8].map(|a| game_read::<u32>(rva(a)));
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
                let _region = crate::race::trap::test_region(&at);
                let (want, got) = unsafe {
                    (
                        angle_asm(crate::pointers::game_address(at.as_ptr())),
                        angle_of(crate::pointers::game_address(at.as_ptr())).unwrap(),
                    )
                };
                assert!(within_ulps(want, got, 1), "{at:x?}: {want:#x} {got:#x}");
                worst = worst.max(want.abs_diff(got));
            }
        });
        eprintln!("sun_slant, angle_of: worst difference {worst} ulps");
    }
}
