//! `physics::object_turn_matrix` (0x45E2D2): the turn of the object hit (`dword_6D321C`)
//! into `flt_5EA444`. `scene::set_turn_y` by minus `dword_63503C`, copied (16 dwords);
//! with `dword_635040` set also: the turn about y by minus `dword_635044` times it into
//! `g_litMatrix` (`scene::multiply_matrices`), the turn about x by minus `dword_635040`
//! times that into `g_turnedMatrix`, the turn about y by `dword_635044` times that into
//! `flt_5EA444`. Then each of its 16 floats times `dword_635038` / 16 (`fild`, `fld1`
//! doubled four times, `fdivp`; the share through `dword_62400C`).
//!
//! In:  nothing
//! Out: the registers as the steps leave them (eax, ebx `dword_635038`, ecx 0, edx, esi
//!      past the matrix, edi, ebp); no flags read (the caller returns)
//!
//! Caller: `sub_45E755`. Refuses (before anything) an object past 0xFF.
//!
//! Plain since 2026-10-23 (phase 3): its arguments are what it reads; the registers
//! above are the original's, for reference.

use crate::errors::{BadInput, Refused};
use crate::framework::{game_addr, game_read, game_write, rva};
use crate::math::x87;
use crate::math::x87_run::{ext_div2, int_80};
use crate::scene::matrix::{multiply_matrices, set_turn_x, set_turn_y};

fn g(x: usize) -> u32 {
    unsafe { game_read(rva(x)) }
}

fn p(x: usize, v: u32) {
    unsafe { game_write(rva(x), v) }
}

fn a(x: usize) -> u32 {
    game_addr(rva(x)) as u32
}

const OUT: usize = 0x5EA444; // flt_5EA444
const TURN_Y: usize = 0x5E9FCC; // g_turnYMatrix
const TURN_X: usize = 0x5E9F8C; // g_turnXMatrix
const LIT: usize = 0x5EA14C; // g_litMatrix
const TURNED: usize = 0x5EA10C; // g_turnedMatrix

/// The Rust version. The registers it leaves are read by none of its callers.
pub unsafe fn object_turn_matrix() -> Result<(), Refused> {
    let object = g(0x6D321C);
    if object > 0xFF {
        return Err(BadInput("an object past 0xFF").into());
    }
    unsafe {
        set_turn_y(g(0x63503C).wrapping_neg());
        for k in 0..16 {
            p(OUT + 4 * k, g(TURN_Y + 4 * k));
        }
        if g(0x635040) != 0 {
            set_turn_y(g(0x635044).wrapping_neg());
            multiply_matrices(a(OUT) as usize, a(TURN_Y) as usize, a(LIT) as usize);
            set_turn_x(g(0x635040).wrapping_neg());
            multiply_matrices(a(LIT) as usize, a(TURN_X) as usize, a(TURNED) as usize);
            set_turn_y(g(0x635044));
            multiply_matrices(a(TURNED) as usize, a(TURN_Y) as usize, a(OUT) as usize);
        }
        let figure = g(0x635038);
        p(0x62400C, figure);
        let sixteen = 16.0f32.to_bits();
        let share = ext_div2(int_80(figure as i32, false), sixteen, 0x3F80_0000);
        p(0x62400C, share);
        for k in 0..16 {
            p(OUT + 4 * k, x87::mul(g(OUT + 4 * k), share));
        }
    }
    Ok(())
}
