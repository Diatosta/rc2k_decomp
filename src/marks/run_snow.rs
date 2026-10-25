//! `marks::run_snow` (0x4C25F6): the snow's frame. `dword_8DB4C4` the settings' +0x58; the
//! light's place (`flt_8DB50C` ..) turned by the view's place (`flt_5EA3CC`) and the view
//! (`g_viewMatrix`) into `flt_8DB430` .., its way (`flt_8DB518` ..) turned without the
//! places into `flt_8DB43C` ..; `marks::make_fade_colours`. `dword_8D21A4` the settings'
//! +0x2760 (the flakes; none: `dword_8D215C` 0 and back), held to 0x800; the settings'
//! +0x28 .. +0x48 into `dword_8D216C` .., +0x1C .. +0x24 into `dword_8D2160` ..;
//! `marks::place_view_2`, `scene::set_texture` (`dword_5D1E10`), `scene::depth_buffer_on`,
//! `scene::depth_test_off`, render states 0x15 = 4, 0x13 = 5, 0x14 = 6, 0x1C = 0, 0x1B = 1;
//! the four vertices' rhw `dword_8DB524`, colour `g_viewLight` | 0xFF000000, specular 0,
//! texture places from `dword_8DB528` ..; `g_polygonVertexCount` 4; 1 over `flt_6241C8`
//! into `flt_8D2110` .. and `flt_8D21F8`; when the settings' +0x10 is set the fall
//! `dword_8D2194` gains the clock's step (since `dword_8D2150`) times `flt_8DB538`
//! (`fistp`), kept to 28 bits, `dword_8D2150` the clock; the turn's nine floats and the
//! focal length each four times into `xmmword_8D2030` ..; `marks::draw_snow`;
//! `scene::depth_test_on`.
//!
//! In:  nothing (`g_marksSettings`)
//! Out: nothing (its caller reads no register after it)
//!
//! Caller: `marks::run_frame` (0x4D166B). The two turns are plain Rust on the x87 model (`math::x87_float`), in the original's order,
//! over the game's variables. Refuses null settings, before anything.

use crate::errors::{BadInput, Refused};
use crate::marks::snow::{SNOW, SNOW_LIGHT, SnowLight};
use crate::math::x87;
use crate::physics::tables_store::PHYSICS;
use crate::race::frame_store::FRAME_WORK;
use crate::scene::draw_store::DRAW;
use crate::scene::kind_texture_store::KIND_TEXTURES;

const MOST: i32 = 0x800;
use crate::d3d::vertices_store::VERTICES;
/// The render states set (value, state).
const STATES: [(u32, u32); 5] = [(4, 0x15), (5, 0x13), (6, 0x14), (0, 0x1C), (1, 0x1B)];

unsafe fn get(a: u32) -> u32 {
    unsafe { (a as *const u32).read_unaligned() }
}

/// The original's 0x4C2605 .. 0x4C276E: the light's place and way turned.
unsafe fn turn_light() {
    unsafe { turn_light_x87(crate::race::store::asm_base() as usize, &SNOW_LIGHT) }
}

