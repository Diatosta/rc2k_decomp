//! `effects::run_particles`: the effects' particle ring's frame.
//! Nothing when the settings' +4 is 0; else `effects::set_quad_uvs`, then the ring's used count
//! (as it is at the start) entries from the ring's first free one along +4:
//! - the life (+0x44) less the frame's ticks; the entry dies when that borrows, or when its y
//!   (+0x10) is below its ground (+0x24), or when its kind's step gives 1;
//! - the kinds' steps (each the place moved by the motion times the ticks): 0 (dust: +0x2C
//!   gains the `dust` gain a tick, +0x3C its whole-number gain; at or above its least
//!   `effects::burst_particles` and death, else `effects::trail_particles` while it rises,
//!   the last place kept), 4 (trail: the `trail` gains; drawn with a colour from `colours`
//!   by the life times +0x40), 8 and 9 (burst: +0x10 also moved by +0x30, which gains the
//!   `burst8` / `burst9` gain, the motion damped by their dampings, colour from the burst
//!   colour ramps by +0x48 + life times +0x40, none: death; 9 one time in 128 drawn 0x386A0
//!   bigger), each drawn by `effects::draw_particle`;
//! - a dead entry: the ring's used count less 1, and the entry moved to just before the ring's
//!   first free one (or that moved on past it when it is the entry).
//!
//! Every eighth effects step: a flag is set and a drift puff is made near the view (its place at
//! +0x30) from two steps of the effects' random by `effects::add_drift_particle`.
//!
//! Works on the effects settings.
//!
//! Refuses null settings, and, at the particle it reaches, a null entry or link, a kind other
//! than 0, 4, 8, 9 and a colour index out of its table, and its callees' own refusals (a broken
//! ring). A particle is updated in place under one lock of the ring.

use crate::effects::blocks::Particle;
use crate::effects::emit_exhaust_smoke::ExhaustSet;
use crate::effects::emit_rolling_dust::{RollingChoice, RollingSet};
use crate::effects::emit_rolling_particles_rated::RollingRatedSet;
use crate::effects::emit_slip_snow_cloud::SlipSprayLongSet;
use crate::effects::emit_slip_spray::SlipSpraySet;
use crate::effects::emit_twin_particle_first::{TwinState, TwinTuning};
use crate::effects::emit_wheel_dust_cloud::WheelFadeSet;
use crate::effects::emit_wheel_grit_bursts::BurstTuning;
use crate::effects::owner::Effects;
use crate::effects::ring_store::Ring;
use crate::errors::{BadInput, Refused};
use crate::math::matrix3::extended;
use crate::math::x87;
use crate::math::x87_float as bh;
use crate::memory::session::{PlainBytes, PlainFloat};
use crate::race::owner_frame::Frame;
use crate::sound::owner::Sound;
const TRAIL_COLOURS: i32 = 298;
const BURST_COLOURS_LEN: i32 = 0x2000;
const BIGGER: u32 = 0x386A0;

/// The dwords of a particle by name (its offset in the record over 4).
mod at {
    pub const PREV: usize = 0;
    pub const NEXT: usize = 1;
    pub const KIND: usize = 2;
    pub const X: usize = 3;
    pub const Y: usize = 4;
    pub const Z: usize = 5;
    pub const LAST_X: usize = 6;
    pub const LAST_Y: usize = 7;
    pub const LAST_Z: usize = 8;
    pub const GROUND: usize = 9;
    pub const MOTION_X: usize = 10;
    pub const MOTION_Y: usize = 11;
    pub const LIFT: usize = 12;
    pub const MOTION_Z: usize = 13;
    pub const SIZE: usize = 14;
    pub const COLOUR: usize = 15;
    pub const COLOUR_STEP: usize = 16;
    pub const LIFE: usize = 17;
    pub const COLOURS_PLACE: usize = 18;
}

