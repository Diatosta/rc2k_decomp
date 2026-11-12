//! `effects::run_rain` (0x4C16F9): the rain's frame. `drops` is the settings' +0x2764 (the
//! drops; none: `drawn` 0 and back), held to 0x100; the settings' +0x28 .. +0x48 into
//! `view_matrix`, +0x1C .. +0x24 into `view_place`; `effects::place_rain_view`,
//! `scene::depth_buffer_off`, render states 0x15 = 4, 0x13 = 5, 0x14 = 6, 0x1C = 0, 0x1B = 1,
//! `scene::clear_texture`; the four vertices' rhw (`rhw`; loaded as a whole number there, as
//! a float here: a fixed bug), colours 0xFFFFFF, 0, 0x28000000, 0x28FFFFFF, the rest 0, the
//! polygon's vertex count 4; `effects::drift_offsets` when the drift switch is set; the
//! view's entry (the settings' +8, 12 bytes in `moves` and `last_views`): when the settings'
//! +0xC or the drift switch is set its move since the last frame scaled; the move (`motion`)
//! held to `move_held` long when longer than `move_limit`; the view's place kept (y plus
//! `drift`); `drift` the clock's low 20 bits times `drift_k` when drifting. Then each drop k
//! from the last: its noise place (`noise`[k] plus the view and the drift, wrapped to 0x40000
//! and centred) as whole numbers to `effects::draw_rain_streak`, and when the settings' +0x10
//! is set its turned place kept (`kept`[k], rounded). `drawn` 1.
//!
//! Screen: the falling rain, seen from inside and outside the car (confirmed: gone
//! with `draw_rain_streak`'s draw cut).
//!
//! Works on the effects settings.
//!
//! Caller: `effects::run_frame` The x87 blocks are plain Rust on the x87 model (`math::x87_float`), in the original's order,
//! over the game's variables. Refuses null settings and a view entry past the five (the
//! settings' +8), before anything.

use crate::effects::cell_x87::CellX87;
use crate::effects::rain::{RainRules, RainState, Weather};
use crate::errors::{BadInput, Refused};
use crate::math::x87;
use crate::race::owner::Race;

const MOST_DROPS: i32 = 0x100;
const ENTRIES: u32 = 5;
/// The render states set (value, state).
const STATES: [(u32, u32); 5] = [(4, 0x15), (5, 0x13), (6, 0x14), (0, 0x1C), (1, 0x1B)];
const COLOURS: [u32; 4] = [0xFF_FFFF, 0, 0x2800_0000, 0x28FF_FFFF];

/// The view's move since the last frame, scaled, into its entry.
fn box_moved(weather: &Weather, i: usize) {
    box_moved_x87(i, &weather.rain, &weather.rain_rules);
}

/// The move's length into `depth`, compared with `move_limit` (the compare's status word
/// goes to the shared FPU status variable).
fn move_length(weather: &Weather) {
    move_length_x87(&weather.rain, &weather.rain_rules);
}

/// The move scaled to `move_held` over its length, times its three factors.
fn move_held(weather: &Weather) {
    move_held_x87(&weather.rain, &weather.rain_rules);
}

/// The view's y plus the drift (rounded) kept in its entry.
fn keep_y(weather: &Weather, i: usize) {
    keep_y_x87(i, &weather.rain);
}

