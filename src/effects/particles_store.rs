//! The particles' state: the puff's work, the emitters' variables, the burst colours and the kinds' tuning.

use crate::effects::blocks_store::Particle;
use crate::effects::emit_exhaust_smoke_store::ExhaustSet;
use crate::effects::emit_rolling_dust_store::RollingSet;
use crate::effects::emit_rolling_particles_rated_store::RollingRatedSet;
use crate::effects::emit_slip_spray_store::SlipSpraySet;
use crate::effects::emit_twin_particle_first_store::{TwinState, TwinTuning};
use crate::effects::emit_wheel_dust_cloud_store::WheelFadeSet;
use crate::effects::emit_wheel_grit_bursts_store::BurstTuning;
use std::cell::RefCell;

/// The puff's work values.
pub struct PuffWork {
    /// The puff's place, size and motion
    pub puff: [f32; 6],
}

impl PuffWork {
    /// The values the game starts with.
    pub const fn new() -> Self {
        PuffWork { puff: [0.0; 6] }
    }
}

/// The particles' state, one part of the effects' owner (`effects::owner::Effects`): the
/// ring's entries, the puff's work, and the emitters' variables. The constants nothing
/// writes (`PARTICLE_TUNING`, `TRAIL_PALETTE`, `BURST_COLOURS`) stay statics.
pub struct Particles {
    pub ring: crate::memory::owned::Owned<Vec<Particle>>,
    pub puff_work: RefCell<PuffWork>,
    /// `emit_rolling_particles_rated` runs when not 0 (`ring_frame` sets it from the rain's drops).
    pub kind1_choice: core::cell::Cell<u32>,
    pub kind1_set: RefCell<RollingSet>,
    pub kind5_set: RefCell<SlipSpraySet>,
    pub kind6_set: RefCell<WheelFadeSet>,
    pub kind11_set: RefCell<SlipSpraySet>,
    pub kind13_set: RefCell<ExhaustSet>,
    pub kind21_set: RefCell<RollingRatedSet>,
    pub twin_state: RefCell<TwinState>,
    pub nineteenth_tuning: RefCell<TwinTuning>,
    pub twenty_second_tuning: RefCell<TwinTuning>,
    pub first_tuning: RefCell<BurstTuning>,
    pub second_tuning: RefCell<BurstTuning>,
}

impl Particles {
    pub const fn new() -> Self {
        Particles {
            ring: crate::memory::owned::Owned::new("no particle ring"),
            puff_work: RefCell::new(PuffWork::new()),
            kind1_choice: core::cell::Cell::new(0),
            kind1_set: RefCell::new(RollingSet::new()),
            kind5_set: RefCell::new(SlipSpraySet::new()),
            kind6_set: RefCell::new(WheelFadeSet::new()),
            kind11_set: RefCell::new(SlipSpraySet::new()),
            kind13_set: RefCell::new(ExhaustSet::new()),
            kind21_set: RefCell::new(RollingRatedSet::new()),
            twin_state: RefCell::new(TwinState::new()),
            nineteenth_tuning: RefCell::new(TwinTuning::nineteenth()),
            twenty_second_tuning: RefCell::new(TwinTuning::twenty_second()),
            first_tuning: RefCell::new(BurstTuning::first()),
            second_tuning: RefCell::new(BurstTuning::second()),
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
pub struct BurstColours {
    /// 8 ramps of 256 dwords (a channel mask times the step)
    table: [u8; 8192],
}

impl BurstColours {
    /// BurstColours's first values.
    pub const fn new() -> Self {
        Self {
            table: burst_colours(),
        }
    }

    /// The little-endian dword at byte `offset`.
    pub fn get_u32(&self, offset: usize) -> u32 {
        u32::from_le_bytes(std::array::from_fn(|i| self.table[offset + i]))
    }
}

/// The particle effects' constants (`run_particles`, `trail_particles`, `burst_particles`,
/// `add_drift_particle`).
pub struct ParticleTuning {
    /// The puff's x wave (three factors)
    pub puff_x: [f32; 3],
    /// The puff's z wave
    pub puff_z: [f32; 3],
    /// The puff's y (add, factor, add)
    pub puff_y: [f32; 3],
    /// The puff's motion factors
    pub puff_m: [f32; 2],
    /// An added drift particle's last y is less this
    pub drift_lower: f32,
    /// The trail's random radius factor
    pub trail_random_k: f32,
    /// Per axis (x, y, z) the trail's two motion factors
    pub trail_axes: [f32; 6],
    /// The trail's spread
    pub trail_spread: f32,
    /// The trail's size factor and base
    pub trail_size_k: [f32; 2],
    /// The trail's life factor
    pub trail_life_k: f32,
    /// The burst radius' base and scale
    pub burst_scale: [f32; 2],
    /// The burst particle's life factor
    pub burst_life: f32,
    /// Kind 0's gain, int gain and least
    pub dust: [f32; 3],
    /// Kind 4's gain and int gain
    pub trail: [f32; 2],
    /// Kind 8's lift gain, three dampings, colour gain
    pub burst8: [f32; 5],
    /// Kind 9's, the same
    pub burst9: [f32; 5],
}

impl ParticleTuning {
    /// ParticleTuning's first values.
    pub const fn new() -> Self {
        Self {
            puff_x: [std::f32::consts::PI, 2.0, 2621440.0],
            puff_z: [std::f32::consts::PI, 2.0, 2621440.0],
            puff_y: [1.0, 2000.0, 500.0],
            puff_m: [0.0001, 0.0001],
            drift_lower: 1000.0,
            trail_random_k: -0.5,
            trail_axes: [0.001, 0.1, 0.001, 0.1, 0.001, 0.1],
            trail_spread: 256.0,
            trail_size_k: [16.0, 2048.0],
            trail_life_k: 256.0,
            burst_scale: [1.0, 1024.0],
            burst_life: 256.0,
            dust: [-1.2, f32::NEG_INFINITY, -1000.0],
            trail: [-0.1, f32::NEG_INFINITY],
            burst8: [-0.1, 0.1, 0.1, 0.1, f32::NEG_INFINITY],
            burst9: [-0.1, 0.1, 0.1, 0.1, f32::NEG_INFINITY],
        }
    }
}