/// A kind's colour index (life times the colour's step), checked against its table.
fn colour_index(p: &Particle, life: u32) -> Result<i32, BadInput> {
    let i = x87::int_times_to_int(life as i32, p[at::COLOUR_STEP]);
    let ok = match p[at::KIND] {
        4 => (0..TRAIL_COLOURS).contains(&i),
        _ => {
            let off = (p[at::COLOURS_PLACE] as i32).wrapping_add(i.wrapping_mul(4));
            i >= 0 && (0..=BURST_COLOURS_LEN - 4).contains(&off)
        }
    };
    if ok {
        Ok(i)
    } else {
        Err(BadInput("a particle colour index out of its table"))
    }
}

/// What the walk does with a particle after its update, outside the ring's lock (these touch
/// the ring or other stores).
enum Outcome {
    /// Dies.
    Dies,
    /// Dust past its least: bursts, then dies.
    Burst,
    /// Dust still rising: leaves a trail particle, then keeps its place as the last one.
    Trail,
    /// Drawn at `place`.
    Draw {
        place: [[u8; 10]; 3],
        colour: u32,
        size: u32,
    },
    /// Nothing more.
    Settled,
}

/// The place moved by the motion times the ticks `t` (the lift too for the burst kinds).
fn move_place(p: &mut Particle, t: i32, with_up: bool) {
    p[at::X] = x87::plus_times_int(p[at::X], p[at::MOTION_X], t).to_bits();
    p[at::Y] = if with_up {
        bh::st((bh::ld(p[at::MOTION_Y]) + bh::ld(p[at::LIFT])) * bh::int(t) + bh::ld(p[at::Y]))
    } else {
        x87::plus_times_int(p[at::Y], p[at::MOTION_Y], t).to_bits()
    };
    p[at::Z] = x87::plus_times_int(p[at::Z], p[at::MOTION_Z], t).to_bits();
}

/// The colour gain: the tuning cell holds a whole number, so its bits are read as one.
fn gain_colour(p: &mut Particle, k: &PlainFloat, t: i32) {
    p[at::COLOUR] = x87::int_times_int_plus_int(k.bits() as i32, t, p[at::COLOUR] as i32) as u32;
}

fn keep_last(p: &mut Particle) {
    p[at::LAST_X] = p[at::X];
    p[at::LAST_Y] = p[at::Y];
    p[at::LAST_Z] = p[at::Z];
}

fn place(p: &Particle) -> [[u8; 10]; 3] {
    [extended(p[at::X]), extended(p[at::Y]), extended(p[at::Z])]
}

