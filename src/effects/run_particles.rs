//! `effects::run_particles` (0x4CFB49, with its chunk 0x4D02B4 .. 0x4D05BD): the marks'
//! particle ring's frame. Nothing when the settings' +4 is 0; else `effects::set_quad_uvs`,
//! then `g_marksRingUsed` (as it is at the start) entries from `g_marksRingFree` along +4:
//! - the life (+0x44) less `g_marksFrameTicks`; the entry dies when that borrows, or when
//!   its y (+0x10) is below its ground (+0x24) (`fcomp`, `g_fpuStatus` 0x100), or when
//!   its kind's step gives 1;
//! - the kinds' steps (each the place moved by the motion times the ticks): 0 (dust:
//!   +0x2C gains `flt_93C680` a tick, +0x3C `dword_93C684`; at or above `flt_93C688`
//!   `effects::burst_particles` and death, else `effects::trail_particles` while it rises,
//!   the last place kept), 4 (trail: `flt_93C68C`, `dword_93C690`; drawn with colour
//!   `dword_922B38`[life times +0x40]), 8 and 9 (burst: +0x10 also moved by +0x30, which
//!   gains `flt_93C694` / `flt_93C6A8`, the motion damped by `flt_93C698` .. / `flt_93C6AC`
//!   .., colour `dword_93A618`[+0x48 + life times +0x40], none: death; 9 one time in 128
//!   drawn 0x386A0 bigger), each drawn by `effects::draw_particle`;
//! - a dead entry: `g_marksRingUsed` less 1, and the entry moved to just before
//!   `g_marksRingFree` (or that moved on past it when it is the entry).
//!
//! Every eighth marks step (`dword_76513C`): `dword_5165E4` 1 and a drift puff near the
//! view (`flt_5EA3CC` + 0x30) from two steps of the marks' random by
//! `effects::add_drift_particle`.
//!
//! In:  nothing (`g_marksSettings`)
//! Out: nothing (its caller reads no register after it)
//!
//! Caller: `effects::run_frame` (0x4D165E). Refuses, before anything, null settings, a null
//! entry or link on the way, a kind other than 0, 4, 8, 9 (the original's stack goes wrong:
//! section 16) and a colour index out of its table; mid-way only its callees' own refusals
//! (a broken ring). its seven calls relocated), manual patch.

use crate::effects::blocks::{Particle, particle, set_particle};
use crate::effects::emit_bursts::BurstTuning;
use crate::effects::emit_kind_1::{Kind1Choice, Kind1Set};
use crate::effects::emit_particle_5::Kind5Set;
use crate::effects::emit_particle_6::Kind6Set;
use crate::effects::emit_particle_11::Kind11Set;
use crate::effects::emit_particle_13::Kind13Set;
use crate::effects::emit_particle_19::{TwinState, TwinTuning};
use crate::effects::emit_particle_21::Kind21Set;
use crate::effects::frame_state::DRAW_WORK;
use crate::effects::ring_store::Ring;
use crate::effects::setup::CLOCK;
use crate::effects::tuning::MARKS_FRAME;
use crate::effects::work_store::MARKS_WORK;
use crate::errors::{BadInput, Refused};
use crate::math::x87;
use crate::race::matrices_store::MATRICES;
use crate::scene::matrix3::extended;
use crate::session::{Bytes, Float};
const TRAIL_COLOURS: i32 = 298;
const BURST_COLOURS_LEN: i32 = 0x2000;
const BIGGER: u32 = 0x386A0;

/// A kind's colour index (life times +0x40, `fistp`) and whether it is in its table.
fn colour_index(particles: &Particles, e: u32, life: u32) -> Result<(i32, bool), Refused> {
    Ok({
        let i = x87::int_times_to_int(life as i32, particle(particles, e, 0x40)?);
        let ok = match particle(particles, e, 8)? {
            4 => (0..TRAIL_COLOURS).contains(&i),
            _ => {
                let off = (particle(particles, e, 0x48)? as i32).wrapping_add(i.wrapping_mul(4));
                i >= 0 && (0..=BURST_COLOURS_LEN - 4).contains(&off)
            }
        };
        (i, ok)
    })
}

