//! The car's shadow (the tree under `cast_car_shadows`, IDA 2026-10-18; a reading: a dark polygon under
//! each car, the hull of its corners on the ground, cut by the course's faces). Its
//! leaves and shared code. Its working values are a `ShadowWork` of its own (frame plan F4):
//! the point records of 0x20 bytes at `work.points` (+0 x, +4 y, +8 z as whole numbers, +0xC
//! the angle, +0x10 the distance, +0x14 0), their count `work.point_count`, the hull's
//! `work.hull_count`, ...

use crate::errors::{BadInput, Refused};
use crate::physics::car_store::Car;
use crate::physics::ground_query::{GroundCells, GroundQuery};
#[cfg(all(test, target_arch = "x86"))]
use crate::pointers::GamePtr;
use crate::race::owner::Race;
#[cfg(all(test, target_arch = "x86"))]
use crate::race::owner::race;
use crate::race::scene_points::{point_word, point_xz, put_point_word};
use crate::race::shadow_skids_store::ShadowPoint;
use crate::race::trap::Trap;
use crate::scene::polygon_store::{POLYGON_WORK, PolygonEntry};
use crate::session::{Byte, Dword, Float, copy_on_in, read_on_in, write_on_in};

/// The shadow's working state: what `cast_car_shadows` writes and reads within itself, one
/// for each call, given down to its parts. The original kept it in the race block
/// (`g_shadowPoints` and after at 0x641AA4 .. 0x643ACC, `g_shadowPointCount` and its
/// neighbours at 0x641928 .. 0x641AA0), left from view to view; nothing else read it, and
/// each call wrote every value before reading it. `#[repr(C)]` for the list's copy into
/// the polygon list (`copy_on_in`).
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct ShadowWork {
    /// The shadow's points, records of 0x20 bytes (0x40 at most), the hull's records after.
    points: [ShadowPoint; 0x100],
    /// The car's corners (`g_shadowPointCount`; from its block's +0x260).
    point_count: Dword,
    /// The length of the farthest point (`shadow_points`).
    farthest_length: Float,
    /// The farthest point's number (the original kept its record's address).
    farthest: Dword,
    /// The first point's angle (`shadow_angles`).
    first_angle: Float,
    /// cos(turn) x cot(sun) (`sun_slant`).
    slant_x: Float,
    /// sin(turn) x cot(sun).
    slant_z: Float,
    /// The shadow hull's points.
    hull_count: Dword,
    /// What the shadow is raised above the face (0x1999, `cast_car_shadows`).
    lift: Dword,
    /// The shadow's texture (0xC3).
    texture: Dword,
    /// The three corners (scene point numbers) of the course face the shadow is cut to.
    face_corners: [Dword; 3],
    /// The edge cut by: x and z from, x and z to (`clip_to_edge`).
    edge: [Dword; 4],
    /// `edge_crossing`'s work dwords (0x641958 .. 0x64198C in the original).
    crossing: [Dword; 14],
    /// The next new scene point (`shadow_on_face`, `edge_crossing`).
    next_point: Dword,
    /// The hull's list.
    list: [Dword; 64],
    /// Its count.
    list_count: Dword,
    /// The ground's height under the car.
    ground: Dword,
    /// The ground searches' fields (the original's were the physics', `GROUND_WORK` ..).
    search: GroundCells,
}

impl ShadowWork {
    pub fn new() -> Self {
        Self {
            points: std::array::from_fn(|_| ShadowPoint::new()),
            point_count: Dword::new(0),
            farthest_length: Float::from_bits(0),
            farthest: Dword::new(0),
            first_angle: Float::from_bits(0),
            slant_x: Float::from_bits(0),
            slant_z: Float::from_bits(0),
            hull_count: Dword::new(0),
            lift: Dword::new(0),
            texture: Dword::new(0),
            face_corners: [const { Dword::new(0) }; 3],
            edge: [const { Dword::new(0) }; 4],
            crossing: [const { Dword::new(0) }; 14],
            next_point: Dword::new(0),
            list: [const { Dword::new(0) }; 64],
            list_count: Dword::new(0),
            ground: Dword::new(0),
            search: GroundCells::new(),
        }
    }

    /// The ground search on the shadow's own fields; the caller's mark (`settle_mark`), which
    /// the search only reads, is the physics' as the last search there left it.
    fn query<'a>(&'a self, race: &'a Race) -> GroundQuery<'a> {
        GroundQuery {
            settle_mark: &race.physics.ground_probe.settle_mark,
            ..self.search.query()
        }
    }
}

/// The circle's radius and unit (`g_shadowCircle`, `flt_641998`: 2^23 and 2^24, never written).
const CIRCLE: [u32; 2] = [0x4B00_0000, 0x4B80_0000];

/// Point `n` (0x20 bytes each; `work.farthest` keeps the farthest one's number), bounded by
/// the table (`work.points`).
fn point_at(work: &ShadowWork, n: u32) -> Result<&'_ ShadowPoint, BadInput> {
    work.points
        .get(n as usize)
        .ok_or(BadInput("a shadow point"))
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

