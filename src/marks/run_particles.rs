//! `marks::run_particles` (0x4CFB49, with its chunk 0x4D02B4 .. 0x4D05BD): the marks'
//! particle ring's frame. Nothing when the settings' +4 is 0; else `marks::set_quad_uvs`,
//! then `g_marksRingUsed` (as it is at the start) entries from `g_marksRingFree` along +4:
//! - the life (+0x44) less `g_marksFrameTicks`; the entry dies when that borrows, or when
//!   its y (+0x10) is below its ground (+0x24) (`fcomp`, `g_fpuStatus` 0x100), or when
//!   its kind's step gives 1;
//! - the kinds' steps (each the place moved by the motion times the ticks): 0 (dust:
//!   +0x2C gains `flt_93C680` a tick, +0x3C `dword_93C684`; at or above `flt_93C688`
//!   `marks::burst_particles` and death, else `marks::trail_particles` while it rises,
//!   the last place kept), 4 (trail: `flt_93C68C`, `dword_93C690`; drawn with colour
//!   `dword_922B38`[life times +0x40]), 8 and 9 (burst: +0x10 also moved by +0x30, which
//!   gains `flt_93C694` / `flt_93C6A8`, the motion damped by `flt_93C698` .. / `flt_93C6AC`
//!   .., colour `dword_93A618`[+0x48 + life times +0x40], none: death; 9 one time in 128
//!   drawn 0x386A0 bigger), each drawn by `marks::draw_particle`;
//! - a dead entry: `g_marksRingUsed` less 1, and the entry moved to just before
//!   `g_marksRingFree` (or that moved on past it when it is the entry).
//!
//! Every eighth marks step (`dword_76513C`): `dword_5165E4` 1 and a drift puff near the
//! view (`flt_5EA3CC` + 0x30) from two steps of the marks' random by
//! `marks::add_drift_particle`.
//!
//! In:  nothing (`g_marksSettings`)
//! Out: nothing (its caller reads no register after it)
//!
//! Caller: `marks::run_frame` (0x4D165E). Refuses, before anything, null settings, a null
//! entry or link on the way, a kind other than 0, 4, 8, 9 (the original's stack goes wrong:
//! section 16) and a colour index out of its table; mid-way only its callees' own refusals
//! (a broken ring). its seven calls relocated), manual patch.

use crate::errors::{BadInput, Refused};
use crate::marks::blocks::RING_POOL;
use crate::marks::frame_state::DRAW_WORK;
use crate::marks::setup::CLOCK;
use crate::marks::tuning::MARKS_FRAME;
use crate::marks::work_store::MARKS_WORK;
use crate::math::x87;
use crate::pointers::GamePtr;
use crate::race::matrices_store::MATRICES;
use crate::scene::matrix3::extended;
use crate::session::{Bytes, Dword, Float};
const TRAIL_COLOURS: i32 = 298;
const BURST_COLOURS_LEN: i32 = 0x2000;
const BIGGER: u32 = 0x386A0;

fn get(a: u32) -> Result<u32, BadInput> {
    GamePtr::new(a).read("a game field")
}

fn put(a: u32, v: u32) -> Result<(), BadInput> {
    GamePtr::new(a).write(v, "a game field")
}

/// A kind's colour index (life times +0x40, `fistp`) and whether it is in its table.
fn colour_index(e: u32, life: u32) -> Result<(i32, bool), Refused> {
    Ok({
        let i = x87::int_times_to_int(life as i32, get(e + 0x40)?);
        let ok = match get(e + 8)? {
            4 => (0..TRAIL_COLOURS).contains(&i),
            _ => {
                let off = (get(e + 0x48)? as i32).wrapping_add(i.wrapping_mul(4));
                i >= 0 && (0..=BURST_COLOURS_LEN - 4).contains(&off)
            }
        };
        (i, ok)
    })
}