/// Why it refuses, if it does: the walk as the original makes it, nothing written.
fn refusal(particles: &Particles, ring: &Ring) -> Result<Option<&'static str>, Refused> {
    Ok({
        let settings: u32 = crate::effects::setup::SETTINGS.is_set.get();
        if settings == 0 {
            return Ok(Some("no marks settings"));
        }
        let marks = crate::effects::setup::marks()?;
        if marks.race_done.get() == 0 {
            return Ok(None);
        }
        let ticks: u32 = CLOCK.frame_ticks.get();
        let mut e: u32 = ring.pool.free.get();
        let used = ring.pool.used.get() as i32;
        if used > 0 && (e == 0 || particle(particles, e, 0)? == 0) {
            return Ok(Some("a null particle ring entry"));
        }
        for _ in 0..used.max(0) {
            if e == 0 || particle(particles, e, 0)? == 0 || particle(particles, e, 4)? == 0 {
                return Ok(Some("a null particle ring entry or link"));
            }
            let life = particle(particles, e, 0x44)?;
            let kind = particle(particles, e, 8)?;
            if life >= ticks
                && x87::compare_status(particle(particles, e, 0x10)?, particle(particles, e, 0x24)?)
                    != 0x100
            {
                if ![0, 4, 8, 9].contains(&kind) {
                    return Ok(Some(
                        "a particle of another kind (the original's stack goes wrong)",
                    ));
                }
                if kind != 0 && !colour_index(particles, e, life - ticks)?.1 {
                    return Ok(Some("a particle colour index out of its table"));
                }
            }
            e = particle(particles, e, 4)?;
        }
        None
    })
}

