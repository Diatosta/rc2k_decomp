//! `scene::fill_engine_tables`: the tables the 3D code looks things up in.
//!
//! Eight parts:
//! - the two root tables `math::normalise_vector` reads: `root_table`[n] = the root of
//!   (n + 0x100) over `k256` (256), `exponent_root_table`[n] = the root of the float whose
//!   bits are n << 23;
//! - the 0x10000 `shuffled_words` = 0 .. 0xFFFF, then each in turn exchanged with the one
//!   `scene::next_noise`'s low sixteen bits name, the generator started from 0x12D687;
//! - the 0x1070 `noise_bytes`, each bits 12 .. 19 of a number of the generator, started from
//!   0x12D687 again;
//! - `face_key_factors`[n] = 1 / n, and 1 for n = 0;
//! - `sines`[n] = the sine of n times pi over 0x800, 0x1400 of them (a turn is 0x1000; the
//!   cosines are the same table from 0x400 on);
//! - the 0x1000 `arctangents`: for the tangent t = n << 20, the angle found by halving twelve
//!   times from 0x1000, each try's tangent being sine over cosine times 65536 times 32768 as a
//!   whole number, doubled, compared unsigned; the angle's low four bits and its bits from 13
//!   up dropped;
//! - the tangent for every 0x40th angle up to 0x2000 and `scene::angle_between` for 0x100
//!   places: nothing kept of either but the work variables' last values.
//!
//! Everything is worked out the same on every call (the generator's state is set first), but for
//! `fsin` being the processor's.

use crate::math::engine_tables::EngineTables;
use crate::math::x87;
use crate::math::x87_float as bh;
use crate::scene::angle_between::angle_of;
use crate::scene::next_noise::next_noise;
use crate::scene::owner::Scene;

// ---- game data ----

const NOISE_LEN: usize = 0x1070;
const SINES: usize = 0x1400;
const SEED: u32 = 0x0012_D687;

/// The tangent of an angle (0x2000 a turn here, its low four bits dropped), from the sine table
/// `sines`.
fn tangent(sines: &[f32; SINES], scene: &Scene, angle: u32) -> u32 {
    let place = (angle >> 4) as usize;
    scene.draw_work_mut().scratch = 0x8000u32;
    (bh::to_int(
        bh::ld(sines[place]) / bh::ld(sines[place + 0x400]) * bh::int(0x1_0000) * bh::int(0x8000),
    ) as u32)
        << 1
}

/// Fills the tables. They are worked out apart and put in whole at the end.
pub fn fill_engine_tables(race: &crate::race::owner::Race, scene: &Scene) {
    let course = &race.course;
    let mut tables = Box::new(EngineTables::new());
    // The root tables.
    for n in 0..0x100usize {
        let root = bh::stf((bh::int(n as i32 + 0x100) / bh::ld(course.numbers.k256.get())).sqrt());
        course.lists.borrow_mut().root_table[n] = root;
    }
    for n in 0..0x100usize {
        course.lists.borrow_mut().exponent_root_table[n] = x87::sqrt((n as u32) << 23);
    }
    // The shuffled words.
    race.shared.run_mut().noise_state = SEED;
    for (n, word) in tables.shuffled_words.iter_mut().enumerate() {
        *word = n as u16;
    }
    for n in 0..0x1_0000usize {
        let other = (next_noise(&race.shared) & 0xFFFF) as usize;
        tables.shuffled_words.swap(n, other);
    }
    // The noise bytes.
    race.shared.run_mut().noise_state = SEED;
    for byte in &mut tables.noise_bytes[..NOISE_LEN] {
        *byte = (next_noise(&race.shared) >> 12) as u8;
    }
    // The face key factors.
    for n in 1..0x100usize {
        course.lists.borrow_mut().face_key_factors[n] = x87::int_ratio(1, n as i32);
    }
    course.lists.borrow_mut().face_key_factors[0] = 1.0f32;
    // The sines.
    for (n, entry) in tables.sines.iter_mut().enumerate() {
        *entry = bh::stf((bh::int(n as i32).times_pi() / bh::int(0x800)).sin());
    }
    // The arctangents.
    for n in 0..0x1000usize {
        let wanted = (n as u32) << 20;
        let (mut angle, mut step) = (0x1000u32, 0x800u32);
        while step != 0 {
            if wanted < tangent(&tables.sines, scene, angle) {
                angle -= step;
            } else {
                angle += step;
            }
            step >>= 1;
        }
        tables.arctangents[n] = (angle & 0x1FF0) as u16;
    }
    // The two loops whose results nothing keeps.
    for angle in (0..0x2000).step_by(0x40) {
        tangent(&tables.sines, scene, angle);
    }
    let mut angle = 0;
    for place in 0..0x100u32 {
        angle = angle_of(&race.course, place, 0x100);
    }
    // `scene::angle_between` leaves the last angle in the shared work dword `scratch`.
    scene.draw_work_mut().scratch = angle;
    *race.shared.buffers.engine_tables.borrow_mut() = *tables;
}