/// `race::shadow_angles` (0x4581F3): the farthest point (`work.farthest`) swapped into
/// the first record; each record's angle made relative to the first's
/// (`work.first_angle`; below 0 plus 2 pi), times 65536 as a whole number, +0x14 0;
/// with two records or more sorted by angle (`sorting::sort_records`). Refuses
/// (before anything) no points (its loop tests at its end) or more than 0x40.
pub fn shadow_angles(race: &Race, work: &ShadowWork) -> Result<(), Refused> {
    use crate::math::x87::{mul_to_int, sub};
    let count: u32 = work.point_count.bits();
    if count == 0 || count > 0x40 {
        return Err(BadInput("no shadow points or more than 0x40").into());
    }
    let records = &work.points;
    let farthest = point_at(work, work.farthest.get())?;
    for (here, there) in records[0].words().into_iter().zip(farthest.words()) {
        let (a, b) = (here.get(), there.get());
        here.set(b);
        there.set(a);
    }
    let first = records[0].angle.get();
    work.first_angle.set_bits(first);
    let zero: u32 = race.course.numbers.zero.bits();
    let unit: u32 = race.course.numbers.k65536.bits();
    for k in 0..count {
        let record = point_at(work, k)?;
        let mut v = sub(record.angle.get(), first);
        if crate::math::x87_load::below_or_unordered(v, zero) {
            v = two_pi_plus(v);
        }
        record.angle.set(mul_to_int(v, unit) as u32);
        record.zeroed.set(0);
    }
    if count >= 2 {
        // Converted: `rc2k_core::race::quick_sort::quick_sort`.
        crate::race::sorting::sort_records::<8>(
            &work.points,
            count,
            3,
            rc2k_core::race::quick_sort::Order::Up,
        )?;
    }
    Ok(())
}

/// `race::shadow_hull` (0x45836F): the first record copied after the last; then, walking the
/// points, each one where the turn from the one before to the one after is not to the left (the
/// 64-bit cross product of whole numbers, `GROUND_PROBE.product` its first half, not negative)
/// is taken out (the rest moved down, starting again), until none is; `work.hull_count`
/// the points left. Refuses (before anything) fewer than two points or more than 0x40.
pub fn shadow_hull(work: &ShadowWork) -> Result<(), Refused> {
    // Converted: `rc2k_core::race::shadow::hull`.
    use rc2k_core::race::shadow::{Record, hull, hull_records};
    let count: u32 = work.point_count.bits();
    if !(2..=0x40).contains(&count) {
        return Err(BadInput("fewer than two shadow points or more than 0x40").into());
    }
    let at = |n: usize| point_at(work, n as u32);
    let mut records: Vec<Record> = Vec::new();
    for n in 0..hull_records(count) {
        records.push(crate::pointers::load_value::<Record>(at(n)?, 0)?);
    }
    let before = records.clone();
    let h = hull(&mut records, count)?;
    for (n, (now, was)) in records.iter().zip(&before).enumerate() {
        if now != was {
            crate::pointers::store_value(at(n)?, 0, *now)?;
        }
    }
    work.search.query().product[0].set(h.product as u32);
    work.search.query().product[1].set((h.product as u64 >> 32) as u32);
    work.hull_count.set(h.count);
    Ok(())
}

/// The circle's texture place of corner `k` of `n` into the work entry `e` (+0x10, +0x14):
/// cos and sin of 2 pi k / n times `radius` plus `radius`, rounded to whole numbers, over
/// `unit`.
fn circle_place(e: &PolygonEntry, k: u32, n: u32, radius: u32, unit: u32) {
    use crate::math::x87_float::{int, ld, st, to_int};
    let angle = (int(k as i32) / int(n as i32)).times_pi();
    let (sin, cos) = (angle + angle).sin_cos();
    let (radius, unit) = (ld(radius), ld(unit));
    let across = to_int(cos * radius + radius);
    let down = to_int(sin * radius + radius);
    e.u2.set(st(int(across) / unit));
    e.v2.set(st(int(down) / unit));
}

/// `race::shadow_polygon` (0x45787F): the hull's points as new scene points (after the scene's,
/// from `g_scenePointCount` + 0x10: x and z the record's, y `work.ground`) and as
/// the polygon's work entries (+0 the point, +0x18 white, +0x21 `stage_prefs.shadow_alpha` / 2,
/// +0x10 / +0x14 a circle's texture place: cos / sin of 2 pi k / n times
/// `CIRCLE[0]` plus it, rounded, over `CIRCLE[1]`). Refuses
/// (before anything) a hull of no points (its loop tests at its end) or more than 0x40.
pub fn shadow_polygon(race: &Race, work: &ShadowWork) -> Result<(), Refused> {
    let n: u32 = work.hull_count.get();
    if n == 0 || n > 0x40 {
        return Err(BadInput("a shadow hull of no points or more than 0x40").into());
    }
    let scene_points: u32 = race.course.course.scene_point_count.get();
    for k in 0..n {
        // The hull's points follow the course's points and 16 more.
        let (record, point) = (
            point_at(work, k)?,
            scene_points.wrapping_add(0x10).wrapping_add(k),
        );
        put_point_word(&race.course, point, 0, record.x.get())?;
        put_point_word(&race.course, point, 2, record.z.get())?;
        put_point_word(&race.course, point, 1, work.ground.bits())?;
        let e = &POLYGON_WORK.entries[k as usize];
        e.point.set(point);
        e.light.set(0x00FF_FFFFu32);
        e.amount
            .set((race.course.stage_prefs.shadow_alpha.get() >> 1) as u8);
        circle_place(e, k, n, CIRCLE[0], CIRCLE[1]);
    }
    Ok(())
}