/// The step of the kind at e (see the header): whether the entry dies.
fn step(particles: &Particles, ring: &Ring, e: u32) -> Result<bool, Refused> {
    use crate::effects::{burst_particles, draw_particle, trail_particles};
    {
        let t: i32 = CLOCK.frame_ticks.get() as i32;
        let move_place = |with_up: bool| -> Result<_, Refused> {
            set_particle(
                particles,
                e,
                0xC,
                x87::plus_times_int(
                    particle(particles, e, 0xC)?,
                    particle(particles, e, 0x28)?,
                    t,
                ),
            )?;
            let y = if with_up {
                x87::plus_sum_times_int(
                    particle(particles, e, 0x10)?,
                    particle(particles, e, 0x2C)?,
                    particle(particles, e, 0x30)?,
                    t,
                )
            } else {
                x87::plus_times_int(
                    particle(particles, e, 0x10)?,
                    particle(particles, e, 0x2C)?,
                    t,
                )
            };
            set_particle(particles, e, 0x10, y)?;
            set_particle(
                particles,
                e,
                0x14,
                x87::plus_times_int(
                    particle(particles, e, 0x14)?,
                    particle(particles, e, 0x34)?,
                    t,
                ),
            )?;
            Ok(())
        };
        let gain_int = |k: &crate::session::Float| -> Result<_, Refused> {
            let v = x87::int_times_int_plus_int(
                k.bits() as i32,
                t,
                particle(particles, e, 0x3C)? as i32,
            );
            set_particle(particles, e, 0x3C, v as u32)?;
            Ok(())
        };
        let keep_last = || -> Result<_, Refused> {
            set_particle(particles, e, 0x18, particle(particles, e, 0xC)?)?;
            set_particle(particles, e, 0x1C, particle(particles, e, 0x10)?)?;
            set_particle(particles, e, 0x20, particle(particles, e, 0x14)?)?;
            Ok(())
        };
        let place = || -> Result<_, Refused> {
            Ok({
                [
                    extended(particle(particles, e, 0xC)?),
                    extended(particle(particles, e, 0x10)?),
                    extended(particle(particles, e, 0x14)?),
                ]
            })
        };
        match particle(particles, e, 8)? {
            0 => {
                move_place(false)?;
                set_particle(
                    particles,
                    e,
                    0x2C,
                    x87::plus_times_int(
                        particle(particles, e, 0x2C)?,
                        PARTICLE_TUNING.dust[0].bits(),
                        t,
                    ),
                )?;
                gain_int(&PARTICLE_TUNING.dust[1])?;
                let s = x87::compare_status(
                    PARTICLE_TUNING.dust[2].bits(),
                    particle(particles, e, 0x2C)?,
                );
                MARKS_WORK.fpu_status.set(s as u32);
                if s != 0x100 {
                    burst_particles::burst_particles(particles, ring, e)?;
                    return Ok(true);
                }
                if particle(particles, e, 0x2C)? & 0x8000_0000 == 0 {
                    trail_particles::trail_particles(particles, ring, e)?;
                }
                keep_last()?;
            }
            4 => {
                move_place(false)?;
                set_particle(
                    particles,
                    e,
                    0x2C,
                    x87::plus_times_int(
                        particle(particles, e, 0x2C)?,
                        PARTICLE_TUNING.trail[0].bits(),
                        t,
                    ),
                )?;
                gain_int(&PARTICLE_TUNING.trail[1])?;
                let (i, _) = colour_index(particles, e, particle(particles, e, 0x44)?)?;
                ring.pool.index.set(i as u32);
                let alpha = particle(particles, e, 0x3C)? & 0xFF00_0000;
                let colour = (TRAIL_PALETTE.colours[i as usize] & 0xFF_FFFF) | alpha;
                draw_particle::draw_particle(place()?, colour, particle(particles, e, 0x38)?);
                keep_last()?;
            }
            kind => {
                let k = if kind == 9 {
                    &PARTICLE_TUNING.burst9
                } else {
                    &PARTICLE_TUNING.burst8
                };
                move_place(true)?;
                set_particle(
                    particles,
                    e,
                    0x30,
                    x87::plus_times_int(particle(particles, e, 0x30)?, k[0].bits(), t),
                )?;
                set_particle(
                    particles,
                    e,
                    0x28,
                    x87::less_own_times(particle(particles, e, 0x28)?, k[1].bits()),
                )?;
                set_particle(
                    particles,
                    e,
                    0x2C,
                    x87::less_own_times(particle(particles, e, 0x2C)?, k[2].bits()),
                )?;
                set_particle(
                    particles,
                    e,
                    0x34,
                    x87::less_own_times(particle(particles, e, 0x34)?, k[3].bits()),
                )?;
                gain_int(&k[4])?;
                let (i, _) = colour_index(particles, e, particle(particles, e, 0x44)?)?;
                ring.pool.index.set(i as u32);
                let tint = BURST_COLOURS
                    .table
                    .get_u32((particle(particles, e, 0x48)? as usize).wrapping_add(4 * i as usize))
                    & 0xFF_FFFF;
                if tint == 0 {
                    return Ok(true);
                }
                let colour = (particle(particles, e, 0x3C)? & 0xFF00_0000) | tint;
                let mut size = particle(particles, e, 0x38)?;
                if kind == 9 {
                    let s = MARKS_WORK.seed.get().wrapping_mul(0x9D).wrapping_add(0x47);
                    MARKS_WORK.seed.set(s);
                    if s.rotate_right(16) & 0x7F == 0 {
                        size = size.wrapping_add(BIGGER);
                    }
                }
                draw_particle::draw_particle(place()?, colour, size);
            }
        }
    }
    Ok(false)
}

