//! The particles' state: the puff's work, the emitters' variables, the burst colours and the kinds' tuning.

use crate::effects::blocks_store::Particle;
use crate::effects::emit_exhaust_smoke_store::ExhaustSet;
use crate::effects::emit_rolling_dust_store::{RollingChoice, RollingSet};
use crate::effects::emit_rolling_particles_rated_store::RollingRatedSet;
use crate::effects::emit_slip_snow_cloud_store::SlipSprayLongSet;
use crate::effects::emit_slip_spray_store::SlipSpraySet;
use crate::effects::emit_twin_particle_first_store::{TwinState, TwinTuning};
use crate::effects::emit_wheel_dust_cloud_store::WheelFadeSet;
use crate::effects::emit_wheel_grit_bursts_store::BurstTuning;
use crate::memory::session::{PlainBytes, PlainFloat};

/// The puff's work values.
#[derive(rc2k_derive::Cells)]
pub struct PuffWork {
    /// The puff's place, size and motion
    pub puff: [PlainFloat; 6],
}

impl PuffWork {
    /// The values the game starts with.
    pub const fn new() -> Self {
        PuffWork {
            puff: [
                PlainFloat::new(0.0),
                PlainFloat::new(0.0),
                PlainFloat::new(0.0),
                PlainFloat::new(0.0),
                PlainFloat::new(0.0),
                PlainFloat::new(0.0),
            ],
        }
    }
}

/// The particles' state, one part of the effects' owner (`effects::owner::Effects`): the
/// ring's entries, the puff's work, and the emitters' variables. The constants nothing
/// writes (`PARTICLE_TUNING`, `TRAIL_PALETTE`, `BURST_COLOURS`) stay statics.
pub struct Particles {
    pub ring: crate::memory::owned::Owned<Vec<Particle>>,
    pub puff_work: PuffWork,
    pub kind1_choice: RollingChoice,
    pub kind1_set: RollingSet,
    pub kind5_set: SlipSpraySet,
    pub kind6_set: WheelFadeSet,
    pub kind11_set: SlipSprayLongSet,
    pub kind13_set: ExhaustSet,
    pub kind21_set: RollingRatedSet,
    pub twin_state: TwinState,
    pub nineteenth_tuning: TwinTuning,
    pub twenty_second_tuning: TwinTuning,
    pub first_tuning: BurstTuning,
    pub second_tuning: BurstTuning,
}

impl Particles {
    pub const fn new() -> Self {
        Particles {
            ring: crate::memory::owned::Owned::new("no particle ring"),
            puff_work: PuffWork::new(),
            kind1_choice: RollingChoice::new(),
            kind1_set: RollingSet::new(),
            kind5_set: SlipSpraySet::new(),
            kind6_set: WheelFadeSet::new(),
            kind11_set: SlipSprayLongSet::new(),
            kind13_set: ExhaustSet::new(),
            kind21_set: RollingRatedSet::new(),
            twin_state: TwinState::new(),
            nineteenth_tuning: TwinTuning::nineteenth(),
            twenty_second_tuning: TwinTuning::twenty_second(),
            first_tuning: BurstTuning::first(),
            second_tuning: BurstTuning::second(),
        }
    }
}

/// The ramps' multipliers: ramp `r` holds `step * RAMPS[r]` for the steps 0 .. 255 (the burst
/// particle colours).
const fn burst_colours() -> [u8; 0x2000] {
    const RAMPS: [u32; 8] = [0x10000, 0x10100, 0x100, 0x101, 1, 0x10001, 0x10101, 0x10101];
    let mut table = [0u8; 0x2000];
    let mut i = 0;
    while i < 0x800 {
        let v = ((i & 255) as u32).wrapping_mul(RAMPS[i >> 8]).to_le_bytes();
        let mut k = 0;
        while k < 4 {
            table[4 * i + k] = v[k];
            k += 1;
        }
        i += 1;
    }
    table
}

/// The burst particles' colour ramps.
#[derive(rc2k_derive::Cells)]
pub struct BurstColours {
    /// 8 ramps of 256 dwords (a channel mask times the step)
    pub table: PlainBytes<8192>,
}

