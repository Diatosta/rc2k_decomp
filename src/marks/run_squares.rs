//! `marks::run_squares` (0x4BFFEC): the squares' frame, with the x87 rounding set to chop
//! throughout (put back at the end): `marks::squares_setup`; the vertices' texture places
//! from `dword_8BC222` ..; with `dword_7652A0` `marks::squares_spawn`; then each live square
//! item: with `dword_7652A0` its age (+8) plus `g_marksFrameTicks`, ended at 0xFA0 or more;
//! ended when its cell's byte in `g_marksEdgeMask` is set (the cell from its place, chopped,
//! shifted right 3); else drawn by `marks::draw_square` (st(1) its x, st(0) `flt_8BC236`
//! less its z; colour `g_viewLight`'s low 24 bits under the alpha: not (age over
//! `dword_8BC232`), shifted left 24). Last `scene::depth_buffer_on`.
//!
//! In:  nothing
//! Out: nothing (its caller reads no register after it)
//!
//! Caller: `sub_4BF60F` (0x4BF7E9). The spawning is its Rust version.

use crate::errors::Refused;
use crate::marks::edges::EDGE_MASK;
use crate::marks::setup::CLOCK;
use crate::marks::squares_spawn::squares_spawn;
use crate::marks::squares_store::{SQUARE_ITEMS, SQUARE_STATE};
use crate::marks::tuning::MARKS_KIND_FLOATS;
use crate::math::x87;
use crate::race::frame_store::FRAME_WORK;
use crate::scene::matrix3::extended;
use crate::session::{address_of, read_bits};

use crate::d3d::vertices_store::VERTICES;
const LIFE: i32 = 0xFA0;

/// `fld k; fsub z`, kept extended.
fn less(k: u32, z: u32) -> [u8; 10] {
    let (v, mut out) = ([k, z], [0u8; 10]);
    less_x87(
        (crate::pointers::game_address(v.as_ptr())) as usize,
        (crate::pointers::game_address(out.as_mut_ptr())) as usize,
    );
    out
}

/// The Rust version (see the header).
pub fn run_squares() -> Result<(), Refused> {
    let old = x87::set_chop();
    run()?;
    x87::set_control(old);
    Ok(())
}

fn run() -> Result<(), Refused> {
    {
        crate::marks::squares_setup::squares_setup();
        let [a, b, c, d]: [u32; 4] = read_bits(&SQUARE_STATE.run_places);
        for (n, (u, v)) in [(a, b), (c, b), (c, d), (a, d)].into_iter().enumerate() {
            VERTICES.vertices[n].u.set_bits(u);
            VERTICES.vertices[n].v.set_bits(v);
        }
        if MARKS_KIND_FLOATS.running.bits() != 0 {
            squares_spawn()?;
        }
        for item in &SQUARE_ITEMS.items {
            if item.live.get() != 1 {
                continue;
            }
            if MARKS_KIND_FLOATS.running.bits() != 0 {
                let age = item.age.get().wrapping_add(CLOCK.frame_ticks.get());
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
            if EDGE_MASK.mask.get(cell as usize) != 0 {
                item.live.set(0);
                continue;
            }
            let alpha =
                x87::int_over_int_to_int(item.age.get() as i32, SQUARE_STATE.fade.get() as i32)
                    as u32;
            let colour = ((!alpha) << 24) | (FRAME_WORK.view_light.bits() & 0xFF_FFFF);
            let values = [
                less(SQUARE_STATE.far.bits(), item.place[1].bits()),
                extended(item.place[0].bits()),
            ];
            crate::marks::draw_square::draw_square(address_of(item), colour, values);
        }
        crate::scene::depth_buffer_on::depth_buffer(true);
    }
    Ok(())
}

use crate::marks::plain::{ld, st80};

/// `less`'s x87 block as plain Rust on the x87 model (3 instructions), same order and memory places.
fn less_x87(v: usize, o: usize) {
    let t0 = ld(v, 0);
    let t1 = t0 - ld(v, 0x4);
    st80(o, 0, t1);
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
            )
        }
    }
}

#[cfg(all(test, target_arch = "x86"))]
mod asm_differential {
    use super::asm_reference as reference;
    use crate::marks::asm_test::Kind::*;
    use crate::marks::asm_test::{Op, Spec, differential};

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
            |b| super::less_x87(b[0], b[1]),
        );
    }
}