pub fn run_rain(
    buffer: &crate::d3d::vertices_store::PolygonVertices,
    race: &Race,
    weather: &Weather,
    scene: &crate::scene::owner::Scene,
    effects: &crate::effects::owner::Effects,
) -> Result<(), Refused> {
    let setup = &effects.setup;
    use crate::scene::set_render_state::set_render_state;
    let r = &weather.rain;
    {
        let marks = effects.settings()?;
        if marks.marks_layout.get() >= ENTRIES {
            return Err(BadInput("a rain view entry past the five").into());
        }
        let mut drops = marks.rain_drops.get() as i32;
        r.drops.set(drops as u32);
        if drops == 0 {
            r.drawn.set(0);
            return Ok(());
        }
        if drops > MOST_DROPS {
            drops = MOST_DROPS;
            r.drops.set(drops as u32);
        }
        for n in 0..9 {
            r.view_matrix[n].set(marks.listener_turn[n].get());
        }
        for n in 0..3 {
            r.view_place[n].set(marks.listener_place[n].get());
        }
        crate::effects::place_rain_view::place_rain_view(weather)?;
        crate::scene::depth_buffer_on::depth_buffer(false);
        for (value, state) in STATES {
            set_render_state(value, state);
        }
        // What `clear_texture` returns is read by nothing here.
        crate::scene::clear_texture::clear_texture(scene);
        // Not the original's: it loaded 256.0's bits as a whole number (`fild`), an rhw of
        // 1132462080.0 that the handheld's GL clipped the streaks away by (`notes/bugs.md`).
        let rhw = weather.rain_rules.rhw;
        for (n, colour) in COLOURS.into_iter().enumerate() {
            let v = &buffer.vertices[n];
            v.rhw.set_bits(rhw);
            v.colour.set(colour);
            v.specular.set(0);
            v.u.set_bits(0);
            v.v.set_bits(0);
        }
        crate::scene::polygon_work::front_end()
            .polygon_vertex_count
            .set_bits(4u32);
        if setup.kind_floats.running.get() != 0 {
            crate::effects::drift_offsets::drift_offsets(race, weather, effects);
        }
        let marks = effects.settings()?;
        let e = marks.marks_layout.get() as usize;
        if marks.own_marks_shown.get() != 0 || setup.kind_floats.running.get() != 0 {
            box_moved(weather, e);
        }
        for n in 0..3 {
            r.motion[n].set(r.moves[e][n].get());
        }
        move_length(weather);
        let status = setup.work.fpu_status.get() & 0x700;
        setup.work.fpu_status.set(status);
        if status != 0x100 {
            move_held(weather);
        }
        r.last_views[e][0].set(r.view[0].get());
        keep_y(weather, e);
        r.last_views[e][2].set(r.view[2].get());
        if setup.kind_floats.running.get() != 0 {
            let clock = race.shared.physics_tables.scene_clock.get() & 0xF_FFFF;
            r.drift
                .set(x87::int_times_to_int(clock as i32, weather.rain_rules.drift_k) as u32);
        }
        for k in (0..drops.max(0) as usize).rev() {
            let drift = r.drift.get();
            let spread = r.drift_halves[0].get();
            let view = |n: usize| r.view[n].get();
            let noise = |n: usize| r.noise[n][k].get();
            let x = noise(0).wrapping_add(view(0)).wrapping_add(spread);
            let y = noise(1).wrapping_add(drift).wrapping_add(view(1));
            let z = noise(2).wrapping_add(view(2)).wrapping_add(spread);
            let [x, y, z] = [x, y, z].map(|v| (v & 0x3_FFFF).wrapping_sub(0x1_FFFF));
            let place = [
                x87::int_80(z as i32, false),
                x87::int_80(y as i32, false),
                x87::int_80(x as i32, false),
            ];
            crate::effects::draw_rain_streak::draw_rain_streak(
                buffer, weather, place, scene, effects,
            )?;
            let marks = effects.settings()?;
            if marks.other_marks_shown.get() != 0 {
                for (n, kept) in r.kept.iter().enumerate() {
                    kept[k].set(x87::to_int(&r.turned[n]) as u32);
                }
            }
        }
        r.drawn.set(1);
    }
    Ok(())
}

use crate::math::x87_float as bh;

/// `box_moved`'s x87 block as plain Rust on the x87 model (13 instructions), same order and memory places.
fn box_moved_x87(i: usize, r: &RainState, k: &RainRules) {
    {
        let t0 = bh::int(r.view[0].get() as i32);
        let t1 = t0 - bh::int(r.last_views[i][0].get() as i32);
        let t2 = t1 * bh::ld(k.move_scale[0]);
        r.moves[i][0].st(t2);
        let t3 = bh::int(r.view[1].get() as i32);
        let t4 = t3 + bh::int(r.drift.get() as i32);
        let t5 = t4 - bh::int(r.last_views[i][1].get() as i32);
        let t6 = t5 * bh::ld(k.move_scale[1]);
        r.moves[i][1].st(t6);
        let t7 = bh::int(r.view[2].get() as i32);
        let t8 = t7 - bh::int(r.last_views[i][2].get() as i32);
        let t9 = t8 * bh::ld(k.move_scale[2]);
        r.moves[i][2].st(t9);
    }
}

/// `move_length`'s x87 block as plain Rust on the x87 model (12 instructions), same order and memory places.
fn move_length_x87(r: &RainState, k: &RainRules) {
    let t0 = r.motion[0].ld();
    let t1 = t0 * t0;
    let t2 = r.motion[1].ld();
    let t3 = t2 * t2;
    let t4 = r.motion[2].ld();
    let t5 = t4 * t4;
    let t6 = t3 + t5;
    let t7 = t1 + t6;
    let t8 = t7.sqrt();
    r.depth.st(t8);
    crate::effects::plain::set_status(bh::compare(t8, bh::ld(k.move_limit)));
}

