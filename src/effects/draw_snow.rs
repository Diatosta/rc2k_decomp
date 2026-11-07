//! `effects::draw_snow` (0x4C2E36): the snowflakes, `dword_8D21A4` of them (also into
//! `dword_8D21A0`; `dword_8DA428` the first, 0, then plus the count; `dword_8D219C` the
//! drawn). For each flake k: its place from `g_marksRandomsA` .. [k], y plus the fall
//! `dword_8D2194`, x and z moved by `g_marksCurve` at (the fall's bits 11 .. 19) plus
//! `g_marksRemaindersA` / `B`[k], plus `dword_8D2214` .., wrapped to 0x100000 and centred;
//! turned by the matrix at `dword_8D2220` .. plus `dword_8D2208` .. into `flt_8D22D4` ..
//! (the depth also whole, `dword_8D21F4`); unless that is at or below 0x10000: when
//! `dword_76523C` is set the colour `g_marksFadeColours`[the square of the light's dot
//! with the way from `flt_8DB430` .. times `flt_76518C`, 0 when the dot is below 0]; the
//! projected square (side `flt_8DB548` over the depth), depth times `flt_8D21F8`,
//! texture u `g_marksSpreads`[k] (+ `flt_8DB54C`), v `dword_8DB550` / `8DB554`;
//! `effects::clip_quad`, and unless all is outside `scene::draw_fan`, the drawn count plus 1
//! and `g_polygonVertexCount` 4.
//!
//! In:  nothing (`dword_8D21A4`)
//! Out: nothing (its caller reads no register after it)
//!
//! Caller: `effects::run_snow` (0x4C2A8F). The x87 blocks are plain Rust on the x87 model (`math::x87_float`), in the original's order,
//! over the game's variables. Refuses, before anything (a dry pass, its scratch put back),
//! more than 0x800 flakes, null marks settings with a flake, a curve index past the 4095
//! entries and a fade index past the 17.

use crate::effects::clip_quad::{clip_quad, draw_fan};
use crate::effects::rain::Weather;
use crate::effects::snow::{SnowLight, SnowState, SnowTables};
use crate::effects::tuning::MarksFloats;
use crate::errors::{BadInput, Refused};

const MOST: i32 = 0x800;
const CURVE: u32 = 4095;
const FADES: u32 = 17;
use crate::d3d::vertices_store::VERTICES;
/// The original's 0x4C2EF0 .. 0x4C2F79: the place (ebx, eax, ecx as `fild`ed) turned.
fn turn(weather: &Weather, v: &[u32; 3]) {
    turn_x87(&v[..], &weather.snow);
}

/// The original's 0x4C2F9C .. 0x4C3038: the light's dot and its fade index.
fn fade(weather: &Weather) {
    fade_x87(&weather.snow, &weather.snow_light, &effects().setup.floats);
}

/// The original's 0x4C305B .. 0x4C30EE: the projected square, its depth and u.
fn square(weather: &Weather, i: u32) {
    let mut w = [0u32; 32];
    square_x87(
        crate::scene::x87_mem::cells(&mut w),
        i as usize,
        &weather.snow_tables,
        &weather.snow,
        &weather.snow_light,
    );
    for (vertex, w) in VERTICES.vertices.iter().zip(w.as_chunks::<8>().0) {
        vertex.x.set_bits(w[0]);
        vertex.y.set_bits(w[1]);
        vertex.z.set_bits(w[2]);
        vertex.u.set_bits(w[6]);
    }
}

/// Flake k's whole-number place and the two curve indices: (eax, ebx, ecx, x index, z
/// index, the shift's edx, ebp, edi).
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
            .map(super::super::session::Dword::get);
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
fn refusal(weather: &Weather) -> Option<&'static str> {
    let n = weather.snow.flakes.get() as i32;
    if n > MOST {
        return Some("more than 0x800 snowflakes");
    }
    if n > 0 && effects().setup.settings.is_set.get() == 0 {
        return Some("no marks settings");
    }
    None
}

