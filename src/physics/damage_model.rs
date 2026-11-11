//! The car's damage and wear model under `physics::damage_step`:
//! `physics::damage_factors` (0x4AB5E6), `physics::damage_wear` (0x4ABEDF),
//! `physics::damage_gauges` (0x4AC817). Their figures: the work car's (`g_car`,
//! 0x719FD8 ..: the damage gauges `brake_heat` .. `tyre_wear`, the handling factors
//! `wheel_ride_height` ..), the model's own (`DamageModel.chance_scale` .. 0x75B940: shares, tables of four
//! (x0, y0, x1, y1) for `physics::flow::table_lerp`, rates). What each figure is, is
//! worked out from its uses (the docs in `physics/store.rs`). The three are long straight runs: their Rust was generated from the
//! disassembly by idiom (x87 runs as blocks, the clamps as `hold_low` / `hold_high`, the
//! table lerps, the compares as `if`), the rest written by hand.
//!
//! This file: the regions and the shared test.

/// 1.0 (`fld1; fstp`).
pub const ONE: u32 = 0x3F80_0000;

use crate::physics::flow::FloatView;
use crate::session::Cell32;

/// The value of a float cell as the model's arithmetic holds it: the game's code loads a
/// float and works at double precision.
pub fn value(cell: &impl Cell32) -> f64 {
    f64::from(cell.float())
}

/// A result of that arithmetic stored into a float cell, rounded to a float.
pub fn store(cell: &impl Cell32, result: f64) {
    cell.set_float(result as f32);
}

/// The same figure for a whole number held in a cell (`fild`).
pub fn whole(cell: &impl Cell32) -> f64 {
    f64::from(cell.bits() as i32)
}

#[cfg(test)]
mod x87_reference {
    pub mod factors;
    pub mod gauges;
    pub mod wear;
}

/// Proofs that the plain versions of the model's arithmetic give what the versions over
/// `X` (`x87_reference`) gave: the same random state in, the same bytes out.
#[cfg(test)]
pub(super) mod proof {
    use super::x87_reference;
    use crate::physics::car_store::Car;
    use crate::physics::store::DamageModel;
    use crate::race::work_store::RaceWork;
    use core::sync::atomic::{AtomicU32, Ordering::Relaxed};

    pub use x87_reference::{factors, gauges, wear};

    /// A seeded generator (splitmix64).
    pub struct Rng(u64);

    impl Rng {
        pub fn new(seed: u64) -> Self {
            Rng(seed)
        }

        pub fn next(&mut self) -> u32 {
            self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
            let mut z = self.0;
            z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
            z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
            (z ^ (z >> 31)) as u32
        }
    }

    /// The bits of a float cell: mostly ordinary figures, now and then zeros, ones, huge
    /// and tiny figures, infinities, no numbers and plain random bits.
    pub fn random_bits(rng: &mut Rng) -> u32 {
        let r = rng.next();
        match r % 16 {
            0 => 0,
            1 => 0x8000_0000,
            2 => 0x3F80_0000,
            3 => 0x42C8_0000,
            4 => 0x7F80_0000 | (rng.next() & 0x8000_0000),
            5 => 0x7FC0_0000 | (rng.next() & 0x803F_FFFF),
            6 => rng.next(),
            7 => rng.next() & 0x00FF_FFFF,
            8 => (rng.next() & 0x8000_0000) | 0x7000_0000 | (rng.next() & 0x07FF_FFFF),
            9 => rng.next() % 2000,
            _ => {
                // within a few hundred, either sign
                let mantissa = rng.next() & 0x007F_FFFF;
                let exponent = 118 + rng.next() % 16;
                (rng.next() & 0x8000_0000) | (exponent << 23) | mantissa
            }
        }
    }

    fn cells<T>(t: &T) -> &[AtomicU32] {
        assert_eq!(size_of::<T>() % 4, 0);
        // SAFETY: the records are made of dword cells only, 4-aligned.
        unsafe { core::slice::from_raw_parts(core::ptr::from_ref(t).cast(), size_of::<T>() / 4) }
    }

    pub fn fill<T>(t: &T, rng: &mut Rng) {
        for c in cells(t) {
            c.store(random_bits(rng), Relaxed);
        }
    }

    fn words<T>(t: &T) -> Vec<u32> {
        cells(t).iter().map(|c| c.load(Relaxed)).collect()
    }

    fn put<T>(t: &T, w: &[u32]) {
        for (c, v) in cells(t).iter().zip(w) {
            c.store(*v, Relaxed);
        }
    }

    /// Whether two results are the same, but for the sign of a no-number: the emulation of
    /// the x87 subtracted by adding the negated operand, which flips the sign of a no-number
    /// operand on its way through; the processor's own subtraction keeps it.
    fn same_word(a: u32, b: u32) -> bool {
        let no_number = |w: u32| w & 0x7F80_0000 == 0x7F80_0000 && w & 0x007F_FFFF != 0;
        a == b || (no_number(a) && a ^ b == 0x8000_0000)
    }

    /// A car, the model and the race work, each with its own state.
    pub struct State {
        pub car: Box<Car>,
        pub damage: Box<DamageModel>,
        pub work: Box<RaceWork>,
    }

    impl State {
        fn new() -> Self {
            State {
                car: Box::new(Car::new()),
                damage: Box::new(DamageModel::new()),
                work: Box::new(RaceWork::new()),
            }
        }

        fn words(&self) -> [Vec<u32>; 3] {
            [words(&*self.car), words(&*self.damage), words(&*self.work)]
        }

        fn put(&self, w: &[Vec<u32>; 3]) {
            put(&*self.car, &w[0]);
            put(&*self.damage, &w[1]);
            put(&*self.work, &w[2]);
        }
    }

    /// `same_result` with 20,000 runs and a seed made of the name.
    pub fn prove(name: &str, old: impl Fn(&State), new: impl Fn(&State)) {
        let seed = name.bytes().fold(0xCBF2_9CE4_8422_2325u64, |h, b| {
            (h ^ u64::from(b)).wrapping_mul(0x100_0000_01B3)
        });
        same_result(name, seed, 20_000, old, new);
    }

    /// Runs `old` and `new` on `runs` copies of one random state each, and fails when any
    /// byte of the car, the model or the work differs afterwards.
    pub fn same_result(
        name: &str,
        seed: u64,
        runs: u32,
        old: impl Fn(&State),
        new: impl Fn(&State),
    ) {
        let mut rng = Rng::new(seed);
        let a = State::new();
        let b = State::new();
        for run in 0..runs {
            fill(&*a.car, &mut rng);
            fill(&*a.damage, &mut rng);
            fill(&*a.work, &mut rng);
            b.put(&a.words());
            old(&a);
            new(&b);
            let (x, y) = (a.words(), b.words());
            for (which, (p, q)) in x.iter().zip(&y).enumerate() {
                if let Some(i) = p.iter().zip(q).position(|(p, q)| !same_word(*p, *q)) {
                    panic!(
                        "{name}: run {run}, block {which}, dword {i}: reference {:#010x}, plain {:#010x}",
                        p[i], q[i]
                    );
                }
            }
        }
    }
}