impl BurstColours {
    /// BurstColours's first values.
    pub const fn new() -> Self {
        Self {
            table: PlainBytes::new(burst_colours()),
        }
    }
}

/// The particle effects' constants (`run_particles`, `trail_particles`, `burst_particles`,
/// `add_drift_particle`).
#[derive(rc2k_derive::Cells)]
pub struct ParticleTuning {
    /// The puff's x wave (three factors)
    pub puff_x: [PlainFloat; 3],
    /// The puff's z wave
    pub puff_z: [PlainFloat; 3],
    /// The puff's y (add, factor, add)
    pub puff_y: [PlainFloat; 3],
    /// The puff's motion factors
    pub puff_m: [PlainFloat; 2],
    /// An added drift particle's last y is less this
    pub drift_lower: PlainFloat,
    /// The trail's random radius factor
    pub trail_random_k: PlainFloat,
    /// Per axis (x, y, z) the trail's two motion factors
    pub trail_axes: [PlainFloat; 6],
    /// The trail's spread
    pub trail_spread: PlainFloat,
    /// The trail's size factor and base
    pub trail_size_k: [PlainFloat; 2],
    /// The trail's life factor
    pub trail_life_k: PlainFloat,
    /// The burst radius' base and scale
    pub burst_scale: [PlainFloat; 2],
    /// The burst particle's life factor
    pub burst_life: PlainFloat,
    /// Kind 0's gain, int gain and least
    pub dust: [PlainFloat; 3],
    /// Kind 4's gain and int gain
    pub trail: [PlainFloat; 2],
    /// Kind 8's lift gain, three dampings, colour gain
    pub burst8: [PlainFloat; 5],
    /// Kind 9's, the same
    pub burst9: [PlainFloat; 5],
}

impl ParticleTuning {
    /// ParticleTuning's first values.
    pub const fn new() -> Self {
        Self {
            puff_x: [
                PlainFloat::new(std::f32::consts::PI),
                PlainFloat::new(2.0),
                PlainFloat::new(2621440.0),
            ],
            puff_z: [
                PlainFloat::new(std::f32::consts::PI),
                PlainFloat::new(2.0),
                PlainFloat::new(2621440.0),
            ],
            puff_y: [
                PlainFloat::new(1.0),
                PlainFloat::new(2000.0),
                PlainFloat::new(500.0),
            ],
            puff_m: [PlainFloat::new(0.0001), PlainFloat::new(0.0001)],
            drift_lower: PlainFloat::new(1000.0),
            trail_random_k: PlainFloat::new(-0.5),
            trail_axes: [
                PlainFloat::new(0.001),
                PlainFloat::new(0.1),
                PlainFloat::new(0.001),
                PlainFloat::new(0.1),
                PlainFloat::new(0.001),
                PlainFloat::new(0.1),
            ],
            trail_spread: PlainFloat::new(256.0),
            trail_size_k: [PlainFloat::new(16.0), PlainFloat::new(2048.0)],
            trail_life_k: PlainFloat::new(256.0),
            burst_scale: [PlainFloat::new(1.0), PlainFloat::new(1024.0)],
            burst_life: PlainFloat::new(256.0),
            dust: [
                PlainFloat::new(-1.2),
                PlainFloat::new(f32::NEG_INFINITY),
                PlainFloat::new(-1000.0),
            ],
            trail: [PlainFloat::new(-0.1), PlainFloat::new(f32::NEG_INFINITY)],
            burst8: [
                PlainFloat::new(-0.1),
                PlainFloat::new(0.1),
                PlainFloat::new(0.1),
                PlainFloat::new(0.1),
                PlainFloat::new(f32::NEG_INFINITY),
            ],
            burst9: [
                PlainFloat::new(-0.1),
                PlainFloat::new(0.1),
                PlainFloat::new(0.1),
                PlainFloat::new(0.1),
                PlainFloat::new(f32::NEG_INFINITY),
            ],
        }
    }
}