/// The Rust version (see the header).
pub fn run_particles(particles: &Particles, ring: &Ring) -> Result<(), Refused> {
    if let Some(why) = refusal(particles, ring)? {
        return Err(BadInput(why).into());
    }
    {
        let marks = crate::effects::setup::marks()?;
        if marks.race_done.get() == 0 {
            return Ok(());
        }
        crate::effects::set_quad_uvs::set_quad_uvs();
        let mut e: u32 = ring.pool.free.get();
        let mut counter = ring.pool.used.get() as i32;
        loop {
            counter = counter.wrapping_sub(1);
            if counter < 0 {
                break;
            }
            let next = particle(particles, e, 4)?;
            let t: u32 = CLOCK.frame_ticks.get();
            let life = particle(particles, e, 0x44)?;
            set_particle(particles, e, 0x44, life.wrapping_sub(t))?;
            let mut dies = life < t;
            if !dies {
                let s = x87::compare_status(
                    particle(particles, e, 0x10)?,
                    particle(particles, e, 0x24)?,
                );
                MARKS_WORK.fpu_status.set(s as u32);
                dies = s == 0x100;
                if !dies {
                    dies = step(particles, ring, e)?;
                }
            }
            if dies {
                ring.pool.used.set(ring.pool.used.get().wrapping_sub(1));
                if e == ring.pool.free.get() {
                    ring.pool.free.set(particle(particles, e, 4)?);
                } else {
                    let (before, after) = (particle(particles, e, 0)?, particle(particles, e, 4)?);
                    set_particle(particles, before, 4, after)?;
                    set_particle(particles, after, 0, before)?;
                    let free: u32 = ring.pool.free.get();
                    let last = particle(particles, free, 0)?;
                    set_particle(particles, free, 0, e)?;
                    set_particle(particles, last, 4, e)?;
                    set_particle(particles, e, 0, last)?;
                    set_particle(particles, e, 4, free)?;
                }
            }
            e = next;
        }
        if MARKS_FRAME.steps.bits() & 7 != 0 {
            return Ok(());
        }
        crate::sound::owner::sound().race.stage.random[0].set(1);
        let seed = || {
            let s = MARKS_WORK.seed.get().wrapping_mul(0x9D).wrapping_add(0x47);
            MARKS_WORK.seed.set(s);
            s & 0xFFFF
        };
        let r = seed();
        let view = |o: usize| MATRICES.view_place[o / 4].bits();
        let uv_k: u32 = DRAW_WORK.uv_k.bits();
        let wave = |k: &[crate::session::Float; 3], add: u32, cosine: bool| {
            x87::int_times3_wave_plus(
                r as i32,
                [uv_k, k[0].bits(), k[1].bits()],
                k[2].bits(),
                add,
                cosine,
            )
        };
        let x = wave(&PARTICLE_TUNING.puff_x, view(0x30), false);
        let y = x87::loaded(view(0x34));
        let z = wave(&PARTICLE_TUNING.puff_z, view(0x38), true);
        let lift = seed();
        let up = x87::int_times_plus_times_plus(
            lift as i32,
            uv_k,
            PARTICLE_TUNING.puff_y[0].bits(),
            PARTICLE_TUNING.puff_y[1].bits(),
            PARTICLE_TUNING.puff_y[2].bits(),
        );
        let mx = x87::sub_mul(view(0x30), x, PARTICLE_TUNING.puff_m[0].bits());
        let mz = x87::sub_mul(view(0x38), z, PARTICLE_TUNING.puff_m[1].bits());
        for (n, v) in [x, y, z, mx, up, mz].into_iter().enumerate() {
            particles.puff_work.puff[n].set_bits(v);
        }
        let six = [mz, up, mx, z, y, x].map(extended);
        crate::effects::add_drift_particle::add_drift_particle(particles, ring, 0, six)?;
    }
    Ok(())
}

