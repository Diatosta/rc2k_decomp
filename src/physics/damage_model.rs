//! The car's damage and wear model under `physics::damage_step`: `physics::damage_factors`,
//! `physics::damage_wear` and `physics::damage_gauges`. Their figures are the work car's (the
//! damage gauges `brake_heat` .. `tyre_wear`, the handling factors `wheel_ride_height` ..) and
//! the model's own (`DamageModel.chance_scale` ..: shares, tables of four (x0, y0, x1, y1) for
//! `physics::flow::table_lerp`, rates). What each figure is was worked out from its uses (see
//! the docs in `physics/store.rs`). The three steps are long straight runs: x87 runs as blocks,
//! the clamps as `hold_low` / `hold_high`, the table lerps and compares as `if`.
//!
//! This file holds the regions and the shared test.

use crate::memory::session::{PlainDword, PlainFloat};

/// The value of a float cell as the model's arithmetic holds it: a float loaded and worked at
/// double precision.
pub fn value(cell: &PlainFloat) -> f64 {
    f64::from(cell.get())
}

/// A result of that arithmetic stored into a float cell, rounded to a float.
pub fn store(cell: &PlainFloat, result: f64) {
    cell.set(result as f32);
}

/// The value of a model figure as the model's arithmetic holds it.
pub fn figure(x: f32) -> f64 {
    f64::from(x)
}

/// A word test on a model figure: true unless every bit is zero.
pub fn figure_is_set(x: f32) -> bool {
    x.to_bits() != 0
}

/// `v` raised to `lim` when below it (or unordered).
pub fn held_low(v: &mut f32, lim: f32) {
    if crate::physics::flow::c0(*v, lim) {
        *v = crate::math::x87::loaded(lim);
    }
}

/// `v` lowered to `lim` unless below it (or unordered).
pub fn held_high(v: &mut f32, lim: f32) {
    if !crate::physics::flow::c0(*v, lim) {
        *v = crate::math::x87::loaded(lim);
    }
}

/// The clamped table lerp of `x` through a table of four (x0, y0, x1, y1).
pub fn lerp_table(x: f32, table: &[f32; 4], zero: f32) -> f32 {
    crate::physics::flow::table_lerp(x, table.map(f32::to_bits), zero)
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

    /// `fill` for the model, whose fields are plain.
    fn fill_model(model: &mut DamageModel, rng: &mut Rng) {
        for word in crate::physics::store::words_of_mut(model) {
            *word = random_bits(rng);
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
            [
                words(&*self.car),
                crate::physics::store::words_of(&self.damage).to_vec(),
                words(&*self.work),
            ]
        }
    }

    /// `same_result` with 20,000 runs and a seed made of the name, against the hash frozen
    /// for the name.
    pub fn prove(name: &str, new: impl Fn(&mut State) + Sync) {
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
    pub fn same_result(seed: u64, runs: u32, new: impl Fn(&mut State) + Sync) -> Digest {
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
        new: &impl Fn(&mut State),
    ) -> Vec<u64> {
        let mut b = State::new();
        runs.map(|_| {
            fill(&*b.car, &mut rng);
            fill_model(&mut b.damage, &mut rng);
            fill(&*b.work, &mut rng);
            new(&mut b);
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
        ("damage_model::add_hit_to_gear_gauge", 0x11f8db37b85d51ce),
        ("damage_model::add_product", 0x411bba95a497ba86),
        ("damage_model::add_rate_to_clutch", 0x0fdb256e9a6321fc),
        ("damage_model::add_rate_to_gauge", 0x31f04384593e5066),
        ("damage_model::add_scaled_triple", 0x150af4d569c2cc2f),
        ("damage_model::add_triple_product", 0x518e7dee7c54b648),
        (
            "damage_model::add_wear into another cell",
            0x8af035af60279c81,
        ),
        ("damage_model::add_wear", 0xec89ae00ca81a814),
        ("damage_model::blend_wear_figures", 0x9f10b11c95326c7b),
        (
            "damage_model::blend_wheel_loads at rest",
            0xe51e8026749b4af3,
        ),
        ("damage_model::blend_wheel_loads", 0xa908737bb52b2aed),
        ("damage_model::brake_share_step", 0xd5cc6b4047865999),
        ("damage_model::brake_wear_step", 0x7227f4aa693df1a9),
        ("damage_model::drive_a_slip_step", 0xecf9ad16f44a4f21),
        (
            "damage_model::drive_extrapolation_step with a separate carried figure",
            0x1ffabc1bb9e28915,
        ),
        ("damage_model::drive_extrapolation_step", 0xadc21ccda3ec1e19),
        ("damage_model::engine_damage_carry", 0xdea97d497afd8b6a),
        ("damage_model::engine_over_limit", 0x1b507f396a494b7d),
        ("damage_model::engine_speed_share_step", 0x1d27663d172da8cc),
        ("damage_model::fail_wheel", 0x8a3d04dfa1b61d33),
        ("damage_model::finish_shares", 0x8bd9ef8b2f66e777),
        ("damage_model::gauge_totals", 0xd7cfffa1b9331d78),
        ("damage_model::gear_wear_from_speed", 0x3d1bcaafe41001f4),
        ("damage_model::hit_figures", 0x68ea7af21b35a43a),
        ("damage_model::hit_size_and_dive", 0x4472be5e3ce21e50),
        ("damage_model::lerp_gap", 0x5448a264ab84b653),
        ("damage_model::means_and_engine_speed", 0x97e0f03c97789afc),
        ("damage_model::part_hit_figures", 0x6a260b41215e1ab7),
        ("damage_model::scale_car_figures", 0x5d4bc0894df6cdb4),
        ("damage_model::scale_hit_size", 0xe1f9bc751dab6234),
        ("damage_model::scale_part_hit_figures", 0xd12862292ad5c7c3),
        ("damage_model::speed_share_step", 0x16c9076b22fa9317),
        ("damage_model::square_dive", 0x3f25024a706b045c),
        ("damage_model::square_into another cell", 0x4ced68c142a12d8c),
        ("damage_model::square_into", 0x880ed04b838c35d3),
        ("damage_model::subtract_rate", 0xeddb5a8d03be7afd),
        ("damage_model::subtract_wear", 0xeb3c0cf88aa1ecc5),
        ("damage_model::the touch-down load", 0xfd7a21512cda4784),
        (
            "damage_model::wheel_ride_height_from_share",
            0x27220bfa69faf96d,
        ),
        (
            "damage_model::wheel_stiffness_from_share",
            0x2e3c6126b55a7366,
        ),
    ];
}
