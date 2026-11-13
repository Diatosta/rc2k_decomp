//! `effects::draw_snow` (0x4C2E36): the snowflakes, `flakes` of them (also into
//! `count_copy`; the walk's `first` is 0, then plus the count; `drawn` counts the drawn).
//! For each flake k: its place from `randoms`[k], y plus the fall, x and z moved by `curve`
//! at (the fall's bits 11 .. 19) plus `remainders`[k], plus the view's place less the point
//! (`view`), wrapped to 0x100000 and centred; turned by `matrix_copy` plus `point` into
//! `turned` (the depth also whole, in `depth`); unless that is at or below 0x10000: when the
//! light is on, the colour `fade_colours`[the square of the light's dot with the way from
//! `light_place`, times the fade step, 0 when the dot is below 0]; the projected square
//! (side `side` over the depth), depth times `inverse`, texture u `spreads`[k] (plus
//! `u_offset`), v `v`; `effects::clip_quad`, and unless all is outside `scene::draw_fan`, the
//! drawn count plus 1 and the polygon's vertex count 4.
//!
//! Works on the snow state.
//!
//! Caller: `effects::run_snow`. The x87 blocks are plain Rust on the x87 model (`math::x87_float`), in the original's order,
//! over the game's variables. Refuses, before anything (a dry pass, its scratch put back),
//! more than 0x800 flakes, null marks settings with a flake, a curve index past the 4095
//! entries and a fade index past the 17.

use crate::effects::cell_x87::CellX87;
use crate::effects::clip_quad::{clip_quad, draw_fan};
use crate::effects::rain::Weather;
use crate::effects::snow::{SnowLight, SnowState, SnowTables};
use crate::effects::tuning::MarksFloats;
use crate::errors::{BadInput, Refused};

const MOST: i32 = 0x800;
const CURVE: u32 = 4095;
const FADES: u32 = 17;
/// The flake's whole-number place turned by the view.
fn turn(weather: &Weather, v: &[u32; 3]) {
    turn_x87(&v[..], &weather.snow);
}

/// The light's dot and its fade index.
fn fade(weather: &Weather, effects: &crate::effects::owner::Effects) {
    fade_x87(&weather.snow, &weather.snow_light, &effects.setup.floats);
}

/// The projected square, its depth and u.
fn square(buffer: &crate::d3d::vertices_store::PolygonVertices, weather: &Weather, i: u32) {
    let mut w = [0u32; 32];
    square_x87(
        crate::scene::x87_mem::cells(&mut w),
        i as usize,
        &weather.snow_tables,
        &weather.snow,
        &weather.snow_light,
    );
    // The scratch holds four vertices' records, eight dwords each.
    for (vertex, w) in buffer.vertices.iter().zip(w.as_chunks::<8>().0) {
        let w = w.map(f32::from_bits);
        vertex.x.set(w[0]);
        vertex.y.set(w[1]);
        vertex.z.set(w[2]);
        vertex.u.set(w[6]);
    }
}

/// Flake k's whole-number place (x, y, z), the two curve indices (x, z) and the three
/// shifted values the indices are made from.
fn place(weather: &Weather, k: u32) -> ([u32; 3], [u32; 2], [u32; 3]) {
    {
        let t = &weather.snow_tables;
        let mut eax = t.randoms[0][k as usize].get();
        let mut ebx = t.randoms[1][k as usize]
            .get()
            .wrapping_add(weather.snow.fall.get());
        let mut ecx = t.randoms[2][k as usize].get();
        let f = (ebx & 0xF_FFFF) >> 11;
        let ix = f.wrapping_add(t.remainders[0][k as usize].get());
        let iz = f.wrapping_add(t.remainders[1][k as usize].get());
        let shift: [u32; 3] = weather
            .snow
            .view
            .each_ref()
            .map(super::super::session::PlainDword::get);
        let curve = |i: u32| {
            if i < CURVE {
                weather.snow_curve.curve[i as usize].get()
            } else {
                0
            }
        };
        eax = eax.wrapping_add(curve(ix)).wrapping_add(shift[0]);
        ecx = ecx.wrapping_add(curve(iz)).wrapping_add(shift[2]);
        ebx = ebx.wrapping_add(shift[1]);
        let wrap = |v: u32| (v & 0xF_FFFF).wrapping_sub(0x7_FFFF);
        ([wrap(eax), wrap(ebx), wrap(ecx)], [ix, iz], shift)
    }
}