/// The trail particles' colours (IDA 0x922B38 .. 0x922FE0, the image's layout; phase 7, typed state).
#[repr(C)]
pub struct TrailColours {
    /// 0x922B38: `dword_922B38`: the trail particles' 298 colours (`effects::run_particles`).
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

#[cfg(test)]
impl TrailColours {
    pub const START: usize = 0x922B38;
    pub const LEN: usize = 0x4A8;
}

/// The puff's work values (IDA 0x93A5F0 .. 0x93A618, the image's layout; phase 7, typed state).
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct PuffWork {
    /// 0x93A5F0: Not named: 16 bytes (a gap, or unreferenced data)
    pub _gap_93a5f0: Bytes<16>,
    /// 0x93A600: `flt_93A600` .. `flt_93A614`: the puff's place, size and motion
    pub puff: [Float; 6],
}

impl PuffWork {
    /// The values the game starts with (the image's bytes).
    pub const fn new() -> Self {
        PuffWork {
            _gap_93a5f0: Bytes::new([
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x80, 0xC5, 0x00, 0x00,
                0x00, 0x00,
            ]),
            puff: [
                Float::from_bits(0x0),
                Float::from_bits(0x0),
                Float::from_bits(0x0),
                Float::from_bits(0x0),
                Float::from_bits(0x0),
                Float::from_bits(0x0),
            ],
        }
    }
}

impl PuffWork {
    pub const START: usize = 0x93A5F0;
    pub const LEN: usize = 0x28;
}

/// The particles' state, one part of the effects' owner (`effects::owner::Effects`): the
/// ring's entries, the puff's work, and the emitters' variables. The constants nothing
/// writes (`PARTICLE_TUNING`, `TRAIL_PALETTE`, `BURST_COLOURS`) stay statics.
pub struct Particles {
    pub ring: crate::memory::owned::Owned<Vec<Particle>>,
    pub puff_work: PuffWork,
    pub kind1_choice: Kind1Choice,
    pub kind1_set: Kind1Set,
    pub kind5_set: Kind5Set,
    pub kind6_set: Kind6Set,
    pub kind11_set: Kind11Set,
    pub kind13_set: Kind13Set,
    pub kind21_set: Kind21Set,
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
            kind1_choice: Kind1Choice::new(),
            kind1_set: Kind1Set::new(),
            kind5_set: Kind5Set::new(),
            kind6_set: Kind6Set::new(),
            kind11_set: Kind11Set::new(),
            kind13_set: Kind13Set::new(),
            kind21_set: Kind21Set::new(),
            twin_state: TwinState::new(),
            nineteenth_tuning: TwinTuning::nineteenth(),
            twenty_second_tuning: TwinTuning::twenty_second(),
            first_tuning: BurstTuning::first(),
            second_tuning: BurstTuning::second(),
        }
    }
}

#[cfg(test)]
mod typed_tests {
    use super::*;
    use core::mem::size_of;

    #[test]
    fn trail_colours_starts_as_the_games() {
        assert_eq!(size_of::<TrailColours>(), TrailColours::LEN);
        let image = crate::moved::image_bytes(TrailColours::START, TrailColours::LEN);
        let p = std::ptr::from_ref(&TRAIL_PALETTE).cast::<u8>();
        let mine: Vec<u8> = (0..TrailColours::LEN)
            .map(|i| unsafe { p.add(i).read() })
            .collect();
        assert!(
            mine == image,
            "TRAIL_PALETTE starts differently from the image"
        );
    }
}

#[cfg(test)]
mod puff_work_tests {
    use super::*;
    use core::mem::offset_of;

    fn bytes<T>(v: &T) -> Vec<u8> {
        let p = std::ptr::from_ref::<T>(v).cast::<u8>();
        (0..size_of::<T>())
            .map(|i| unsafe { p.add(i).read() })
            .collect()
    }

    #[test]
    fn puff_work_starts_as_the_games() {
        assert_eq!(size_of::<PuffWork>(), PuffWork::LEN);
        assert!(
            bytes(&crate::effects::owner::EFFECTS.particles.puff_work)
                == crate::moved::image_bytes(PuffWork::START, PuffWork::LEN),
            "PUFF_WORK starts differently from the image"
        );
        assert_eq!(offset_of!(PuffWork, puff), 0x93A600 - PuffWork::START);
    }
}

/// The ramps' multipliers: ramp `r` holds `step * RAMPS[r]` for the steps 0 .. 255 (the burst particle colours).
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

/// The burst particles' colour ramps (IDA 0x93A618 .. 0x93C618, the image's layout; phase 7, typed state).
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct BurstColours {
    /// 0x93A618: `dword_93A618`: 8 ramps of 256 dwords (a channel mask times the step)
    pub table: Bytes<8192>,
}

/// BurstColours's first values.
pub static BURST_COLOURS: BurstColours = BurstColours {
    table: Bytes::new(burst_colours()),
};

impl BurstColours {
    pub const START: usize = 0x93A618;
    pub const LEN: usize = 0x2000;
}

#[cfg(test)]
mod burst_colours_tests {
    use super::*;
    use core::mem::offset_of;

    fn bytes<T>(v: &T) -> Vec<u8> {
        let p = std::ptr::from_ref::<T>(v).cast::<u8>();
        (0..size_of::<T>())
            .map(|i| unsafe { p.add(i).read() })
            .collect()
    }

