//! The skidmarks' drawing (`race::draw_skidmarks_if_on` / `race::draw_skidmarks`). A skidmark is a
//! chain of up to 0x400 links (`skid_links`, 8 bytes each: +0 the link before, -1 none; +6 the
//! views it shows in, a bit each) with two points each (`skid_points`, 0xC bytes a point: x, y, z
//! whole numbers); each link and the one before it give a quad.

use crate::errors::BadInput;
use crate::errors::Refused;

/// A whole number plus a float, as an 80-bit extended value.
#[cfg(test)]
fn int_plus_float_80(i: u32, f: u32) -> [u8; 10] {
    use crate::math::x87_float::{f80, int, ld};
    f80(int(i as i32) + ld(f))
}

/// A whole number as an 80-bit extended value.
pub fn int_80(i: u32) -> [u8; 10] {
    crate::math::x87_float::f80(crate::math::x87_float::int(i as i32))
}

/// An 80-bit extended value rounded to a float's bits.
#[cfg(test)]
fn float_of_80(t: [u8; 10]) -> u32 {
    crate::math::x87_float::st(crate::math::x87_float::from_f80(t))
}

/// The projection of a turned vertex record (words 4, 5, 6: x, y, depth, floats) onto the screen
/// (words 3 and 7). The scale is `view_scale` over `flat_depth` + 1 when the stage is flat
/// (`flat_depth`), else over the depth. x is x times that scale plus `view_centre`, y is
/// `view_centre[1]` less y times it.
pub(crate) fn project_record(
    prefs: crate::race::prefs_store::StagePrefsRef<'_>,
    flat_depth: u32,
    v: &mut crate::scene::point_records::Record,
    window: &crate::scene::view_window::ViewWindow,
) {
    use crate::math::x87_float::{ONE, int, ld, st};
    let (x, y, z) = (ld(v[4]), ld(v[5]), ld(v[6]));
    let reach = crate::math::x87_float::ld(window.borrow().view_scale);
    let scale = if prefs.read(|p| p.flat_depth) != 0 {
        reach / (int(flat_depth as i32) + ONE)
    } else {
        reach / z
    };
    v[3] = st(x * scale + crate::math::x87_float::ld(window.borrow().view_centre[0]));
    v[7] = st(-(y * scale) + crate::math::x87_float::ld(window.borrow().view_centre[1]));
}

/// A skidmark point turned and projected: words 3 .. 8 of its vertex record.
#[derive(Clone, Copy)]
struct Turned {
    words: [u32; 5],
}

/// One skidmark point (`skid_points`, 0xC bytes each, number `n`) into the vertex record `v`:
/// raised by `skid_lift`, turned by `view_matrix` (`scene::rotate_point`), and projected when its
/// depth is not below 4096.0. The projected x and y start at 0.
fn skid_vertex(
    ctx: &crate::race::view_scratch::DrawCtx<'_>,
    matrix: &[u32; 15],
    n: u32,
    v: u32,
) -> Result<Turned, BadInput> {
    let turned = turn_skid_point(ctx, matrix, n)?;
    put_turned(ctx.part, &ctx.view.polygon.quads, v, turned)?;
    Ok(turned)
}

/// A point as `skid_vertex` turned it, put into the vertex record `v`.
fn put_turned(
    part: &crate::scene::scratch_store::PartScratch,
    quad: &crate::scene::polygon_work_store::QuadSlot,
    v: u32,
    turned: Turned,
) -> Result<(), BadInput> {
    let mut made = crate::scene::point_records::get_in(part, Some(quad), v)?;
    made[3..8].copy_from_slice(&turned.words);
    crate::scene::point_records::set_in(part, Some(quad), v, made)
}

/// Skidmark point `n` turned and projected.
fn turn_skid_point(
    ctx: &crate::race::view_scratch::DrawCtx<'_>,
    matrix: &[u32; 15],
    n: u32,
) -> Result<Turned, BadInput> {
    {
        let at = n as usize * 0xC;
        if at + 0xC > crate::race::shadow_skids_store::POINTS_LEN {
            return Err(BadInput("a skidmark point past the table"));
        }
        let (words, lift) = ctx
            .reads
            .shadow_skids
            .read(|skids| ([0, 4, 8].map(|k| skids.point_word(at + k)), skids.skid_lift));
        use crate::math::x87_float::{Rounds, at_rounding};
        let turned = at_rounding!(|r| {
            let point = [
                r.int(words[0] as i32),
                r.int(words[1] as i32) + r.ld(lift),
                r.int(words[2] as i32),
            ];
            crate::scene::rotate_point::turned(r, &matrix[..], point).map(|v| r.st(v))
        });
        // Only words 3 .. 8 are written; the projection reads words 4 .. 6.
        let mut made = [0u32; 8];
        made[4..7].copy_from_slice(&turned);
        if !crate::math::x87::below_or_unordered(made[6], ctx.reads.numbers.k4096_a) {
            project_record(
                ctx.reads.stage_prefs,
                ctx.reads.course_work.get().flat_depth,
                &mut made,
                &ctx.view.polygon.window,
            );
        }
        Ok(Turned {
            words: made[3..8].try_into().expect("5 dwords"),
        })
    }
}