/// Why it refuses before anything is drawn, if it does.
fn refusal(weather: &Weather, effects: &crate::effects::owner::Effects) -> Option<&'static str> {
    let n = weather.snow.flakes.get() as i32;
    if n > MOST {
        return Some("more than 0x800 snowflakes");
    }
    if n > 0 && effects.setup.settings.is_set.get() == 0 {
        return Some("no marks settings");
    }
    None
}

pub fn draw_snow(
    buffer: &crate::d3d::vertices_store::PolygonVertices,
    weather: &Weather,
    scene: &crate::scene::owner::Scene,
    effects: &crate::effects::owner::Effects,
) -> Result<(), Refused> {
    let (snow, snow_light) = (&weather.snow, &weather.snow_light);
    let snow_vars = &weather.snow_vars;
    if let Some(why) = refusal(weather, effects) {
        return Err(BadInput(why).into());
    }
    {
        let n = snow.flakes.get() as i32;
        snow.count_copy.set(n as u32);
        snow_vars.first.set(0);
        snow.drawn.set(0);
        snow_vars.first.set(n as u32);
        for k in 0..n.max(0) as u32 {
            let ([eax, ebx, ecx], curve_at, _) = place(weather, k);
            if curve_at.iter().any(|&i| i >= CURVE) {
                return Err(BadInput("a snow curve index past the table").into());
            }
            turn(weather, &[ebx, eax, ecx]);
            if snow.depth.get() as i32 > 0x1_0000 {
                if effects.setup.kind_floats.fading.get() != 0 {
                    fade(weather, effects);
                    let mut index = snow_light.fade_index.get();
                    if snow_light.dot.get().is_sign_negative() {
                        index = 0;
                    }
                    if index >= FADES {
                        return Err(BadInput("a snow fade index past the table").into());
                    }
                    let colour = snow_light.fade_colours[index as usize].get();
                    for item in buffer.vertices.iter().take(4) {
                        item.colour.set(colour);
                    }
                }
                square(buffer, weather, 4 * k);
                let [top, bottom] = snow_light.v.each_ref().map(crate::session::PlainFloat::get);
                for (n, vertex) in buffer.vertices.iter().enumerate().take(4) {
                    vertex.v.set(if n < 2 { top } else { bottom });
                }
                if !clip_quad(buffer, effects)? {
                    draw_fan(buffer, scene);
                    snow.drawn.set(snow.drawn.get().wrapping_add(1));
                    crate::scene::polygon_work::front_end()
                        .polygon_vertex_count
                        .set(4);
                }
            }
        }
    }
    Ok(())
}

use crate::math::x87_float as bh;
use crate::scene::x87_mem::{Base, fild as ldi, fld as ld, fstp as st};

/// `turn`'s x87 block as plain Rust on the x87 model (34 instructions), same order and memory places.
fn turn_x87(v: impl Base, s: &SnowState) {
    let t0 = ldi(v, 0);
    let t1 = ldi(v, 0x4);
    let t2 = ldi(v, 0x8);
    let t3 = t1 * s.matrix_copy[0].ld();
    let t4 = t0 * s.matrix_copy[3].ld();
    let t5 = t2 * s.matrix_copy[6].ld();
    let t6 = t4 + t5;
    let t7 = t3 + t6;
    let t8 = t7 + bh::int(s.point[0].get() as i32);
    s.turned[0].st(t8);
    let t9 = t1 * s.matrix_copy[1].ld();
    let t10 = t0 * s.matrix_copy[4].ld();
    let t11 = t2 * s.matrix_copy[7].ld();
    let t12 = t10 + t11;
    let t13 = t9 + t12;
    let t14 = t13 + bh::int(s.point[1].get() as i32);
    s.turned[1].st(t14);
    let t15 = t1 * s.matrix_copy[2].ld();
    let t16 = t0 * s.matrix_copy[5].ld();
    let t17 = t2 * s.matrix_copy[8].ld();
    let t18 = t16 + t17;
    let t19 = t15 + t18;
    let t20 = t19 + bh::int(s.point[2].get() as i32);
    s.depth.set(bh::to_int(t20) as u32);
    s.turned[2].st(t20);
}