/// The update of a particle by its kind (see the header), in place; what to do after it.
fn advance(p: &mut Particle, effects: &Effects, ring: &Ring) -> Result<Outcome, BadInput> {
    let t: i32 = effects.setup.clock.frame_ticks.get() as i32;
    let tuning = &effects.kinds.particle_tuning;
    Ok(match p[at::KIND] {
        0 => {
            move_place(p, t, false);
            p[at::MOTION_Y] = x87::plus_times_int(p[at::MOTION_Y], &tuning.dust[0], t).to_bits();
            gain_colour(p, &tuning.dust[1], t);
            let s = x87::compare_status(&tuning.dust[2], p[at::MOTION_Y]);
            effects.setup.work.fpu_status.set(s as u32);
            if s != 0x100 {
                Outcome::Burst
            } else if p[at::MOTION_Y] & 0x8000_0000 == 0 {
                Outcome::Trail
            } else {
                keep_last(p);
                Outcome::Settled
            }
        }
        4 => {
            move_place(p, t, false);
            p[at::MOTION_Y] = x87::plus_times_int(p[at::MOTION_Y], &tuning.trail[0], t).to_bits();
            gain_colour(p, &tuning.trail[1], t);
            let index = colour_index(p, p[at::LIFE])?;
            ring.pool.index.set(index as u32);
            let alpha = p[at::COLOUR] & 0xFF00_0000;
            let colour = (TRAIL_PALETTE.colours[index as usize] & 0xFF_FFFF) | alpha;
            let drawn = Outcome::Draw {
                place: place(p),
                colour,
                size: p[at::SIZE],
            };
            keep_last(p);
            drawn
        }
        kind @ (8 | 9) => {
            let k = if kind == 9 {
                &tuning.burst9
            } else {
                &tuning.burst8
            };
            move_place(p, t, true);
            p[at::LIFT] = x87::plus_times_int(p[at::LIFT], &k[0], t).to_bits();
            p[at::MOTION_X] = x87::less_own_times(p[at::MOTION_X], &k[1]).to_bits();
            p[at::MOTION_Y] = x87::less_own_times(p[at::MOTION_Y], &k[2]).to_bits();
            p[at::MOTION_Z] = x87::less_own_times(p[at::MOTION_Z], &k[3]).to_bits();
            gain_colour(p, &k[4], t);
            let index = colour_index(p, p[at::LIFE])?;
            ring.pool.index.set(index as u32);
            let tint = effects
                .kinds
                .burst_colours
                .table
                .get_u32((p[at::COLOURS_PLACE] as usize).wrapping_add(4 * index as usize))
                & 0xFF_FFFF;
            if tint == 0 {
                return Ok(Outcome::Dies);
            }
            let mut size = p[at::SIZE];
            if kind == 9 {
                let s = effects
                    .setup
                    .work
                    .seed
                    .get()
                    .wrapping_mul(0x9D)
                    .wrapping_add(0x47);
                effects.setup.work.seed.set(s);
                if s.rotate_right(16) & 0x7F == 0 {
                    size = size.wrapping_add(BIGGER);
                }
            }
            Outcome::Draw {
                place: place(p),
                colour: (p[at::COLOUR] & 0xFF00_0000) | tint,
                size,
            }
        }
        _ => return Err(BadInput("a particle of another kind")),
    })
}

/// A dead particle (number `e`) taken out of the walk's order: the ring's first free one moves
/// on past it when it is that one, else it is unlinked and linked in just before the first
/// free one, so the live ones stay together from there on. The ring's new first free one.
fn retire(r: &mut [Particle], first: u32, e: u32) -> Result<u32, BadInput> {
    let bad = || BadInput("a particle past the ring");
    let len = r.len();
    let slot = |n: u32| {
        (n as usize)
            .checked_sub(1)
            .filter(|&i| i < len)
            .ok_or_else(bad)
    };
    let (e_slot, first_slot) = (slot(e)?, slot(first)?);
    if e == first {
        return Ok(r[e_slot][at::NEXT]);
    }
    let (before, after) = (r[e_slot][at::PREV], r[e_slot][at::NEXT]);
    let (before_slot, after_slot) = (slot(before)?, slot(after)?);
    r[before_slot][at::NEXT] = after;
    r[after_slot][at::PREV] = before;
    let last = r[first_slot][at::PREV];
    let last_slot = slot(last)?;
    r[first_slot][at::PREV] = e;
    r[last_slot][at::NEXT] = e;
    r[e_slot][at::PREV] = last;
    r[e_slot][at::NEXT] = first;
    Ok(first)
}

