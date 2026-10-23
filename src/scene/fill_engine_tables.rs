//! `scene::fill_engine_tables` (0x41F6BA): the tables the 3D code looks things up in.
//!
//! In:  nothing
//! Out: eax = ebx = 0, ecx = 0xFF, edx = ebp = 0x100, edi = the arctangent table's end;
//!      esi preserved
//!
//! Eight parts, each a function with this one caller, all inlined:
//! - `sub_42142B`: the two root tables `math::normalise_vector` reads: `g_rootTable`[n]
//!   = the root of (n + 0x100) over `flt_5C5E2C` (256), `g_exponentRootTable`[n] = the
//!   root of the float whose bits are n << 23;
//! - `sub_41F661`: the 0x10000 words at `word_A6BB54` = 0 .. 0xFFFF, then each in turn
//!   exchanged with the one `scene::next_noise`'s low sixteen bits name, the generator
//!   started from 0x12D687;
//! - `sub_41F635`: 0x1070 bytes at `byte_A2AAE4`, each bits 12 .. 19 of a number of the
//!   generator, started from 0x12D687 again;
//! - `sub_41F716`: `g_faceKeyFactors`[n] = 1 / n, and 1 for n = 0;
//! - `sub_41F5EF`: `g_sines`[n] = the sine of n times pi over 0x800, 0x1400 of them (a
//!   turn is 0x1000; the cosines are the same table from 0x400 on);
//! - `sub_41F74A`: 0x1000 words at `unk_A28AE4`, an arctangent table: for the tangent
//!   t = n << 20, the angle found by halving twelve times from 0x1000 (`sub_41F768`),
//!   each try's tangent being sine over cosine times 65536 times 32768 as a whole
//!   number, doubled (`sub_41F792`, into `dword_5C5F48`), compared unsigned; the
//!   angle's low four bits and its bits from 13 up dropped;
//! - `sub_41F792` for every 0x40th angle up to 0x2000 and `scene::angle_between` for
//!   0x100 places: nothing kept of either but the work variables' last values.
//!
//! Everything is worked out the same on every call (the generator's state is set
//! first), but for `fsin` being the processor's.
//! One caller, `sub_442AF6` (eight callers: around the front end's 3D set-up and at
//! the ends of events).
//! Shadow mode: Rust, its writes put back, the original, compared.
//! Self-test: both on the game's tables filled with a pattern first, put back
//! afterwards.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::framework::{Regs, game_addr, game_read, game_write, rva};
use crate::math::x87;
use crate::scene::angle_between::angle_of;
use crate::scene::draw_polygon::CHECKED_RVA;
use crate::scene::next_noise::{STATE_RVA, next_noise};

// ---- game data ----

const ROOTS_RVA: usize = rva(0x5C70F8); // g_rootTable
const EXPONENT_ROOTS_RVA: usize = rva(0x5C74F8); // g_exponentRootTable
const ROOT_DIVISOR_RVA: usize = rva(0x5C5E2C); // flt_5C5E2C, 256.0
const SHUFFLED_RVA: usize = rva(0xA6BB54); // word_A6BB54, 0x10000 words
const NOISE_RVA: usize = rva(0xA2AAE4); // byte_A2AAE4, 0x1070 bytes
const NOISE_LEN: usize = 0x1070;
const FACTORS_RVA: usize = rva(0x5C6950); // g_faceKeyFactors
const SINES_RVA: usize = rva(0xA23AE4); // g_sines, 0x1400 floats
const SINES: usize = 0x1400;
const ARCTANGENTS_RVA: usize = rva(0xA28AE4); // unk_A28AE4, 0x1000 words
const TANGENT_RVA: usize = rva(0x5C5F48); // dword_5C5F48
const SEED: u32 = 0x0012_D687;

// ---- Rust version ----

/// `sub_41F792`: the tangent of an angle (0x2000 a turn here, its low four bits
/// dropped) into `dword_5C5F48`.
unsafe fn tangent(angle: u32) -> u32 {
    unsafe {
        let place = SINES_RVA + (angle >> 4) as usize * 4;
        game_write(CHECKED_RVA, 0x8000u32);
        let tangent = (x87::ratio_scaled_to_int(
            game_read(place),
            game_read(place + 0x1000),
            0x1_0000,
            0x8000,
        ) as u32)
            << 1;
        game_write(TANGENT_RVA, tangent);
        tangent
    }
}

pub unsafe fn fill_engine_tables(r: &mut Regs) {
    unsafe {
        // sub_42142B
        for n in 0..0x100usize {
            let root = x87::int_over_root(n as i32 + 0x100, game_read(ROOT_DIVISOR_RVA));
            game_write(ROOTS_RVA + n * 4, root);
        }
        for n in 0..0x100usize {
            game_write(EXPONENT_ROOTS_RVA + n * 4, x87::sqrt((n as u32) << 23));
        }
        // sub_41F661
        game_write(STATE_RVA, SEED);
        let word = |n: usize| SHUFFLED_RVA + n * 2;
        for n in 0..0x1_0000usize {
            game_write(word(n), n as u16);
        }
        for n in 0..0x1_0000usize {
            let other = (next_noise() & 0xFFFF) as usize;
            let (here, there): (u16, u16) = (game_read(word(n)), game_read(word(other)));
            game_write(word(n), there);
            game_write(word(other), here);
        }
        // sub_41F635
        game_write(STATE_RVA, SEED);
        for n in 0..NOISE_LEN {
            game_write(NOISE_RVA + n, (next_noise() >> 12) as u8);
        }
        // sub_41F716
        for n in 1..0x100usize {
            game_write(FACTORS_RVA + n * 4, x87::int_ratio(1, n as i32));
        }
        game_write(FACTORS_RVA, 1.0f32.to_bits());
        // sub_41F5EF
        for n in 0..SINES {
            game_write(SINES_RVA + n * 4, x87::sine_of_pi_fraction(n as i32, 0x800));
        }
        // sub_41F74A
        for n in 0..0x1000usize {
            let wanted = (n as u32) << 20;
            let (mut angle, mut step) = (0x1000u32, 0x800u32);
            while step != 0 {
                if wanted < tangent(angle) {
                    angle -= step;
                } else {
                    angle += step;
                }
                step >>= 1;
            }
            game_write(ARCTANGENTS_RVA + n * 2, (angle & 0x1FF0) as u16);
        }
        // The two loops whose results nothing keeps.
        for angle in (0..0x2000).step_by(0x40) {
            tangent(angle);
        }
        for place in 0..0x100u32 {
            angle_of(place, 0x100);
        }
        r.eax = 0;
        r.ebx = 0;
        r.ecx = 0xFF;
        r.edx = 0x100;
        r.ebp = 0x100;
        r.edi = (game_addr(ARCTANGENTS_RVA) + 0x2000) as u32;
    }
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    unsafe { fill_engine_tables(r) };
    Ok(())
}