/// `move_held`'s x87 block as plain Rust on the x87 model (13 instructions), same order and memory places.
fn move_held_x87(r: &RainState, k: &RainRules) {
    {
        let t0 = r.depth.ld();
        let t1 = bh::ld(k.move_held) / t0;
        let t2 = t1 * r.motion[0].ld();
        let t3 = t2 * bh::ld(k.move_factors[0]);
        r.motion[0].st(t3);
        let t4 = t1 * r.motion[1].ld();
        let t5 = t4 * bh::ld(k.move_factors[1]);
        r.motion[1].st(t5);
        let t6 = t1 * r.motion[2].ld();
        let t7 = t6 * bh::ld(k.move_factors[2]);
        r.motion[2].st(t7);
    }
}

/// `keep_y`'s x87 block as plain Rust on the x87 model (3 instructions), same order and memory places.
fn keep_y_x87(i: usize, r: &RainState) {
    {
        let t0 = bh::int(r.view[1].get() as i32);
        let t1 = t0 + bh::int(r.drift.get() as i32);
        r.last_views[i][1].set(bh::to_int(t1) as u32);
    }
}

#[cfg(all(test, target_arch = "x86"))]
mod asm_reference {
    //! The original assembly blocks, kept verbatim as the reference the plain versions are tested against.

    pub(super) fn box_moved_x87(mk: usize, e: usize) {
        unsafe {
            core::arch::asm!(
                    "fild dword ptr [{mk} + 0x158E3D]",
            "fisub dword ptr [{mk} + {e} + 0x157585]",
            "fmul dword ptr [{mk} + 0x161055]",
            "fstp dword ptr [{mk} + {e} + 0x157549]",
            "fild dword ptr [{mk} + 0x158E41]",
            "fiadd dword ptr [{mk} + 0x1575C5]",
            "fisub dword ptr [{mk} + {e} + 0x157589]",
            "fmul dword ptr [{mk} + 0x161059]",
            "fstp dword ptr [{mk} + {e} + 0x15754D]",
            "fild dword ptr [{mk} + 0x158E45]",
            "fisub dword ptr [{mk} + {e} + 0x15758D]",
            "fmul dword ptr [{mk} + 0x16105D]",
            "fstp dword ptr [{mk} + {e} + 0x157551]",
            mk = in(reg) mk,
            e = in(reg) e,
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
            );
        }
    }

    pub(super) fn move_length_x87(rc: usize, mk: usize) {
        unsafe {
            core::arch::asm!(
                    "fld dword ptr [{mk} + 0x15753D]",
            ".byte 0xD8, 0xC8",
            "fld dword ptr [{mk} + 0x157541]",
            ".byte 0xD8, 0xC8",
            "fld dword ptr [{mk} + 0x157545]",
            ".byte 0xD8, 0xC8",
            ".byte 0xDE, 0xC1",
            ".byte 0xDE, 0xC1",
            ".byte 0xD9, 0xFA",
            "fst dword ptr [{mk} + 0x158E1D]",
            "fcomp dword ptr [{mk} + 0x161061]",
            "fstsw word ptr [{rc} + 0x31C814]",
            rc = in(reg) rc,
            mk = in(reg) mk,
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
            );
        }
    }

    pub(super) fn move_held_x87(mk: usize) {
        unsafe {
            core::arch::asm!(
                    "fld dword ptr [{mk} + 0x158E1D]",
            "fdivr dword ptr [{mk} + 0x161065]",
            ".byte 0xD9, 0xC0",
            ".byte 0xD9, 0xC0",
            "fmul dword ptr [{mk} + 0x15753D]",
            "fmul dword ptr [{mk} + 0x161069]",
            "fstp dword ptr [{mk} + 0x15753D]",
            "fmul dword ptr [{mk} + 0x157541]",
            "fmul dword ptr [{mk} + 0x16106D]",
            "fstp dword ptr [{mk} + 0x157541]",
            "fmul dword ptr [{mk} + 0x157545]",
            "fmul dword ptr [{mk} + 0x161071]",
            "fstp dword ptr [{mk} + 0x157545]",
            mk = in(reg) mk,
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
            );
        }
    }