/// The Rust version (see the header).
pub fn draw_snow(weather: &Weather, scene: &crate::scene::owner::Scene) -> Result<(), Refused> {
    let (snow, snow_light) = (&weather.snow, &weather.snow_light);
    let snow_vars = &weather.snow_vars;
    if let Some(why) = refusal(weather) {
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
                if effects().setup.kind_floats.fading.bits() != 0 {
                    fade(weather);
                    let mut index = snow_light.fade_index.get();
                    if (snow_light.dot.bits() as i32) < 0 {
                        index = 0;
                    }
                    if index >= FADES {
                        return Err(BadInput("a snow fade index past the table").into());
                    }
                    let colour = snow_light.fade_colours[index as usize].get();
                    for n in 0..4 {
                        VERTICES.vertices[n].colour.set(colour);
                    }
                }
                square(weather, 4 * k);
                let [top, bottom] = snow_light
                    .v
                    .each_ref()
                    .map(super::super::session::Dword::get);
                for n in 0..4 {
                    VERTICES.vertices[n]
                        .v
                        .set_bits(if n < 2 { top } else { bottom });
                }
                if !clip_quad(scene)? {
                    draw_fan();
                    snow.drawn.set(snow.drawn.get().wrapping_add(1));
                    scene.drawing.scene.polygon_vertex_count.set_bits(4u32);
                }
            }
        }
    }
    Ok(())
}

use crate::effects::owner::effects;
use crate::math::x87_float as bh;
use crate::scene::x87_mem::{Base, fild as ldi, fld as ld, fstp as st};

/// `turn`'s x87 block as plain Rust on the x87 model (34 instructions), same order and memory places.
fn turn_x87(v: impl Base, s: &SnowState) {
    let t0 = ldi(v, 0);
    let t1 = ldi(v, 0x4);
    let t2 = ldi(v, 0x8);
    let t3 = t1 * bh::ld(s.matrix_copy[0].bits());
    let t4 = t0 * bh::ld(s.matrix_copy[3].bits());
    let t5 = t2 * bh::ld(s.matrix_copy[6].bits());
    let t6 = t4 + t5;
    let t7 = t3 + t6;
    let t8 = t7 + bh::int(s.point[0].get() as i32);
    s.turned[0].set_bits(bh::st(t8));
    let t9 = t1 * bh::ld(s.matrix_copy[1].bits());
    let t10 = t0 * bh::ld(s.matrix_copy[4].bits());
    let t11 = t2 * bh::ld(s.matrix_copy[7].bits());
    let t12 = t10 + t11;
    let t13 = t9 + t12;
    let t14 = t13 + bh::int(s.point[1].get() as i32);
    s.turned[1].set_bits(bh::st(t14));
    let t15 = t1 * bh::ld(s.matrix_copy[2].bits());
    let t16 = t0 * bh::ld(s.matrix_copy[5].bits());
    let t17 = t2 * bh::ld(s.matrix_copy[8].bits());
    let t18 = t16 + t17;
    let t19 = t15 + t18;
    let t20 = t19 + bh::int(s.point[2].get() as i32);
    s.depth.set(bh::to_int(t20) as u32);
    s.turned[2].set_bits(bh::st(t20));
}