/// A polygon work entry's address (`POLYGON_WORK.entries` + 0x44 x n).
fn work_entry(n: u32) -> Result<&'static PolygonEntry, BadInput> {
    POLYGON_WORK
        .entries
        .get(n as usize)
        .ok_or(BadInput("a polygon work entry past 0x80"))
}

/// `inside_edge`: whether scene point `n` is on the inner side of the edge from
/// (`work.edge`, `work.edge[1]`) to (`work.edge[2]`, `work.edge[3]`): the cross product
/// of whole numbers on the x87 (the original's instructions) above 0, or not a number
/// (`fldz; fcompp`, C0). The point's x and z into `GROUND_WORK.ground_x` / `GROUND_WORK.ground_z`.
fn inside_edge(work: &ShadowWork, n: u32) -> Result<bool, BadInput> {
    let (x, z) = point_xz(n)?;
    let q = work.search.query();
    q.x.set(x);
    q.z.set(z);
    use crate::math::x87_float::{X, compare, int};
    let word = |d: &crate::session::Dword| int(d.bits() as i32);
    let (x, z) = (q.x, q.z);
    let c = &work.edge;
    let first = (word(x) - word(&c[0])) * (word(&c[3]) - word(&c[1]));
    let second = (word(z) - word(&c[1])) * (word(&c[2]) - word(&c[0]));
    // `fldz; fcompp`: above 0, or not a number (C0)
    Ok(compare(X(0.0), first - second) & 0x0100 != 0)
}

/// `edge_crossing`'s arithmetic: the share of the edge's crossing (-(`work.crossing[8]` x
/// `work.crossing[11]` - `work.crossing[10]` x `work.crossing[9]`) over the difference of the two cross
/// products, rounded to a float into `GROUND_WORK.plane_f` and used as read back), the crossing's
/// place (x and z whole numbers into `DRAW.clip_work[2]` / `DRAW.clip_work[4]`) and the entry's
/// texture floats (`DRAW.clip_work[10]`, `DRAW.clip_work[11]`) and two bytes (`DRAW.vertex_byte_21`, `DRAW.clip_work_2`)
/// shared the same way between the scene points numbered `pa`, `pb` and the entries `ea`, `eb`;
/// The original's store of the share's bits to `g_scratch` is not kept (nothing read it).
fn crossing_block(
    poly: &crate::scene::polygon_work::PolygonWork,
    work: &ShadowWork,
    pa: u32,
    pb: u32,
    ea: &PolygonEntry,
    eb: &PolygonEntry,
) -> Result<(), BadInput> {
    use crate::math::x87_float::{int, ld, st, to_int};
    let word = |n: usize| int(work.crossing[n].bits() as i32);
    let first = word(11) * word(8) - word(10) * word(9);
    let second = word(13) * word(8) - word(12) * word(9);
    let share = st(-(first / (second - first)));
    work.search.query().plane_f.set_bits(share);
    let share = ld(share);
    // a + (b - a) x share
    let whole = |k: u32| -> Result<i32, BadInput> {
        let (a, b) = (point_word(pa, k)?, point_word(pb, k)?);
        Ok(to_int(
            int(a as i32) + (int(b as i32) - int(a as i32)) * share,
        ))
    };
    poly.clip_work[2].set_bits(whole(0)? as u32);
    poly.clip_work[4].set_bits(whole(2)? as u32);
    let float = |a: &Dword, b: &Dword| st(ld(a.get()) + (ld(b.get()) - ld(a.get())) * share);
    poly.clip_work[10].set_bits(float(&ea.u2, &eb.u2));
    poly.clip_work[11].set_bits(float(&ea.v2, &eb.v2));
    let byte = |a: &Byte, b: &Byte| {
        let (a, b) = (int(a.get().into()), int(b.get().into()));
        to_int(a + (b - a) * share) as u32
    };
    poly.vertex_byte_21.set(byte(&ea.amount, &eb.amount));
    poly.clip_work_2[0].set(byte(&ea.fog, &eb.fog));
    Ok(())
}