/// The Rust version (see the header).
pub unsafe fn run_snow() -> Result<(), Refused> {
    use crate::scene::set_render_state::set_render_state;
    let (s, l) = (&SNOW, &SNOW_LIGHT);
    unsafe {
        let settings: u32 = crate::marks::setup::SETTINGS.pointer.get();
        if settings == 0 {
            return Err(BadInput("no marks settings").into());
        }
        l.kept.set(get(settings + 0x58));
        turn_light();
        crate::marks::make_fade_colours::make_fade_colours();
        let settings: u32 = crate::marks::setup::SETTINGS.pointer.get();
        let mut flakes = get(settings + 0x2760) as i32;
        s.flakes.set(flakes as u32);
        if flakes == 0 {
            s.flag.set(0);
            return Ok(());
        }
        if flakes > MOST {
            flakes = MOST;
            s.flakes.set(flakes as u32);
        }
        for n in 0..9 {
            s.view_matrix[n].set_bits(get(settings + 0x28 + 4 * n as u32));
        }
        for n in 0..3 {
            s.view_place[n].set(get(settings + 0x1C + 4 * n as u32));
        }
        crate::marks::place_view_2::place_view_2()?;
        crate::scene::set_texture::set_texture(KIND_TEXTURES.kind_first_slots[202].bits());
        crate::scene::depth_buffer_on::depth_buffer(true);
        crate::scene::depth_test_off::depth_test_off();
        for (value, state) in STATES {
            set_render_state(value, state);
        }
        let rhw = l.rhw.get();
        let light = FRAME_WORK.view_light.bits() | 0xFF00_0000;
        let [u0, v0, u1, v1] = l.uv.each_ref().map(|d| d.get());
        let uv = [(u0, v0), (u1, v0), (u1, v1), (u0, v1)];
        for (n, (u, v)) in uv.into_iter().enumerate() {
            let at = &VERTICES.vertices[n];
            at.rhw.set_bits(rhw);
            at.colour.set(light);
            at.specular.set(0u32);
            at.u.set_bits(u);
            at.v.set_bits(v);
        }
        DRAW.polygon_vertex_count.set_bits(4u32);
        let inverse = x87::reciprocal(DRAW.farthest.bits());
        for r in &s.inverses {
            r.set_bits(inverse);
        }
        s.inverse.set_bits(inverse);
        let settings: u32 = crate::marks::setup::SETTINGS.pointer.get();
        if get(settings + 0x10) != 0 {
            let ticks = PHYSICS.scene_clock.get().wrapping_sub(s.last_clock.get());
            let fall =
                x87::int_times_plus_int_to_int(ticks as i32, l.fall_k.bits(), s.fall.get() as i32);
            s.fall.set(fall as u32);
        }
        s.fall.set(s.fall.get() & 0x0FFF_FFFF);
        s.last_clock.set(PHYSICS.scene_clock.get());
        for (n, v) in s.matrix_copy.iter().enumerate() {
            for m in 0..4 {
                s.wide_turn[n][m].set_bits(v.bits());
            }
        }
        let focal: u32 = DRAW.view_scale.bits();
        for m in 0..4 {
            s.wide_focal[m].set_bits(focal);
        }
        crate::marks::draw_snow::draw_snow()?;
        crate::scene::depth_test_on::depth_test_on();
    }
    Ok(())
}

use crate::marks::plain::ld;
use crate::math::x87_float as bh;