/// `fade`'s x87 block as plain Rust on the x87 model (34 instructions), same order and memory places.
fn fade_x87(s: &SnowState, l: &SnowLight, f: &MarksFloats) {
    {
        let t0 = bh::ld(s.turned[0].bits());
        let t1 = t0 - bh::ld(l.light_place[0].bits());
        l.fade_delta[0].set_bits(bh::st(t1));
        let t2 = t1 * t1;
        let t3 = bh::ld(s.turned[1].bits());
        let t4 = t3 - bh::ld(l.light_place[1].bits());
        l.fade_delta[1].set_bits(bh::st(t4));
        let t5 = t4 * t4;
        let t6 = bh::ld(s.turned[2].bits());
        let t7 = t6 - bh::ld(l.light_place[2].bits());
        l.fade_delta[2].set_bits(bh::st(t7));
        let t8 = t7 * t7;
        let t9 = t5 + t8;
        let t10 = t2 + t9;
        let t11 = t10.sqrt();
        let t12 = bh::ld(f.one.bits()) / t11;
        let t13 = t12 * bh::ld(l.fade_delta[0].bits());
        l.fade_unit[0].set_bits(bh::st(t13));
        let t14 = t13 * bh::ld(l.light_way[0].bits());
        let t15 = t12 * bh::ld(l.fade_delta[1].bits());
        l.fade_unit[1].set_bits(bh::st(t15));
        let t16 = t15 * bh::ld(l.light_way[1].bits());
        let t17 = t12 * bh::ld(l.fade_delta[2].bits());
        l.fade_unit[2].set_bits(bh::st(t17));
        let t18 = t17 * bh::ld(l.light_way[2].bits());
        let t19 = t14 + t18;
        let t20 = t16 + t19;
        l.dot.set_bits(bh::st(t20));
        let t21 = t20 * t20;
        let t22 = t21 * bh::ld(f.sixteen.bits());
        l.fade_index.set(bh::to_int(t22) as u32);
    }
}

/// `square`'s x87 block as plain Rust on the x87 model (38 instructions), same order and memory places.
fn square_x87(vt: impl Base, i: usize, t: &SnowTables, s: &SnowState, l: &SnowLight) {
    let t0 = bh::ld(s.turned[2].bits());
    let t1 =
        crate::math::x87_float::ld(crate::scene::owner::scene().drawing.scene.view_scale.bits())
            / t0;
    s.depth.set(bh::st(t1));
    let t2 = bh::ld(s.turned[1].bits());
    let t3 = t2 * t1;
    let t4 = t1 * bh::ld(s.turned[0].bits());
    let t5 = -t3;
    let t6 = t5
        + crate::math::x87_float::ld(
            crate::scene::owner::scene().drawing.scene.view_centre[1].bits(),
        );
    st(vt, 0x4, t6);
    let t7 = t4
        + crate::math::x87_float::ld(
            crate::scene::owner::scene().drawing.scene.view_centre[0].bits(),
        );
    st(vt, 0x0, t7);
    let t8 = bh::ld(l.side.bits());
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
    let t15 = bh::ld(s.turned[2].bits());
    let t16 = t15 * bh::ld(s.inverse.bits());
    st(vt, 0x8, t16);
    st(vt, 0x28, t16);
    st(vt, 0x48, t16);
    st(vt, 0x68, t16);
    let t17 = crate::math::x87_float::ld(t.spreads[i / 4].bits());
    st(vt, 0x18, t17);
    st(vt, 0x78, t17);
    let t18 = t17 + bh::ld(l.u_offset.bits());
    st(vt, 0x38, t18);
    st(vt, 0x58, t18);
}

#[cfg(all(test, target_arch = "x86"))]
mod asm_reference {
    //! The original assembly blocks, kept verbatim as the reference the plain versions are tested against.