/// `race::draw_skidmarks`: draws the skidmarks shown in the drawn view (all in modes 0 and 2, else
/// those with the view's bit). The polygon's four work entries are set once (texture places
/// `fade_uv`, amount 0x60, textures 0xC0). Then each link with a link before gives a quad of its
/// two points and the two before (vertices 3, 2, 0, 1), fogged (`scene::fog_polygon`), cut by
/// depth and to the screen and drawn in black in mode 3 without the depth test. Refuses what the
/// scene's functions refuse (mid-way; not met with four vertices).
fn draw_skidmarks(ctx: &crate::race::view_scratch::DrawCtx<'_>) -> Result<(), Refused> {
    let race = ctx.race;
    let mut views = u32::MAX;
    let mode: u32 = ctx.reads.vars.run_kind();
    if mode != 0 && mode != 2 {
        views = 1u32.wrapping_shl(ctx.view.drawn_view(race) & 0x1F);
    }
    let entries = &ctx.part.polygon.entries[..4];
    for (k, e) in (0..4u32).zip(entries) {
        e.record.set(crate::scene::point_records::of_quad(k));
        e.light.set(0);
        e.lamp_light.set(0);
    }
    let (u, w) = (
        crate::race::vars_store::FADE_UV[0],
        crate::race::vars_store::FADE_UV[1],
    );
    for e in entries {
        e.u.set(u);
        e.u2.set(u);
        e.v.set(w);
        e.v2.set(w);
        e.point.set(u32::MAX);
        e.amount.set(0x60);
        e.fog.set(0);
    }
    ctx.view.polygon.scalars.borrow_mut().polygon_textures[0] = 0xC0u32;
    ctx.view.polygon.scalars.borrow_mut().polygon_textures[1] = 0xC0u32;
    let matrix: [u32; 15] = {
        let kept = ctx.view.matrices.borrow().view_matrix;
        std::array::from_fn(|n| kept[n].to_bits())
    };
    // The last quad's two points (its link's number and the points as turned): the next quad of
    // the skidmark has them as its two before, and nothing it is drawn with changes what they
    // are turned from.
    let mut last: Option<(u32, Turned, Turned)> = None;
    for n in 0..0x400u32 {
        let link = |at: u32| -> Result<u16, BadInput> {
            let at = at as usize;
            // A link is -1 (the table's start values) or a point of the ring: the links are written
            // only by `init_skidmarks` and `add_skid_point`, which refuses one outside the 0x400
            // points, so `before * 8 + 6` ends inside the table's 0x2000 bytes.
            if at + 2 <= crate::race::shadow_skids_store::LINKS_LEN {
                Ok(ctx.reads.shadow_skids.read(|skids| {
                    u16::from_le_bytes([skids.skid_links[at], skids.skid_links[at + 1]])
                }))
            } else {
                Err(BadInput("a skid link past the links' table"))
            }
        };
        let before = link(8 * n)? as i16 as i32 as u32;
        if before == u32::MAX {
            continue;
        }
        let shown = link(before.wrapping_mul(8).wrapping_add(6))? as u32;
        if shown & views == 0 {
            continue;
        }
        use crate::scene::point_records::of_quad;
        let first = skid_vertex(ctx, &matrix, 2 * n, of_quad(3))?;
        let second = skid_vertex(ctx, &matrix, 2 * n + 1, of_quad(2))?;
        match last {
            Some((m, a, b)) if m == before => {
                put_turned(ctx.part, &ctx.view.polygon.quads, of_quad(0), a)?;
                put_turned(ctx.part, &ctx.view.polygon.quads, of_quad(1), b)?;
            }
            _ => {
                skid_vertex(ctx, &matrix, before.wrapping_mul(2), of_quad(0))?;
                skid_vertex(
                    ctx,
                    &matrix,
                    before.wrapping_mul(2).wrapping_add(1),
                    of_quad(1),
                )?;
            }
        }
        last = Some((n, first, second));
        ctx.view.polygon.scalars.borrow_mut().polygon_vertex_count = 4u32;
        ctx.view.polygon.scalars.borrow_mut().polygon_next_record =
            crate::scene::polygon::pool_start();
        let list = &ctx.part.polygon.list;
        for k in 0..4u32 {
            list[k as usize].set(k);
        }
        list[4].set(0);
        ctx.view.polygon.scalars.borrow_mut().polygon_next_entry = 4u32;
        crate::scene::fog_polygon::fog_polygon(ctx.part, ctx.view)?;
        crate::scene::clip_polygon_by_depth::clip_polygon_by_depth(
            ctx.part,
            &ctx.view.polygon,
            ctx.reads.course(),
            &ctx.reads.draw_switches,
            ctx.far_clip,
            ctx.view,
        )?;
        if ctx.view.polygon.scalars.borrow().polygon_vertex_count < 3 {
            continue;
        }
        crate::scene::clip_polygon_to_screen::clip_polygon_to_screen(
            ctx.part,
            &ctx.view.polygon,
            ctx.reads.course(),
        )?;
        if ctx.view.polygon.scalars.borrow().polygon_vertex_count >= 3 {
            ctx.view.polygon.scalars.borrow_mut().polygon_colour = 0u32;
            crate::scene::depth_test_off::depth_test_off(ctx.part);
            ctx.view.face.borrow_mut().polygon_mode = 3u32;
            ctx.view.polygon.scalars.borrow_mut().polygon_sides[0] = 0u32;
            ctx.view.polygon.scalars.borrow_mut().polygon_sides[1] = 1u32;
            crate::scene::draw_polygon::draw_polygon_in_game(
                ctx.view,
                ctx.part,
                ctx.reads,
                ctx.reads.screen,
            )?;
            ctx.view.face.borrow_mut().polygon_mode = 0u32;
            crate::scene::depth_test_on::depth_test_on(ctx.part);
        }
        ctx.view.polygon.scalars.borrow_mut().polygon_colour = 0x00FF_FFFFu32;
    }
    Ok(())
}

