//! The skidmarks' drawing (`race::draw_skidmarks_if_on` / `race::draw_skidmarks`).
//! A skidmark is a chain of up to 0x400 links (`skid_links`, 8 bytes each: +0 the link before, -1
//! none; +6 the views it shows in, a bit each) with two points each (`skid_points`, 0xC bytes a
//! point: x, y, z whole numbers); each link and the one before give a quad.

use crate::effects::cell_x87::CellX87;
use crate::errors::BadInput;
use crate::errors::Refused;
use crate::race::owner_course::Course;

/// `fild i; fadd f; fstp tbyte`: a whole number plus a float in the x87's 80 bits.
#[cfg(test)]
fn int_plus_float_80(i: u32, f: u32) -> [u8; 10] {
    use crate::math::x87_float::{f80, int, ld};
    f80(int(i as i32) + ld(f))
}

/// `fild i; fstp tbyte`.
pub fn int_80(i: u32) -> [u8; 10] {
    crate::math::x87_float::f80(crate::math::x87_float::int(i as i32))
}

/// `fld tbyte; fstp dword`.
#[cfg(test)]
fn float_of_80(t: [u8; 10]) -> u32 {
    crate::math::x87_float::st(crate::math::x87_float::from_f80(t))
}

/// The projection of a turned vertex record (`v` +0x10 x, +0x14 y, +0x18 depth, floats)
/// onto the screen (+0xC, +0x1C): flat (`flat_depth`) by `flat_depth` + 1, else by the
/// depth (the original's `fdivrp st(1), st` is `view_scale` over that). x is
/// x times that scale plus `view_centre`, y is `SceneDraw.view_centre[1]` less y times it. When flat,
/// the original also stored the depth at `scratch`; nothing read it, so it is not kept.
pub(crate) fn project_record(
    course: &Course,
    v: &mut crate::scene::point_records::Record,
    window: &crate::scene::view_window::ViewWindow,
) {
    use crate::math::x87_float::{ONE, int, ld, st};
    let (x, y, z) = (ld(v[4]), ld(v[5]), ld(v[6]));
    let reach = window.view_scale.ld();
    let scale = if course.stage_prefs.flat_depth.get() != 0 {
        reach / (int(course.course.flat_depth.get() as i32) + ONE)
    } else {
        reach / z
    };
    v[3] = st(x * scale + window.view_centre[0].ld());
    v[7] = st(-(y * scale) + window.view_centre[1].ld());
}

/// A skidmark point turned and projected: its vertex record's +0xC .. +0x20.
#[derive(Clone, Copy)]
struct Turned {
    words: [u32; 5],
}

/// One skidmark point (`skid_points` + 0xC x n) into the vertex record `v`: raised by
/// `skid_lift`, turned by `g_viewMatrix` (`scene::rotate_point`), +0xC / +0x1C 0, and
/// projected when its depth is not below 4096.0.
fn skid_vertex(
    ctx: &crate::race::view_scratch::DrawCtx<'_>,
    matrix: &[u32; 15],
    n: u32,
    v: u32,
) -> Result<Turned, BadInput> {
    let turned = turn_skid_point(ctx, matrix, n)?;
    put_turned(ctx.part, &ctx.view.polygon.quad_vertices, v, turned)?;
    Ok(turned)
}

/// A point as `skid_vertex` turns it, put into the vertex record `v` again.
fn put_turned(
    part: &crate::workers::PartScratch,
    quad: &[crate::session::Dword; 32],
    v: u32,
    turned: Turned,
) -> Result<(), BadInput> {
    let mut made = crate::scene::point_records::get_in(part, Some(quad), v)?;
    made[3..8].copy_from_slice(&turned.words);
    crate::scene::point_records::set_in(part, Some(quad), v, made)
}