    pub(super) fn turn_x87(mk: usize, v: usize) {
        unsafe {
            core::arch::asm!(
                    "fild dword ptr [{v}]",
            "fild dword ptr [{v} + 4]",
            "fild dword ptr [{v} + 8]",
            ".byte 0xD9, 0xC1",
            "fmul dword ptr [{mk} + 0x16D2E0]",
            ".byte 0xD9, 0xC3",
            "fmul dword ptr [{mk} + 0x16D2EC]",
            ".byte 0xD9, 0xC2",
            "fmul dword ptr [{mk} + 0x16D2F8]",
            ".byte 0xDE, 0xC1",
            ".byte 0xDE, 0xC1",
            "fiadd dword ptr [{mk} + 0x16D2C8]",
            "fstp dword ptr [{mk} + 0x16D394]",
            ".byte 0xD9, 0xC1",
            "fmul dword ptr [{mk} + 0x16D2E4]",
            ".byte 0xD9, 0xC3",
            "fmul dword ptr [{mk} + 0x16D2F0]",
            ".byte 0xD9, 0xC2",
            "fmul dword ptr [{mk} + 0x16D2FC]",
            ".byte 0xDE, 0xC1",
            ".byte 0xDE, 0xC1",
            "fiadd dword ptr [{mk} + 0x16D2CC]",
            "fstp dword ptr [{mk} + 0x16D398]",
            ".byte 0xD9, 0xC9",
            "fmul dword ptr [{mk} + 0x16D2E8]",
            ".byte 0xD9, 0xCA",
            "fmul dword ptr [{mk} + 0x16D2F4]",
            ".byte 0xD9, 0xC9",
            "fmul dword ptr [{mk} + 0x16D300]",
            ".byte 0xDE, 0xC1",
            ".byte 0xDE, 0xC1",
            "fiadd dword ptr [{mk} + 0x16D2D0]",
            "fist dword ptr [{mk} + 0x16D2B4]",
            "fstp dword ptr [{mk} + 0x16D39C]",
            mk = in(reg) mk,
            v = in(reg) v,
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
            );
        }
    }

    pub(super) fn fade_x87(mk: usize) {
        unsafe {
            core::arch::asm!(
                    "fld dword ptr [{mk} + 0x16D394]",
            "fsub dword ptr [{mk} + 0x1764F0]",
            "fst dword ptr [{mk} + 0x176508]",
            ".byte 0xD8, 0xC8",
            "fld dword ptr [{mk} + 0x16D398]",
            "fsub dword ptr [{mk} + 0x1764F4]",
            "fst dword ptr [{mk} + 0x17650C]",
            ".byte 0xD8, 0xC8",
            "fld dword ptr [{mk} + 0x16D39C]",
            "fsub dword ptr [{mk} + 0x1764F8]",
            "fst dword ptr [{mk} + 0x176510]",
            ".byte 0xD8, 0xC8",
            ".byte 0xDE, 0xC1",
            ".byte 0xDE, 0xC1",
            ".byte 0xD9, 0xFA",
            "fdivr dword ptr [{mk} + 0x22C]",
            ".byte 0xD9, 0xC0",
            "fmul dword ptr [{mk} + 0x176508]",
            "fst dword ptr [{mk} + 0x176514]",
            "fmul dword ptr [{mk} + 0x1764FC]",
            ".byte 0xD9, 0xC1",
            "fmul dword ptr [{mk} + 0x17650C]",
            "fst dword ptr [{mk} + 0x176518]",
            "fmul dword ptr [{mk} + 0x176500]",
            ".byte 0xD9, 0xCA",
            "fmul dword ptr [{mk} + 0x176510]",
            "fst dword ptr [{mk} + 0x17651C]",
            "fmul dword ptr [{mk} + 0x176504]",
            ".byte 0xDE, 0xC1",
            ".byte 0xDE, 0xC1",
            "fst dword ptr [{mk} + 0x176588]",
            ".byte 0xD8, 0xC8",
            "fmul dword ptr [{mk} + 0x24C]",
            "fistp dword ptr [{mk} + 0x176520]",
            mk = in(reg) mk,
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
            );
        }
    }

