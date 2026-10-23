//! `physics::move_slot` (0x462485): one step of a loose object's slot (esi, 0x78 bytes of
//! `g_raceSlots`; +4 .. +0xC its place, whole numbers; +0x10 its matrix; +0x50 .. +0x58
//! its speed; +0x68 .. +0x70 its spin; +0x60 its last face):
//! - the place kept (0x71BE90 ..); +0x74 = the spin's length; when not 0, turned by it:
//!   the axis (`scene::unit_vector` of +0x74 .. +0x7C, so two dwords past the slot, as
//!   the original has it) and angle (+0x74 times `flt_61388C`, the step) through
//!   `physics::axis_turn_matrix` (`flt_5C7010`), `scene::square_matrix`, then
//!   `scene::multiply_matrices` with the matrix into `g_turnedMatrix`, copied back;
//! - the place moved by the speed (times the step and 65536, `fistp`'d through
//!   `dword_62400C`); y's speed less gravity (`flt_613FC8`) times the step;
//! - the ground under it (`physics::find_ground`, mask 0x80, last face +0x60): none, the
//!   place put back and the speed and spin cleared; else the face kept, and when below
//!   the ground (the sign of the difference) set on it, the speed halved (y's turned
//!   round), the spin halved and turned round; then, with |y's speed| below 1, the
//!   matrix from `unk_71BA88` (12 dwords), or from `unk_71BA48` when +0 mod 20 is 5 .. 8;
//!   else `dword_62400C` = |y's speed| (and a call to an empty function).
//!
//! By the look of it debris thrown about in a crash (a reading).
//!
//! Refuses (before anything) what `find_ground` would refuse where the slot moves to.
//!
//! In:  esi the slot
//! Out: the registers as the path leaves them (see `move_slot_at`); no flags read
//!
//! Caller: `physics::move_slots` (inside `pusha` / `popa`).
//! Self-test: in a race only (it uses the stage's ground): a copy of the original (its
//! calls the game's) against the Rust version on test slots placed about the work car,
//! with speeds and spins by lot; every game variable it writes saved and put back.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{Regs, game_addr, game_read, game_write, rva};
use crate::math::x87;
use crate::math::x87_run::{abs_compare, abs_of, mul2_to_int, neg_mul, neg_product_plus};
use crate::physics::axis_turn_matrix::axis_turn_matrix;
use crate::physics::ground::Ground;
use crate::scene::matrix::multiply_matrices;
use crate::scene::matrix3::rounded;
use crate::scene::view_matrix::{square_matrix, unit_vector};

pub const SLOT_LEN: u32 = 0x78;
fn a(x: u32) -> u32 {
    game_addr(rva(x as usize)) as u32
}

unsafe fn at(p: u32) -> u32 {
    unsafe { (p as *const u32).read_unaligned() }
}

unsafe fn put(p: u32, v: u32) {
    unsafe { (p as *mut u32).write_unaligned(v) }
}

unsafe fn copy(from: u32, to: u32, n: u32) {
    for k in 0..n {
        unsafe { put(to + 4 * k, at(from + 4 * k)) };
    }
}

/// `find_ground`'s refusal at the place the slot at `s` will move to (worked out without
/// writing: the point is put back).
pub unsafe fn slot_refusal(s: u32) -> Option<&'static str> {
    unsafe {
        let dt = game_read::<u32>(rva(0x61388C));
        let k65536 = game_read::<u32>(rva(0x5C5D98));
        let moved =
            |c: u32, v: u32| at(s + c).wrapping_add(mul2_to_int(at(s + v), dt, k65536) as u32);
        let (x, z) = (moved(4, 0x50), moved(0xC, 0x58));
        let g = Ground::game();
        let keep = (g.get(0x639824), g.get(0x63982C));
        g.put(0x639824, x);
        g.put(0x63982C, z);
        let why = crate::physics::ground::ground_refusal(&g);
        g.put(0x639824, keep.0);
        g.put(0x63982C, keep.1);
        why
    }
}

/// The Rust core; `r` as the original leaves it.
pub unsafe fn move_slot_at(r: &mut Regs) -> Result<(), Refused> {
    unsafe {
        let s = r.esi;
        if let Some(why) = slot_refusal(s) {
            return Err(crate::errors::BadInput(why).into());
        }
        let dt = game_read::<u32>(rva(0x61388C));
        let k65536 = game_read::<u32>(rva(0x5C5D98));
        game_write(rva(0x71BE90), at(s + 4));
        game_write(rva(0x71BE94), at(s + 8));
        game_write(rva(0x71BE98), at(s + 0xC));
        put(
            s + 0x74,
            rounded(crate::physics::vector_length::vector_length(s + 0x68)),
        );
        if at(s + 0x74) != 0 {
            unit_vector((s + 0x74) as usize, a(0x5C78FC) as usize);
            game_write(rva(0x5C78F8), x87::mul(at(s + 0x74), dt));
            r.edi = a(0x5C7010);
            axis_turn_matrix(r.edi);
            square_matrix(a(0x5C7010) as usize);
            multiply_matrices(
                (s + 0x10) as usize,
                a(0x5C7010) as usize,
                a(0x5EA10C) as usize,
            );
            r.ecx = 0;
            r.edx = 0;
            r.ebp = a(0x5EA10C);
            copy(a(0x5EA10C), s + 0x10, 16);
        }
        for (v, c) in [(0x54, 8), (0x50, 4), (0x58, 0xC)] {
            let d = mul2_to_int(at(s + v), dt, k65536) as u32;
            game_write(rva(0x62400C), d);
            r.eax = d;
            put(s + c, at(s + c).wrapping_add(d));
        }
        put(
            s + 0x54,
            neg_product_plus(game_read(rva(0x613FC8)), dt, at(s + 0x54)),
        );
        game_write(rva(0x639824), at(s + 4));
        game_write(rva(0x63982C), at(s + 0xC));
        r.eax = at(s + 0x60);
        game_write(rva(0x5CCC78), r.eax);
        game_write(rva(0x620470), 0x80u32);
        crate::physics::find_ground::find_ground()?;
        let face: u32 = game_read(rva(0x5CCC74));
        if face == u32::MAX {
            copy(a(0x71BE90), s + 4, 3);
            for o in [0x50, 0x54, 0x58, 0x68, 0x6C, 0x70] {
                put(s + o, 0);
            }
            return Ok(());
        }
        put(s + 0x60, face);
        r.eax = at(s + 8);
        let height: u32 = game_read(rva(0x639828));
        if (r.eax.wrapping_sub(height) as i32) >= 0 {
            return Ok(());
        }
        r.eax = height;
        put(s + 8, height);
        let half = game_read::<u32>(rva(0x5C5E40));
        put(s + 0x50, x87::mul(at(s + 0x50), half));
        put(s + 0x54, neg_mul(at(s + 0x54), half));
        put(s + 0x58, x87::mul(at(s + 0x58), half));
        for o in [0x68, 0x6C, 0x70] {
            put(s + o, neg_mul(at(s + o), half));
        }
        if abs_compare(at(s + 0x54), game_read(rva(0x5C5D38))) & 0x100 != 0 {
            copy(a(0x71BA88), s + 0x10, 12);
            let n = at(s);
            r.eax = n / 20;
            r.edx = n % 20;
            r.ebx = 20;
            if (r.edx.wrapping_sub(5) as i32) >= 0 && (r.edx.wrapping_sub(9) as i32) < 0 {
                copy(a(0x71BA48), s + 0x10, 12);
                return Ok(());
            }
        }
        game_write(rva(0x62400C), abs_of(at(s + 0x54)));
        Ok(())
    }
}