    #[test]
    fn burst_colours_starts_as_the_games() {
        assert_eq!(size_of::<BurstColours>(), BurstColours::LEN);
        assert!(
            bytes(&BURST_COLOURS)
                == crate::moved::image_bytes(BurstColours::START, BurstColours::LEN),
            "BURST_COLOURS starts differently from the image"
        );
        assert_eq!(
            offset_of!(BurstColours, table),
            0x93A618 - BurstColours::START
        );
    }
}

/// The particle effects' constants (`run_particles`, `trail_particles`, `burst_particles`, `add_drift_particle`) (IDA 0x93C618 .. 0x93C6C8, the image's layout; phase 7, typed state).
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct ParticleTuning {
    /// 0x93C618: `flt_93C618` .. `flt_93C620`: the puff's x wave (three factors)
    pub puff_x: [Float; 3],
    /// 0x93C624: `flt_93C624` .. `flt_93C62C`: the puff's z wave
    pub puff_z: [Float; 3],
    /// 0x93C630: `flt_93C630` .. `flt_93C638`: the puff's y (add, factor, add)
    pub puff_y: [Float; 3],
    /// 0x93C63C: `flt_93C63C`, `flt_93C640`: the puff's motion factors
    pub puff_m: [Float; 2],
    /// 0x93C644: `flt_93C644`: an added drift particle's last y is less this
    pub drift_lower: Float,
    /// 0x93C648: `flt_93C648`: the trail's random radius factor
    pub trail_random_k: Float,
    /// 0x93C64C: `flt_93C64C` .. `flt_93C660`: per axis (x, y, z) the trail's two motion factors
    pub trail_axes: [Float; 6],
    /// 0x93C664: `flt_93C664`: the trail's spread
    pub trail_spread: Float,
    /// 0x93C668: `flt_93C668`, `flt_93C66C`: the trail's size factor and base
    pub trail_size_k: [Float; 2],
    /// 0x93C670: `flt_93C670`: the trail's life factor
    pub trail_life_k: Float,
    /// 0x93C674: `flt_93C674`, `flt_93C678`: the burst radius' base and scale
    pub burst_scale: [Float; 2],
    /// 0x93C67C: `flt_93C67C`: the burst particle's life factor
    pub burst_life: Float,
    /// 0x93C680: `flt_93C680`, `dword_93C684`, `flt_93C688`: kind 0's gain, int gain and least
    pub dust: [Float; 3],
    /// 0x93C68C: `flt_93C68C`, `dword_93C690`: kind 4's gain and int gain
    pub trail: [Float; 2],
    /// 0x93C694: `flt_93C694` .. `dword_93C6A4`: kind 8's +0x30 gain, three dampings, +0x3C gain
    pub burst8: [Float; 5],
    /// 0x93C6A8: `flt_93C6A8` .. `dword_93C6B8`: kind 9's, the same
    pub burst9: [Float; 5],
    /// 0x93C6BC: Not named: 12 bytes (a gap, or unreferenced data)
    pub _gap_93c6bc: Bytes<12>,
}