/// `race::draw_skidmarks_if_on`: `draw_skidmarks` unless `drawing` is set.
pub(super) fn draw_skidmarks_if_on(
    ctx: &crate::race::view_scratch::DrawCtx<'_>,
) -> Result<(), Refused> {
    if ctx.reads.frame_work.drawing() != 0 {
        return Ok(());
    }
    draw_skidmarks(ctx)
}

#[cfg(test)]
mod reference_vectors {
    use super::*;
    use crate::math::x87_float::{X, f80};
    use crate::race::asm_test::{Rng, at_each_control_word};
    use crate::test_support::vectors::{Recorder, bytes, word};

    fn same_bits(a: u32, b: u32) -> bool {
        crate::race::asm_test::same(a, b)
    }

    #[test]
    fn conversions_match() {
        let mut r = Rng(3);
        let mut file = Recorder::replay("race.skidmarks.conversions");
        at_each_control_word(|| {
            for _ in 0..9000 {
                let (i, f) = (r.int() as u32, r.float());
                file.input(&[i.to_le_bytes(), f.to_le_bytes()].concat());
                assert_eq!(int_plus_float_80(i, f), bytes(&mut file), "{i} {f:#x}");
                assert_eq!(int_80(i), bytes(&mut file), "{i}");
                let v = if r.below(2) == 0 {
                    f64::from(r.int())
                } else {
                    f64::from(f32::from_bits(r.float())) * f64::from(f32::from_bits(r.float()))
                };
                let t = f80(X(v));
                file.input(&t);
                let (want, got) = (word(&mut file), float_of_80(t));
                assert!(same_bits(want, got), "{v}: {want:#x} {got:#x}");
            }
        });
        file.finish();
    }
}

/// The projection against recorded results (`test_support::race_vectors`).
#[cfg(test)]
mod vector_tests {
    use super::*;
    use crate::race::asm_test::{Rng, at_each_control_word, same};
    use crate::test_support::own::race;
    use crate::test_support::race_vectors::{Kind, Vectors};
    use crate::test_support::state::Snapshot;

    /// The stores the test writes.
    const STORES: [&str; 1] = ["owners.scene.draw_work"];

    #[test]
    fn project_vertex_is_the_originals() {
        let course = &race().course;
        let kept = Snapshot::of(&STORES);
        let kept_course = *course.course.borrow();
        let kept_flat = course.stage_prefs.borrow().flat_depth;
        let mut vectors = Vectors::open("race.skidmarks.project_vertex", Kind::Exact, 8);
        let mut r = Rng(4);
        let win = crate::scene::view_window::first_window();
        at_each_control_word(|| {
            for n in 0..20000 {
                let flat = n % 2 == 0;
                course.stage_prefs.borrow_mut().flat_depth = flat as u32;
                course.course.borrow_mut().flat_depth = r.int() as u32;
                // The recorded run read the scene's window; this code reads the window it is given.
                {
                    let mut win = win.borrow_mut();
                    win.view_scale = f32::from_bits(r.float());
                    win.view_centre[1] = f32::from_bits(r.float());
                    win.view_centre[0] = f32::from_bits(r.float());
                }
                crate::test_support::own::scene().draw_work_mut().scratch = 0xAAAA_AAAAu32;
                let mut a = [0xAAAA_AAAAu32; 8];
                for q in a.iter_mut().skip(4).take(3) {
                    *q = r.float();
                }
                let from = a;
                project_record(
                    crate::race::prefs_store::StagePrefsRef::Store(&course.stage_prefs),
                    course.course.borrow().flat_depth,
                    &mut a,
                    &win,
                );
                vectors.check(&a, same, || format!("flat {flat} from {from:x?}"));
            }
        });
        kept.restore();
        course.stage_prefs.borrow_mut().flat_depth = kept_flat;
        *course.course.borrow_mut() = kept_course;
        vectors.finish();
    }
}