#[allow(
    clippy::too_many_arguments,
    reason = "the effects' groups, the owners and the vertex buffer"
)]
pub fn run_particles(
    buffer: &crate::d3d::vertices_store::PolygonVertices,
    frame: &Frame,
    particles: &Particles,
    ring: &Ring,
    scene: &crate::scene::owner::Scene,
    effects: &crate::effects::owner::Effects,
    sound: &Sound,
    view: crate::effects::run_frame::EffectsView<'_>,
) -> Result<(), Refused> {
    use crate::effects::{burst_particles, draw_particle, trail_particles};
    if effects.setup.settings.is_set.get() == 0 {
        return Err(BadInput("no marks settings").into());
    }
    {
        let marks = effects.settings()?;
        if marks.race_done.get() == 0 {
            return Ok(());
        }
        crate::effects::set_quad_uvs::set_quad_uvs(buffer, scene, effects);
        let mut e: u32 = ring.pool.free.get();
        let mut counter = ring.pool.used.get() as i32;
        loop {
            counter = counter.wrapping_sub(1);
            if counter < 0 {
                break;
            }
            let t: u32 = effects.setup.clock.frame_ticks.get();
            // One lock for the particle's whole update; what reaches the ring again (the
            // callees that add particles) runs after it.
            let (next, outcome) = particles.ring.with(|r| -> Result<_, BadInput> {
                let p = (e as usize)
                    .checked_sub(1)
                    .and_then(|n| r.get_mut(n))
                    .ok_or(BadInput("a particle past the ring"))?;
                let next = p[at::NEXT];
                let life = p[at::LIFE];
                p[at::LIFE] = life.wrapping_sub(t);
                if life < t {
                    return Ok((next, Outcome::Dies));
                }
                let s = x87::compare_status(p[at::Y], p[at::GROUND]);
                effects.setup.work.fpu_status.set(s as u32);
                if s == 0x100 {
                    return Ok((next, Outcome::Dies));
                }
                Ok((next, advance(p, effects, ring)?))
            })??;
            let dies = match outcome {
                Outcome::Dies => true,
                Outcome::Burst => {
                    burst_particles::burst_particles(
                        view.engine_tables,
                        particles,
                        ring,
                        e,
                        effects,
                        sound,
                    )?;
                    true
                }
                Outcome::Trail => {
                    trail_particles::trail_particles(frame, particles, ring, e, effects)?;
                    particles.ring.with(|r| {
                        (e as usize)
                            .checked_sub(1)
                            .and_then(|n| r.get_mut(n))
                            .map(keep_last)
                    })?;
                    false
                }
                Outcome::Draw {
                    place,
                    colour,
                    size,
                } => {
                    draw_particle::draw_particle(
                        buffer,
                        view.view_matrix,
                        place,
                        colour,
                        size,
                        scene,
                        effects,
                    );
                    false
                }
                Outcome::Settled => false,
            };
            if dies {
                ring.pool.used.set(ring.pool.used.get().wrapping_sub(1));
                let first = ring.pool.free.get();
                let new_first = particles.ring.with(|r| retire(r, first, e))??;
                ring.pool.free.set(new_first);
            }
            e = next;
        }
        if effects.setup.frame.steps.get() & 7 != 0 {
            return Ok(());
        }
        sound.race.stage.get().random[0].set(1);
        let seed = || {
            let s = effects
                .setup
                .work
                .seed
                .get()
                .wrapping_mul(0x9D)
                .wrapping_add(0x47);
            effects.setup.work.seed.set(s);
            s & 0xFFFF
        };
        let r = seed();
        let view = |o: usize| frame.matrices.get().view_place[o / 4].get();
        let uv_k: f32 = effects.frame.draw_work.uv_k.get();
        let wave = |k: &[crate::memory::session::PlainFloat; 3], add: f32, cosine: bool| {
            x87::int_times3_wave_plus(r as i32, [uv_k, k[0].get(), k[1].get()], &k[2], add, cosine)
        };
        let x = wave(&effects.kinds.particle_tuning.puff_x, view(0x30), false);
        let y = x87::loaded(view(0x34));
        let z = wave(&effects.kinds.particle_tuning.puff_z, view(0x38), true);
        let lift = seed();
        let up = f32::from_bits(bh::st(
            (bh::int(lift as i32) * bh::ld(uv_k)
                + bh::ld(effects.kinds.particle_tuning.puff_y[0].get()))
                * bh::ld(effects.kinds.particle_tuning.puff_y[1].get())
                + bh::ld(effects.kinds.particle_tuning.puff_y[2].get()),
        ));
        let mx = x87::sub_mul(view(0x30), x, &effects.kinds.particle_tuning.puff_m[0]);
        let mz = x87::sub_mul(view(0x38), z, &effects.kinds.particle_tuning.puff_m[1]);
        for (n, v) in [x, y, z, mx, up, mz].into_iter().enumerate() {
            particles.puff_work.puff[n].set(v);
        }
        let six = [mz, up, mx, z, y, x].map(extended);
        crate::effects::add_drift_particle::add_drift_particle(
            frame, particles, ring, 0, six, effects,
        )?;
    }
    Ok(())
}