/// Skidmark point `n` turned and projected, as `skid_vertex` does it.
fn turn_skid_point(
    ctx: &crate::race::view_scratch::DrawCtx<'_>,
    matrix: &[u32; 15],
    n: u32,
) -> Result<Turned, BadInput> {
    let race = ctx.race;
    {
        let points = &race.frame.shadow_skids.skid_points;
        let at = n as usize * 0xC;
        if at + 0xC > 0x6000 {
            return Err(BadInput("a skidmark point past the table"));
        }
        let lift = race.frame.shadow_skids.skid_lift.get();
        use crate::math::x87_float::{int, ld, st};
        let point = [
            int(points.get_u32(at) as i32),
            int(points.get_u32(at + 4) as i32) + ld(lift),
            int(points.get_u32(at + 8) as i32),
        ];
        let t = crate::scene::rotate_point::turned(&matrix[..], point);
        // Only +0xC .. +0x20 are written; the projection reads +0x10 .. +0x18.
        let mut made = [0u32; 8];
        made[4..7].copy_from_slice(&t.map(st));
        if !crate::math::x87::below_or_unordered(made[6], &race.course.numbers.k4096_a) {
            project_record(&race.course, &mut made, &ctx.view.polygon.window);
        }
        Ok(Turned {
            words: made[3..8].try_into().expect("5 dwords"),
        })
    }
}

