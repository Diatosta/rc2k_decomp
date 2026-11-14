//! The car's damage and wear model under `physics::damage_step`: `physics::damage_factors`,
//! `physics::damage_wear` and `physics::damage_gauges`. Their figures are the work car's (the
//! damage gauges `brake_heat` .. `tyre_wear`, the handling factors `wheel_ride_height` ..) and
//! the model's own (`DamageModel.chance_scale` ..: shares, tables of four (x0, y0, x1, y1) for
//! `physics::flow::table_lerp`, rates). What each figure is was worked out from its uses (see
//! the docs in `physics/store.rs`). The three steps are long straight runs: x87 runs as blocks,
//! the clamps as `hold_low` / `hold_high`, the table lerps and compares as `if`.
//!
//! This file holds the regions and the shared test.

use crate::session::{PlainDword, PlainFloat};

/// The value of a float cell as the model's arithmetic holds it: a float loaded and worked at
/// double precision.
pub fn value(cell: &PlainFloat) -> f64 {
    f64::from(cell.get())
}

/// A result of that arithmetic stored into a float cell, rounded to a float.
pub fn store(cell: &PlainFloat, result: f64) {
    cell.set(result as f32);
}

/// A word test on a float cell: true unless every bit is zero, so a -0.0 counts as set and a
/// NaN as set, as a compare of the float would not.
pub fn is_set(cell: &PlainFloat) -> bool {
    cell.bits() != 0
}

/// The same figure for a whole number held in a cell.
pub fn whole(cell: &PlainDword) -> f64 {
    f64::from(cell.get() as i32)
}

