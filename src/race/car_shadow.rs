//! The car's shadow (the tree under `cast_car_shadows`; a dark polygon under
//! each car, the hull of its corners on the ground, cut by the course's faces; drawn per car whose
//! +0x6FC is set, once for the "reduced" car shadows option and twice for "full" (strings 0x396 /
//! 0x397, `settings_words[2]`): once cast along the stage's sun height, once with the sun straight
//! above at height 0x40. Settled from `cast_car_shadows` and the options page). Its
//! leaves and shared code. Its working values are a `ShadowWork` of its own (frame plan F4):
//! the point records of 0x20 bytes at `work.points` (+0 x, +4 y, +8 z as whole numbers, +0xC
//! the angle, +0x10 the distance, +0x14 0), their count `work.point_count`, the hull's
//! `work.hull_count`, ...

use crate::effects::cell_x87::CellX87;
use crate::errors::{BadInput, Refused};
use crate::physics::car_store::Car;
use crate::physics::ground_probe_store::GroundProbe;
use crate::physics::ground_query::{GroundCells, GroundQuery};
use crate::race::owner_course::CourseReads;
use crate::race::scene_points::{OwnPoints, point_word, point_xz, put_point_word};
use crate::race::shadow_skids_store::ShadowPoint;
use crate::race::trap::Trap;
use crate::scene::polygon::{list_cell, old_list_cell};
use crate::scene::polygon_store::PolygonEntry;
use crate::session::{Byte, PlainDword, PlainFloat};

/// The shadow's working state: what `cast_car_shadows` writes and reads within itself, one
/// for each call, given down to its parts. The original kept it in the race block
/// (`g_shadowPoints` and after at 0x641AA4 .. 0x643ACC, `g_shadowPointCount` and its
/// neighbours at 0x641928 .. 0x641AA0), left from view to view; nothing else read it, and
/// each call wrote every value before reading it.
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct ShadowWork {
    /// The shadow's points, records of 0x20 bytes (0x40 at most), the hull's records after.
    points: [ShadowPoint; 0x100],
    /// The car's corners (`g_shadowPointCount`; from its block's +0x260).
    point_count: PlainDword,
    /// The length of the farthest point (`shadow_points`).
    farthest_length: PlainFloat,
    /// The farthest point's number (the original kept its record's address).
    farthest: PlainDword,
    /// The first point's angle (`shadow_angles`).
    first_angle: PlainFloat,
    /// cos(turn) x cot(sun) (`sun_slant`).
    slant_x: PlainFloat,
    /// sin(turn) x cot(sun).
    slant_z: PlainFloat,
    /// The shadow hull's points.
    hull_count: PlainDword,
    /// What the shadow is raised above the face (0x1999, `cast_car_shadows`).
    lift: PlainDword,
    /// The shadow's texture (0xC3).
    texture: PlainDword,
    /// The three corners (scene point numbers) of the course face the shadow is cut to.
    face_corners: [PlainDword; 3],
    /// The edge cut by: x and z from, x and z to (`clip_to_edge`).
    edge: [PlainDword; 4],
    /// `edge_crossing`'s work dwords (0x641958 .. 0x64198C in the original).
    crossing: [PlainDword; 14],
    /// The next new scene point (`shadow_on_face`, `edge_crossing`).
    next_point: PlainDword,
    /// The hull's list.
    list: [PlainDword; 64],
    /// Its count.
    list_count: PlainDword,
    /// The ground's height under the car.
    ground: PlainDword,
    /// The ground searches' fields (the original's were the physics', `GROUND_WORK` ..).
    search: GroundCells,
}

impl ShadowWork {
    pub fn new() -> Self {
        Self {
            points: std::array::from_fn(|_| ShadowPoint::new()),
            point_count: PlainDword::new(0),
            farthest_length: PlainFloat::new(0.0),
            farthest: PlainDword::new(0),
            first_angle: PlainFloat::new(0.0),
            slant_x: PlainFloat::new(0.0),
            slant_z: PlainFloat::new(0.0),
            hull_count: PlainDword::new(0),
            lift: PlainDword::new(0),
            texture: PlainDword::new(0),
            face_corners: [const { PlainDword::new(0) }; 3],
            edge: [const { PlainDword::new(0) }; 4],
            crossing: [const { PlainDword::new(0) }; 14],
            next_point: PlainDword::new(0),
            list: [const { PlainDword::new(0) }; 64],
            list_count: PlainDword::new(0),
            ground: PlainDword::new(0),
            search: GroundCells::new(),
        }
    }

    /// The ground search on the shadow's own fields, the scene points through `points`; the
    /// caller's mark (`settle_mark`), which the search only reads, is the physics' (`probe`)
    /// as the last search there left it.
    fn query<'a>(&'a self, probe: &'a GroundProbe, points: &'a OwnPoints) -> GroundQuery<'a> {
        GroundQuery {
            settle_mark: &probe.settle_mark,
            ..self.search.query(points)
        }
    }
}

/// The circle's radius and unit (`g_shadowCircle`, `FrameWork.shadow_circle[1]`: 2^23 and 2^24, never written).
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