/// `turn_light`'s x87 block as plain Rust on the x87 model (126 instructions), same order and memory places.
unsafe fn turn_light_x87(rc: usize, l: &SnowLight) {
    unsafe {
        let t0 = bh::ld(l.light_source[0].bits());
        let t1 = bh::ld(l.light_source[1].bits());
        let t2 = bh::ld(l.light_source[2].bits());
        let t3 = ld(rc, 0x1EA3CC);
        let t4 = t3 * t2;
        let t5 = ld(rc, 0x1EA3DC);
        let t6 = t5 * t1;
        let t7 = ld(rc, 0x1EA3EC);
        let t8 = t7 * t0;
        let t9 = t6 + t8;
        let t10 = t4 + t9;
        let t11 = t10 + ld(rc, 0x1EA3FC);
        let t12 = ld(rc, 0x1EA3D0);
        let t13 = t12 * t2;
        let t14 = ld(rc, 0x1EA3E0);
        let t15 = t14 * t1;
        let t16 = ld(rc, 0x1EA3F0);
        let t17 = t16 * t0;
        let t18 = t15 + t17;
        let t19 = t13 + t18;
        let t20 = t19 + ld(rc, 0x1EA400);
        let t21 = ld(rc, 0x1EA3D4);
        let t22 = t2 * t21;
        let t23 = ld(rc, 0x1EA3E4);
        let t24 = t1 * t23;
        let t25 = ld(rc, 0x1EA3F4);
        let t26 = t0 * t25;
        let t27 = t26 + t24;
        let t28 = t22 + t27;
        let t29 = t28 + ld(rc, 0x1EA404);
        let t30 = ld(rc, 0x1EA2CC);
        let t31 = t30 * t11;
        let t32 = ld(rc, 0x1EA2DC);
        let t33 = t32 * t20;
        let t34 = ld(rc, 0x1EA2EC);
        let t35 = t34 * t29;
        let t36 = t33 + t35;
        let t37 = t31 + t36;
        let t38 = t37 + ld(rc, 0x1EA2FC);
        let t39 = ld(rc, 0x1EA2D0);
        let t40 = t39 * t11;
        let t41 = ld(rc, 0x1EA2E0);
        let t42 = t41 * t20;
        let t43 = ld(rc, 0x1EA2F0);
        let t44 = t43 * t29;
        let t45 = t42 + t44;
        let t46 = t40 + t45;
        let t47 = t46 + ld(rc, 0x1EA300);
        let t48 = ld(rc, 0x1EA2D4);
        let t49 = t11 * t48;
        let t50 = ld(rc, 0x1EA2E4);
        let t51 = t20 * t50;
        let t52 = ld(rc, 0x1EA2F4);
        let t53 = t29 * t52;
        let t54 = t53 + t51;
        let t55 = t49 + t54;
        let t56 = t55 + ld(rc, 0x1EA304);
        l.light_place[0].set_bits(bh::st(t38));
        l.light_place[1].set_bits(bh::st(t47));
        l.light_place[2].set_bits(bh::st(t56));
        let t57 = bh::ld(l.light_source_way[0].bits());
        let t58 = bh::ld(l.light_source_way[1].bits());
        let t59 = bh::ld(l.light_source_way[2].bits());
        let t60 = ld(rc, 0x1EA3CC);
        let t61 = t60 * t59;
        let t62 = ld(rc, 0x1EA3DC);
        let t63 = t62 * t58;
        let t64 = ld(rc, 0x1EA3EC);
        let t65 = t64 * t57;
        let t66 = t63 + t65;
        let t67 = t61 + t66;
        let t68 = ld(rc, 0x1EA3D0);
        let t69 = t68 * t59;
        let t70 = ld(rc, 0x1EA3E0);
        let t71 = t70 * t58;
        let t72 = ld(rc, 0x1EA3F0);
        let t73 = t72 * t57;
        let t74 = t71 + t73;
        let t75 = t69 + t74;
        let t76 = ld(rc, 0x1EA3D4);
        let t77 = t59 * t76;
        let t78 = ld(rc, 0x1EA3E4);
        let t79 = t58 * t78;
        let t80 = ld(rc, 0x1EA3F4);
        let t81 = t57 * t80;
        let t82 = t81 + t79;
        let t83 = t77 + t82;
        let t84 = ld(rc, 0x1EA2CC);
        let t85 = t84 * t67;
        let t86 = ld(rc, 0x1EA2DC);
        let t87 = t86 * t75;
        let t88 = ld(rc, 0x1EA2EC);
        let t89 = t88 * t83;
        let t90 = t87 + t89;
        let t91 = t85 + t90;
        let t92 = ld(rc, 0x1EA2D0);
        let t93 = t92 * t67;
        let t94 = ld(rc, 0x1EA2E0);
        let t95 = t94 * t75;
        let t96 = ld(rc, 0x1EA2F0);
        let t97 = t96 * t83;
        let t98 = t95 + t97;
        let t99 = t93 + t98;
        let t100 = ld(rc, 0x1EA2D4);
        let t101 = t67 * t100;
        let t102 = ld(rc, 0x1EA2E4);
        let t103 = t75 * t102;
        let t104 = ld(rc, 0x1EA2F4);
        let t105 = t83 * t104;
        let t106 = t105 + t103;
        let t107 = t101 + t106;
        l.light_way[0].set_bits(bh::st(t91));
        l.light_way[1].set_bits(bh::st(t99));
        l.light_way[2].set_bits(bh::st(t107));
    }
}

#[cfg(test)]
mod asm_reference {
    //! The original assembly blocks, kept verbatim as the reference the plain versions are tested against.

