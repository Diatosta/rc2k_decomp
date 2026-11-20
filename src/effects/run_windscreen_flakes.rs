//! `effects::run_windscreen_flakes`: the windscreen flakes' frame, with the x87 rounding set to
//! chop throughout (put back at the end). After the set-up (and, with the drift switch, the
//! spawning), each live flake item ages by the frame's ticks (ended at 0xFA0 or more) and ends when
//! its cell's byte in the edge mask is set; else it is drawn by `effects::draw_windscreen_flake`,
//! coloured by the view light under an alpha that fades with age.

use crate::effects::mask_store::Masks;
use crate::effects::windscreen_drop_vars_store::WindscreenDrops;
use crate::effects::windscreen_flakes_spawn::windscreen_flakes_spawn;
use crate::effects::windscreen_flakes_store::WindscreenFlakes;

use crate::errors::Refused;
use crate::math::matrix3::extended;
use crate::math::x87;
use crate::memory::session::PlainFloat;

const LIFE: i32 = 0xFA0;

/// `fld k; fsub z`, kept extended.
fn less(k: f32, z: f32) -> [u8; 10] {
    less_x87(k, z)
}

#[allow(
    clippy::too_many_arguments,
    reason = "the effects' groups, the owners and the vertex buffer"
)]
pub(super) fn run_windscreen_flakes(
    buffer: &crate::d3d::vertices_store::PolygonVertices,
    masks: &Masks,
    drops: &WindscreenDrops,
    fl: &WindscreenFlakes,
    view_light: u32,
    effects: &crate::effects::owner::Effects,
    scene: &crate::scene::owner::Scene,
    shared: &crate::race::owner_shared::RaceShared,
) -> Result<(), Refused> {
    let old = x87::set_chop();
    run(buffer, masks, drops, fl, view_light, effects, scene, shared)?;
    x87::set_control(old);
    Ok(())
}

#[allow(
    clippy::too_many_arguments,
    reason = "the effects' groups, the owners and the vertex buffer"
)]
fn run(
    buffer: &crate::d3d::vertices_store::PolygonVertices,
    masks: &Masks,
    drops: &WindscreenDrops,
    fl: &WindscreenFlakes,
    view_light: u32,
    effects: &crate::effects::owner::Effects,
    scene: &crate::scene::owner::Scene,
    shared: &crate::race::owner_shared::RaceShared,
) -> Result<(), Refused> {
    let setup = &effects.setup;
    {
        crate::effects::windscreen_flakes_setup::windscreen_flakes_setup(buffer, fl, scene);
        let [a, b, c, d] = fl.state.run_places.each_ref().map(PlainFloat::get);
        for (n, (u, v)) in [(a, b), (c, b), (c, d), (a, d)].into_iter().enumerate() {
            buffer.vertices[n].u.set(u);
            buffer.vertices[n].v.set(v);
        }
        if setup.kind_floats.running.get() != 0 {
            windscreen_flakes_spawn(shared, drops, fl, effects)?;
        }
        for item in &fl.items.items {
            if item.live.get() != 1 {
                continue;
            }
            if setup.kind_floats.running.get() != 0 {
                let age = item.age.get().wrapping_add(setup.clock.frame_ticks.get());
                item.age.set(age);
                if age as i32 >= LIFE {
                    item.live.set(0);
                    continue;
                }
            }
            let x = x87::to_int(&item.place[0]) as u32;
            let z = x87::to_int(&item.place[1]) as u32;
            let column = (x >> 3) & 0xFF;
            let cell = (((z >> 3) & 0x7F) << 8) + column;
            if masks.edge_mask.mask.get(cell as usize) != 0 {
                item.live.set(0);
                continue;
            }
            let alpha =
                x87::int_over_int_to_int(item.age.get() as i32, fl.state.fade.get() as i32) as u32;
            let colour = ((!alpha) << 24) | (view_light & 0xFF_FFFF);
            let values = [
                less(fl.state.far.get(), item.place[1].get()),
                extended(item.place[0].get()),
            ];
            crate::effects::draw_windscreen_flake::draw_windscreen_flake(
                buffer, fl, item, colour, values, scene, effects,
            );
        }
        crate::scene::depth_buffer_on::depth_buffer(scene.scratch.game_scratch(), true);
    }
    Ok(())
}

use crate::math::x87_float as bh;

/// `less`'s x87 block as plain Rust on the x87 model (3 instructions), same order: the two
/// floats `v[0]` and `v[1]`, the difference as ten bytes.
fn less_x87(k: f32, z: f32) -> [u8; 10] {
    let t0 = bh::ld(k);
    let t1 = t0 - bh::ld(z);
    bh::f80(t1)
}

#[cfg(test)]
mod reference_vectors {
    use crate::effects::asm_test::Kind::*;
    use crate::effects::asm_test::{Op, Spec, recorded};

    #[test]
    fn less_x87_matches() {
        const SPEC: Spec = Spec {
            operands: &[
                Op::Base {
                    lo: 0x0,
                    hi: 0x8,
                    reads: &[(0x0, F32), (0x4, F32)],
                    writes: &[],
                },
                Op::Base {
                    lo: 0x0,
                    hi: 0xA,
                    reads: &[],
                    writes: &[(0x0, F80)],
                },
            ],
            sincos: false,
        };
        recorded("effects.run_windscreen_flakes.less_x87", &SPEC, |b| {
            // SAFETY: `at` checks that the four bytes lie in the operand's test buffer, which the
            // recorded run keeps alive and nothing else reaches meanwhile; unaligned, any bits a
            // `u32`.
            let word = |off: usize| unsafe { (b[0] + off).at(0, 4).cast::<u32>().read_unaligned() };
            let out = super::less_x87(f32::from_bits(word(0)), f32::from_bits(word(4)));
            for (i, byte) in out.into_iter().enumerate() {
                // SAFETY: `host` checks that the byte lies in the operand's test buffer, which the
                // recorded run keeps alive and nothing else reaches meanwhile.
                unsafe { (b[1] + i).host().write(byte) };
            }
        });
    }
}