    pub(super) fn square_x87(rc: usize, bf: usize, mk: usize, i: usize) {
        unsafe {
            core::arch::asm!(
                    "fld dword ptr [{mk} + 0x16D39C]",
            "fdivr dword ptr [{rc} + 0x224088]",
            "fst dword ptr [{mk} + 0x16D2B4]",
            "fld dword ptr [{mk} + 0x16D398]",
            ".byte 0xD8, 0xC9",
            ".byte 0xD9, 0xC9",
            "fmul dword ptr [{mk} + 0x16D394]",
            ".byte 0xD9, 0xC9",
            ".byte 0xD9, 0xE0",
            "fadd dword ptr [{rc} + 0x2240A0]",
            "fstp dword ptr [{bf} + 0x5605A4]",
            "fadd dword ptr [{rc} + 0x22409C]",
            "fstp dword ptr [{bf} + 0x5605A0]",
            "fld dword ptr [{mk} + 0x176608]",
            "fmul dword ptr [{mk} + 0x16D2B4]",
            "fld dword ptr [{bf} + 0x5605A0]",
            ".byte 0xD8, 0xC1",
            "fst dword ptr [{bf} + 0x5605C0]",
            "fst dword ptr [{bf} + 0x5605E0]",
            ".byte 0xD8, 0xE1",
            "fstp dword ptr [{bf} + 0x560600]",
            "fld dword ptr [{bf} + 0x5605A4]",
            "fst dword ptr [{bf} + 0x5605C4]",
            ".byte 0xDE, 0xC1",
            "fst dword ptr [{bf} + 0x5605E4]",
            "fstp dword ptr [{bf} + 0x560604]",
            "fld dword ptr [{mk} + 0x16D39C]",
            "fmul dword ptr [{mk} + 0x16D2B8]",
            "fst dword ptr [{bf} + 0x5605A8]",
            "fst dword ptr [{bf} + 0x5605C8]",
            "fst dword ptr [{bf} + 0x5605E8]",
            "fstp dword ptr [{bf} + 0x560608]",
            "fld dword ptr [{mk} + {i} + 0x16B0C0]",
            "fst dword ptr [{bf} + 0x5605B8]",
            "fst dword ptr [{bf} + 0x560618]",
            "fadd dword ptr [{mk} + 0x17660C]",
            "fst dword ptr [{bf} + 0x5605D8]",
            "fstp dword ptr [{bf} + 0x5605F8]",
            rc = in(reg) rc,
            bf = in(reg) bf,
            mk = in(reg) mk,
            i = in(reg) i,
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
            );
        }
    }
}

#[cfg(all(test, target_arch = "x86"))]
mod asm_differential {
    use super::asm_reference as reference;
    use crate::effects::asm_test::Kind::*;
    use crate::effects::asm_test::with_copy;
    use crate::effects::asm_test::{Op, Spec, differential};
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
        differential(
            &SPEC,
            |b| reference::turn_x87(b[0], b[1]),
            |b| with_copy(b[0], SnowState::START, |s| super::turn_x87(b[1], s)),
        );
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
        differential(
            &SPEC,
            |b| reference::fade_x87(b[0]),
            |b| {
                with_copy(b[0], SnowState::START, |s| {
                    with_copy(b[0], SnowLight::START, |l| {
                        with_copy(b[0], MarksFloats::START, |f| super::fade_x87(s, l, f));
                    });
                });
            },
        );
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
                    lo: 0x5605A0,
                    hi: 0x56061C,
                    reads: &[(0x5605A0, F32), (0x5605A4, F32)],
                    writes: &[
                        (0x5605A0, F32),
                        (0x5605A4, F32),
                        (0x5605A8, F32),
                        (0x5605B8, F32),
                        (0x5605C0, F32),
                        (0x5605C4, F32),
                        (0x5605C8, F32),
                        (0x5605D8, F32),
                        (0x5605E0, F32),
                        (0x5605E4, F32),
                        (0x5605E8, F32),
                        (0x5605F8, F32),
                        (0x560600, F32),
                        (0x560604, F32),
                        (0x560608, F32),
                        (0x560618, F32),
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
        differential(
            &SPEC,
            |b| reference::square_x87(b[0], b[1], b[2], b[3]),
            |b| {
                with_copy(b[2], SnowTables::START, |t| {
                    with_copy(b[2], SnowState::START, |s| {
                        with_copy(b[2], SnowLight::START, |l| {
                            super::square_x87(b[1].wrapping_add(0x5605A0), b[3], t, s, l);
                        });
                    });
                });
            },
        );
    }
}