    pub(super) unsafe fn turn_light_x87(rc: usize, mk: usize) {
        unsafe {
            core::arch::asm!(
                    "fld dword ptr [{mk} + 0x1765CC]",
            "fld dword ptr [{mk} + 0x1765D0]",
            "fld dword ptr [{mk} + 0x1765D4]",
            "fld dword ptr [{rc} + 0x1EA3CC]",
            ".byte 0xD8, 0xC9",
            "fld dword ptr [{rc} + 0x1EA3DC]",
            ".byte 0xD8, 0xCB",
            "fld dword ptr [{rc} + 0x1EA3EC]",
            ".byte 0xD8, 0xCD",
            ".byte 0xDE, 0xC1",
            ".byte 0xDE, 0xC1",
            "fadd dword ptr [{rc} + 0x1EA3FC]",
            ".byte 0xD9, 0xCB",
            "fld dword ptr [{rc} + 0x1EA3D0]",
            ".byte 0xD8, 0xCA",
            "fld dword ptr [{rc} + 0x1EA3E0]",
            ".byte 0xD8, 0xCC",
            "fld dword ptr [{rc} + 0x1EA3F0]",
            ".byte 0xD8, 0xCB",
            ".byte 0xDE, 0xC1",
            ".byte 0xDE, 0xC1",
            "fadd dword ptr [{rc} + 0x1EA400]",
            ".byte 0xD9, 0xCB",
            "fld dword ptr [{rc} + 0x1EA3D4]",
            ".byte 0xDE, 0xCB",
            "fld dword ptr [{rc} + 0x1EA3E4]",
            ".byte 0xDE, 0xC9",
            "fld dword ptr [{rc} + 0x1EA3F4]",
            ".byte 0xDE, 0xCA",
            ".byte 0xDE, 0xC1",
            ".byte 0xDE, 0xC1",
            "fadd dword ptr [{rc} + 0x1EA404]",
            ".byte 0xD9, 0xCA",
            "fld dword ptr [{rc} + 0x1EA2CC]",
            ".byte 0xD8, 0xC9",
            "fld dword ptr [{rc} + 0x1EA2DC]",
            ".byte 0xD8, 0xCB",
            "fld dword ptr [{rc} + 0x1EA2EC]",
            ".byte 0xD8, 0xCD",
            ".byte 0xDE, 0xC1",
            ".byte 0xDE, 0xC1",
            "fadd dword ptr [{rc} + 0x1EA2FC]",
            ".byte 0xD9, 0xCB",
            "fld dword ptr [{rc} + 0x1EA2D0]",
            ".byte 0xD8, 0xCA",
            "fld dword ptr [{rc} + 0x1EA2E0]",
            ".byte 0xD8, 0xCC",
            "fld dword ptr [{rc} + 0x1EA2F0]",
            ".byte 0xD8, 0xCB",
            ".byte 0xDE, 0xC1",
            ".byte 0xDE, 0xC1",
            "fadd dword ptr [{rc} + 0x1EA300]",
            ".byte 0xD9, 0xCB",
            "fld dword ptr [{rc} + 0x1EA2D4]",
            ".byte 0xDE, 0xCB",
            "fld dword ptr [{rc} + 0x1EA2E4]",
            ".byte 0xDE, 0xC9",
            "fld dword ptr [{rc} + 0x1EA2F4]",
            ".byte 0xDE, 0xCA",
            ".byte 0xDE, 0xC1",
            ".byte 0xDE, 0xC1",
            "fadd dword ptr [{rc} + 0x1EA304]",
            ".byte 0xD9, 0xCA",
            "fstp dword ptr [{mk} + 0x1764F0]",
            "fstp dword ptr [{mk} + 0x1764F4]",
            "fstp dword ptr [{mk} + 0x1764F8]",
            "fld dword ptr [{mk} + 0x1765D8]",
            "fld dword ptr [{mk} + 0x1765DC]",
            "fld dword ptr [{mk} + 0x1765E0]",
            "fld dword ptr [{rc} + 0x1EA3CC]",
            ".byte 0xD8, 0xC9",
            "fld dword ptr [{rc} + 0x1EA3DC]",
            ".byte 0xD8, 0xCB",
            "fld dword ptr [{rc} + 0x1EA3EC]",
            ".byte 0xD8, 0xCD",
            ".byte 0xDE, 0xC1",
            ".byte 0xDE, 0xC1",
            ".byte 0xD9, 0xCB",
            "fld dword ptr [{rc} + 0x1EA3D0]",
            ".byte 0xD8, 0xCA",
            "fld dword ptr [{rc} + 0x1EA3E0]",
            ".byte 0xD8, 0xCC",
            "fld dword ptr [{rc} + 0x1EA3F0]",
            ".byte 0xD8, 0xCB",
            ".byte 0xDE, 0xC1",
            ".byte 0xDE, 0xC1",
            ".byte 0xD9, 0xCB",
            "fld dword ptr [{rc} + 0x1EA3D4]",
            ".byte 0xDE, 0xCB",
            "fld dword ptr [{rc} + 0x1EA3E4]",
            ".byte 0xDE, 0xC9",
            "fld dword ptr [{rc} + 0x1EA3F4]",
            ".byte 0xDE, 0xCA",
            ".byte 0xDE, 0xC1",
            ".byte 0xDE, 0xC1",
            ".byte 0xD9, 0xCA",
            "fld dword ptr [{rc} + 0x1EA2CC]",
            ".byte 0xD8, 0xC9",
            "fld dword ptr [{rc} + 0x1EA2DC]",
            ".byte 0xD8, 0xCB",
            "fld dword ptr [{rc} + 0x1EA2EC]",
            ".byte 0xD8, 0xCD",
            ".byte 0xDE, 0xC1",
            ".byte 0xDE, 0xC1",
            ".byte 0xD9, 0xCB",
            "fld dword ptr [{rc} + 0x1EA2D0]",
            ".byte 0xD8, 0xCA",
            "fld dword ptr [{rc} + 0x1EA2E0]",
            ".byte 0xD8, 0xCC",
            "fld dword ptr [{rc} + 0x1EA2F0]",
            ".byte 0xD8, 0xCB",
            ".byte 0xDE, 0xC1",
            ".byte 0xDE, 0xC1",
            ".byte 0xD9, 0xCB",
            "fld dword ptr [{rc} + 0x1EA2D4]",
            ".byte 0xDE, 0xCB",
            "fld dword ptr [{rc} + 0x1EA2E4]",
            ".byte 0xDE, 0xC9",
            "fld dword ptr [{rc} + 0x1EA2F4]",
            ".byte 0xDE, 0xCA",
            ".byte 0xDE, 0xC1",
            ".byte 0xDE, 0xC1",
            ".byte 0xD9, 0xCA",
            "fstp dword ptr [{mk} + 0x1764FC]",
            "fstp dword ptr [{mk} + 0x176500]",
            "fstp dword ptr [{mk} + 0x176504]",
            rc = in(reg) rc,
            mk = in(reg) mk,
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
            )
        }
    }
}