/// `edge_crossing`: the edge's crossing between work entries `ea` and `eb` as a new scene
/// point (`work.next_point`, then counted on: x and z from the share `GROUND_WORK.plane_f`, y 0) and
/// the new work entry `ed` (its point, texture places +0x10 / +0x14 and the two bytes
/// +0x20 / +0x21 shared the same way); the original's work dwords (0x641958 .. 0x64198C,
/// `DRAW.clip_work[2]` ..) kept as it leaves them.
fn edge_crossing(
    ctx: &crate::race::view_scratch::DrawCtx<'_>,
    work: &ShadowWork,
    ea: &PolygonEntry,
    eb: &PolygonEntry,
    ed: &PolygonEntry,
) -> Result<(), BadInput> {
    let race = ctx.race;
    let (pa, pb) = (ea.point.get(), eb.point.get());
    let c = &work.edge;
    let w = &work.crossing;
    let rd = |d: &crate::session::Dword| d.bits();
    let ((bx, bz), (ax, az)) = (point_xz(pb)?, point_xz(pa)?);
    w[1].set(bx);
    w[5].set(bz);
    w[0].set(ax);
    w[4].set(az);
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
    crossing_block(&ctx.view.polygon, work, pa, pb, ea, eb)?;
    let n = rd(&work.next_point);
    put_point_word(&race.course, n, 0, ctx.view.polygon.clip_work[2].bits())?;
    put_point_word(&race.course, n, 1, 0)?;
    put_point_word(&race.course, n, 2, ctx.view.polygon.clip_work[4].bits())?;
    ed.point.set(n);
    work.next_point.set(n.wrapping_add(1));
    ed.u2.set(ctx.view.polygon.clip_work[10].bits());
    ed.v2.set(ctx.view.polygon.clip_work[11].bits());
    ed.amount.set(rd(&ctx.view.polygon.vertex_byte_21) as u8);
    ed.fog.set(rd(&ctx.view.polygon.clip_work_2[0]) as u8);
    Ok(())
}

