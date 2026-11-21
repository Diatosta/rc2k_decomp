//! The car's shadow (the tree under `cast_car_shadows`): a dark polygon under each car, the hull of
//! its corners on the ground, cut by the course's faces. It is drawn for each car whose state bits
//! say it casts a shadow, once for the "reduced" car shadows option and twice for "full"
//! (`settings_words[2]`): once cast along the stage's sun height, once with the sun straight above
//! at height 0x40. This file holds its leaves and shared code. The working values are a
//! `ShadowWork` of its own: the point records of 0x20 bytes at `work.points` (`ShadowPoint`: x, y,
//! z as whole numbers, then the angle, the distance and a zero), their count `work.point_count`,
//! the hull's `work.hull_count`, ...

use crate::errors::{BadInput, Refused};
use crate::math::cell_x87::CellX87;
use crate::memory::session::{PlainByte, PlainDword, PlainFloat};
use crate::physics::car_store::Car;
use crate::physics::ground_query::{GroundEnv, GroundSearch};
use crate::race::owner_course::CourseReads;
use crate::race::scene_points::{OwnPoints, point_word, point_xz, put_point_word};
use crate::race::trap::Trap;
use crate::scene::polygon::{list_cell, old_list_cell};
use crate::scene::polygon_store::PolygonEntry;

/// One shadow point (0x20 bytes): the corner of the car's shadow, first as floats, then
/// as whole numbers.
#[repr(C)]
#[derive(rc2k_derive::Cells, rc2k_derive::ZeroCells, rc2k_derive::ByteView)]
pub(super) struct ShadowPoint {
    /// X.
    pub x: PlainDword,
    /// Y.
    pub y: PlainDword,
    /// Z.
    pub z: PlainDword,
    /// The angle from the first point (`fpatan` of x and z), then whole numbers.
    pub angle: PlainDword,
    /// The distance from the car.
    pub length: PlainDword,
    /// 0 once the angles are laid (`race::car_shadow::shadow_angles`).
    pub zeroed: PlainDword,
    /// The record's last two dwords; only moved with the record when points are swapped or
    /// sorted, never read for a value.
    pub carried_words: [PlainDword; 2],
}

const _: () = assert!(core::mem::size_of::<ShadowPoint>() == 0x20);

impl ShadowPoint {
    /// A record of zeros.
    pub const fn new() -> Self {
        Self {
            x: PlainDword::new(0),
            y: PlainDword::new(0),
            z: PlainDword::new(0),
            angle: PlainDword::new(0),
            length: PlainDword::new(0),
            zeroed: PlainDword::new(0),
            carried_words: [PlainDword::new(0), PlainDword::new(0)],
        }
    }

    /// The record's eight dwords, in order (it is swapped and sorted whole).
    pub fn words(&self) -> [&PlainDword; 8] {
        [
            &self.x,
            &self.y,
            &self.z,
            &self.angle,
            &self.length,
            &self.zeroed,
            &self.carried_words[0],
            &self.carried_words[1],
        ]
    }
}

/// The shadow's working state: what `cast_car_shadows` writes and reads within itself, one for each
/// call, given down to its parts. Every value is written before it is read, so nothing carries over
/// from one call to the next.
#[derive(rc2k_derive::Cells)]
pub(super) struct ShadowWork {
    /// The shadow's points, records of 0x20 bytes (0x40 at most), the hull's records after.
    points: [ShadowPoint; 0x100],
    /// The car's corners (`shadow_point_count`; from its block's `probe_count`).
    point_count: PlainDword,
    /// The length of the farthest point (`shadow_points`).
    farthest_length: PlainFloat,
    /// The farthest point's number.
    farthest: PlainDword,
    /// The first point's angle (`shadow_angles`).
    first_angle: PlainFloat,
    /// Cos(turn) x cot(sun) (`sun_slant`).
    slant_x: PlainFloat,
    /// Sin(turn) x cot(sun).
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
    /// `edge_crossing`'s work dwords.
    crossing: [PlainDword; 14],
    /// The next new scene point (`shadow_on_face`, `edge_crossing`).
    next_point: PlainDword,
    /// The hull's list.
    list: [PlainDword; 64],
    /// Its count.
    list_count: PlainDword,
    /// The ground's height under the car.
    ground: PlainDword,
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
        }
    }
}