/// The trail particles' colours.
pub struct TrailColours {
    /// The trail particles' 298 colours (`effects::run_particles`).
    pub colours: [u32; 298],
}

pub static TRAIL_PALETTE: TrailColours = TrailColours {
    colours: [
        0xFF0000, 0xFF0200, 0xFF0400, 0xFF0600, 0xFF0800, 0xFF0A00, 0xFF0C00, 0xFF0E00, 0xFF1000,
        0xFF1200, 0xFF1400, 0xFF1600, 0xFF1800, 0xFF1A00, 0xFF1C00, 0xFF1E00, 0xFF2000, 0xFF2200,
        0xFF2400, 0xFF2600, 0xFF2800, 0xFF2A00, 0xFF2C00, 0xFF2E00, 0xFF3000, 0xFF3200, 0xFF3400,
        0xFF3600, 0xFF3800, 0xFF3A00, 0xFF3C00, 0xFF3E00, 0xFF4000, 0xFF4200, 0xFF4400, 0xFF4600,
        0xFF4800, 0xFF4A00, 0xFF4C00, 0xFF4E00, 0xFF5000, 0xFF5200, 0xFF5400, 0xFF5600, 0xFF5800,
        0xFF5A00, 0xFF5C00, 0xFF5E00, 0xFF6000, 0xFF6200, 0xFF6400, 0xFF6600, 0xFF6800, 0xFF6A00,
        0xFF6C00, 0xFF6E00, 0xFF7000, 0xFF7200, 0xFF7400, 0xFF7600, 0xFF7800, 0xFF7A00, 0xFF7C00,
        0xFF7E00, 0xFF8000, 0xFF8200, 0xFF8400, 0xFF8600, 0xFF8800, 0xFF8A00, 0xFF8C00, 0xFF8E00,
        0xFF9000, 0xFF9200, 0xFF9400, 0xFF9600, 0xFF9800, 0xFF9A00, 0xFF9C00, 0xFF9E00, 0xFFA000,
        0xFFA200, 0xFFA400, 0xFFA600, 0xFFA800, 0xFFAA00, 0xFFAC00, 0xFFAE00, 0xFFB000, 0xFFB200,
        0xFFB400, 0xFFB600, 0xFFB800, 0xFFBA00, 0xFFBC00, 0xFFBE00, 0xFFC000, 0xFFC200, 0xFFC400,
        0xFFC600, 0xFFC800, 0xFFCA00, 0xFFCC00, 0xFFCE00, 0xFFD000, 0xFFD200, 0xFFD400, 0xFFD600,
        0xFFD800, 0xFFDA00, 0xFFDC00, 0xFFDE00, 0xFFE000, 0xFFE200, 0xFFE400, 0xFFE600, 0xFFE800,
        0xFFEA00, 0xFFEC00, 0xFFEE00, 0xFFF000, 0xFFF200, 0xFFF400, 0xFFF600, 0xFFF800, 0xFFFA00,
        0xFFFC00, 0xFFFE00, 0xFFFF00, 0xFFFF02, 0xFFFF04, 0xFFFF06, 0xFFFF08, 0xFFFF0A, 0xFFFF0C,
        0xFFFF0E, 0xFFFF10, 0xFFFF12, 0xFFFF14, 0xFFFF16, 0xFFFF18, 0xFFFF1A, 0xFFFF1C, 0xFFFF1E,
        0xFFFF20, 0xFFFF22, 0xFFFF24, 0xFFFF26, 0xFFFF28, 0xFFFF2A, 0xFFFF2C, 0xFFFF2E, 0xFFFF30,
        0xFFFF32, 0xFFFF34, 0xFFFF36, 0xFFFF38, 0xFFFF3A, 0xFFFF3C, 0xFFFF3E, 0xFFFF40, 0xFFFF42,
        0xFFFF44, 0xFFFF46, 0xFFFF48, 0xFFFF4A, 0xFFFF4C, 0xFFFF4E, 0xFFFF50, 0xFFFF52, 0xFFFF54,
        0xFFFF56, 0xFFFF58, 0xFFFF5A, 0xFFFF5C, 0xFFFF5E, 0xFFFF60, 0xFFFF62, 0xFFFF64, 0xFFFF66,
        0xFFFF68, 0xFFFF6A, 0xFFFF6C, 0xFFFF6E, 0xFFFF70, 0xFFFF72, 0xFFFF74, 0xFFFF76, 0xFFFF78,
        0xFFFF7A, 0xFFFF7C, 0xFFFF7E, 0xFFFF80, 0xFFFF82, 0xFFFF84, 0xFFFF86, 0xFFFF88, 0xFFFF8A,
        0xFFFF8C, 0xFFFF8E, 0xFFFF90, 0xFFFF92, 0xFFFF94, 0xFFFF96, 0xFFFF98, 0xFFFF9A, 0xFFFF9C,
        0xFFFF9E, 0xFFFFA0, 0xFFFFA2, 0xFFFFA4, 0xFFFFA6, 0xFFFFA8, 0xFFFFAA, 0xFFFFAC, 0xFFFFAE,
        0xFFFFB0, 0xFFFFB2, 0xFFFFB4, 0xFFFFB6, 0xFFFFB8, 0xFFFFBA, 0xFFFFBC, 0xFFFFBE, 0xFFFFC0,
        0xFFFFC2, 0xFFFFC4, 0xFFFFC6, 0xFFFFC8, 0xFFFFCA, 0xFFFFCC, 0xFFFFCE, 0xFFFFD0, 0xFFFFD2,
        0xFFFFD4, 0xFFFFD6, 0xFFFFD8, 0xFFFFDA, 0xFFFFDC, 0xFFFFDE, 0xFFFFE0, 0xFFFFE2, 0xFFFFE4,
        0xFFFFE6, 0xFFFFE8, 0xFFFFEA, 0xFFFFEC, 0xFFFFEE, 0xFFFFF0, 0xFFFFF2, 0xFFFFF4, 0xFFFFF6,
        0xFFFFF8, 0xFFFFFA, 0xFFFFFC, 0xFFFFFE, 0xFFFFFF, 0xFFFFFF, 0xFFFFFF, 0xFFFFFF, 0xFFFFFF,
        0xFFFFFF, 0xFFFFFF, 0xFFFFFF, 0xFFFFFF, 0xFFFFFF, 0xFFFFFF, 0xFFFFFF, 0xFFFFFF, 0xFFFFFF,
        0xFFFFFF, 0xFFFFFF, 0xFFFFFF, 0xFFFFFF, 0xFFFFFF, 0xFFFFFF, 0xFFFFFF, 0xFFFFFF, 0xFFFFFF,
        0xFFFFFF, 0xFFFFFF, 0xFFFFFF, 0xFFFFFF, 0xFFFFFF, 0xFFFFFF, 0xFFFFFF, 0xFFFFFF, 0xFFFFFF,
        0xFFFFFF, 0xFFFFFF, 0xFFFFFF, 0xFFFFFF, 0xFFFFFF, 0xFFFFFF, 0xFFFFFF, 0xFFFFFF, 0xFFFFFF,
        0xFFFFFF,
    ],
};

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
    /// Kind 8's +0x30 gain, three dampings, +0x3C gain
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