/// Proofs that the plain versions of the model's arithmetic give what the versions over `X`
/// gave: the same random state in, the same bytes out. Each proof's results are kept as a hash
/// (`test_support::frozen`, table `FROZEN` at the end).
#[cfg(test)]
pub(super) mod proof {
    use crate::physics::car_store::Car;
    use crate::physics::store::DamageModel;
    use crate::race::work_store::RaceWork;
    use crate::test_support::frozen::{Canon, Digest, check_one};
    use core::sync::atomic::{AtomicU32, Ordering::Relaxed};

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
    }

    /// `same_result` with 20,000 runs and a seed made of the name, against the hash frozen
    /// for the name.
    pub fn prove(name: &str, new: impl Fn(&State) + Sync) {
        let seed = name.bytes().fold(0xCBF2_9CE4_8422_2325u64, |h, b| {
            (h ^ u64::from(b)).wrapping_mul(0x100_0000_01B3)
        });
        let digest = same_result(seed, 20_000, new);
        check_one("damage_model", name, digest, FROZEN);
    }

    /// Runs `new` on `runs` copies of one random state each and hashes every dword of the
    /// car, the model and the work afterwards (a NaN as one NaN: which NaN an operation
    /// gives is not fixed in Rust, and the comparison counted any NaN as any other).
    ///
    /// The runs are split over threads, each with its own state (the tests are bound by the
    /// processor). The cases are the ones a single thread made: every run's
    /// state is drawn from where the last run's draws ended, so the generator is first run
    /// through all the runs' draws (as many `random_bits` as the three records have cells)
    /// to find where each part's first run begins. Each part runs at the caller's x87
    /// control word, the one thread-local state the arithmetic reads, and hashes its runs
    /// one by one; the runs' hashes are hashed in order.
    pub fn same_result(seed: u64, runs: u32, new: impl Fn(&State) + Sync) -> Digest {
        // `fill`'s draws: one a cell.
        let draws_a_run =
            size_of::<Car>() / 4 + size_of::<DamageModel>() / 4 + size_of::<RaceWork>() / 4;
        let parts = std::thread::available_parallelism()
            .map_or(1, std::num::NonZero::get)
            .min(8) as u32;
        let per_part = runs.div_ceil(parts).max(1);
        let mut starts = Vec::new();
        let mut rng = Rng::new(seed);
        for run in 0..runs {
            if run % per_part == 0 {
                starts.push((run, rng.0));
            }
            for _ in 0..draws_a_run {
                random_bits(&mut rng);
            }
        }
        let control = crate::math::x87::control_word();
        let new = &new;
        let outcomes: Vec<std::thread::Result<Vec<u64>>> = std::thread::scope(|scope| {
            let parts: Vec<_> = starts
                .into_iter()
                .map(|(first, at)| {
                    scope.spawn(move || {
                        crate::math::x87::set_control_word(control);
                        let end = (first + per_part).min(runs);
                        same_result_part(Rng(at), first..end, new)
                    })
                })
                .collect();
            parts
                .into_iter()
                .map(std::thread::ScopedJoinHandle::join)
                .collect()
        });
        let mut digest = Digest::default();
        for outcome in outcomes {
            match outcome {
                Ok(hashes) => hashes.iter().for_each(|h| digest.bytes(&h.to_le_bytes())),
                Err(panic) => std::panic::resume_unwind(panic),
            }
        }
        digest
    }

    /// The runs `runs` of `same_result`, the first drawn from `rng`: each run's hash.
    fn same_result_part(
        mut rng: Rng,
        runs: std::ops::Range<u32>,
        new: &impl Fn(&State),
    ) -> Vec<u64> {
        let b = State::new();
        runs.map(|_| {
            fill(&*b.car, &mut rng);
            fill(&*b.damage, &mut rng);
            fill(&*b.work, &mut rng);
            new(&b);
            let mut digest = Digest::default();
            for block in b.words() {
                for word in block {
                    word.feed(&mut digest);
                }
            }
            digest.value()
        })
        .collect()
    }

    /// The proofs' hashes, frozen (`RC2K_FREEZE_OUT` to make them again).
    const FROZEN: &[(&str, u64)] = &[
        ("damage_model::add_hit_to_gear_gauge", 0xdc5a49e3da27fefa),
        ("damage_model::add_product", 0xacf7f98bf1b70004),
        ("damage_model::add_rate_to_clutch", 0x3e88e515b76d6dd8),
        ("damage_model::add_rate_to_gauge", 0xdef686ce7f0c6e90),
        ("damage_model::add_scaled_triple", 0x430d2b014a0612ad),
        ("damage_model::add_triple_product", 0xb317082186089d97),
        (
            "damage_model::add_wear into another cell",
            0x4646202c78ed51d9,
        ),
        ("damage_model::add_wear", 0x30179634a7879737),
        ("damage_model::blend_wear_figures", 0xf3dff4eed52b6072),
        (
            "damage_model::blend_wheel_loads at rest",
            0x6d5bc02de4751f63,
        ),
        ("damage_model::blend_wheel_loads", 0x043f7d36df02d5cc),
        ("damage_model::brake_share_step", 0x01b45014c44e4302),
        ("damage_model::brake_wear_step", 0xa58edb99db580f37),
        ("damage_model::drive_a_slip_step", 0xcd2f2352d6be1326),
        (
            "damage_model::drive_extrapolation_step with a separate carried figure",
            0x8fad3fc5706935b8,
        ),
        ("damage_model::drive_extrapolation_step", 0x51aac32af32624d8),
        ("damage_model::engine_damage_carry", 0x5123368f4ec10463),
        ("damage_model::engine_over_limit", 0x714b81976fa090ea),
        ("damage_model::engine_speed_share_step", 0x1dc6c9b51cadad29),
        ("damage_model::fail_wheel", 0xc8e8da73a9158914),
        ("damage_model::finish_shares", 0x9fc66479f68b642f),
        ("damage_model::gauge_totals", 0x4d61077363b7b454),
        ("damage_model::gear_wear_from_speed", 0x6f66bfab340dafc1),
        ("damage_model::hit_figures", 0xd873ac7230fd77d3),
        ("damage_model::hit_size_and_dive", 0x0e143904751edb93),
        ("damage_model::lerp_gap", 0xceda1e81eb53fa25),
        ("damage_model::means_and_engine_speed", 0xc8e98ddb88942dc2),
        ("damage_model::part_hit_figures", 0xe0f487b9e28a1897),
        ("damage_model::scale_car_figures", 0xa1bc4cf7bfd2237e),
        ("damage_model::scale_hit_size", 0x11581deabc7d6879),
        ("damage_model::scale_part_hit_figures", 0xdc1988cdf8763988),
        ("damage_model::speed_share_step", 0x4f0b1424d8006c60),
        ("damage_model::square_dive", 0xd24f64fbde41a216),
        ("damage_model::square_into another cell", 0x9a1154a17afc77a7),
        ("damage_model::square_into", 0x533b8c3fd38c145a),
        ("damage_model::subtract_rate", 0xb003031a3d9caba7),
        ("damage_model::subtract_wear", 0x8489991ba7ae7357),
        ("damage_model::the touch-down load", 0x1855350b49096790),
        (
            "damage_model::wheel_ride_height_from_share",
            0xf513add342443458,
        ),
        (
            "damage_model::wheel_stiffness_from_share",
            0x3f94658625f5c30b,
        ),
    ];
}