/// ParticleTuning's first values.
pub static PARTICLE_TUNING: ParticleTuning = ParticleTuning {
    puff_x: [
        Float::from_bits(0x40490FDB),
        Float::from_bits(0x40000000),
        Float::from_bits(0x4A200000),
    ],
    puff_z: [
        Float::from_bits(0x40490FDB),
        Float::from_bits(0x40000000),
        Float::from_bits(0x4A200000),
    ],
    puff_y: [
        Float::from_bits(0x3F800000),
        Float::from_bits(0x44FA0000),
        Float::from_bits(0x43FA0000),
    ],
    puff_m: [Float::from_bits(0x38D1B717), Float::from_bits(0x38D1B717)],
    drift_lower: Float::from_bits(0x447A0000),
    trail_random_k: Float::from_bits(0xBF000000),
    trail_axes: [
        Float::from_bits(0x3A83126F),
        Float::from_bits(0x3DCCCCCD),
        Float::from_bits(0x3A83126F),
        Float::from_bits(0x3DCCCCCD),
        Float::from_bits(0x3A83126F),
        Float::from_bits(0x3DCCCCCD),
    ],
    trail_spread: Float::from_bits(0x43800000),
    trail_size_k: [Float::from_bits(0x41800000), Float::from_bits(0x45000000)],
    trail_life_k: Float::from_bits(0x43800000),
    burst_scale: [Float::from_bits(0x3F800000), Float::from_bits(0x44800000)],
    burst_life: Float::from_bits(0x43800000),
    dust: [
        Float::from_bits(0xBF99999A),
        Float::from_bits(0xFF800000),
        Float::from_bits(0xC47A0000),
    ],
    trail: [Float::from_bits(0xBDCCCCCD), Float::from_bits(0xFF800000)],
    burst8: [
        Float::from_bits(0xBDCCCCCD),
        Float::from_bits(0x3DCCCCCD),
        Float::from_bits(0x3DCCCCCD),
        Float::from_bits(0x3DCCCCCD),
        Float::from_bits(0xFF800000),
    ],
    burst9: [
        Float::from_bits(0xBDCCCCCD),
        Float::from_bits(0x3DCCCCCD),
        Float::from_bits(0x3DCCCCCD),
        Float::from_bits(0x3DCCCCCD),
        Float::from_bits(0xFF800000),
    ],
    _gap_93c6bc: Bytes::new([0; 12]),
};

impl ParticleTuning {
    pub const START: usize = 0x93C618;
    pub const LEN: usize = 0xB0;
}

#[cfg(test)]
mod particle_tuning_tests {
    use super::*;
    use core::mem::offset_of;

    fn bytes<T>(v: &T) -> Vec<u8> {
        let p = std::ptr::from_ref::<T>(v).cast::<u8>();
        (0..size_of::<T>())
            .map(|i| unsafe { p.add(i).read() })
            .collect()
    }

    #[test]
    fn particle_tuning_starts_as_the_games() {
        assert_eq!(size_of::<ParticleTuning>(), ParticleTuning::LEN);
        assert!(
            bytes(&PARTICLE_TUNING)
                == crate::moved::image_bytes(ParticleTuning::START, ParticleTuning::LEN),
            "PARTICLE_TUNING starts differently from the image"
        );
        assert_eq!(
            offset_of!(ParticleTuning, puff_x),
            0x93C618 - ParticleTuning::START
        );
        assert_eq!(
            offset_of!(ParticleTuning, puff_z),
            0x93C624 - ParticleTuning::START
        );
        assert_eq!(
            offset_of!(ParticleTuning, puff_y),
            0x93C630 - ParticleTuning::START
        );
        assert_eq!(
            offset_of!(ParticleTuning, puff_m),
            0x93C63C - ParticleTuning::START
        );
        assert_eq!(
            offset_of!(ParticleTuning, drift_lower),
            0x93C644 - ParticleTuning::START
        );
        assert_eq!(
            offset_of!(ParticleTuning, trail_random_k),
            0x93C648 - ParticleTuning::START
        );
        assert_eq!(
            offset_of!(ParticleTuning, trail_axes),
            0x93C64C - ParticleTuning::START
        );
        assert_eq!(
            offset_of!(ParticleTuning, trail_spread),
            0x93C664 - ParticleTuning::START
        );
        assert_eq!(
            offset_of!(ParticleTuning, trail_size_k),
            0x93C668 - ParticleTuning::START
        );
        assert_eq!(
            offset_of!(ParticleTuning, trail_life_k),
            0x93C670 - ParticleTuning::START
        );
        assert_eq!(
            offset_of!(ParticleTuning, burst_scale),
            0x93C674 - ParticleTuning::START
        );
        assert_eq!(
            offset_of!(ParticleTuning, burst_life),
            0x93C67C - ParticleTuning::START
        );
        assert_eq!(
            offset_of!(ParticleTuning, dust),
            0x93C680 - ParticleTuning::START
        );
        assert_eq!(
            offset_of!(ParticleTuning, trail),
            0x93C68C - ParticleTuning::START
        );
        assert_eq!(
            offset_of!(ParticleTuning, burst8),
            0x93C694 - ParticleTuning::START
        );
        assert_eq!(
            offset_of!(ParticleTuning, burst9),
            0x93C6A8 - ParticleTuning::START
        );
    }
}