/// Why it refuses, if it does: the walk as the original makes it, nothing written.
fn refusal() -> Result<Option<&'static str>, Refused> {
    Ok({
        let settings: u32 = crate::marks::setup::SETTINGS.pointer.get();
        if settings == 0 {
            return Ok(Some("no marks settings"));
        }
        if get(settings + 4)? == 0 {
            return Ok(None);
        }
        let ticks: u32 = CLOCK.frame_ticks.get();
        let mut e: u32 = RING_POOL.free.get();
        let used = RING_POOL.used.get() as i32;
        if used > 0 && (e == 0 || get(e)? == 0) {
            return Ok(Some("a null particle ring entry"));
        }
        for _ in 0..used.max(0) {
            if e == 0 || get(e)? == 0 || get(e + 4)? == 0 {
                return Ok(Some("a null particle ring entry or link"));
            }
            let life = get(e + 0x44)?;
            let kind = get(e + 8)?;
            if life >= ticks && x87::compare_status(get(e + 0x10)?, get(e + 0x24)?) != 0x100 {
                if ![0, 4, 8, 9].contains(&kind) {
                    return Ok(Some(
                        "a particle of another kind (the original's stack goes wrong)",
                    ));
                }
                if kind != 0 && !colour_index(e, life - ticks)?.1 {
                    return Ok(Some("a particle colour index out of its table"));
                }
            }
            e = get(e + 4)?;
        }
        None
    })
}