/// `race::shadow_angles` (0x4581F3): the farthest point (`work.farthest`) swapped into the first
/// record; each record's angle made relative to the first's (`work.first_angle`; below 0 plus 2
/// pi), times 65536 as a whole number, +0x14 0; with two records or more sorted by angle
/// (`sorting::sort_records`). Refuses (before anything) no points (its loop tests at its end) or
/// more than 0x40.
pub fn shadow_angles(
    numbers: &crate::race::numbers_store::Numbers,
    work: &ShadowWork,
) -> Result<(), Refused> {
    use crate::math::x87::{mul_to_int, sub};
    let count: u32 = work.point_count.get();
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
    work.first_angle.set(f32::from_bits(first));
    let zero = numbers.zero.get();
    let unit = numbers.k65536.get();
    for k in 0..count {
        let record = point_at(work, k)?;
        let mut v = sub(record.angle.get(), first);
        if crate::math::x87::below_or_unordered(v, zero) {
            v = f32::from_bits(two_pi_plus(v.to_bits()));
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
/// 64-bit cross product of whole numbers, `GROUND_PROBE.product` its first half, not negative) is
/// taken out (the rest moved down, starting again), until none is; `work.hull_count` the points
/// left. Refuses (before anything) fewer than two points or more than 0x40.
pub fn shadow_hull(work: &ShadowWork) -> Result<(), Refused> {
    // Converted: `rc2k_core::race::shadow::hull`.
    use rc2k_core::race::shadow::{Record, hull, hull_records};
    let count: u32 = work.point_count.get();
    if !(2..=0x40).contains(&count) {
        return Err(BadInput("fewer than two shadow points or more than 0x40").into());
    }
    let at = |n: usize| point_at(work, n as u32);
    let mut records: Vec<Record> = Vec::new();
    for n in 0..hull_records(count) {
        records.push(crate::cells::load_value::<Record>(at(n)?, 0)?);
    }
    let before = records.clone();
    let h = hull(&mut records, count)?;
    for (n, (now, was)) in records.iter().zip(&before).enumerate() {
        if now != was {
            crate::cells::store_value(at(n)?, 0, *now)?;
        }
    }
    work.search.product()[0].set(h.product as u32);
    work.search.product()[1].set((h.product as u64 >> 32) as u32);
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
/// from `g_scenePointCount` + 0x10: x and z the record's, y `work.ground`) and as the polygon's
/// work entries (+0 the point, +0x18 white, +0x21 `stage_prefs.shadow_alpha` / 2, +0x10 / +0x14 a
/// circle's texture place: cos / sin of 2 pi k / n times `CIRCLE[0]` plus it, rounded, over
/// `CIRCLE[1]`). Refuses (before anything) a hull of no points (its loop tests at its end) or more
/// than 0x40.
pub fn shadow_polygon(
    polygon_work: &crate::scene::polygon_store::PolygonWork,
    points: &OwnPoints,
    work: &ShadowWork,
    reads: crate::workers::DrawReads<'_>,
) -> Result<(), Refused> {
    let n: u32 = work.hull_count.get();
    if n == 0 || n > 0x40 {
        return Err(BadInput("a shadow hull of no points or more than 0x40").into());
    }
    let scene_points: u32 = reads.course_work.scene_point_count.get();
    for k in 0..n {
        // The hull's points follow the course's points and 16 more.
        let (record, point) = (
            point_at(work, k)?,
            scene_points.wrapping_add(0x10).wrapping_add(k),
        );
        put_point_word(reads.course(), points, point, 0, record.x.get())?;
        put_point_word(reads.course(), points, point, 2, record.z.get())?;
        put_point_word(reads.course(), points, point, 1, work.ground.get())?;
        let e = &polygon_work.entries[k as usize];
        e.point.set(point);
        e.light.set(0x00FF_FFFFu32);
        e.amount
            .set((reads.stage_prefs.shadow_alpha.get() >> 1) as u8);
        circle_place(e, k, n, CIRCLE[0], CIRCLE[1]);
    }
    Ok(())
}

/// A polygon work entry's address (`POLYGON_WORK.entries` + 0x44 x n).
fn work_entry(
    polygon_work: &crate::scene::polygon_store::PolygonWork,
    n: u32,
) -> Result<&PolygonEntry, BadInput> {
    polygon_work
        .entries
        .get(n as usize)
        .ok_or(BadInput("a polygon work entry past 0x80"))
}

/// `inside_edge`: whether scene point `n` is on the inner side of the edge from
/// (`work.edge`, `work.edge[1]`) to (`work.edge[2]`, `work.edge[3]`): the cross product
/// of whole numbers on the x87 (the original's instructions) above 0, or not a number
/// (`fldz; fcompp`, C0). The point's x and z into `GROUND_WORK.ground_x` / `GROUND_WORK.ground_z`.
fn inside_edge(
    course: CourseReads<'_>,
    points: &OwnPoints,
    work: &ShadowWork,
    n: u32,
) -> Result<bool, BadInput> {
    let (x, z) = point_xz(course, points, n)?;
    let q = work.search.query(points);
    q.x.set(x);
    q.z.set(z);
    use crate::math::x87_float::{X, compare, int};
    let word = |d: &dyn crate::session::DwordLike| int(d.get() as i32);
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
/// texture floats (`DRAW.clip_work[10]`, `DRAW.clip_work[11]`) and two bytes (`DRAW.crossing_amount`, `DRAW.clip_out_words`)
/// shared the same way between the scene points numbered `pa`, `pb` and the entries `ea`, `eb`;
/// The original's store of the share's bits to `g_scratch` is not kept (nothing read it).
fn crossing_block(
    course: CourseReads<'_>,
    points: &OwnPoints,
    poly: &crate::scene::polygon_work::PolygonWork,
    work: &ShadowWork,
    (pa, pb): (u32, u32),
    ea: &PolygonEntry,
    eb: &PolygonEntry,
) -> Result<(), BadInput> {
    use crate::math::x87_float::{int, ld, stf, to_int};
    let word = |n: usize| int(work.crossing[n].get() as i32);
    let first = word(11) * word(8) - word(10) * word(9);
    let second = word(13) * word(8) - word(12) * word(9);
    let share = stf(-(first / (second - first)));
    work.search.plane_f().set(share);
    let share = ld(share);
    // a + (b - a) x share
    let whole = |k: u32| -> Result<i32, BadInput> {
        let (a, b) = (
            point_word(course, points, pa, k)?,
            point_word(course, points, pb, k)?,
        );
        Ok(to_int(
            int(a as i32) + (int(b as i32) - int(a as i32)) * share,
        ))
    };
    poly.clip_work[2].set_bits(whole(0)? as u32);
    poly.clip_work[4].set_bits(whole(2)? as u32);
    let float = |a: &dyn crate::session::DwordLike, b: &dyn crate::session::DwordLike| {
        stf(ld(a.get()) + (ld(b.get()) - ld(a.get())) * share)
    };
    poly.clip_work[10].set(float(&ea.u2, &eb.u2));
    poly.clip_work[11].set(float(&ea.v2, &eb.v2));
    let byte = |a: &Byte, b: &Byte| {
        let (a, b) = (int(a.get().into()), int(b.get().into()));
        to_int(a + (b - a) * share) as u32
    };
    poly.crossing_amount.set(byte(&ea.amount, &eb.amount));
    poly.clip_out_words[0].set(byte(&ea.fog, &eb.fog));
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
    let (pa, pb) = (ea.point.get(), eb.point.get());
    let c = &work.edge;
    let w = &work.crossing;
    let rd = |d: &dyn crate::session::DwordLike| d.get();
    let points = &ctx.part.points;
    let ((bx, bz), (ax, az)) = (
        point_xz(ctx.reads.course(), points, pb)?,
        point_xz(ctx.reads.course(), points, pa)?,
    );
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
    crossing_block(
        ctx.reads.course(),
        points,
        &ctx.view.polygon,
        work,
        (pa, pb),
        ea,
        eb,
    )?;
    let n = rd(&work.next_point);
    put_point_word(
        ctx.reads.course(),
        points,
        n,
        0,
        ctx.view.polygon.clip_work[2].bits(),
    )?;
    put_point_word(ctx.reads.course(), points, n, 1, 0)?;
    put_point_word(
        ctx.reads.course(),
        points,
        n,
        2,
        ctx.view.polygon.clip_work[4].bits(),
    )?;
    ed.point.set(n);
    work.next_point.set(n.wrapping_add(1));
    ed.u2.set(ctx.view.polygon.clip_work[10].get().to_bits());
    ed.v2.set(ctx.view.polygon.clip_work[11].get().to_bits());
    ed.amount.set(rd(&ctx.view.polygon.crossing_amount) as u8);
    ed.fog.set(rd(&ctx.view.polygon.clip_out_words[0]) as u8);
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
    let count: u32 = ctx.view.polygon.polygon_vertex_count.get();
    if count < 3 {
        return Ok(());
    }
    let points = &ctx.part.points;
    let ((tx, tz), (fx, fz)) = (
        point_xz(ctx.reads.course(), points, to)?,
        point_xz(ctx.reads.course(), points, from)?,
    );
    work.edge[2].set(tx);
    work.edge[3].set(tz);
    work.edge[0].set(fx);
    work.edge[1].set(fz);
    // The polygon list holds 0x40 numbers and the old list follows it: a cut that adds
    // crossings past 0x40 runs on from the list into the old list, and from the old list
    // into the gap after it, as the original (`list_cell` names the three fields). A hull of
    // at most 0x40 points cut by three edges adds at most as many again as it holds each time.
    let polygon = &ctx.part.polygon;
    for k in 0..count as usize {
        old_list_cell(polygon, k).set(list_cell(polygon, k).get());
    }
    old_list_cell(polygon, count as usize).set(list_cell(polygon, 0).get());
    // Converted: `rc2k_core::race::shadow::clip_by_edge`.
    let mut added = 0u32;
    let trap = Trap::new();
    rc2k_core::race::shadow::clip_by_edge(
        count,
        |k| old_list_cell(polygon, k as usize).get(),
        |e| {
            let entry = match work_entry(&ctx.part.polygon, e) {
                Ok(x) => x.point.get(),
                Err(err) => {
                    trap.keep(err);
                    0
                }
            };
            inside_edge(ctx.reads.course(), points, work, entry).unwrap_or_else(|err| {
                trap.keep(err);
                false
            })
        },
        |a, b| {
            let next: u32 = ctx.view.polygon.polygon_next_entry.get();
            let crossing = (|| {
                edge_crossing(
                    ctx,
                    work,
                    work_entry(&ctx.part.polygon, a)?,
                    work_entry(&ctx.part.polygon, b)?,
                    work_entry(&ctx.part.polygon, next)?,
                )
            })();
            if let Err(err) = crossing {
                trap.keep(err);
            }
            ctx.view
                .polygon
                .polygon_next_entry
                .set(next.wrapping_add(1));
            next
        },
        |e| {
            list_cell(polygon, added as usize).set(e);
            added += 1;
        },
    );
    trap.finish()?;
    ctx.view.polygon.polygon_vertex_count.set(added);
    Ok(())
}

/// `race::clip_shadow_to_face` (0x457960): the shadow's polygon cut to the course face whose
/// corners are `work.face_corners` .. `work.face_corners[2]` (scene points): none (count 0) when a
/// corner is more than 0x100000 off the work car in x before one within 0x100000 in z is met (or
/// none is); else cut by its three edges in turn while three vertices are left (`clip_to_edge`,
/// with `inside_edge` the side test and `edge_crossing` the crossing).
pub fn clip_shadow_to_face(
    ctx: &crate::race::view_scratch::DrawCtx<'_>,
    work: &ShadowWork,
    car: &Car,
) -> Result<(), BadInput> {
    let corners = [
        work.face_corners[0].get(),
        work.face_corners[1].get(),
        work.face_corners[2].get(),
    ];
    // Converted: `rc2k_core::race::shadow::face_near`.
    let trap = Trap::new();
    let found =
        rc2k_core::race::shadow::face_near((car.position[0].get(), car.position[2].get()), |n| {
            point_xz(ctx.reads.course(), &ctx.part.points, corners[n]).unwrap_or_else(|e| {
                trap.keep(e);
                (0, 0)
            })
        });
    trap.finish()?;
    if !found.near {
        ctx.view.polygon.polygon_vertex_count.set(0u32);
        return Ok(());
    }
    for (a, b) in [(0usize, 1usize), (1, 2), (2, 0)] {
        if ctx.view.polygon.polygon_vertex_count.get() < 3 {
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
fn raise_onto_face(
    polygon_work: &crate::scene::polygon_store::PolygonWork,
    points: &OwnPoints,
    reads: crate::workers::DrawReads<'_>,
    work: &ShadowWork,
    index: u32,
) -> Result<(), Refused> {
    use crate::physics::ground::face_shares_at;
    let (probe, switches) = (reads.ground_probe, reads.draw_switches);
    let p = work_entry(polygon_work, index)?.point.get();
    let q = work.query(probe, points);
    for k in 0..3 {
        q.corner_numbers[k].set(work.face_corners[k].get());
    }
    let (x, z) = point_xz(reads.course(), points, p)?;
    q.x.set(x);
    q.z.set(z);
    face_shares_at(reads.course(), &q, switches)?;
    crate::physics::plane_height::plane_height(&q);
    put_point_word(
        reads.course(),
        points,
        p,
        1,
        q.height.get().wrapping_add(work.lift.get()),
    )?;
    Ok(())
}

/// `race::shadow_on_face` (0x45773C): the shadow's polygon (the hull's list at `work.list`, its
/// count `work.list_count`) cut to the face `work.face_corners` .. and drawn on it: new points from
/// `g_scenePointCount` + 0x30 (`work.next_point`), entries from 0x20, the 0x40 point records after
/// the scene's marked unmade; `clip_shadow_to_face`; with three vertices or more each raised onto
/// the face, projected (`scene::project_polygon`, `scene::fog_polygon`), cut by depth (unless all
/// or none are outside) and to the screen and drawn (`scene::draw_polygon_in_game`) in mode 5 with
/// texture `work.texture`. Refuses what the projection and the clips refuse (after the work is set;
/// not met with a hull's few vertices) and a hull list past 0x40.
pub fn shadow_on_face(
    ctx: &crate::race::view_scratch::DrawCtx<'_>,
    work: &ShadowWork,
    car: &Car,
) -> Result<(), Refused> {
    let count: u32 = work.list_count.get();
    if count > 0x40 {
        return Err(BadInput("a shadow list past 0x40").into());
    }
    ctx.view.polygon.polygon_drawn.set(0u32);
    let polygon = &ctx.part.polygon;
    for k in 0..count as usize {
        list_cell(polygon, k).set(work.list[k].get());
    }
    ctx.view.polygon.polygon_vertex_count.set(count);
    list_cell(polygon, count as usize).set(work.list[0].get());
    ctx.view.polygon.polygon_next_entry.set(0x20u32);
    let scene_points: u32 = ctx.reads.course_work.scene_point_count.get();
    work.next_point.set(scene_points.wrapping_add(0x30));
    ctx.view.polygon.polygon_sides[0].set(0u32);
    ctx.view.polygon.polygon_sides[1].set(1u32);
    let mut at = (scene_points & 0xFFF) * 0x20;
    for _ in 0..0x40 {
        crate::scene::point_records::put_field(
            ctx.part,
            crate::scene::point_records::at_offset(at),
            0,
            u32::MAX,
        )?;
        at = (at + 0x20) & 0x1FFE0;
    }
    clip_shadow_to_face(ctx, work, car)?;
    if ctx.view.polygon.polygon_vertex_count.get() < 3 {
        return Ok(());
    }
    let n: u32 = ctx.view.polygon.polygon_vertex_count.get();
    for k in 0..n {
        raise_onto_face(
            &ctx.part.polygon,
            &ctx.part.points,
            ctx.reads,
            work,
            list_cell(&ctx.part.polygon, k as usize).get(),
        )?;
    }
    crate::scene::project_polygon::project_polygon(
        ctx.part,
        &ctx.view.polygon,
        ctx.reads,
        ctx.view,
    )?;
    crate::scene::fog_polygon::fog_polygon(ctx.part, ctx.view)?;
    ctx.view
        .polygon
        .polygon_next_record
        .set(crate::scene::polygon::pool_start());
    let outside: u32 = ctx.view.polygon.polygon_outside_count.get();
    if outside != 0 {
        if outside == ctx.view.polygon.polygon_vertex_count.get() {
            return Ok(());
        }
        crate::scene::clip_polygon_by_depth::clip_polygon_by_depth(
            ctx.part,
            &ctx.view.polygon,
            ctx.reads.course(),
            ctx.reads.draw_switches,
            ctx.far_clip,
            ctx.view,
        )?;
        if ctx.view.polygon.polygon_vertex_count.get() < 3 {
            return Ok(());
        }
    }
    crate::scene::clip_polygon_to_screen::clip_polygon_to_screen(
        ctx.part,
        &ctx.view.polygon,
        ctx.reads.course(),
    )?;
    if ctx.view.polygon.polygon_vertex_count.get() < 3 {
        return Ok(());
    }
    ctx.view.polygon.polygon_textures[0].set(u32::MAX);
    ctx.view.polygon.polygon_textures[1].set(work.texture.get());
    ctx.view.polygon.polygon_sides[0].set(0u32);
    ctx.view.polygon.polygon_sides[1].set(1u32);
    ctx.view.face.polygon_mode.set(5u32);
    crate::scene::draw_polygon::draw_polygon_in_game(
        ctx.view,
        ctx.part,
        ctx.reads,
        ctx.reads.screen,
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
        record_field(ctx.reads.course(), record, 8)?,
        record_field(ctx.reads.course(), record, 0xC)?,
    ) else {
        return Ok(());
    };
    for face in first..=last {
        ctx.view.face.course_face.set(face);
        let f = face_number(ctx.reads.course(), face)?;
        let takes = takes_shadow(f.kind(), || f.switches());
        if takes {
            for corners in SHADOW_TRIANGLES {
                for (k, &o) in corners.iter().enumerate() {
                    work.face_corners[k].set(f.dword_at(o));
                }
                shadow_on_face(ctx, work, car)?;
            }
        }
    }
    Ok(())
}

/// `race::shadow_faces` (0x457618): the shadow (`shadow_on_face`) on the last course record's
/// faces, then on every record in the 3 x 3 cells round the work car (`shadow_on_record` each).
pub fn shadow_faces(
    ctx: &crate::race::view_scratch::DrawCtx<'_>,
    work: &ShadowWork,
    car: &Car,
) -> Result<(), Refused> {
    let count: u32 = ctx.reads.course_work.course_record_count.get();
    shadow_on_record(ctx, work, car, count.wrapping_sub(1))?;
    if count == 0 {
        return Ok(());
    }
    // Converted: `rc2k_core::race::shadow::{record_cell, cell_near_car}`.
    use rc2k_core::race::shadow::{cell_near_car, record_cell};
    // Every record of the course is looked at each frame: the run is resolved once.
    let all = crate::race::course_places::records_table(ctx.reads.course(), count)?;
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
        if cell_near_car((car.position[0].get(), car.position[2].get()), (x, z)) {
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
    use crate::math::x87_float::int;
    let cot = {
        let (sin, cos) = (int(sun as i32) / int(0x80)).times_pi().sin_cos();
        cos / sin
    };
    let (sin, cos) = (int(turn as i32) / int(0x80)).times_pi().sin_cos();
    work.slant_x.st(cos * cot);
    work.slant_z.st(cot * sin);
}

/// `fld x; fld z; fpatan; fstp`: the angle of the corner at `at` (x at +0, z at +8).
fn angle_of(at: &ShadowPoint) -> u32 {
    use crate::math::x87_float::{ld, st};
    st(ld(at.x.get()).atan2(ld(at.z.get())))
}

/// `race::shadow_points` (0x458078): the shadow's points: the ground under the work car
/// (`physics::find_ground`, its height to `work.ground`); the sun's slant from the sun's height
/// `sun` (at least 0x10) and -`stage_prefs.sun_turn` - 0x40 (`work.slant_x`, `work.slant_z`); each
/// of the car's `work.point_count` corners (`g_car` +0xAF0 .. +0xCF0 by axis) turned by its matrix
/// (+0x840), moved along the slant by its height plus 0.7, its angle (`fpatan`) and distance, the
/// farthest kept (`work.farthest_length`, `work.farthest`), as whole numbers (x 65536) plus the
/// car's place. Refuses what `find_ground` refuses (before anything), and no corners or more than
/// 0x40.
#[allow(
    clippy::too_many_arguments,
    reason = "the drawing's stores the part reads, as they are lent"
)]
pub fn shadow_points(
    reads: crate::workers::DrawReads<'_>,
    points: &OwnPoints,
    work: &ShadowWork,
    car: &Car,
    sun: u32,
) -> Result<(), Refused> {
    let (probe, switches) = (reads.ground_probe, reads.draw_switches);
    use crate::math::x87::mul_to_int;
    use crate::math::x87::{below_or_unordered, to_tbyte};
    let count: u32 = work.point_count.get();
    if count == 0 || count > 0x40 {
        return Err(BadInput("no shadow corners or more than 0x40").into());
    }
    let q = work.query(probe, points);
    q.x.set(car.position[0].get());
    q.z.set(car.position[2].get());
    q.last.set(car.probe_point_face[4].get());
    crate::physics::find_ground::find_ground_with(reads.course(), &q, switches)?;
    work.farthest_length.set(-1.0f32);
    work.ground.set(q.height.get());
    let sun = sun.max(0x10);
    let turn = reads
        .stage_prefs
        .sun_turn
        .get()
        .wrapping_neg()
        .wrapping_sub(0x40);
    sun_slant(work, sun, turn);
    let (sx, sz) = (work.slant_x.get(), work.slant_z.get());
    let lift = reads.physics_tables.camera_drop.get();
    let unit = reads.numbers.k65536.get();
    for k in 0..count {
        let at = point_at(work, k)?;
        let k = k as usize;
        let p = [
            to_tbyte(&car.probe_point_x[k]),
            to_tbyte(&car.probe_point_y[k]),
            to_tbyte(&car.probe_point_z[k]),
        ];
        let t = crate::scene::rotate_point::rotate_point(&car.body_matrix, p);
        let [x, y, z] = t.map(tbyte_to_float);
        at.y.set(y);
        let x = lifted(y, lift, sx, x);
        at.x.set(x);
        let z = lifted(y, lift, sz, z);
        at.z.set(z);
        at.angle.set(angle_of(at));
        let length = crate::math::x87::float_hypot(x, z);
        at.length.set(length.to_bits());
        if !below_or_unordered(length, work.farthest_length.get()) {
            work.farthest_length.set(length);
            work.farthest.set(k as u32);
        }
        at.x.set((mul_to_int(x, unit) as u32).wrapping_add(car.position[0].get()));
        at.z.set((mul_to_int(z, unit) as u32).wrapping_add(car.position[2].get()));
        at.y.set(work.ground.get());
    }
    Ok(())
}

/// `fld y; fadd lift; fmul slant; fadd base; fstp`: a corner moved along the sun's slant,
/// rounded once.
fn lifted(
    y: u32,
    lift: impl crate::math::x87_float::Loadable,
    slant: impl crate::math::x87_float::Loadable,
    base: u32,
) -> u32 {
    use crate::math::x87_float::{ld, st};
    st((ld(y) + ld(lift)) * ld(slant) + ld(base))
}

/// `race::cast_car_shadow` (0x4575C1): with three corners or more, the shadow's points, their
/// order, hull and polygon, the hull's list (`work.list`, `work.list_count`), texture 0xC3
/// (`work.texture`), and the shadow on the faces round the car (`shadow_faces`), with the sun at
/// height `sun`. Refuses what its parts refuse (before anything for the points and the counts).
pub fn cast_car_shadow(
    ctx: &crate::race::view_scratch::DrawCtx<'_>,
    work: &ShadowWork,
    car: &Car,
    sun: u32,
) -> Result<(), Refused> {
    // Converted: `rc2k_core::race::shadow::{casts, hull_drawn, hull_list,
    // SHADOW_TEXTURE}`.
    use rc2k_core::race::shadow::{SHADOW_TEXTURE, casts, hull_drawn, hull_list};
    if !casts(work.point_count.get()) {
        return Ok(());
    }
    shadow_points(ctx.reads, &ctx.part.points, work, car, sun)?;
    shadow_angles(ctx.reads.numbers, work)?;
    shadow_hull(work)?;
    if !hull_drawn(work.hull_count.get()) {
        return Ok(());
    }
    shadow_polygon(&ctx.part.polygon, &ctx.part.points, work, ctx.reads)?;
    let n: u32 = work.hull_count.get();
    work.list_count.set(n);
    // The hull has at most 0x40 points (`shadow_hull`), the list's length.
    for k in hull_list(n) {
        work.list
            .get(k as usize)
            .ok_or(BadInput("a shadow hull past the shadow list"))?
            .set(k);
    }
    work.texture.set(SHADOW_TEXTURE);
    shadow_faces(ctx, work, car)
}

/// `race::cast_car_shadows` (0x45754C): the depth test off, `work.lift` 0x1999, then for each race
/// car with +0x6FC not 0: its block read directly (the original loaded it into the work copy,
/// `drive::load_car`), its corner count from its +0x260, `cast_car_shadow` once with
/// `RACE_VARS.settings_words[2]` set, then once more with the sun at height 0x40 (the original set
/// `stage_prefs.sun_height` to it and put it back after: an argument here); the depth test on.
/// Refuses (before anything) no cars or more than eight, or a null block.
pub fn cast_car_shadows(ctx: &crate::race::view_scratch::DrawCtx<'_>) -> Result<(), Refused> {
    // Converted: `rc2k_core::race::shadow::{check_shadow_cars, casts_shadow, first_pass,
    // SHADOW_STRENGTH, SUN_PASS}`.
    use rc2k_core::race::shadow::{
        SHADOW_STRENGTH, SUN_PASS, casts_shadow, check_shadow_cars, first_pass,
    };
    let cars: u32 = ctx.reads.view.race_cars.get();
    check_shadow_cars(cars, |n| {
        u32::from(ctx.reads.work.has_car_block(n as usize))
    })?;
    let work = ShadowWork::new();
    crate::scene::depth_test_off::depth_test_off(ctx.part);
    work.lift.set(SHADOW_STRENGTH);
    for n in 0..cars {
        // Each car's shadow from its own block (the original loaded it into the work car,
        // `drive::load_car`, and left the last one there for the rest of the view:
        // `notes/bugs.md`).
        let car = crate::race::owner_shared::RaceShared::race_car_or_in(
            ctx.reads.work,
            ctx.reads.car_blocks,
            n as usize,
            "a car's block",
        )?;
        if !casts_shadow(
            ctx.view
                .cars
                .state_of(ctx.reads.work.block_number(n as usize)?)?
                .get(),
        ) {
            continue;
        }
        work.point_count.set(car.probe_count.get());
        if first_pass(ctx.reads.vars.settings_words[2].get()) {
            let sun: u32 = ctx.reads.stage_prefs.sun_height.get();
            cast_car_shadow(ctx, &work, car, sun)?;
        }
        cast_car_shadow(ctx, &work, car, SUN_PASS)?;
    }
    crate::scene::depth_test_on::depth_test_on(ctx.part);
    Ok(())
}

#[cfg(test)]
mod reference_vectors {
    use super::*;

    use crate::race::asm_test::{Rng, at_each_control_word, lock, same};
    use crate::test_support::vectors::{Recorder, word, words};

    #[test]
    fn two_pi_plus_matches() {
        let _l = lock();
        let mut r = Rng(21);
        let mut file = Recorder::replay("race.car_shadow.two_pi_plus");
        at_each_control_word(|| {
            for n in 0..40000 {
                let v = if n % 2 == 0 {
                    // the angles it is used for: below 0, above -2 pi
                    (-(f64::from(r.u32()) / 4294967296.0 * std::f64::consts::TAU) as f32).to_bits()
                } else {
                    r.float()
                };
                file.input(&v.to_le_bytes());
                let (want, got) = (word(&mut file), two_pi_plus(v));
                assert!(same(want, got), "{v:#x}: {want:#x} {got:#x}");
            }
        });
        file.finish();
    }

    #[test]
    fn float_blocks_match() {
        let _l = lock();
        let mut r = Rng(22);
        let mut file = Recorder::replay("race.car_shadow.float_blocks");
        at_each_control_word(|| {
            for _ in 0..20000 {
                let t = crate::math::x87_float::f80(crate::math::x87_float::X(
                    f64::from(f32::from_bits(r.float())) * f64::from(f32::from_bits(r.float())),
                ));
                file.input(&t);
                let (want, got) = (word(&mut file), tbyte_to_float(t));
                assert!(same(want, got), "{t:x?}: {want:#x} {got:#x}");
                let (y, l, sl, b) = (r.float(), r.float(), r.float(), r.float());
                file.input(
                    &[y, l, sl, b]
                        .iter()
                        .flat_map(|w| w.to_le_bytes())
                        .collect::<Vec<u8>>(),
                );
                let (want, got) = (word(&mut file), lifted(y, l, sl, b));
                assert!(
                    same(want, got),
                    "{y:#x} {l:#x} {sl:#x} {b:#x}: {want:#x} {got:#x}"
                );
            }
        });
        file.finish();
    }

    #[test]
    fn circle_place_matches() {
        let _l = lock();
        let mut r = Rng(23);
        let mut file = Recorder::replay("race.car_shadow.circle_place");
        let mut worst = 0;
        at_each_control_word(|| {
            for _ in 0..20000 {
                let n = 3 + r.below(0x3E);
                let k = r.below(n);
                let radius = r.float();
                let unit = r.float();
                file.input(
                    &[k, n, radius, unit]
                        .iter()
                        .flat_map(|w| w.to_le_bytes())
                        .collect::<Vec<u8>>(),
                );
                let entry = super::vector_tests::entry_of([0xAAAA_AAAA; 17]);
                let cells = crate::cells::dwords_in::<_, 17>(&entry, 0);
                // The original writes the entry's dwords 4 and 5 only.
                let a: [u32; 2] = words(&mut file);
                circle_place(&entry, k, n, radius, unit);
                let b: [u32; 17] = std::array::from_fn(|w| cells[w].get());
                for (w, &want) in [4, 5].iter().zip(&a) {
                    let got = b[*w];
                    assert!(
                        same(want, got),
                        "{k}/{n} {radius:#x} {unit:#x}: {want:#x} {got:#x}"
                    );
                    worst = worst.max(want.abs_diff(got));
                }
                assert!(b[..4].iter().chain(&b[6..]).all(|&x| x == 0xAAAA_AAAA));
            }
        });
        file.finish();
        eprintln!("circle_place: worst difference {worst} ulps");
    }
}

/// The blocks that read the race's state against the original's outputs, recorded
/// (`test_support::race_vectors`).
#[cfg(test)]
mod vector_tests {
    use super::*;
    use crate::race::asm_test::{Rng, at_each_control_word, lock, same, within_ulps};
    use crate::race::owner::race;
    use crate::test_support::race_vectors::{Kind, Vectors};
    use crate::test_support::state::Snapshot;

    /// A polygon entry of the words `words`.
    pub(super) fn entry_of(words: [u32; 17]) -> PolygonEntry {
        let mut bytes = [0u8; 68];
        for (b, w) in bytes.as_chunks_mut::<4>().0.iter_mut().zip(words) {
            b.copy_from_slice(&w.to_le_bytes());
        }
        crate::cells::cells_of_bytes(bytes)
    }

    #[test]
    fn inside_edge_is_the_originals() {
        let _l = lock();
        let kept = Snapshot::of(&[
            "race.owner.RACE.physics.ground",
            "race.owner.RACE.frame.frame_work",
            "race.owner.RACE.course.course",
        ]);
        let mut vectors = Vectors::open("race.car_shadow.inside_edge", Kind::Exact, 3);
        let mut r = Rng(24);
        let mut points = [0u32; 12];
        let (mut yes, mut no) = (0, 0);
        at_each_control_word(|| {
            for _ in 0..20000 {
                for q in points.iter_mut() {
                    *q = r.int() as u32;
                }
                let edge: [u32; 4] = std::array::from_fn(|_| r.int() as u32);
                let n = r.below(4);
                let work = ShadowWork::new();
                for (cell, &e) in work.edge.iter().zip(&edge) {
                    cell.set(e);
                }
                let inside = crate::race::scene_points::with_test_points(&points, || {
                    inside_edge(
                        race().course.reads(),
                        &crate::race::scene_points::NO_OWN,
                        &work,
                        n,
                    )
                })
                .expect("the test's memory is registered");
                // The point the original left in the ground work, the shadow's own search in
                // the shadow's own search fields.
                let q = work.search.query(&crate::race::scene_points::NO_OWN);
                let got = [inside as u32, q.x.get(), q.z.get()];
                vectors.check(&got, |a, b| a == b, || format!("{points:x?} {edge:x?} {n}"));
                if got[0] != 0 { yes += 1 } else { no += 1 }
            }
        });
        kept.restore();
        vectors.finish();
        assert!(yes > 1000 && no > 1000, "{yes} {no}");
    }

    #[test]
    fn crossing_block_is_the_originals() {
        let _l = lock();
        let kept = Snapshot::of(&[
            "scene.owner.SCENE.drawing.scene",
            "race.owner.RACE.frame.frame_work",
            "race.owner.RACE.physics.ground",
        ]);
        let mut vectors = Vectors::open("race.car_shadow.crossing_block", Kind::Exact, 13);
        let mut r = Rng(25);
        at_each_control_word(|| {
            for _ in 0..20000 {
                let small = r.below(2) == 0;
                // The work dwords the original read from its frame work, the shadow's own in
                // its work.
                let inputs: [u32; 6] = std::array::from_fn(|_| {
                    let v = if small {
                        r.below(200) as i32 - 100
                    } else {
                        r.int()
                    };
                    v as u32
                });
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
                let work = ShadowWork::new();
                let poly = crate::scene::polygon_work::PolygonWork::new();
                for (k, &v) in inputs.iter().enumerate() {
                    work.crossing[8 + k].set(v);
                }
                let both = [pa, pb].concat();
                let (a, b) = (entry_of(ea), entry_of(eb));
                crate::race::scene_points::with_test_points(&both, || {
                    crossing_block(
                        race().course.reads(),
                        &crate::race::scene_points::NO_OWN,
                        &poly,
                        &work,
                        (0, 1),
                        &a,
                        &b,
                    )
                })
                .expect("the test's memory is registered");
                // The inputs, `plane_f` (kept in the shadow's own search fields) and the
                // view's work the block writes.
                let mut got = inputs.to_vec();
                got.push(work.search.plane_f().get().to_bits());
                for k in [2, 4, 10, 11] {
                    got.push(poly.clip_work[k].bits());
                }
                got.push(poly.crossing_amount.get());
                got.push(poly.clip_out_words[0].get());
                vectors.check(&got, same, || format!("inputs {inputs:x?}"));
            }
        });
        kept.restore();
        vectors.finish();
    }

    #[test]
    fn sun_slant_and_angle_are_the_originals() {
        let _l = lock();
        let kept = Snapshot::of(&[
            "scene.owner.SCENE.drawing.scene",
            "race.owner.RACE.frame.shadow_skids",
        ]);
        let mut vectors = Vectors::open("race.car_shadow.sun_slant_and_angle", Kind::Close, 3);
        let mut r = Rng(26);
        at_each_control_word(|| {
            for _ in 0..4096 {
                let sun = 0x10 + r.below(0x400);
                let turn = (r.below(0x800) as i32 - 0x400) as u32;
                // At a multiple of a quarter turn (sun 0x40, ...) the sine or the cosine is
                // about 1e-16: the x87 reduces by a 66-bit pi, libm exactly, so those few
                // digits differ (the slants are then about 1e-17 or 1e16 in the original,
                // whatever the processor). Left out here.
                if sun.is_multiple_of(64) || turn.is_multiple_of(64) {
                    continue;
                }
                let work = ShadowWork::new();
                sun_slant(&work, sun, turn);
                let at = [r.float(), r.float(), r.float()];
                let point = ShadowPoint {
                    x: PlainDword::new(at[0]),
                    y: PlainDword::new(at[1]),
                    z: PlainDword::new(at[2]),
                    angle: PlainDword::new(0),
                    length: PlainDword::new(0),
                    zeroed: PlainDword::new(0),
                    carried_words: [PlainDword::new(0), PlainDword::new(0)],
                };
                let got = [
                    work.slant_x.get().to_bits(),
                    work.slant_z.get().to_bits(),
                    angle_of(&point),
                ];
                vectors.check(
                    &got,
                    |a, b| within_ulps(a, b, 1),
                    || format!("{sun:#x} {turn:#x} {at:x?}"),
                );
            }
        });
        kept.restore();
        vectors.finish();
    }
}
