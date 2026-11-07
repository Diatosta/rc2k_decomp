//! `effects::run_windscreen_flakes` (0x4BFFEC): the windscreen flakes' frame, with the x87 rounding set to chop
//! throughout (put back at the end): `effects::windscreen_flakes_setup`; the vertices' texture places
//! from `dword_8BC222` ..; with `dword_7652A0` `effects::windscreen_flakes_spawn`; then each live flake
//! item: with `dword_7652A0` its age (+8) plus `g_marksFrameTicks`, ended at 0xFA0 or more;
//! ended when its cell's byte in `g_marksEdgeMask` is set (the cell from its place, chopped,
//! shifted right 3); else drawn by `effects::draw_windscreen_flake` (st(1) its x, st(0) `flt_8BC236`
//! less its z; colour `g_viewLight`'s low 24 bits under the alpha: not (age over
//! `dword_8BC232`), shifted left 24). Last `scene::depth_buffer_on`.
//!
//! Caller: `sub_4BF60F` (0x4BF7E9). The spawning is its Rust version.

use crate::effects::mask_store::Masks;
use crate::effects::windscreen_drop_vars::WindscreenDrops;
use crate::effects::windscreen_flakes_spawn::windscreen_flakes_spawn;
use crate::effects::windscreen_flakes_store::WindscreenFlakes;
use crate::race::owner_frame::Frame;

use crate::errors::Refused;
use crate::math::x87;
use crate::scene::matrix3::extended;
use crate::session::read_bits;

use crate::d3d::vertices_store::VERTICES;
const LIFE: i32 = 0xFA0;

/// `fld k; fsub z`, kept extended.
fn less(k: u32, z: u32) -> [u8; 10] {
    less_x87(k, z)
}

/// The Rust version (see the header).
pub fn run_windscreen_flakes(
    frame: &Frame,
    masks: &Masks,
    drops: &WindscreenDrops,
    fl: &WindscreenFlakes,
) -> Result<(), Refused> {
    let old = x87::set_chop();
    run(frame, masks, drops, fl)?;
    x87::set_control(old);
    Ok(())
}

fn run(
    frame: &Frame,
    masks: &Masks,
    drops: &WindscreenDrops,
    fl: &WindscreenFlakes,
) -> Result<(), Refused> {
    let effects = crate::effects::owner::effects();
    let scene = crate::scene::owner::scene();
    let shared = &crate::race::owner::race().shared;
    let setup = &effects.setup;
    {
        crate::effects::windscreen_flakes_setup::windscreen_flakes_setup(fl, scene);
        let [a, b, c, d]: [u32; 4] = read_bits(&fl.state.run_places);
        for (n, (u, v)) in [(a, b), (c, b), (c, d), (a, d)].into_iter().enumerate() {
            VERTICES.vertices[n].u.set_bits(u);
            VERTICES.vertices[n].v.set_bits(v);
        }
        if setup.kind_floats.running.bits() != 0 {
            windscreen_flakes_spawn(shared, drops, fl, effects)?;
        }
        for item in &fl.items.items {
            if item.live.get() != 1 {
                continue;
            }
            if setup.kind_floats.running.bits() != 0 {
                let age = item.age.get().wrapping_add(setup.clock.frame_ticks.get());
                item.age.set(age);
                if age as i32 >= LIFE {
                    item.live.set(0);
                    continue;
                }
            }
            let x = x87::to_int(item.place[0].bits()) as u32;
            let z = x87::to_int(item.place[1].bits()) as u32;
            let column = (x >> 3) & 0xFF;
            let cell = (((z >> 3) & 0x7F) << 8) + column;
            if masks.edge_mask.mask.get(cell as usize) != 0 {
                item.live.set(0);
                continue;
            }
            let alpha =
                x87::int_over_int_to_int(item.age.get() as i32, fl.state.fade.get() as i32) as u32;
            let colour = ((!alpha) << 24) | (frame.frame_work.view_light.bits() & 0xFF_FFFF);
            let values = [
                less(fl.state.far.bits(), item.place[1].bits()),
                extended(item.place[0].bits()),
            ];
            crate::effects::draw_windscreen_flake::draw_windscreen_flake(
                fl, item, colour, values, effects, scene,
            );
        }
        crate::scene::depth_buffer_on::depth_buffer(true);
    }
    Ok(())
}

use crate::math::x87_float as bh;

/// `less`'s x87 block as plain Rust on the x87 model (3 instructions), same order: the two
/// floats `v[0]` and `v[1]`, the difference as ten bytes.
fn less_x87(k: u32, z: u32) -> [u8; 10] {
    let t0 = bh::ld(k);
    let t1 = t0 - bh::ld(z);
    bh::f80(t1)
}

#[cfg(all(test, target_arch = "x86"))]
mod asm_reference {
    //! The original assembly blocks, kept verbatim as the reference the plain versions are tested against.

    pub(super) fn less_x87(v: usize, o: usize) {
        unsafe {
            core::arch::asm!(
                    "fld dword ptr [{v}]",
            "fsub dword ptr [{v} + 4]",
            "fstp tbyte ptr [{o}]",
            v = in(reg) v,
            o = in(reg) o,
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
        differential(
            &SPEC,
            |b| reference::less_x87(b[0], b[1]),
            |b| {
                let word = |off: usize| unsafe {
                    crate::pointers::host_at(b[0] + off)
                        .cast::<u32>()
                        .read_unaligned()
                };
                let out = super::less_x87(word(0), word(4));
                for (i, byte) in out.into_iter().enumerate() {
                    unsafe { crate::pointers::host_at(b[1] + i).write(byte) };
                }
            },
        );
    }
}
