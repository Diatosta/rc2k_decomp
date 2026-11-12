//! `scene::fill_engine_tables` (0x41F6BA): the tables the 3D code looks things up in.
//!
//! Eight parts, each a function with this one caller, all inlined:
//! - the two root tables `math::normalise_vector` reads: `root_table`[n] = the root of
//!   (n + 0x100) over `k256` (256), `exponent_root_table`[n] = the root of the float whose
//!   bits are n << 23;
//! - the 0x10000 `shuffled_words` = 0 .. 0xFFFF, then each in turn exchanged with the one
//!   `scene::next_noise`'s low sixteen bits name, the generator started from 0x12D687;
//! - the 0x1070 `noise_bytes`, each bits 12 .. 19 of a number of the generator, started
//!   from 0x12D687 again;
//! - `face_key_factors`[n] = 1 / n, and 1 for n = 0;
//! - `sines`[n] = the sine of n times pi over 0x800, 0x1400 of them (a turn is 0x1000; the
//!   cosines are the same table from 0x400 on);
//! - the 0x1000 `arctangents`: for the tangent t = n << 20, the angle found by halving
//!   twelve times from 0x1000, each try's tangent being sine over cosine times 65536 times
//!   32768 as a whole number, doubled, compared unsigned; the angle's low four bits and its
//!   bits from 13 up dropped;
//! - the tangent for every 0x40th angle up to 0x2000 and `scene::angle_between` for 0x100
//!   places: nothing kept of either but the work variables' last values.
//!
//! Everything is worked out the same on every call (the generator's state is set first), but
//! for `fsin` being the processor's. One caller, `scene::set_up_engine` (itself called around the
//! front end's 3D set-up and at the ends of events).

use crate::math::x87;
use crate::math::x87_float as bh;
use crate::scene::angle_between::angle_of;
use crate::scene::next_noise::next_noise;
use crate::scene::owner::Scene;

// ---- game data ----

const NOISE_LEN: usize = 0x1070;
const SINES: usize = 0x1400;
const SEED: u32 = 0x0012_D687;

// ---- Rust version ----

/// The tangent of an angle (0x2000 a turn here, its low four bits dropped), also kept in
/// the frame's `tangent_scratch`.
fn tangent(race: &crate::race::owner::Race, scene: &Scene, angle: u32) -> u32 {
    {
        let frame = &race.frame;
        let tables = race.engine_tables();
        let place = (angle >> 4) as usize;
        scene.drawing.scene.get().scratch.set(0x8000u32);
        let tangent = (bh::to_int(
            bh::ld(tables.sines[place].get()) / bh::ld(tables.sines[place + 0x400].get())
                * bh::int(0x1_0000)
                * bh::int(0x8000),
        ) as u32)
            << 1;
        frame.scratch().tangent_scratch.set(tangent);
        tangent
    }
}

/// Fills the tables; the register leftovers of the original are read by no caller.
pub fn fill_engine_tables(race: &crate::race::owner::Race, scene: &Scene) {
    let course = &race.course;
    let tables = race.engine_tables();
    // The root tables.
    for n in 0..0x100usize {
        let root = bh::stf((bh::int(n as i32 + 0x100) / bh::ld(course.numbers.k256.get())).sqrt());
        course.lists.root_table[n].set(root);
    }
    for n in 0..0x100usize {
        course.lists.exponent_root_table[n].set(x87::sqrt((n as u32) << 23));
    }
    // The shuffled words.
    race.shared.physics_tables.noise_state.set(SEED);
    let word = |n: usize| &tables.shuffled_words[n];
    for n in 0..0x1_0000usize {
        word(n).set(n as u16);
    }
    for n in 0..0x1_0000usize {
        let other = (next_noise(&race.shared) & 0xFFFF) as usize;
        let (here, there) = (word(n).get(), word(other).get());
        word(n).set(there);
        word(other).set(here);
    }
    // The noise bytes.
    race.shared.physics_tables.noise_state.set(SEED);
    for n in 0..NOISE_LEN {
        tables
            .noise_bytes
            .set(n, (next_noise(&race.shared) >> 12) as u8);
    }
    // The face key factors.
    for n in 1..0x100usize {
        course.lists.face_key_factors[n].set(x87::int_ratio(1, n as i32));
    }
    course.lists.face_key_factors[0].set(1.0f32);
    // The sines.
    for n in 0..SINES {
        tables.sines[n].set(bh::stf(
            (bh::int(n as i32).times_pi() / bh::int(0x800)).sin(),
        ));
    }
    // The arctangents.
    for n in 0..0x1000usize {
        let wanted = (n as u32) << 20;
        let (mut angle, mut step) = (0x1000u32, 0x800u32);
        while step != 0 {
            if wanted < tangent(race, scene, angle) {
                angle -= step;
            } else {
                angle += step;
            }
            step >>= 1;
        }
        tables.arctangents[n].set((angle & 0x1FF0) as u16);
    }
    // The two loops whose results nothing keeps.
    for angle in (0..0x2000).step_by(0x40) {
        tangent(race, scene, angle);
    }
    let mut angle = 0;
    for place in 0..0x100u32 {
        angle = angle_of(&race.course, place, 0x100);
    }
    // `scene::angle_between` leaves the last angle in the shared work dword `scratch`.
    scene.drawing.scene.get().scratch.set(angle);
}