/// The circle's radius and unit (`shadow_circle`, `FrameWork.shadow_circle[1]`): 2^23 and 2^24,
/// never written.
const CIRCLE: [u32; 2] = [0x4B00_0000, 0x4B80_0000];

/// Point `n` (0x20 bytes each; `work.farthest` keeps the farthest one's number), bounded by the
/// table (`work.points`).
fn point_at(work: &ShadowWork, n: u32) -> Result<&'_ ShadowPoint, BadInput> {
    work.points
        .get(n as usize)
        .ok_or(BadInput("a shadow point"))
}

/// 2 pi plus a float, as the x87 adds them.
fn two_pi_plus(v: u32) -> u32 {
    use crate::math::x87::{Rounding, rounding};
    use crate::math::x87_float::{X, ld, st};
    // Twice the 64-bit pi plus v, rounded once: the f64 pi's double, plus what the 64-bit pi has
    // above it (565 units of 2^-62, one less rounding down or chop).
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

/// Moves the farthest point (`work.farthest`) into the first record and makes each record's angle
/// relative to the first's (`work.first_angle`; below 0 plus 2 pi), times 65536 as a whole number,
/// `zeroed` 0; with two records or more they are sorted by angle (`sorting::sort_records`). Refused
/// (before anything): no points or more than 0x40.
pub(super) fn shadow_angles(
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
        // The sort is `rc2k_core::race::quick_sort::quick_sort`.
        crate::race::sorting::sort_records(
            &work.points,
            count,
            3,
            rc2k_core::race::quick_sort::Order::Up,
            ShadowPoint::words,
        )?;
    }
    Ok(())
}

/// Copies the first record after the last, then walks the points and takes out each one where the
/// turn from the one before to the one after is not to the left (the 64-bit cross product of whole
/// numbers, `product` its first half, not negative), moving the rest down and starting
/// again, until none is; `work.hull_count` is the points left. Refused (before anything): fewer
/// than two points or more than 0x40.
fn shadow_hull(work: &ShadowWork) -> Result<(), Refused> {
    use rc2k_core::race::shadow::{Record, hull, hull_records};
    let count: u32 = work.point_count.get();
    if !(2..=0x40).contains(&count) {
        return Err(BadInput("fewer than two shadow points or more than 0x40").into());
    }
    let at = |n: usize| point_at(work, n as u32);
    let mut records: Vec<Record> = Vec::new();
    for n in 0..hull_records(count) {
        records.push(crate::memory::cells::load_value::<Record>(at(n)?, 0)?);
    }
    let before = records.clone();
    let h = hull(&mut records, count)?;
    for (n, (now, was)) in records.iter().zip(&before).enumerate() {
        if now != was {
            crate::memory::cells::store_value(at(n)?, 0, *now)?;
        }
    }
    work.hull_count.set(h.count);
    Ok(())
}

/// The circle's texture place of corner `k` of `n` into the work entry `e` (`u2`, `v2`): cos and
/// sin of 2 pi k / n times `radius` plus `radius`, rounded to whole numbers, over `unit`.
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

