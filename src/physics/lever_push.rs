//! `physics::lever_push` (0x43DEF4): the work car's spin (`dword_71A15C` .., `g_car`
//! +0x184) copied to `dword_614B64` .. and turned into the car's frame
//! (`physics::turn_by_inverse`); crossed with the lever `flt_614B44` .. (`scene::cross_product`
//! into `flt_614B54` ..), turned back (`physics::turn_by_car_matrix`), scaled by the share
//! `flt_614C48` and by `flt_614768` (`physics::scale_vector`, each factor then stored in
//! `dword_62400C`); its x and z times 65536 (`flt_5C5D98`) and the step (`flt_61388C`),
//! `fistp` into `dword_6140E4` / `dword_6140E8` and added to the car's x / z
//! (`dword_71A10C` / `dword_71A114`). A reading: the car moved by a point's speed about
//! its spin.
//!
//! In:  nothing
//! Out: eax the z push; ebx 0x614B44, ecx 0x614B64, edx and esi 0x614B54; the rest kept;
//!      no flags read
//!
//! Caller: `physics::lever_pushes` (twice).

use crate::framework::{game_addr, game_read, game_write, rva};
use crate::math::x87_run::mul2_to_int;
use crate::physics::scale_vector::scale_vector;
use crate::physics::turn_by_car_matrix::turn_by_car_matrix;
use crate::physics::turn_by_inverse::turn_by_inverse;
use crate::scene::matrix3::{extended, rounded};
use crate::scene::view_matrix::cross_product;

fn a(x: usize) -> u32 {
    game_addr(rva(x)) as u32
}

fn g(x: usize) -> u32 {
    unsafe { game_read(rva(x)) }
}

fn p(x: usize, v: u32) {
    unsafe { game_write(rva(x), v) }
}

/// The Rust version.
pub unsafe fn lever_push() {
    unsafe {
        for k in 0..3 {
            p(0x614B64 + 4 * k, g(0x71A15C + 4 * k));
        }
        turn_by_inverse(a(0x614B64));
        cross_product(
            a(0x614B44) as usize,
            a(0x614B64) as usize,
            a(0x614B54) as usize,
        );
        turn_by_car_matrix(a(0x614B54));
        for factor in [0x614C48, 0x614768] {
            let f = extended(g(factor));
            scale_vector(a(0x614B54), f);
            p(0x62400C, rounded(f));
        }
        for (from, int, place) in [
            (0x614B54, 0x6140E4, 0x71A10C),
            (0x614B5C, 0x6140E8, 0x71A114),
        ] {
            let v = mul2_to_int(g(from), g(0x5C5D98), g(0x61388C)) as u32;
            p(int, v);
            let eax = g(int);
            p(place, g(place).wrapping_add(eax));
        }
    }
}
