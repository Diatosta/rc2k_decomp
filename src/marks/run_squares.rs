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

use crate::framework::rva;
use crate::marks::expire_squares::{ITEM_LEN, ITEMS, ITEMS_RVA};
use crate::marks::setup::CLOCK;
use crate::marks::squares_spawn::squares_spawn;
use crate::marks::store::{Slot, game_addr, game_read, game_write};
use crate::math::x87;
use crate::race::frame_store::FRAME_WORK;
use crate::scene::matrix3::extended;

const VERTICES_RVA: usize = rva(0x9605A0); // g_polygonVertices
const PLACES_RVA: Slot = Slot::at(0x8BC222); // dword_8BC222 .. dword_8BC22E
const RUNNING_RVA: Slot = Slot::at(0x7652A0); // dword_7652A0
const EDGE_RVA: Slot = Slot::at(0x809BC3); // g_marksEdgeMask
const FADE_RVA: Slot = Slot::at(0x8BC232); // dword_8BC232
const FAR_RVA: Slot = Slot::at(0x8BC236); // flt_8BC236
const LIFE: i32 = 0xFA0;

/// `fld k; fsub z`, kept extended.
fn less(k: u32, z: u32) -> [u8; 10] {
    let (v, mut out) = ([k, z], [0u8; 10]);
    unsafe { less_x87(v.as_ptr() as usize, out.as_mut_ptr() as usize) };
    out
}

/// The Rust version (see the header).
pub unsafe fn run_squares() {
    let old = x87::set_chop();
    unsafe { run() };
    x87::set_control(old);
}

unsafe fn run() {
    unsafe {
        crate::marks::squares_setup::squares_setup();
        let [a, b, c, d]: [u32; 4] = game_read(PLACES_RVA);
        for (n, (u, v)) in [(a, b), (c, b), (c, d), (a, d)].into_iter().enumerate() {
            game_write(VERTICES_RVA + 0x20 * n + 0x18, u);
            game_write(VERTICES_RVA + 0x20 * n + 0x1C, v);
        }
        if game_read::<u32>(RUNNING_RVA) != 0 {
            squares_spawn();
        }
        let base = game_addr(ITEMS_RVA) as u32;
        for n in 0..ITEMS {
            let e = base + n * ITEM_LEN;
            let get = |o: u32| ((e + o) as *const u32).read_unaligned();
            if get(0) != 1 {
                continue;
            }
            if game_read::<u32>(RUNNING_RVA) != 0 {
                let age = get(8).wrapping_add(CLOCK.frame_ticks.get());
                ((e + 8) as *mut u32).write_unaligned(age);
                if age as i32 >= LIFE {
                    (e as *mut u32).write_unaligned(0);
                    continue;
                }
            }
            let x = x87::to_int(get(0x20)) as u32;
            let z = x87::to_int(get(0x24)) as u32;
            let column = (x >> 3) & 0xFF;
            let cell = (((z >> 3) & 0x7F) << 8) + column;
            if game_read::<u8>(EDGE_RVA + cell as usize) != 0 {
                (e as *mut u32).write_unaligned(0);
                continue;
            }
            let alpha = x87::int_over_int_to_int(get(8) as i32, game_read(FADE_RVA)) as u32;
            let colour = ((!alpha) << 24) | (FRAME_WORK.view_light.bits() & 0xFF_FFFF);
            let values = [less(game_read(FAR_RVA), get(0x24)), extended(get(0x20))];
            crate::marks::draw_square::draw_square(e, colour, values);
        }
        crate::scene::depth_buffer_on::depth_buffer(true);
    }
}

use crate::marks::plain::{ld, st80};

/// `less`'s x87 block as plain Rust on the x87 model (3 instructions), same order and memory places.
unsafe fn less_x87(v: usize, o: usize) {
    unsafe {
        let t0 = ld(v, 0);
        let t1 = t0 - ld(v, 0x4);
        st80(o, 0, t1);
    }
}

#[cfg(test)]
mod asm_reference {
    //! The original assembly blocks, kept verbatim as the reference the plain versions are tested against.

    pub(super) unsafe fn less_x87(v: usize, o: usize) {
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

#[cfg(test)]
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
            |b| unsafe { reference::less_x87(b[0], b[1]) },
            |b| unsafe { super::less_x87(b[0], b[1]) },
        );
    }
}