/// Makes the hull's points new scene points (after the scene's, from `scene_point_count` + 0x10: x
/// and z the record's, y `work.ground`) and the polygon's work entries (`point`, `light` white,
/// `amount` `stage_prefs.shadow_alpha` / 2, `u2` / `v2` a circle's texture place: cos / sin of 2 pi
/// k / n times `CIRCLE[0]` plus it, rounded, over `CIRCLE[1]`). Refused (before anything): a hull
/// of no points or more than 0x40.
fn shadow_polygon(
    polygon_work: &crate::scene::polygon_store::PolygonWork,
    points: &OwnPoints,
    work: &ShadowWork,
    reads: crate::scene::workers::DrawReads<'_>,
) -> Result<(), Refused> {
    let n: u32 = work.hull_count.get();
    if n == 0 || n > 0x40 {
        return Err(BadInput("a shadow hull of no points or more than 0x40").into());
    }
    let scene_points: u32 = reads.course_work.get().scene_point_count;
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

/// A polygon work entry's place: `entries` + 0x44 x n.
fn work_entry(
    polygon_work: &crate::scene::polygon_store::PolygonWork,
    n: u32,
) -> Result<&PolygonEntry, BadInput> {
    polygon_work
        .entries
        .get(n as usize)
        .ok_or(BadInput("a polygon work entry past 0x80"))
}

/// Whether scene point `n` is on the inner side of the edge from (`work.edge`, `work.edge[1]`) to
/// (`work.edge[2]`, `work.edge[3]`): the cross product of whole numbers, on the x87, above 0 or not
/// a number.
fn inside_edge(
    course: CourseReads<'_>,
    points: &OwnPoints,
    work: &ShadowWork,
    n: u32,
) -> Result<bool, BadInput> {
    let (x, z) = point_xz(course, points, n)?;
    use crate::math::x87_float::{X, compare, int};
    let word = |d: &dyn crate::memory::session::DwordLike| int(d.get() as i32);
    let c = &work.edge;
    let first = (int(x as i32) - word(&c[0])) * (word(&c[3]) - word(&c[1]));
    let second = (int(z as i32) - word(&c[1])) * (word(&c[2]) - word(&c[0]));
    // Above 0, or not a number.
    Ok(compare(X(0.0), first - second) & 0x0100 != 0)
}

/// `edge_crossing`'s arithmetic: the share of the edge's crossing (-(`work.crossing[8]` x
/// `work.crossing[11]` - `work.crossing[10]` x `work.crossing[9]`) over the difference of the two
/// cross products, rounded to a float, returned, and used as read back), the
/// crossing's place (x and z whole numbers into `clip_work[2]` / `clip_work[4]`) and the
/// entry's texture floats (`clip_work[10]`, `clip_work[11]`) and two bytes
/// (`crossing_amount`, `clip_out_words`), shared the same way between the scene points
/// numbered `pa`, `pb` and the entries `ea`, `eb`. The share's bits are not stored to `scratch`:
/// nothing reads them.
fn crossing_block(
    course: CourseReads<'_>,
    points: &OwnPoints,
    poly: &crate::scene::polygon_work_store::PolygonWork,
    work: &ShadowWork,
    (pa, pb): (u32, u32),
    ea: &PolygonEntry,
    eb: &PolygonEntry,
) -> Result<f32, BadInput> {
    use crate::math::x87_float::{int, ld, stf, to_int};
    let word = |n: usize| int(work.crossing[n].get() as i32);
    let first = word(11) * word(8) - word(10) * word(9);
    let second = word(13) * word(8) - word(12) * word(9);
    let stored_share = stf(-(first / (second - first)));
    let share = ld(stored_share);
    // a + (b - a) x share.
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
    let float = |a: &dyn crate::memory::session::DwordLike,
                 b: &dyn crate::memory::session::DwordLike| {
        stf(ld(a.get()) + (ld(b.get()) - ld(a.get())) * share)
    };
    poly.clip_work[10].set(float(&ea.u2, &eb.u2));
    poly.clip_work[11].set(float(&ea.v2, &eb.v2));
    let byte = |a: &PlainByte, b: &PlainByte| {
        let (a, b) = (int(a.get().into()), int(b.get().into()));
        to_int(a + (b - a) * share) as u32
    };
    poly.crossing_amount.set(byte(&ea.amount, &eb.amount));
    poly.clip_out_words[0].set(byte(&ea.fog, &eb.fog));
    Ok(stored_share)
}

/// The edge's crossing between work entries `ea` and `eb` as a new scene point (`work.next_point`,
/// then counted on: x and z from the share `plane_f`, y 0) and the new work entry `ed`
/// (its point, texture places `u2` / `v2` and the two bytes `fog` / `amount` shared the same way).
/// The work dwords (`clip_work[2]` ..) are left as `edge_crossing` leaves them.
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
    let rd = |d: &dyn crate::memory::session::DwordLike| d.get();
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

/// Cuts the polygon by the edge from scene point `from` to scene point `to` (kept: those on the
/// inner side, and the crossings), with three vertices or more.
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
    // The polygon list holds 0x40 numbers and the old list follows it: a cut that adds crossings
    // past 0x40 runs on from the list into the old list, and from the old list into the gap after
    // it (`list_cell` names the three fields). A hull of at most 0x40 points cut by three edges
    // adds at most as many again as it holds each time.
    let polygon = &ctx.part.polygon;
    for k in 0..count as usize {
        old_list_cell(polygon, k).set(list_cell(polygon, k).get());
    }
    old_list_cell(polygon, count as usize).set(list_cell(polygon, 0).get());
    // The cut is `rc2k_core::race::shadow::clip_by_edge`.
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

/// Whether the course face whose corners are `work.face_corners` .. is near the work car
/// (`rc2k_core::race::shadow::face_near`).
fn face_is_near(
    ctx: &crate::race::view_scratch::DrawCtx<'_>,
    work: &ShadowWork,
    car: &Car,
) -> Result<bool, BadInput> {
    let corners = [
        work.face_corners[0].get(),
        work.face_corners[1].get(),
        work.face_corners[2].get(),
    ];
    let trap = Trap::new();
    let found =
        rc2k_core::race::shadow::face_near((car.position[0].get(), car.position[2].get()), |n| {
            point_xz(ctx.reads.course(), &ctx.part.points, corners[n]).unwrap_or_else(|e| {
                trap.keep(e);
                (0, 0)
            })
        });
    trap.finish()?;
    Ok(found.near)
}

/// Cuts the shadow's polygon to the course face whose corners are `work.face_corners` ..
/// `work.face_corners[2]` (scene points): none (count 0) when a corner is more than 0x100000 off
/// the work car in x before one within 0x100000 in z is met (or none is); else cut by its three
/// edges in turn while three vertices are left (`clip_to_edge`, with `inside_edge` the side test
/// and `edge_crossing` the crossing).
fn clip_shadow_to_face(
    ctx: &crate::race::view_scratch::DrawCtx<'_>,
    work: &ShadowWork,
    car: &Car,
) -> Result<(), BadInput> {
    let corners = [
        work.face_corners[0].get(),
        work.face_corners[1].get(),
        work.face_corners[2].get(),
    ];
    if !face_is_near(ctx, work, car)? {
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

/// Raises work entry `index`'s scene point onto the face (`work.face_corners` ..): the face's
/// corners (into `face_corner_numbers` ..) and the point (into `ground_x` /
/// `ground_z`) go through `physics::face_shares` and `physics::plane_height`, its y the
/// height (`ground_height`) plus `work.lift`.
fn raise_onto_face(
    polygon_work: &crate::scene::polygon_store::PolygonWork,
    points: &OwnPoints,
    reads: crate::scene::workers::DrawReads<'_>,
    work: &ShadowWork,
    index: u32,
) -> Result<(), Refused> {
    use crate::physics::ground::face_shares_at;
    let p = work_entry(polygon_work, index)?.point.get();
    let mut q = GroundSearch::default();
    for k in 0..3 {
        q.corner_numbers[k] = work.face_corners[k].get();
    }
    let (x, z) = point_xz(reads.course(), points, p)?;
    q.x = x;
    q.z = z;
    let env = GroundEnv {
        course: reads.course(),
        points,
        mark: reads.ground_probe.settle_mark,
    };
    face_shares_at(env, &mut q)?;
    crate::physics::plane_height::plane_height(&mut q);
    put_point_word(
        reads.course(),
        points,
        p,
        1,
        q.height.wrapping_add(work.lift.get()),
    )?;
    Ok(())
}

/// Cuts the shadow's polygon (the hull's list at `work.list`, its count `work.list_count`) to the
/// face `work.face_corners` .. and draws it there: new points from `scene_point_count` + 0x30
/// (`work.next_point`), entries from 0x20, the 0x40 point records after the scene's marked unmade;
/// `clip_shadow_to_face`; with three vertices or more each is raised onto the face, projected
/// (`scene::project_polygon`, `scene::fog_polygon`), cut by depth (unless all or none are outside)
/// and to the screen and drawn (`scene::draw_polygon_in_game`) in mode 5 with texture
/// `work.texture`. Refused: what the projection and the clips refuse (after the work is set; not
/// met with a hull's few vertices) and a hull list past 0x40.
fn shadow_on_face(
    ctx: &crate::race::view_scratch::DrawCtx<'_>,
    work: &ShadowWork,
    car: &Car,
) -> Result<(), Refused> {
    let count: u32 = work.list_count.get();
    if count > 0x40 {
        return Err(BadInput("a shadow list past 0x40").into());
    }
    if !face_is_near(ctx, work, car)? {
        // The face is cut away whole (`clip_shadow_to_face`), whatever the polygon holds: the
        // polygon's list and the point marks are made afresh by the next face that is near.
        ctx.view.polygon.polygon_vertex_count.set(0u32);
        ctx.view.polygon.polygon_next_entry.set(0x20u32);
        let scene_points: u32 = ctx.reads.course_work.get().scene_point_count;
        work.next_point.set(scene_points.wrapping_add(0x30));
        ctx.view.polygon.polygon_sides[0].set(0u32);
        ctx.view.polygon.polygon_sides[1].set(1u32);
        return Ok(());
    }
    let polygon = &ctx.part.polygon;
    for (k, cell) in work.list.iter().enumerate().take(count as usize) {
        list_cell(polygon, k).set(cell.get());
    }
    ctx.view.polygon.polygon_vertex_count.set(count);
    list_cell(polygon, count as usize).set(work.list[0].get());
    ctx.view.polygon.polygon_next_entry.set(0x20u32);
    let scene_points: u32 = ctx.reads.course_work.get().scene_point_count;
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
            &ctx.reads.draw_switches,
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
    Ok(())
}

/// Each face of course record number `record` (from its first to its last face number) with a kind
/// below 0x10 and without bit 7 of its switches word gets the shadow (`shadow_on_face`) on its two
/// triangles (corner points 0, 1, 2 and 3, 2, 1); `current_course_face` follows each face.
fn shadow_on_record(
    ctx: &crate::race::view_scratch::DrawCtx<'_>,
    work: &ShadowWork,
    car: &Car,
    record: u32,
) -> Result<(), Refused> {
    use crate::race::course_places::{face_number, record_field};
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

/// Puts the shadow (`shadow_on_face`) on the last course record's faces, then on every record in
/// the 3 x 3 cells round the work car (`shadow_on_record` each).
fn shadow_faces(
    ctx: &crate::race::view_scratch::DrawCtx<'_>,
    work: &ShadowWork,
    car: &Car,
) -> Result<(), Refused> {
    let count: u32 = ctx.reads.course_work.get().course_record_count;
    shadow_on_record(ctx, work, car, count.wrapping_sub(1))?;
    if count == 0 {
        return Ok(());
    }
    use rc2k_core::race::shadow::{cell_near_car, record_cell};
    // Every record of the course is looked at each frame: each one's first dword is read in
    // place, after a check that the whole run is held.
    crate::race::course_places::records_held(ctx.reads.course(), count)?;
    for n in 0..count {
        let Some((x, z)) = record_cell(crate::race::course_places::record_field::<u32>(
            ctx.reads.course(),
            n,
            0,
        )?) else {
            continue;
        };
        if cell_near_car((car.position[0].get(), car.position[2].get()), (x, z)) {
            shadow_on_record(ctx, work, car, n)?;
        }
    }
    Ok(())
}

/// 80 bits as a float.
fn tbyte_to_float(t: [u8; 10]) -> u32 {
    crate::math::x87_float::st(crate::math::x87_float::from_f80(t))
}

/// The sun's slant from `sun` and `turn` (the angles as 128ths of a half turn): `work.slant_x` =
/// cos(turn) x cot(sun), `work.slant_z` = sin(turn) x cot(sun).
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

/// The angle of the corner at `at` (its `x` and `z`): arctangent of x over z.
fn angle_of(at: &ShadowPoint) -> u32 {
    use crate::math::x87_float::{ld, st};
    st(ld(at.x.get()).atan2(ld(at.z.get())))
}

/// Works out the shadow's points: the ground under the work car (`physics::find_ground`, its height
/// to `work.ground`); the sun's slant from the sun's height `sun` (at least 0x10) and
/// -`stage_prefs.sun_turn` - 0x40 (`work.slant_x`, `work.slant_z`); each of the car's
/// `work.point_count` corners (`car.probe_point_x` / `_y` / `_z`) is turned by its `body_matrix`,
/// moved along the slant by its height plus 0.7, and given its angle and distance, the farthest
/// kept (`work.farthest_length`, `work.farthest`), as whole numbers (x 65536) plus the car's place.
/// Refused: what `find_ground` refuses (before anything), and no corners or more than 0x40.
#[allow(
    clippy::too_many_arguments,
    reason = "the drawing's stores the part reads, as they are lent"
)]
pub(super) fn shadow_points(
    reads: crate::scene::workers::DrawReads<'_>,
    points: &OwnPoints,
    work: &ShadowWork,
    car: &Car,
    sun: u32,
) -> Result<(), Refused> {
    use crate::math::x87::mul_to_int;
    use crate::math::x87::{below_or_unordered, to_tbyte};
    let count: u32 = work.point_count.get();
    if count == 0 || count > 0x40 {
        return Err(BadInput("no shadow corners or more than 0x40").into());
    }
    // The mark is the physics' as the last search there left it.
    let env = GroundEnv {
        course: reads.course(),
        points,
        mark: reads.ground_probe.settle_mark,
    };
    let mut q = GroundSearch {
        x: car.position[0].get(),
        z: car.position[2].get(),
        last: car.probe_point_face[4].get(),
        ..GroundSearch::default()
    };
    crate::physics::find_ground::find_ground_with(env, &mut q)?;
    work.farthest_length.set(-1.0f32);
    work.ground.set(q.height);
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

/// A corner moved along the sun's slant: (y + lift) x slant + base, rounded once.
fn lifted(
    y: u32,
    lift: impl crate::math::x87_float::Loadable,
    slant: impl crate::math::x87_float::Loadable,
    base: u32,
) -> u32 {
    use crate::math::x87_float::{ld, st};
    st((ld(y) + ld(lift)) * ld(slant) + ld(base))
}

/// With three corners or more, makes the shadow's points, their order, hull and polygon, the hull's
/// list (`work.list`, `work.list_count`) and texture 0xC3 (`work.texture`), and puts the shadow on
/// the faces round the car (`shadow_faces`), with the sun at height `sun`. Refused: what its parts
/// refuse (before anything for the points and the counts).
fn cast_car_shadow(
    ctx: &crate::race::view_scratch::DrawCtx<'_>,
    work: &ShadowWork,
    car: &Car,
    sun: u32,
) -> Result<(), Refused> {
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

/// Turns the depth test off and sets `work.lift` to 0x1999; then for each race car whose state bits
/// are not 0 reads its block directly and its corner count from its `probe_count`, and calls
/// `cast_car_shadow` once with `settings_words[2]` set, then once more with the sun at height 0x40;
/// the depth test goes back on. Refused (before anything): no cars or more than eight, or a null
/// block.
pub(super) fn cast_car_shadows(
    ctx: &crate::race::view_scratch::DrawCtx<'_>,
) -> Result<(), Refused> {
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
        // Each car's shadow from its own block.
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

    use crate::race::asm_test::{Rng, at_each_control_word, same};
    use crate::test_support::vectors::{Recorder, word, words};

    #[test]
    fn two_pi_plus_matches() {
        let mut r = Rng(21);
        let mut file = Recorder::replay("race.car_shadow.two_pi_plus");
        at_each_control_word(|| {
            for n in 0..40000 {
                let v = if n % 2 == 0 {
                    // The angles it is used for: below 0, above -2 pi.
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
                let cells = crate::memory::cells::dwords_in::<_, 17>(&entry, 0);
                // Only the entry's dwords 4 and 5 are written.
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

/// The blocks that read the race's state against the recorded outputs
/// (`test_support::race_vectors`).
#[cfg(test)]
mod vector_tests {
    use super::*;
    use crate::race::asm_test::{Rng, at_each_control_word, same, within_ulps};
    use crate::test_support::own::race;
    use crate::test_support::race_vectors::{Kind, Vectors};
    use crate::test_support::state::Snapshot;

    /// A polygon entry of the words `words`.
    pub(super) fn entry_of(words: [u32; 17]) -> PolygonEntry {
        let mut bytes = [0u8; 68];
        for (b, w) in bytes.as_chunks_mut::<4>().0.iter_mut().zip(words) {
            b.copy_from_slice(&w.to_le_bytes());
        }
        crate::memory::cells::cells_of_bytes(bytes)
    }

    #[test]
    fn inside_edge_is_the_originals() {
        let kept_course = *crate::test_support::own::race().course.course.borrow();
        let kept = Snapshot::of(&["owners.race.physics.ground", "owners.race.frame.frame_work"]);
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
                let inside = crate::race::scene_points::with_test_points(
                    &race().course.stage.course,
                    &points,
                    || {
                        inside_edge(
                            race().course.reads(),
                            &crate::race::scene_points::no_own(),
                            &work,
                            n,
                        )
                    },
                )
                .expect("the test's memory is registered");
                // The point the recorded run left in the ground work: the point the test asked about.
                let (x, z) = crate::race::scene_points::with_test_points(
                    &race().course.stage.course,
                    &points,
                    || {
                        point_xz(
                            race().course.reads(),
                            &crate::race::scene_points::no_own(),
                            n,
                        )
                    },
                )
                .expect("the test's memory is registered");
                let got = [inside as u32, x, z];
                vectors.check(&got, |a, b| a == b, || format!("{points:x?} {edge:x?} {n}"));
                if got[0] != 0 { yes += 1 } else { no += 1 }
            }
        });
        kept.restore();
        *crate::test_support::own::race().course.course.borrow_mut() = kept_course;
        vectors.finish();
        assert!(yes > 1000 && no > 1000, "{yes} {no}");
    }

    #[test]
    fn crossing_block_is_the_originals() {
        let kept = Snapshot::of(&[
            "owners.scene.draw_work",
            "owners.race.frame.frame_work",
            "owners.race.physics.ground",
        ]);
        let mut vectors = Vectors::open("race.car_shadow.crossing_block", Kind::Exact, 13);
        let mut r = Rng(25);
        at_each_control_word(|| {
            for _ in 0..20000 {
                let small = r.below(2) == 0;
                // The work dwords the recorded run read from its frame work, here the shadow's own
                // in its work.
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
                let poly = crate::scene::polygon_work_store::PolygonWork::new();
                for (k, &v) in inputs.iter().enumerate() {
                    work.crossing[8 + k].set(v);
                }
                let both = [pa, pb].concat();
                let (a, b) = (entry_of(ea), entry_of(eb));
                let stored_share = crate::race::scene_points::with_test_points(
                    &race().course.stage.course,
                    &both,
                    || {
                        crossing_block(
                            race().course.reads(),
                            &crate::race::scene_points::no_own(),
                            &poly,
                            &work,
                            (0, 1),
                            &a,
                            &b,
                        )
                    },
                )
                .expect("the test's memory is registered");
                // The inputs, the share the block returns and the view's
                // work the block writes.
                let mut got = inputs.to_vec();
                got.push(stored_share.to_bits());
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
        let kept = Snapshot::of(&["owners.scene.draw_work", "owners.race.frame.shadow_skids"]);
        let mut vectors = Vectors::open("race.car_shadow.sun_slant_and_angle", Kind::Close, 3);
        let mut r = Rng(26);
        at_each_control_word(|| {
            for _ in 0..4096 {
                let sun = 0x10 + r.below(0x400);
                let turn = (r.below(0x800) as i32 - 0x400) as u32;
                // At a multiple of a quarter turn (sun 0x40, ...) the sine or the cosine is about
                // 1e-16: the x87 reduces by a 66-bit pi, libm exactly, so those few digits differ
                // (the slants are then about 1e-17 or 1e16, whatever the processor). Left out here.
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