/// `fade`'s x87 block as plain Rust on the x87 model (34 instructions), same order and memory places.
fn fade_x87(s: &SnowState, l: &SnowLight, f: &MarksFloats) {
    {
        let t0 = s.turned[0].ld();
        let t1 = t0 - l.light_place[0].ld();
        l.fade_delta[0].st(t1);
        let t2 = t1 * t1;
        let t3 = s.turned[1].ld();
        let t4 = t3 - l.light_place[1].ld();
        l.fade_delta[1].st(t4);
        let t5 = t4 * t4;
        let t6 = s.turned[2].ld();
        let t7 = t6 - l.light_place[2].ld();
        l.fade_delta[2].st(t7);
        let t8 = t7 * t7;
        let t9 = t5 + t8;
        let t10 = t2 + t9;
        let t11 = t10.sqrt();
        let t12 = f.one.ld() / t11;
        let t13 = t12 * l.fade_delta[0].ld();
        l.fade_unit[0].st(t13);
        let t14 = t13 * l.light_way[0].ld();
        let t15 = t12 * l.fade_delta[1].ld();
        l.fade_unit[1].st(t15);
        let t16 = t15 * l.light_way[1].ld();
        let t17 = t12 * l.fade_delta[2].ld();
        l.fade_unit[2].st(t17);
        let t18 = t17 * l.light_way[2].ld();
        let t19 = t14 + t18;
        let t20 = t16 + t19;
        l.dot.st(t20);
        let t21 = t20 * t20;
        let t22 = t21 * f.sixteen.ld();
        l.fade_index.set(bh::to_int(t22) as u32);
    }
}

/// `square`'s x87 block as plain Rust on the x87 model (38 instructions), same order and memory places.
fn square_x87(vt: impl Base, i: usize, t: &SnowTables, s: &SnowState, l: &SnowLight) {
    let t0 = s.turned[2].ld();
    let t1 = crate::math::x87_float::ld(
        crate::scene::polygon_work::front_end()
            .window
            .view_scale
            .get(),
    ) / t0;
    s.depth.set(bh::st(t1));
    let t2 = s.turned[1].ld();
    let t3 = t2 * t1;
    let t4 = t1 * s.turned[0].ld();
    let t5 = -t3;
    let t6 = t5
        + crate::math::x87_float::ld(
            crate::scene::polygon_work::front_end().window.view_centre[1].get(),
        );
    st(vt, 0x4, t6);
    let t7 = t4
        + crate::math::x87_float::ld(
            crate::scene::polygon_work::front_end().window.view_centre[0].get(),
        );
    st(vt, 0x0, t7);
    let t8 = l.side.ld();
    let t9 = t8 * bh::ld(s.depth.get());
    let t10 = ld(vt, 0x0);
    let t11 = t10 + t9;
    st(vt, 0x20, t11);
    st(vt, 0x40, t11);
    let t12 = t11 - t9;
    st(vt, 0x60, t12);
    let t13 = ld(vt, 0x4);
    st(vt, 0x24, t13);
    let t14 = t9 + t13;
    st(vt, 0x44, t14);
    st(vt, 0x64, t14);
    let t15 = s.turned[2].ld();
    let t16 = t15 * s.inverse.ld();
    st(vt, 0x8, t16);
    st(vt, 0x28, t16);
    st(vt, 0x48, t16);
    st(vt, 0x68, t16);
    let t17 = t.spreads[i / 4].ld();
    st(vt, 0x18, t17);
    st(vt, 0x78, t17);
    let t18 = t17 + l.u_offset.ld();
    st(vt, 0x38, t18);
    st(vt, 0x58, t18);
}

#[cfg(test)]
mod reference_vectors {
    use crate::effects::asm_layout::with_laid;
    use crate::effects::asm_test::Kind::*;

    use crate::effects::asm_test::{Op, Spec, recorded_writes};
    use crate::effects::snow::{SnowLight, SnowState, SnowTables};
    use crate::effects::tuning::MarksFloats;