/// The step of the kind at e (see the header): whether the entry dies.
fn step(e: u32) -> Result<bool, Refused> {
    use crate::marks::{burst_particles, draw_particle, trail_particles};
    {
        let t: i32 = CLOCK.frame_ticks.get() as i32;
        let move_place = |with_up: bool| -> Result<_, Refused> {
            put(
                e + 0xC,
                x87::plus_times_int(get(e + 0xC)?, get(e + 0x28)?, t),
            )?;
            let y = if with_up {
                x87::plus_sum_times_int(get(e + 0x10)?, get(e + 0x2C)?, get(e + 0x30)?, t)
            } else {
                x87::plus_times_int(get(e + 0x10)?, get(e + 0x2C)?, t)
            };
            put(e + 0x10, y)?;
            put(
                e + 0x14,
                x87::plus_times_int(get(e + 0x14)?, get(e + 0x34)?, t),
            )?;
            Ok(())
        };
        let gain_int = |k: &crate::session::Float| -> Result<_, Refused> {
            let v = x87::int_times_int_plus_int(k.bits() as i32, t, get(e + 0x3C)? as i32);
            put(e + 0x3C, v as u32)?;
            Ok(())
        };
        let keep_last = || -> Result<_, Refused> {
            put(e + 0x18, get(e + 0xC)?)?;
            put(e + 0x1C, get(e + 0x10)?)?;
            put(e + 0x20, get(e + 0x14)?)?;
            Ok(())
        };
        let place = || -> Result<_, Refused> {
            Ok({
                [
                    extended(get(e + 0xC)?),
                    extended(get(e + 0x10)?),
                    extended(get(e + 0x14)?),
                ]
            })
        };
        match get(e + 8)? {
            0 => {
                move_place(false)?;
                put(
                    e + 0x2C,
                    x87::plus_times_int(get(e + 0x2C)?, PARTICLE_TUNING.dust[0].bits(), t),
                )?;
                gain_int(&PARTICLE_TUNING.dust[1])?;
                let s = x87::compare_status(PARTICLE_TUNING.dust[2].bits(), get(e + 0x2C)?);
                MARKS_WORK.fpu_status.set(s as u32);
                if s != 0x100 {
                    burst_particles::burst_particles(e)?;
                    return Ok(true);
                }
                if get(e + 0x2C)? & 0x8000_0000 == 0 {
                    trail_particles::trail_particles(e)?;
                }
                keep_last()?;
            }
            4 => {
                move_place(false)?;
                put(
                    e + 0x2C,
                    x87::plus_times_int(get(e + 0x2C)?, PARTICLE_TUNING.trail[0].bits(), t),
                )?;
                gain_int(&PARTICLE_TUNING.trail[1])?;
                let (i, _) = colour_index(e, get(e + 0x44)?)?;
                RING_POOL.index.set(i as u32);
                let alpha = get(e + 0x3C)? & 0xFF00_0000;
                let colour = (TRAIL_PALETTE.colours[i as usize].get() & 0xFF_FFFF) | alpha;
                draw_particle::draw_particle(place()?, colour, get(e + 0x38)?);
                keep_last()?;
            }
            kind => {
                let k = if kind == 9 {
                    &PARTICLE_TUNING.burst9
                } else {
                    &PARTICLE_TUNING.burst8
                };
                move_place(true)?;
                put(
                    e + 0x30,
                    x87::plus_times_int(get(e + 0x30)?, k[0].bits(), t),
                )?;
                put(e + 0x28, x87::less_own_times(get(e + 0x28)?, k[1].bits()))?;
                put(e + 0x2C, x87::less_own_times(get(e + 0x2C)?, k[2].bits()))?;
                put(e + 0x34, x87::less_own_times(get(e + 0x34)?, k[3].bits()))?;
                gain_int(&k[4])?;
                let (i, _) = colour_index(e, get(e + 0x44)?)?;
                RING_POOL.index.set(i as u32);
                let tint = BURST_COLOURS
                    .table
                    .get_u32((get(e + 0x48)? as usize).wrapping_add(4 * i as usize))
                    & 0xFF_FFFF;
                if tint == 0 {
                    return Ok(true);
                }
                let colour = (get(e + 0x3C)? & 0xFF00_0000) | tint;
                let mut size = get(e + 0x38)?;
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
pub fn run_particles() -> Result<(), Refused> {
    if let Some(why) = refusal()? {
        return Err(BadInput(why).into());
    }
    {
        let settings: u32 = crate::marks::setup::SETTINGS.pointer.get();
        if get(settings + 4)? == 0 {
            return Ok(());
        }
        crate::marks::set_quad_uvs::set_quad_uvs();
        let mut e: u32 = RING_POOL.free.get();
        let mut counter = RING_POOL.used.get() as i32;
        loop {
            counter = counter.wrapping_sub(1);
            if counter < 0 {
                break;
            }
            let next = get(e + 4)?;
            let t: u32 = CLOCK.frame_ticks.get();
            let life = get(e + 0x44)?;
            put(e + 0x44, life.wrapping_sub(t))?;
            let mut dies = life < t;
            if !dies {
                let s = x87::compare_status(get(e + 0x10)?, get(e + 0x24)?);
                MARKS_WORK.fpu_status.set(s as u32);
                dies = s == 0x100;
                if !dies {
                    dies = step(e)?;
                }
            }
            if dies {
                RING_POOL.used.set(RING_POOL.used.get().wrapping_sub(1));
                if e == RING_POOL.free.get() {
                    RING_POOL.free.set(get(e + 4)?);
                } else {
                    let (before, after) = (get(e)?, get(e + 4)?);
                    put(before + 4, after)?;
                    put(after, before)?;
                    let free: u32 = RING_POOL.free.get();
                    let last = get(free)?;
                    put(free, e)?;
                    put(last + 4, e)?;
                    put(e, last)?;
                    put(e + 4, free)?;
                }
            }
            e = next;
        }
        if MARKS_FRAME.steps.bits() & 7 != 0 {
            return Ok(());
        }
        crate::sound::store::STAGE_SOUNDS.random[0].set(1);
        let seed = || {
            let s = MARKS_WORK.seed.get().wrapping_mul(0x9D).wrapping_add(0x47);
            MARKS_WORK.seed.set(s);
            s & 0xFFFF
        };
        let r = seed();
        let at = MATRICES.view_place[0].addr();
        let view = |o: u32| -> Result<_, Refused> { Ok(get(at + o)?) };
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
        let x = wave(&PARTICLE_TUNING.puff_x, view(0x30)?, false);
        let y = x87::loaded(view(0x34)?);
        let z = wave(&PARTICLE_TUNING.puff_z, view(0x38)?, true);
        let lift = seed();
        let up = x87::int_times_plus_times_plus(
            lift as i32,
            uv_k,
            PARTICLE_TUNING.puff_y[0].bits(),
            PARTICLE_TUNING.puff_y[1].bits(),
            PARTICLE_TUNING.puff_y[2].bits(),
        );
        let mx = x87::sub_mul(view(0x30)?, x, PARTICLE_TUNING.puff_m[0].bits());
        let mz = x87::sub_mul(view(0x38)?, z, PARTICLE_TUNING.puff_m[1].bits());
        for (n, v) in [x, y, z, mx, up, mz].into_iter().enumerate() {
            PUFF_WORK.puff[n].set_bits(v);
        }
        let six = [mz, up, mx, z, y, x].map(extended);
        crate::marks::add_drift_particle::add_drift_particle(0, six)?;
    }
    Ok(())
}

/// The trail particles' colours (IDA 0x922B38 .. 0x922FE0, the image's layout; phase 7, typed state).
#[repr(C)]
pub struct TrailColours {
    /// 0x922B38: `dword_922B38`: the trail particles' 298 colours (`marks::run_particles`).
    pub colours: [Dword; 298],
}

pub static TRAIL_PALETTE: TrailColours = TrailColours {
    colours: [
        Dword::new(0xFF0000),
        Dword::new(0xFF0200),
        Dword::new(0xFF0400),
        Dword::new(0xFF0600),
        Dword::new(0xFF0800),
        Dword::new(0xFF0A00),
        Dword::new(0xFF0C00),
        Dword::new(0xFF0E00),
        Dword::new(0xFF1000),
        Dword::new(0xFF1200),
        Dword::new(0xFF1400),
        Dword::new(0xFF1600),
        Dword::new(0xFF1800),
        Dword::new(0xFF1A00),
        Dword::new(0xFF1C00),
        Dword::new(0xFF1E00),
        Dword::new(0xFF2000),
        Dword::new(0xFF2200),
        Dword::new(0xFF2400),
        Dword::new(0xFF2600),
        Dword::new(0xFF2800),
        Dword::new(0xFF2A00),
        Dword::new(0xFF2C00),
        Dword::new(0xFF2E00),
        Dword::new(0xFF3000),
        Dword::new(0xFF3200),
        Dword::new(0xFF3400),
        Dword::new(0xFF3600),
        Dword::new(0xFF3800),
        Dword::new(0xFF3A00),
        Dword::new(0xFF3C00),
        Dword::new(0xFF3E00),
        Dword::new(0xFF4000),
        Dword::new(0xFF4200),
        Dword::new(0xFF4400),
        Dword::new(0xFF4600),
        Dword::new(0xFF4800),
        Dword::new(0xFF4A00),
        Dword::new(0xFF4C00),
        Dword::new(0xFF4E00),
        Dword::new(0xFF5000),
        Dword::new(0xFF5200),
        Dword::new(0xFF5400),
        Dword::new(0xFF5600),
        Dword::new(0xFF5800),
        Dword::new(0xFF5A00),
        Dword::new(0xFF5C00),
        Dword::new(0xFF5E00),
        Dword::new(0xFF6000),
        Dword::new(0xFF6200),
        Dword::new(0xFF6400),
        Dword::new(0xFF6600),
        Dword::new(0xFF6800),
        Dword::new(0xFF6A00),
        Dword::new(0xFF6C00),
        Dword::new(0xFF6E00),
        Dword::new(0xFF7000),
        Dword::new(0xFF7200),
        Dword::new(0xFF7400),
        Dword::new(0xFF7600),
        Dword::new(0xFF7800),
        Dword::new(0xFF7A00),
        Dword::new(0xFF7C00),
        Dword::new(0xFF7E00),
        Dword::new(0xFF8000),
        Dword::new(0xFF8200),
        Dword::new(0xFF8400),
        Dword::new(0xFF8600),
        Dword::new(0xFF8800),
        Dword::new(0xFF8A00),
        Dword::new(0xFF8C00),
        Dword::new(0xFF8E00),
        Dword::new(0xFF9000),
        Dword::new(0xFF9200),
        Dword::new(0xFF9400),
        Dword::new(0xFF9600),
        Dword::new(0xFF9800),
        Dword::new(0xFF9A00),
        Dword::new(0xFF9C00),
        Dword::new(0xFF9E00),
        Dword::new(0xFFA000),
        Dword::new(0xFFA200),
        Dword::new(0xFFA400),
        Dword::new(0xFFA600),
        Dword::new(0xFFA800),
        Dword::new(0xFFAA00),
        Dword::new(0xFFAC00),
        Dword::new(0xFFAE00),
        Dword::new(0xFFB000),
        Dword::new(0xFFB200),
        Dword::new(0xFFB400),
        Dword::new(0xFFB600),
        Dword::new(0xFFB800),
        Dword::new(0xFFBA00),
        Dword::new(0xFFBC00),
        Dword::new(0xFFBE00),
        Dword::new(0xFFC000),
        Dword::new(0xFFC200),
        Dword::new(0xFFC400),
        Dword::new(0xFFC600),
        Dword::new(0xFFC800),
        Dword::new(0xFFCA00),
        Dword::new(0xFFCC00),
        Dword::new(0xFFCE00),
        Dword::new(0xFFD000),
        Dword::new(0xFFD200),
        Dword::new(0xFFD400),
        Dword::new(0xFFD600),
        Dword::new(0xFFD800),
        Dword::new(0xFFDA00),
        Dword::new(0xFFDC00),
        Dword::new(0xFFDE00),
        Dword::new(0xFFE000),
        Dword::new(0xFFE200),
        Dword::new(0xFFE400),
        Dword::new(0xFFE600),
        Dword::new(0xFFE800),
        Dword::new(0xFFEA00),
        Dword::new(0xFFEC00),
        Dword::new(0xFFEE00),
        Dword::new(0xFFF000),
        Dword::new(0xFFF200),
        Dword::new(0xFFF400),
        Dword::new(0xFFF600),
        Dword::new(0xFFF800),
        Dword::new(0xFFFA00),
        Dword::new(0xFFFC00),
        Dword::new(0xFFFE00),
        Dword::new(0xFFFF00),
        Dword::new(0xFFFF02),
        Dword::new(0xFFFF04),
        Dword::new(0xFFFF06),
        Dword::new(0xFFFF08),
        Dword::new(0xFFFF0A),
        Dword::new(0xFFFF0C),
        Dword::new(0xFFFF0E),
        Dword::new(0xFFFF10),
        Dword::new(0xFFFF12),
        Dword::new(0xFFFF14),
        Dword::new(0xFFFF16),
        Dword::new(0xFFFF18),
        Dword::new(0xFFFF1A),
        Dword::new(0xFFFF1C),
        Dword::new(0xFFFF1E),
        Dword::new(0xFFFF20),
        Dword::new(0xFFFF22),
        Dword::new(0xFFFF24),
        Dword::new(0xFFFF26),
        Dword::new(0xFFFF28),
        Dword::new(0xFFFF2A),
        Dword::new(0xFFFF2C),
        Dword::new(0xFFFF2E),
        Dword::new(0xFFFF30),
        Dword::new(0xFFFF32),
        Dword::new(0xFFFF34),
        Dword::new(0xFFFF36),
        Dword::new(0xFFFF38),
        Dword::new(0xFFFF3A),
        Dword::new(0xFFFF3C),
        Dword::new(0xFFFF3E),
        Dword::new(0xFFFF40),
        Dword::new(0xFFFF42),
        Dword::new(0xFFFF44),
        Dword::new(0xFFFF46),
        Dword::new(0xFFFF48),
        Dword::new(0xFFFF4A),
        Dword::new(0xFFFF4C),
        Dword::new(0xFFFF4E),
        Dword::new(0xFFFF50),
        Dword::new(0xFFFF52),
        Dword::new(0xFFFF54),
        Dword::new(0xFFFF56),
        Dword::new(0xFFFF58),
        Dword::new(0xFFFF5A),
        Dword::new(0xFFFF5C),
        Dword::new(0xFFFF5E),
        Dword::new(0xFFFF60),
        Dword::new(0xFFFF62),
        Dword::new(0xFFFF64),
        Dword::new(0xFFFF66),
        Dword::new(0xFFFF68),
        Dword::new(0xFFFF6A),
        Dword::new(0xFFFF6C),
        Dword::new(0xFFFF6E),
        Dword::new(0xFFFF70),
        Dword::new(0xFFFF72),
        Dword::new(0xFFFF74),
        Dword::new(0xFFFF76),
        Dword::new(0xFFFF78),
        Dword::new(0xFFFF7A),
        Dword::new(0xFFFF7C),
        Dword::new(0xFFFF7E),
        Dword::new(0xFFFF80),
        Dword::new(0xFFFF82),
        Dword::new(0xFFFF84),
        Dword::new(0xFFFF86),
        Dword::new(0xFFFF88),
        Dword::new(0xFFFF8A),
        Dword::new(0xFFFF8C),
        Dword::new(0xFFFF8E),
        Dword::new(0xFFFF90),
        Dword::new(0xFFFF92),
        Dword::new(0xFFFF94),
        Dword::new(0xFFFF96),
        Dword::new(0xFFFF98),
        Dword::new(0xFFFF9A),
        Dword::new(0xFFFF9C),
        Dword::new(0xFFFF9E),
        Dword::new(0xFFFFA0),
        Dword::new(0xFFFFA2),
        Dword::new(0xFFFFA4),
        Dword::new(0xFFFFA6),
        Dword::new(0xFFFFA8),
        Dword::new(0xFFFFAA),
        Dword::new(0xFFFFAC),
        Dword::new(0xFFFFAE),
        Dword::new(0xFFFFB0),
        Dword::new(0xFFFFB2),
        Dword::new(0xFFFFB4),
        Dword::new(0xFFFFB6),
        Dword::new(0xFFFFB8),
        Dword::new(0xFFFFBA),
        Dword::new(0xFFFFBC),
        Dword::new(0xFFFFBE),
        Dword::new(0xFFFFC0),
        Dword::new(0xFFFFC2),
        Dword::new(0xFFFFC4),
        Dword::new(0xFFFFC6),
        Dword::new(0xFFFFC8),
        Dword::new(0xFFFFCA),
        Dword::new(0xFFFFCC),
        Dword::new(0xFFFFCE),
        Dword::new(0xFFFFD0),
        Dword::new(0xFFFFD2),
        Dword::new(0xFFFFD4),
        Dword::new(0xFFFFD6),
        Dword::new(0xFFFFD8),
        Dword::new(0xFFFFDA),
        Dword::new(0xFFFFDC),
        Dword::new(0xFFFFDE),
        Dword::new(0xFFFFE0),
        Dword::new(0xFFFFE2),
        Dword::new(0xFFFFE4),
        Dword::new(0xFFFFE6),
        Dword::new(0xFFFFE8),
        Dword::new(0xFFFFEA),
        Dword::new(0xFFFFEC),
        Dword::new(0xFFFFEE),
        Dword::new(0xFFFFF0),
        Dword::new(0xFFFFF2),
        Dword::new(0xFFFFF4),
        Dword::new(0xFFFFF6),
        Dword::new(0xFFFFF8),
        Dword::new(0xFFFFFA),
        Dword::new(0xFFFFFC),
        Dword::new(0xFFFFFE),
        Dword::new(0xFFFFFF),
        Dword::new(0xFFFFFF),
        Dword::new(0xFFFFFF),
        Dword::new(0xFFFFFF),
        Dword::new(0xFFFFFF),
        Dword::new(0xFFFFFF),
        Dword::new(0xFFFFFF),
        Dword::new(0xFFFFFF),
        Dword::new(0xFFFFFF),
        Dword::new(0xFFFFFF),
        Dword::new(0xFFFFFF),
        Dword::new(0xFFFFFF),
        Dword::new(0xFFFFFF),
        Dword::new(0xFFFFFF),
        Dword::new(0xFFFFFF),
        Dword::new(0xFFFFFF),
        Dword::new(0xFFFFFF),
        Dword::new(0xFFFFFF),
        Dword::new(0xFFFFFF),
        Dword::new(0xFFFFFF),
        Dword::new(0xFFFFFF),
        Dword::new(0xFFFFFF),
        Dword::new(0xFFFFFF),
        Dword::new(0xFFFFFF),
        Dword::new(0xFFFFFF),
        Dword::new(0xFFFFFF),
        Dword::new(0xFFFFFF),
        Dword::new(0xFFFFFF),
        Dword::new(0xFFFFFF),
        Dword::new(0xFFFFFF),
        Dword::new(0xFFFFFF),
        Dword::new(0xFFFFFF),
        Dword::new(0xFFFFFF),
        Dword::new(0xFFFFFF),
        Dword::new(0xFFFFFF),
        Dword::new(0xFFFFFF),
        Dword::new(0xFFFFFF),
        Dword::new(0xFFFFFF),
        Dword::new(0xFFFFFF),
        Dword::new(0xFFFFFF),
        Dword::new(0xFFFFFF),
        Dword::new(0xFFFFFF),
    ],
};

impl TrailColours {
    pub const START: usize = 0x922B38;
    pub const LEN: usize = 0x4A8;
    /// Where the byte IDA calls `START` is now (`moved::MOVED`'s home).
    pub fn home() -> usize {
        crate::pointers::game_address(&TRAIL_PALETTE as *const TrailColours) as usize
    }
}

/// The puff's work values (IDA 0x93A5F0 .. 0x93A618, the image's layout; phase 7, typed state).
#[repr(C)]
pub struct PuffWork {
    /// 0x93A5F0: Not named: 16 bytes (a gap, or unreferenced data)
    pub _gap_93a5f0: Bytes<16>,
    /// 0x93A600: `flt_93A600` .. `flt_93A614`: the puff's place, size and motion
    pub puff: [Float; 6],
}

/// PuffWork's first values.
pub static PUFF_WORK: PuffWork = PuffWork {
    _gap_93a5f0: Bytes::new([
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x80, 0xC5, 0x00, 0x00, 0x00,
        0x00,
    ]),
    puff: [
        Float::from_bits(0x0),
        Float::from_bits(0x0),
        Float::from_bits(0x0),
        Float::from_bits(0x0),
        Float::from_bits(0x0),
        Float::from_bits(0x0),
    ],
};

impl PuffWork {
    pub const START: usize = 0x93A5F0;
    pub const LEN: usize = 0x28;
    /// Where the byte IDA calls `START` is now (`moved::MOVED`'s home).
    pub fn home() -> usize {
        crate::pointers::game_address(&PUFF_WORK as *const PuffWork) as usize
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
        let mine = unsafe {
            std::slice::from_raw_parts(
                crate::pointers::host_at(TrailColours::home()) as *const u8,
                TrailColours::LEN,
            )
        };
        assert!(
            mine == &image[..],
            "TRAIL_PALETTE starts differently from the image"
        );
    }
}

#[cfg(test)]
mod puff_work_tests {
    use super::*;
    use core::mem::offset_of;

    fn bytes<T>(v: &T) -> Vec<u8> {
        let p = v as *const T as *const u8;
        (0..size_of::<T>())
            .map(|i| unsafe { p.add(i).read() })
            .collect()
    }

    #[test]
    fn puff_work_starts_as_the_games() {
        assert_eq!(size_of::<PuffWork>(), PuffWork::LEN);
        assert!(
            bytes(&PUFF_WORK) == crate::moved::image_bytes(PuffWork::START, PuffWork::LEN),
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
    /// Where the byte IDA calls `START` is now (`moved::MOVED`'s home).
    pub fn home() -> usize {
        crate::pointers::game_address(&BURST_COLOURS as *const BurstColours) as usize
    }
}

#[cfg(test)]
mod burst_colours_tests {
    use super::*;
    use core::mem::offset_of;

    fn bytes<T>(v: &T) -> Vec<u8> {
        let p = v as *const T as *const u8;
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
    /// Where the byte IDA calls `START` is now (`moved::MOVED`'s home).
    pub fn home() -> usize {
        crate::pointers::game_address(&PARTICLE_TUNING as *const ParticleTuning) as usize
    }
}

#[cfg(test)]
mod particle_tuning_tests {
    use super::*;
    use core::mem::offset_of;

    fn bytes<T>(v: &T) -> Vec<u8> {
        let p = v as *const T as *const u8;
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