/// `clip_to_edge`: the polygon cut by the edge from scene point `from` to scene point `to`
/// (kept: those on the inner side, and the crossings), with three vertices or more.
fn clip_to_edge(
    ctx: &crate::race::view_scratch::DrawCtx<'_>,
    work: &ShadowWork,
    from: u32,
    to: u32,
) -> Result<(), BadInput> {
    let race = ctx.race;
    let count: u32 = ctx.view.polygon.polygon_vertex_count.bits();
    if count < 3 {
        return Ok(());
    }
    let ((tx, tz), (fx, fz)) = (point_xz(to)?, point_xz(from)?);
    work.edge[2].set(tx);
    work.edge[3].set(tz);
    work.edge[0].set(fx);
    work.edge[1].set(fz);
    let (list, old) = (&POLYGON_WORK.list, &POLYGON_WORK.old_list);
    copy_on_in(
        (&race.shared.buffers, list),
        (&race.shared.buffers, old),
        count as usize,
        "the polygon list",
    )?;
    write_on_in(
        &race.shared.buffers,
        old,
        count as usize,
        list[0].get(),
        "the old polygon list",
    )?;
    // Converted: `rc2k_core::race::shadow::clip_by_edge`.
    let mut added = 0u32;
    let trap = Trap::new();
    rc2k_core::race::shadow::clip_by_edge(
        count,
        |k| {
            read_on_in(
                &race.shared.buffers,
                old,
                k as usize,
                "the old polygon list",
            )
            .unwrap_or_else(|err| {
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
            inside_edge(work, entry).unwrap_or_else(|err| {
                trap.keep(err);
                false
            })
        },
        |a, b| {
            let next: u32 = ctx.view.polygon.polygon_next_entry.bits();
            let crossing =
                (|| edge_crossing(ctx, work, work_entry(a)?, work_entry(b)?, work_entry(next)?))();
            if let Err(err) = crossing {
                trap.keep(err);
            }
            ctx.view
                .polygon
                .polygon_next_entry
                .set_bits(next.wrapping_add(1));
            next
        },
        |e| {
            if let Err(err) = write_on_in(
                &race.shared.buffers,
                list,
                added as usize,
                e,
                "the polygon list",
            ) {
                trap.keep(err);
            }
            added += 1;
        },
    );
    trap.finish()?;
    ctx.view.polygon.polygon_vertex_count.set_bits(added);
    Ok(())
}

/// `race::clip_shadow_to_face` (0x457960): the shadow's polygon cut to the course face whose
/// corners are `work.face_corners` .. `work.face_corners[2]` (scene points): none
/// (count 0) when a corner is more than 0x100000 off the work car in x before one within
/// 0x100000 in z is met (or none is); else cut by its three edges in turn while three vertices
/// are left (`clip_to_edge`, with `inside_edge` the side test and `edge_crossing` the crossing).
pub fn clip_shadow_to_face(
    ctx: &crate::race::view_scratch::DrawCtx<'_>,
    work: &ShadowWork,
    car: &Car,
) -> Result<(), BadInput> {
    let corners = [
        work.face_corners[0].get(),
        work.face_corners[1].bits(),
        work.face_corners[2].bits(),
    ];
    // Converted: `rc2k_core::race::shadow::face_near`.
    let trap = Trap::new();
    let found =
        rc2k_core::race::shadow::face_near((car.position[0].bits(), car.position[2].bits()), |n| {
            point_xz(corners[n]).unwrap_or_else(|e| {
                trap.keep(e);
                (0, 0)
            })
        });
    trap.finish()?;
    if !found.near {
        ctx.view.polygon.polygon_vertex_count.set_bits(0u32);
        return Ok(());
    }
    for (a, b) in [(0usize, 1usize), (1, 2), (2, 0)] {
        if ctx.view.polygon.polygon_vertex_count.bits() < 3 {
            return Ok(());
        }
        clip_to_edge(ctx, work, corners[a], corners[b])?;
    }
    Ok(())
}

/// `raise_onto_face`: work entry `index`'s scene point raised onto the face (`work.face_corners` ..):
/// the face's corners (into `GROUND_WORK.face_corner_numbers` ..) and the point (into `GROUND_WORK.ground_x` /
/// `GROUND_WORK.ground_z`) through `physics::face_shares` and `physics::plane_height`, its y the
/// height (`GROUND_WORK.ground_height`) plus `work.lift`.
fn raise_onto_face(race: &Race, work: &ShadowWork, index: u32) -> Result<(), Refused> {
    use crate::physics::ground::face_shares_at;
    let p = work_entry(index)?.point.get();
    let q = work.query(race);
    for k in 0..3 {
        q.corner_numbers[k].set_bits(work.face_corners[k].bits());
    }
    let (x, z) = point_xz(p)?;
    q.x.set(x);
    q.z.set(z);
    face_shares_at(race, &q, crate::scene::owner::scene())?;
    crate::physics::plane_height::plane_height(&q);
    put_point_word(
        &race.course,
        p,
        1,
        q.height.get().wrapping_add(work.lift.bits()),
    )?;
    Ok(())
}

/// `race::shadow_on_face` (0x45773C): the shadow's polygon (the hull's list at
/// `work.list`, its count `work.list_count`) cut to the face
/// `work.face_corners` .. and drawn on it: new points from `g_scenePointCount` + 0x30
/// (`work.next_point`), entries from 0x20, the 0x40 point records after the scene's
/// marked unmade; `clip_shadow_to_face`; with three vertices or more each raised onto the face,
/// projected (`scene::project_polygon`, `scene::fog_polygon`), cut by depth (unless all or none
/// are outside) and to the screen and drawn (`scene::draw_polygon_in_game`) in mode 5 with
/// texture `work.texture`. The registers it leaves are not read (its caller calls
/// it. Refuses what the projection and the clips refuse (after the work is set; not met with a
/// hull's few vertices) and a hull list past 0x40.
pub fn shadow_on_face(
    ctx: &crate::race::view_scratch::DrawCtx<'_>,
    work: &ShadowWork,
    car: &Car,
) -> Result<(), Refused> {
    let (race, scene) = (ctx.race, ctx.scene);
    let count: u32 = work.list_count.get();
    if count > 0x40 {
        return Err(BadInput("a shadow list past 0x40").into());
    }
    ctx.view.polygon.polygon_drawn.set(0u32);
    let list = &POLYGON_WORK.list;
    copy_on_in(
        (work, &work.list),
        (&race.shared.buffers, list),
        count as usize,
        "the shadow list",
    )?;
    ctx.view.polygon.polygon_vertex_count.set_bits(count);
    write_on_in(
        &race.shared.buffers,
        list,
        count as usize,
        work.list[0].get(),
        "the polygon list",
    )?;
    ctx.view.polygon.polygon_next_entry.set_bits(0x20u32);
    let scene_points: u32 = race.course.course.scene_point_count.get();
    work.next_point.set(scene_points.wrapping_add(0x30));
    ctx.view.polygon.polygon_sides[0].set(0u32);
    ctx.view.polygon.polygon_sides[1].set(1u32);
    let mut at = (scene_points & 0xFFF) * 0x20;
    for _ in 0..0x40 {
        crate::scene::point_records::put_field(
            crate::scene::point_records::at_offset(at),
            0,
            u32::MAX,
        )?;
        at = (at + 0x20) & 0x1FFE0;
    }
    clip_shadow_to_face(ctx, work, car)?;
    if ctx.view.polygon.polygon_vertex_count.bits() < 3 {
        return Ok(());
    }
    let n: u32 = ctx.view.polygon.polygon_vertex_count.bits();
    for k in 0..n {
        raise_onto_face(
            race,
            work,
            read_on_in(&race.shared.buffers, list, k as usize, "the polygon list")?,
        )?;
    }
    crate::scene::project_polygon::project_polygon(&ctx.view.polygon, race, scene)?;
    crate::scene::fog_polygon::fog_polygon(ctx.view)?;
    ctx.view
        .polygon
        .polygon_next_record
        .set(crate::scene::polygon::pool_start());
    let outside: u32 = ctx.view.polygon.polygon_outside_count.get();
    if outside != 0 {
        if outside == ctx.view.polygon.polygon_vertex_count.bits() {
            return Ok(());
        }
        crate::scene::clip_polygon_by_depth::clip_polygon_by_depth(
            &ctx.view.polygon,
            race,
            scene,
            ctx.far_clip,
            ctx.view,
        )?;
        if ctx.view.polygon.polygon_vertex_count.bits() < 3 {
            return Ok(());
        }
    }
    crate::scene::clip_polygon_to_screen::clip_polygon_to_screen(&ctx.view.polygon, race, scene)?;
    if ctx.view.polygon.polygon_vertex_count.bits() < 3 {
        return Ok(());
    }
    ctx.view.polygon.polygon_textures[0].set(u32::MAX);
    ctx.view.polygon.polygon_textures[1].set(work.texture.get());
    ctx.view.polygon.polygon_sides[0].set(0u32);
    ctx.view.polygon.polygon_sides[1].set(1u32);
    ctx.view.face.polygon_mode.set(5u32);
    crate::scene::draw_polygon::draw_polygon_in_game(
        &ctx.view.polygon,
        race,
        &ctx.view.face,
        scene,
    )?;
    ctx.view.polygon.polygon_drawn.set(1u32);
    Ok(())
}

/// `shadow_on_record`: each face of course record number `record` (its
/// address; +8 .. +0xC) with a kind below 0x10 and
/// without bit 7 of its +2 word gets the shadow (`shadow_on_face`) on its two triangles
/// (corners +4, +8, +0xC and +0x10, +0xC, +8); `g_currentCourseFace` each face.
fn shadow_on_record(
    ctx: &crate::race::view_scratch::DrawCtx<'_>,
    work: &ShadowWork,
    car: &Car,
    record: u32,
) -> Result<(), Refused> {
    use crate::race::course_places::{face_number, record_field};
    // Converted: `rc2k_core::race::shadow::{takes_shadow, SHADOW_TRIANGLES}`,
    // `rc2k_core::race::view::record_faces`.
    use rc2k_core::race::shadow::{SHADOW_TRIANGLES, takes_shadow};
    let Some((first, last)) = rc2k_core::race::view::record_faces(
        record_field(record, 8, "a course record")?,
        record_field(record, 0xC, "a course record")?,
    ) else {
        return Ok(());
    };
    for face in first..=last {
        ctx.view.face.course_face.set(face);
        let f = face_number(face, "a course face")?;
        let takes = takes_shadow(f.kind(), || f.switches());
        if takes {
            for corners in SHADOW_TRIANGLES {
                for (k, &o) in corners.iter().enumerate() {
                    work.face_corners[k].set_bits(f.dword_at(o));
                }
                shadow_on_face(ctx, work, car)?;
            }
        }
    }
    Ok(())
}

/// `race::shadow_faces` (0x457618): the shadow (`shadow_on_face`) on the last course
/// record's faces, then on every record in the 3 x 3 cells round the work car
/// (`shadow_on_record` each).
pub fn shadow_faces(
    ctx: &crate::race::view_scratch::DrawCtx<'_>,
    work: &ShadowWork,
    car: &Car,
) -> Result<(), Refused> {
    let race = ctx.race;
    let course = &race.course;
    let count: u32 = course.course.course_record_count.get();
    shadow_on_record(ctx, work, car, count.wrapping_sub(1))?;
    if count == 0 {
        return Ok(());
    }
    // Converted: `rc2k_core::race::shadow::{record_cell, cell_near_car}`.
    use rc2k_core::race::shadow::{cell_near_car, record_cell};
    // Every record of the course is looked at each frame: the run is resolved once.
    let all = crate::race::course_places::records_table(&race.course, count, "the course records")?;
    for n in 0..count {
        let o = (n as usize) << 5;
        let Some((x, z)) = record_cell(u32::from_le_bytes([
            all[o],
            all[o + 1],
            all[o + 2],
            all[o + 3],
        ])) else {
            continue;
        };
        if cell_near_car((car.position[0].bits(), car.position[2].bits()), (x, z)) {
            shadow_on_record(ctx, work, car, n)?;
        }
    }
    Ok(())
}

/// `fld tbyte; fstp dword`: 80 bits as a float.
fn tbyte_to_float(t: [u8; 10]) -> u32 {
    crate::math::x87_float::st(crate::math::x87_float::from_f80(t))
}

/// The sun's slant from `sun` and `turn` (the angles as 128ths of a half turn): `work.slant_x`
/// = cos(turn) x cot(sun), `work.slant_z` = sin(turn) x cot(sun)
/// (the original's store of 0x80 to `g_scratch` is not kept, nothing read it).
fn sun_slant(work: &ShadowWork, sun: u32, turn: u32) {
    use crate::math::x87_float::{int, st};
    let cot = {
        let (sin, cos) = (int(sun as i32) / int(0x80)).times_pi().sin_cos();
        cos / sin
    };
    let (sin, cos) = (int(turn as i32) / int(0x80)).times_pi().sin_cos();
    work.slant_x.set_bits(st(cos * cot));
    work.slant_z.set_bits(st(cot * sin));
}

/// `fld x; fld z; fpatan; fstp`: the angle of the corner at `at` (x at +0, z at +8).
fn angle_of(at: &ShadowPoint) -> u32 {
    use crate::math::x87_float::{ld, st};
    st(ld(at.x.get()).atan2(ld(at.z.get())))
}

/// `race::shadow_points` (0x458078): the shadow's points: the ground under the work car
/// (`physics::find_ground`, its height to `work.ground`); the sun's slant from
/// the sun's height `sun` (at least 0x10) and -`stage_prefs.sun_turn` - 0x40 (`work.slant_x`, `work.slant_z`);
/// each of the car's `work.point_count` corners (`g_car` +0xAF0 .. +0xCF0 by axis) turned by
/// its matrix (+0x840), moved along the slant by its height plus 0.7, its angle (`fpatan`)
/// and distance, the farthest kept (`work.farthest_length`, `work.farthest`), as whole numbers (x
/// 65536) plus the car's place. Refuses what `find_ground` refuses (before anything), and
/// no corners or more than 0x40.
pub fn shadow_points(race: &Race, work: &ShadowWork, car: &Car, sun: u32) -> Result<(), Refused> {
    use crate::math::x87::mul_to_int;
    use crate::math::x87_load::{below_or_unordered, to_tbyte};
    let count: u32 = work.point_count.bits();
    if count == 0 || count > 0x40 {
        return Err(BadInput("no shadow corners or more than 0x40").into());
    }
    let q = work.query(race);
    q.x.set_bits(car.position[0].bits());
    q.z.set_bits(car.position[2].bits());
    q.last.set_bits(car.probe_point_face[4].bits());
    crate::physics::find_ground::find_ground_with(race, &q, crate::scene::owner::scene())?;
    work.farthest_length.set(-1.0f32);
    work.ground.set_bits(q.height.get());
    let sun = sun.max(0x10);
    let turn = race
        .course
        .stage_prefs
        .sun_turn
        .get()
        .wrapping_neg()
        .wrapping_sub(0x40);
    sun_slant(work, sun, turn);
    let (sx, sz) = (work.slant_x.bits(), work.slant_z.bits());
    let lift: u32 = race.shared.physics_tables.camera_drop.bits();
    let unit: u32 = race.course.numbers.k65536.bits();
    for k in 0..count {
        let at = point_at(work, k)?;
        let k = k as usize;
        let p = [
            to_tbyte(car.probe_point_x[k].bits()),
            to_tbyte(car.probe_point_y[k].bits()),
            to_tbyte(car.probe_point_z[k].bits()),
        ];
        let t = crate::scene::rotate_point::rotate_point(&car.body_matrix, p);
        let [x, y, z] = t.map(tbyte_to_float);
        at.y.set(y);
        let x = lifted(y, lift, sx, x);
        at.x.set(x);
        let z = lifted(y, lift, sz, z);
        at.z.set(z);
        at.angle.set(angle_of(at));
        let length = crate::math::x87_load::float_hypot(x, z);
        at.length.set(length);
        if !below_or_unordered(length, work.farthest_length.bits()) {
            work.farthest_length.set_bits(length);
            work.farthest.set(k as u32);
        }
        at.x.set((mul_to_int(x, unit) as u32).wrapping_add(car.position[0].bits()));
        at.z.set((mul_to_int(z, unit) as u32).wrapping_add(car.position[2].bits()));
        at.y.set(work.ground.bits());
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
/// order, hull and polygon, the hull's list (`work.list`,
/// `work.list_count`), texture 0xC3 (`work.texture`), and the shadow
/// on the faces round the car (`shadow_faces`), with the sun at height `sun`. Refuses what its
/// parts refuse (before anything for the points and the counts).
pub fn cast_car_shadow(
    ctx: &crate::race::view_scratch::DrawCtx<'_>,
    work: &ShadowWork,
    car: &Car,
    sun: u32,
) -> Result<(), Refused> {
    let race = ctx.race;
    // Converted: `rc2k_core::race::shadow::{casts, hull_drawn, hull_list,
    // SHADOW_TEXTURE}`.
    use rc2k_core::race::shadow::{SHADOW_TEXTURE, casts, hull_drawn, hull_list};
    if !casts(work.point_count.bits()) {
        return Ok(());
    }
    shadow_points(race, work, car, sun)?;
    shadow_angles(race, work)?;
    shadow_hull(work)?;
    if !hull_drawn(work.hull_count.get()) {
        return Ok(());
    }
    shadow_polygon(race, work)?;
    let n: u32 = work.hull_count.get();
    work.list_count.set(n);
    for k in hull_list(n) {
        crate::session::write_on_in(work, &work.list, k as usize, k, "the shadow list")?;
    }
    work.texture.set(SHADOW_TEXTURE);
    shadow_faces(ctx, work, car)
}

/// `race::cast_car_shadows` (0x45754C): the depth test off, `work.lift`
/// 0x1999, then for each race car with +0x6FC not 0: its block read directly (the original
/// loaded it into the work copy, `drive::load_car`), its corner count from its +0x260, `cast_car_shadow` once with
/// `RACE_VARS.settings_words[2]` set, then once more with the sun at height 0x40 (the original
/// set `stage_prefs.sun_height` to it and put it back after: an argument here); the depth
/// test on. Refuses (before anything) no cars or more than eight, or a
/// null block.
pub fn cast_car_shadows(ctx: &crate::race::view_scratch::DrawCtx<'_>) -> Result<(), Refused> {
    let race = ctx.race;
    // Converted: `rc2k_core::race::shadow::{check_shadow_cars, casts_shadow, first_pass,
    // SHADOW_STRENGTH, SUN_PASS}`.
    use rc2k_core::race::shadow::{
        SHADOW_STRENGTH, SUN_PASS, casts_shadow, check_shadow_cars, first_pass,
    };
    let cars: u32 = race.shared.view.race_cars.get();
    let block = |n: u32| race.shared.work.car_block(n as usize);
    check_shadow_cars(cars, block)?;
    let work = ShadowWork::new();
    crate::scene::depth_test_off::depth_test_off();
    work.lift.set(SHADOW_STRENGTH);
    for n in 0..cars {
        // Each car's shadow from its own block (the original loaded it into the work car,
        // `drive::load_car`, and left the last one there for the rest of the view:
        // `notes/bugs.md`).
        let car = crate::race::blocks_store::block_at(block(n), "a car's block")?;
        if !casts_shadow(ctx.view.cars.state_of(block(n))?.get()) {
            continue;
        }
        work.point_count.set_bits(car.probe_count.bits());
        if first_pass(race.shared.vars.settings_words[2].bits()) {
            let sun: u32 = race.course.stage_prefs.sun_height.get();
            cast_car_shadow(ctx, &work, car, sun)?;
        }
        cast_car_shadow(ctx, &work, car, SUN_PASS)?;
    }
    crate::scene::depth_test_on::depth_test_on();
    Ok(())
}

#[cfg(all(test, target_arch = "x86"))]
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

    pub(super) unsafe fn inside_edge_asm(n: u32, points: &[u32]) -> bool {
        let physics = &race().physics;
        unsafe {
            physics.ground.ground_x.set(points[3 * n as usize]);
            physics.ground.ground_z.set(points[3 * n as usize + 2]);
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
        let race = race();
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
                race.course
                    .course
                    .scene_points
                    .set(crate::pointers::game_address(points.as_ptr()));
                for a in &places[2..6] {
                    game_write(rva(*a), r.int() as u32);
                }
                race.physics.ground.ground_x.set(0xAAAA_AAAAu32);
                race.physics.ground.ground_z.set(0xAAAA_AAAAu32);
                let n = r.below(4);
                let want = unsafe { inside_edge_asm(n, &points) };
                let seen = (
                    race.physics.ground.ground_x.get(),
                    race.physics.ground.ground_z.get(),
                );
                // The edge the original read at 0x641948 .., the Rust version's in its work.
                let work = ShadowWork::new();
                for (k, a) in places[2..6].iter().enumerate() {
                    work.edge[k].set(game_read::<u32>(rva(*a)));
                }
                let got =
                    crate::race::scene_points::with_test_points(&points, || inside_edge(&work, n))
                        .expect("the test's memory is registered");
                assert_eq!(want, got, "{points:x?} {n}");
                // The point the original left in `GROUND_WORK`, the Rust version in the
                // shadow's own search fields.
                let q = work.search.query();
                assert_eq!(seen, (q.x.get(), q.z.get()));
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
                // The work dwords the original read at 0x641978 .., the Rust version's in its
                // work.
                let work = ShadowWork::new();
                let poly = crate::scene::polygon_work::PolygonWork::new();
                for (k, a) in places[1..7].iter().enumerate() {
                    work.crossing[8 + k].set(game_read::<u32>(rva(*a)));
                }
                let both = [pa, pb].concat();
                crate::race::scene_points::with_test_points(&both, || {
                    let entry = |a: u32| {
                        GamePtr::new(a)
                            .record::<PolygonEntry>("the test's entry")
                            .expect("the test's memory is registered")
                    };
                    crossing_block(&poly, &work, 0, 1, entry(args.2), entry(args.3))
                })
                .expect("the test's memory is registered");
                // The Rust version writes the view's work: put its pieces where `read` looks.
                let old = &crate::scene::owner::scene().drawing.scene;
                for k in [2, 4, 10, 11] {
                    old.clip_work[k].set_bits(poly.clip_work[k].bits());
                }
                old.clip_work_2[0].set(poly.clip_work_2[0].get());
                old.vertex_byte_21.set(poly.vertex_byte_21.get());
                let mut got = read();
                // `plane_f` (0x639810) the Rust version keeps in the shadow's own search fields.
                got[7] = work.search.query().plane_f.bits();
                // Place 0 (0x62400C, the assembly's spill dword) is not kept by the Rust version.
                for k in 1..places.len() {
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
                // digits differ (shadow_skids.slant_x and shadow_skids.slant_z are then about 1e-17 or 1e16 in
                // the original, whatever the processor). Left out here, reported.
                if sun.is_multiple_of(64) || turn.is_multiple_of(64) {
                    continue;
                }
                unsafe { sun_slant_asm(sun, turn) };
                let want = [0x62400C, 0x643AC4, 0x643AC8].map(|a| game_read::<u32>(rva(a)));
                for a in [0x62400C, 0x643AC4, 0x643AC8] {
                    game_write(rva(a), 0xAAAA_AAAAu32);
                }
                let work = ShadowWork::new();
                sun_slant(&work, sun, turn);
                // The assembly's 0x80 store to 0x62400C (want[0]) is not kept by the Rust version.
                let got = [0, work.slant_x.bits(), work.slant_z.bits()];
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
                use crate::race::shadow_skids_store::ShadowPoint;
                use crate::session::Dword;
                let point = ShadowPoint {
                    x: Dword::new(at[0]),
                    y: Dword::new(at[1]),
                    z: Dword::new(at[2]),
                    angle: Dword::new(0),
                    length: Dword::new(0),
                    zeroed: Dword::new(0),
                    unknown_18: [Dword::new(0), Dword::new(0)],
                };
                let (want, got) = (
                    unsafe { angle_asm(crate::pointers::game_address(at.as_ptr())) },
                    angle_of(&point),
                );
                assert!(within_ulps(want, got, 1), "{at:x?}: {want:#x} {got:#x}");
                worst = worst.max(want.abs_diff(got));
            }
        });
        eprintln!("sun_slant, angle_of: worst difference {worst} ulps");
    }
}
