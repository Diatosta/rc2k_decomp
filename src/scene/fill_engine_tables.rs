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
//! Everything is worked out the same on every call (the generator's state is set first), but
//! for `fsin` being the processor's. One caller, `sub_442AF6` (eight callers: around the front
//! end's 3D set-up and at the ends of events).

use crate::math::x87;
use crate::scene::angle_between::angle_of;
use crate::scene::next_noise::next_noise;
use crate::scene::owner::scene;

// ---- game data ----

const NOISE_LEN: usize = 0x1070;
const SINES: usize = 0x1400;
const SEED: u32 = 0x0012_D687;

// ---- Rust version ----

/// `sub_41F792`: the tangent of an angle (0x2000 a turn here, its low four bits
/// dropped) into `dword_5C5F48`.
fn tangent(frame: &crate::race::owner_frame::Frame, angle: u32) -> u32 {
    {
        let place = (angle >> 4) as usize;
        scene().drawing.scene.scratch.set_bits(0x8000u32);
        let tangent = (x87::ratio_scaled_to_int(
            crate::scene::engine_tables_store::ENGINE_TABLES.sines[place].bits(),
            crate::scene::engine_tables_store::ENGINE_TABLES.sines[place + 0x400].bits(),
            0x1_0000,
            0x8000,
        ) as u32)
            << 1;
        frame.scratch.tangent_scratch.set_bits(tangent);
        tangent
    }
}

/// The Rust version. The registers the original leaves (eax, ebx 0, ecx 0xFF, edx and ebp
/// 0x100, edi the end of the arctangent table) are overwritten or unread by its one caller,
/// `scene::set_up_engine`, whose leftovers are `physics::test_drive`'s (edi, ebp: read by
/// none of its sites, `lq.py -s 442AF6`).
pub fn fill_engine_tables(race: &crate::race::owner::Race) {
    let course = &race.course;
    // sub_42142B
    for n in 0..0x100usize {
        let root = x87::int_over_root(n as i32 + 0x100, course.numbers.k256.bits());
        course.lists.root_table[n].set_bits(root);
    }
    for n in 0..0x100usize {
        course.lists.exponent_root_table[n].set_bits(x87::sqrt((n as u32) << 23));
    }
    // sub_41F661
    race.shared.physics_tables.noise_state.set_bits(SEED);
    let word = |n: usize| &crate::scene::engine_tables_store::ENGINE_TABLES.shuffled_words[n];
    for n in 0..0x1_0000usize {
        word(n).set(n as u16);
    }
    for n in 0..0x1_0000usize {
        let other = (next_noise(&race.shared) & 0xFFFF) as usize;
        let (here, there) = (word(n).get(), word(other).get());
        word(n).set(there);
        word(other).set(here);
    }
    // sub_41F635
    race.shared.physics_tables.noise_state.set_bits(SEED);
    for n in 0..NOISE_LEN {
        crate::scene::engine_tables_store::ENGINE_TABLES
            .noise_bytes
            .set(n, (next_noise(&race.shared) >> 12) as u8);
    }
    // sub_41F716
    for n in 1..0x100usize {
        course.lists.face_key_factors[n].set_bits(x87::int_ratio(1, n as i32));
    }
    course.lists.face_key_factors[0].set_bits(1.0f32.to_bits());
    // sub_41F5EF
    for n in 0..SINES {
        crate::scene::engine_tables_store::ENGINE_TABLES.sines[n]
            .set_bits(x87::sine_of_pi_fraction(n as i32, 0x800));
    }
    // sub_41F74A
    for n in 0..0x1000usize {
        let wanted = (n as u32) << 20;
        let (mut angle, mut step) = (0x1000u32, 0x800u32);
        while step != 0 {
            if wanted < tangent(&race.frame, angle) {
                angle -= step;
            } else {
                angle += step;
            }
            step >>= 1;
        }
        crate::scene::engine_tables_store::ENGINE_TABLES.arctangents[n]
            .set((angle & 0x1FF0) as u16);
    }
    // The two loops whose results nothing keeps.
    for angle in (0..0x2000).step_by(0x40) {
        tangent(&race.frame, angle);
    }
    for place in 0..0x100u32 {
        angle_of(&race.course, place, 0x100);
    }
}