#[cfg(test)]
mod asm_differential {
    use super::asm_reference as reference;
    use crate::marks::asm_test::Kind::*;
    use crate::marks::asm_test::with_copy;
    use crate::marks::asm_test::{Op, Spec, differential};
    use crate::marks::snow::SnowLight;

    #[test]
    fn turn_light_x87_matches() {
        const SPEC: Spec = Spec {
            operands: &[
                Op::Base {
                    lo: 0x1EA2CC,
                    hi: 0x1EA408,
                    reads: &[
                        (0x1EA2CC, F32),
                        (0x1EA2D0, F32),
                        (0x1EA2D4, F32),
                        (0x1EA2DC, F32),
                        (0x1EA2E0, F32),
                        (0x1EA2E4, F32),
                        (0x1EA2EC, F32),
                        (0x1EA2F0, F32),
                        (0x1EA2F4, F32),
                        (0x1EA2FC, F32),
                        (0x1EA300, F32),
                        (0x1EA304, F32),
                        (0x1EA3CC, F32),
                        (0x1EA3D0, F32),
                        (0x1EA3D4, F32),
                        (0x1EA3DC, F32),
                        (0x1EA3E0, F32),
                        (0x1EA3E4, F32),
                        (0x1EA3EC, F32),
                        (0x1EA3F0, F32),
                        (0x1EA3F4, F32),
                        (0x1EA3FC, F32),
                        (0x1EA400, F32),
                        (0x1EA404, F32),
                    ],
                    writes: &[],
                },
                Op::Base {
                    lo: 0x1764F0,
                    hi: 0x176618,
                    reads: &[
                        (0x1765CC, F32),
                        (0x1765D0, F32),
                        (0x1765D4, F32),
                        (0x1765D8, F32),
                        (0x1765DC, F32),
                        (0x1765E0, F32),
                    ],
                    writes: &[
                        (0x1764F0, F32),
                        (0x1764F4, F32),
                        (0x1764F8, F32),
                        (0x1764FC, F32),
                        (0x176500, F32),
                        (0x176504, F32),
                    ],
                },
            ],
            sincos: false,
        };
        differential(
            &SPEC,
            |b| unsafe { reference::turn_light_x87(b[0], b[1]) },
            |b| {
                with_copy(b[1], SnowLight::START, |l| unsafe {
                    super::turn_light_x87(b[0], l)
                })
            },
        );
    }
}