    pub(super) fn keep_y_x87(mk: usize, e: usize) {
        unsafe {
            core::arch::asm!(
                    "fild dword ptr [{mk} + 0x158E41]",
            "fiadd dword ptr [{mk} + 0x1575C5]",
            "fistp dword ptr [{mk} + {e} + 0x157589]",
            mk = in(reg) mk,
            e = in(reg) e,
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
    use crate::effects::asm_test::{Op, Spec, differential};
    use crate::effects::rain::tests::with_state_at;

    #[test]
    fn box_moved_x87_matches() {
        const SPEC: Spec = Spec {
            operands: &[
                Op::Base {
                    lo: 0x1574C5,
                    hi: 0x1610BD,
                    reads: &[
                        (0x157585, I32),
                        (0x157589, I32),
                        (0x15758D, I32),
                        (0x157591, I32),
                        (0x157595, I32),
                        (0x157599, I32),
                        (0x15759D, I32),
                        (0x1575A1, I32),
                        (0x1575A5, I32),
                        (0x1575C1, I32),
                        (0x1575C5, I32),
                        (0x1575C9, I32),
                        (0x1575FD, I32),
                        (0x157601, I32),
                        (0x157605, I32),
                        (0x158E3D, I32),
                        (0x158E41, I32),
                        (0x158E45, I32),
                        (0x161055, F32),
                        (0x161059, F32),
                        (0x16105D, F32),
                    ],
                    writes: &[
                        (0x157549, F32),
                        (0x15754D, F32),
                        (0x157551, F32),
                        (0x157555, F32),
                        (0x157559, F32),
                        (0x15755D, F32),
                        (0x157561, F32),
                        (0x157565, F32),
                        (0x157569, F32),
                        (0x157585, F32),
                        (0x157589, F32),
                        (0x15758D, F32),
                        (0x1575C1, F32),
                        (0x1575C5, F32),
                        (0x1575C9, F32),
                    ],
                },
                Op::Index(&[0, 12, 24, 36, 48]),
            ],
            sincos: false,
        };
        differential(
            &SPEC,
            |b| reference::box_moved_x87(b[0], b[1]),
            |b| with_state_at(b[0], |r, k| super::box_moved_x87(b[1] / 12, r, k)),
        );
    }

    #[test]
    fn move_length_x87_matches() {
        const SPEC: Spec = Spec {
            operands: &[
                Op::RaceBlock {
                    reads: &[],
                    writes: &[(0x31C814, Status)],
                },
                Op::Base {
                    lo: 0x1574C5,
                    hi: 0x1610BD,
                    reads: &[
                        (0x15753D, F32),
                        (0x157541, F32),
                        (0x157545, F32),
                        (0x161061, F32),
                    ],
                    writes: &[(0x158E1D, F32)],
                },
            ],
            sincos: false,
        };
        differential(
            &SPEC,
            |b| reference::move_length_x87(b[0], b[1]),
            |b| with_state_at(b[1], super::move_length_x87),
        );
    }

    #[test]
    fn move_held_x87_matches() {
        const SPEC: Spec = Spec {
            operands: &[Op::Base {
                lo: 0x1574C5,
                hi: 0x1610BD,
                reads: &[
                    (0x15753D, F32),
                    (0x157541, F32),
                    (0x157545, F32),
                    (0x158E1D, F32),
                    (0x161065, F32),
                    (0x161069, F32),
                    (0x16106D, F32),
                    (0x161071, F32),
                ],
                writes: &[(0x15753D, F32), (0x157541, F32), (0x157545, F32)],
            }],
            sincos: false,
        };
        differential(
            &SPEC,
            |b| reference::move_held_x87(b[0]),
            |b| with_state_at(b[0], super::move_held_x87),
        );
    }

    #[test]
    fn keep_y_x87_matches() {
        const SPEC: Spec = Spec {
            operands: &[
                Op::Base {
                    lo: 0x1574C5,
                    hi: 0x1610BD,
                    reads: &[(0x1575C5, I32), (0x158E41, I32)],
                    writes: &[
                        (0x157589, I32),
                        (0x157595, I32),
                        (0x1575A1, I32),
                        (0x1575C5, I32),
                        (0x157601, I32),
                    ],
                },
                Op::Index(&[0, 12, 24, 36, 48]),
            ],
            sincos: false,
        };
        differential(
            &SPEC,
            |b| reference::keep_y_x87(b[0], b[1]),
            |b| with_state_at(b[0], |r, _| super::keep_y_x87(b[1] / 12, r)),
        );
    }
}