    #[test]
    fn turn_x87_matches() {
        const SPEC: Spec = Spec {
            operands: &[
                Op::Base {
                    lo: 0x16D0C0,
                    hi: 0x16D3D0,
                    reads: &[
                        (0x16D2C8, I32),
                        (0x16D2CC, I32),
                        (0x16D2D0, I32),
                        (0x16D2E0, F32),
                        (0x16D2E4, F32),
                        (0x16D2E8, F32),
                        (0x16D2EC, F32),
                        (0x16D2F0, F32),
                        (0x16D2F4, F32),
                        (0x16D2F8, F32),
                        (0x16D2FC, F32),
                        (0x16D300, F32),
                    ],
                    writes: &[
                        (0x16D2B4, I32),
                        (0x16D394, F32),
                        (0x16D398, F32),
                        (0x16D39C, F32),
                    ],
                },
                Op::Base {
                    lo: 0x0,
                    hi: 0xC,
                    reads: &[(0x0, I32), (0x4, I32), (0x8, I32)],
                    writes: &[],
                },
            ],
            sincos: false,
        };
        recorded_writes("effects.draw_snow.turn_x87", &SPEC, |b| {
            with_laid::<SnowState, _>(b[0], |s| super::turn_x87(b[1], s));
        });
    }

    #[test]
    fn fade_x87_matches() {
        const SPEC: Spec = Spec {
            operands: &[Op::Base {
                lo: 0x200,
                hi: 0x176618,
                reads: &[
                    (0x22C, F32),
                    (0x24C, F32),
                    (0x16D394, F32),
                    (0x16D398, F32),
                    (0x16D39C, F32),
                    (0x1764F0, F32),
                    (0x1764F4, F32),
                    (0x1764F8, F32),
                    (0x1764FC, F32),
                    (0x176500, F32),
                    (0x176504, F32),
                    (0x176508, F32),
                    (0x17650C, F32),
                    (0x176510, F32),
                ],
                writes: &[
                    (0x176508, F32),
                    (0x17650C, F32),
                    (0x176510, F32),
                    (0x176514, F32),
                    (0x176518, F32),
                    (0x17651C, F32),
                    (0x176520, I32),
                    (0x176588, F32),
                ],
            }],
            sincos: false,
        };
        recorded_writes("effects.draw_snow.fade_x87", &SPEC, |b| {
            with_laid::<SnowState, _>(b[0], |s| {
                with_laid::<SnowLight, _>(b[0], |l| {
                    with_laid::<MarksFloats, _>(b[0], |f| super::fade_x87(s, l, f));
                });
            });
        });
    }

    #[test]
    fn square_x87_matches() {
        const SPEC: Spec = Spec {
            operands: &[
                Op::RaceBlock {
                    reads: &[(0x224088, F32), (0x22409C, F32), (0x2240A0, F32)],
                    writes: &[],
                },
                Op::Base {
                    lo: 0x0,
                    hi: 0x7C,
                    reads: &[(0x0, F32), (0x4, F32)],
                    writes: &[
                        (0x0, F32),
                        (0x4, F32),
                        (0x8, F32),
                        (0x18, F32),
                        (0x20, F32),
                        (0x24, F32),
                        (0x28, F32),
                        (0x38, F32),
                        (0x40, F32),
                        (0x44, F32),
                        (0x48, F32),
                        (0x58, F32),
                        (0x60, F32),
                        (0x64, F32),
                        (0x68, F32),
                        (0x78, F32),
                    ],
                },
                Op::Base {
                    lo: 0x1610C0,
                    hi: 0x176618,
                    reads: &[
                        (0x16B0C0, F32),
                        (0x16B0C4, F32),
                        (0x16B0C8, F32),
                        (0x16B250, F32),
                        (0x16D2B4, F32),
                        (0x16D2B8, F32),
                        (0x16D394, F32),
                        (0x16D398, F32),
                        (0x16D39C, F32),
                        (0x16F0BC, F32),
                        (0x176608, F32),
                        (0x17660C, F32),
                    ],
                    writes: &[(0x16D2B4, F32)],
                },
                Op::Index(&[0, 4, 8, 400, 8188]),
            ],
            sincos: false,
        };
        recorded_writes("effects.draw_snow.square_x87", &SPEC, |b| {
            with_laid::<SnowTables, _>(b[2], |t| {
                with_laid::<SnowState, _>(b[2], |s| {
                    with_laid::<SnowLight, _>(b[2], |l| {
                        super::square_x87(b[1], b[3].number(), t, s, l);
                    });
                });
            });
        });
    }
}