/// `race::draw_skidmarks` (0x458C03): the skidmarks shown in the drawn view (all in modes 0 and 2,
/// else those with the view's bit): the polygon's four work entries set once
/// (vertices 0x623E40 .., texture places `fade_uv` / `RaceVars.fade_uv[1]`, bytes 0 and 0x60,
/// textures 0xC0); then for each link with a link before, its two points and the two before as a
/// quad (vertices 3, 2, 0, 1), `scene::fog_polygon`, cut by depth and to the screen and drawn in
/// black in mode 3 without the depth test. Refuses what the scene's functions refuse (mid-way; not
/// met with four vertices).
pub fn draw_skidmarks(ctx: &crate::race::view_scratch::DrawCtx<'_>) -> Result<(), Refused> {
    let (race, scene) = (ctx.race, ctx.scene);
    let frame = &race.frame;
    let mut views = u32::MAX;
    let mode: u32 = race.shared.vars.run_kind.get();
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
        race.shared.vars.fade_uv[0].get(),
        race.shared.vars.fade_uv[1].get(),
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
    ctx.view.polygon.polygon_textures[0].set(0xC0u32);
    ctx.view.polygon.polygon_textures[1].set(0xC0u32);
    let matrix: [u32; 15] = crate::math::cells::load(&ctx.view.matrices.view_matrix);
    // The last quad's two points (its link's number and the points as turned): a skidmark's
    // next quad has them as its two before, and nothing it is drawn with changes what they
    // are turned from.
    let mut last: Option<(u32, Turned, Turned)> = None;
    for n in 0..0x400u32 {
        let link = |at: u32| -> Result<u16, BadInput> {
            let table = &frame.shadow_skids.skid_links;
            let at = at as usize;
            if at + 2 <= size_of_val(table) {
                Ok(u16::from_le_bytes([table.get(at), table.get(at + 1)]))
            } else {
                // past the table, by address, as the original reads on
                crate::pointers::load_value::<u16>(table, at)
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
                put_turned(ctx.part, &ctx.view.polygon.quad_vertices, of_quad(0), a)?;
                put_turned(ctx.part, &ctx.view.polygon.quad_vertices, of_quad(1), b)?;
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
        ctx.view.polygon.polygon_vertex_count.set(4u32);
        ctx.view
            .polygon
            .polygon_next_record
            .set(crate::scene::polygon::pool_start());
        let list = &ctx.part.polygon.list;
        for k in 0..4u32 {
            list[k as usize].set(k);
        }
        list[4].set(0);
        ctx.view.polygon.polygon_next_entry.set(4u32);
        crate::scene::fog_polygon::fog_polygon(ctx.part, ctx.view)?;
        crate::scene::clip_polygon_by_depth::clip_polygon_by_depth(
            ctx.part,
            &ctx.view.polygon,
            race,
            scene,
            ctx.far_clip,
            ctx.view,
        )?;
        // Each quad inside the original's `pusha` .. `popa`.
        if ctx.view.polygon.polygon_vertex_count.get() < 3 {
            continue;
        }
        crate::scene::clip_polygon_to_screen::clip_polygon_to_screen(
            ctx.part,
            &ctx.view.polygon,
            race,
        )?;
        if ctx.view.polygon.polygon_vertex_count.get() >= 3 {
            ctx.view.polygon.polygon_colour.set(0u32);
            crate::scene::depth_test_off::depth_test_off();
            ctx.view.face.polygon_mode.set(3u32);
            ctx.view.polygon.polygon_sides[0].set(0u32);
            ctx.view.polygon.polygon_sides[1].set(1u32);
            crate::scene::draw_polygon::draw_polygon_in_game(
                &ctx.view.polygon,
                ctx.part,
                race,
                &ctx.view.face,
                scene,
                &ctx.system.session.screen,
            )?;
            ctx.view.face.polygon_mode.set(0u32);
            ctx.view.polygon.polygon_drawn.set(1u32);
            crate::scene::depth_test_on::depth_test_on();
        }
        ctx.view.polygon.polygon_colour.set(0x00FF_FFFFu32);
    }
    Ok(())
}

/// `race::draw_skidmarks_if_on` (0x4585E3): `draw_skidmarks` unless `drawing` is set.
pub fn draw_skidmarks_if_on(ctx: &crate::race::view_scratch::DrawCtx<'_>) -> Result<(), Refused> {
    let race = ctx.race;
    if race.frame.frame_work.drawing.get() != 0 {
        return Ok(());
    }
    draw_skidmarks(ctx)
}

#[cfg(test)]
mod reference_vectors {
    use super::*;
    use crate::math::x87_float::{X, f80};
    use crate::race::asm_test::{Rng, at_each_control_word, lock};
    use crate::test_support::vectors::{Recorder, bytes, word};

    fn same_bits(a: u32, b: u32) -> bool {
        crate::race::asm_test::same(a, b)
    }

    #[test]
    fn conversions_match() {
        let _l = lock();
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

/// The projection against the original's outputs, recorded (`test_support::race_vectors`).
#[cfg(test)]
mod vector_tests {
    use super::*;
    use crate::race::asm_test::{Rng, at_each_control_word, lock, same};
    use crate::race::owner::race;
    use crate::scene::owner::scene;
    use crate::test_support::race_vectors::{Kind, Vectors};
    use crate::test_support::state::Snapshot;

    /// The stores the test writes.
    const STORES: [&str; 3] = [
        "scene.owner.SCENE.drawing.scene",
        "race.owner.RACE.course.course",
        "race.owner.RACE.course.stage_prefs",
    ];

    #[test]
    fn project_vertex_is_the_originals() {
        let drawing = &scene().drawing;
        let course = &race().course;
        let _l = lock();
        let kept = Snapshot::of(&STORES);
        let mut vectors = Vectors::open("race.skidmarks.project_vertex", Kind::Exact, 8);
        let mut r = Rng(4);
        let win = crate::scene::view_window::ViewWindow::new();
        at_each_control_word(|| {
            for n in 0..20000 {
                let flat = n % 2 == 0;
                course.stage_prefs.flat_depth.set(flat as u32);
                course.course.flat_depth.set(r.int() as u32);
                // The original read the scene's window; this code the window it is given.
                for (old, own) in [
                    (&drawing.scene.view_scale, &win.view_scale),
                    (&drawing.scene.view_centre[1], &win.view_centre[1]),
                    (&drawing.scene.view_centre[0], &win.view_centre[0]),
                ] {
                    let value = r.float();
                    old.set_bits(value);
                    own.set_bits(value);
                }
                drawing.scene.scratch.set(0xAAAA_AAAAu32);
                let mut a = [0xAAAA_AAAAu32; 8];
                for q in a.iter_mut().skip(4).take(3) {
                    *q = r.float();
                }
                let from = a;
                project_record(course, &mut a, &win);
                vectors.check(&a, same, || format!("flat {flat} from {from:x?}"));
            }
        });
        kept.restore();
        vectors.finish();
    }
}
