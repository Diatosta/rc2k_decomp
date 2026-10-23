//! `marks::view_motion` (0x4C4D40): the view's motion since the last frame. With the
//! view matrix `flt_5EA3CC`: its third row (+0x20 ..) kept on the stack; its place (the
//! origin turned by it, `scene::rotate_point`, unrounded) less the last (`flt_91FFBE` ..)
//! into `flt_8EB951` .. and the stack; that step made length 1
//! (`scene::normalise_vector3`); `flt_8F397D` the third row's dot product with it; the
//! place (turned again) kept as the last; then `flt_8EB951` .. each over
//! `g_marksFrameTicks` (`fidiv`).
//!
//! In:  nothing
//! Out: esi the view matrix; edi and ebp its stack scratch (not compared); the rest kept
//!
//! Caller: `sub_4C3EEE`. Shadow mode and the self-test set edi and ebp back to the
//! caller's on both sides (the original's are addresses on its stack). Self-test: view
//! matrices, last places and ticks by lot; compared on the seven results.
//!
//! Converted 2026-10-21: the hook's Rust version is `view_motion_plain`, through
//! `rc2k_core::marks::view::view_motion` (plain floats; the x87 version stays for the Rust
//! callers in unconverted code). Shadow mode and the self-test compare within
//! `service::floats`' tolerances (the self-test at both test precisions, exact at double).
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::framework::{Regs, game_addr, game_read, game_write, rva};
use crate::math::x87;
use crate::scene::matrix3::{extended, rounded};
use crate::scene::rotate_point::rotate_point;
use crate::scene::vector3::{dot_product3, normalise_vector3};
use crate::service::harness::{Region, at};

const VIEW_RVA: usize = rva(0x5EA3CC); // flt_5EA3CC: the view matrix
const LAST_RVA: usize = rva(0x91FFBE); // flt_91FFBE .. flt_91FFC6
const STEP_RVA: usize = rva(0x8EB951); // flt_8EB951 .. flt_8EB959
const FACING_RVA: usize = rva(0x8F397D); // flt_8F397D
const TICKS_RVA: usize = rva(0x764FD0); // g_marksFrameTicks

/// The Rust version (see the header): the registers it leaves.
pub unsafe fn view_motion(entry: &Regs) -> Regs {
    let mut g = *entry;
    unsafe {
        let m = game_addr(VIEW_RVA) as u32;
        let get = |a: u32| (a as *const u32).read_unaligned();
        let mut local = [0u32; 6];
        for (k, v) in local.iter_mut().take(3).enumerate() {
            *v = x87::loaded(get(m + 0x20 + 4 * k as u32));
        }
        let zero = [extended(0); 3];
        let place = rotate_point(m, zero);
        for k in 0..3 {
            let step = x87::ext_less(place[k], game_read(LAST_RVA + 4 * k));
            game_write(STEP_RVA + 4 * k, step);
            local[3 + k] = step;
        }
        let at = local.as_mut_ptr() as u32;
        normalise_vector3(at + 12);
        game_write(FACING_RVA, rounded(dot_product3(at, at + 12)));
        let place = rotate_point(m, zero);
        for (k, p) in place.iter().enumerate() {
            game_write(LAST_RVA + 4 * k, rounded(*p));
        }
        let ticks: i32 = game_read(TICKS_RVA);
        for k in 0..3 {
            let a = STEP_RVA + 4 * k;
            game_write(a, x87::over_int(game_read(a), ticks));
        }
        g.esi = m;
    }
    g
}

/// The converted version (plain floats): the hook's.
// Converted: `rc2k_core::marks::view::view_motion`
pub unsafe fn view_motion_plain(entry: &Regs) -> Regs {
    unsafe {
        let m = game_addr(VIEW_RVA) as u32;
        // The fourth float of each row is not read.
        let matrix: [f32; 15] = std::array::from_fn(|n| {
            if n % 4 == 3 {
                0.0
            } else {
                f32::from_bits(((m + 4 * n as u32) as *const u32).read_unaligned())
            }
        });
        let last: [u32; 3] = game_read(LAST_RVA);
        let ticks: i32 = game_read(TICKS_RVA);
        let v = rc2k_core::marks::view::view_motion(&matrix, last.map(f32::from_bits), ticks);
        game_write(STEP_RVA, v.step.map(f32::to_bits));
        game_write(FACING_RVA, v.facing.to_bits());
        game_write(LAST_RVA, v.last.map(f32::to_bits));
        game_write(STEP_RVA, v.per_tick.map(f32::to_bits));
        Regs { esi: m, ..*entry }
    }
}

pub fn regions() -> Vec<Region> {
    vec![
        (game_addr(STEP_RVA) as u32, 12),
        at(FACING_RVA),
        (game_addr(LAST_RVA) as u32, 12),
    ]
}
