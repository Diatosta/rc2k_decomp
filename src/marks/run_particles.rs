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
use crate::marks::blocks::{RING_FREE_RVA, RING_USED_RVA};
use crate::marks::setup::CLOCK;
use crate::marks::setup::SEED_RVA;
use crate::marks::store::{Slot, game_read, game_write};
use crate::math::x87;
use crate::race::matrices_store::MATRICES;
use crate::scene::matrix3::extended;
use crate::scene::vector3::STATUS_RVA;

const STEPS_RVA: Slot = Slot::at(0x76513C); // dword_76513C
const INDEX_RVA: Slot = Slot::at(0x93A5E8); // dword_93A5E8
const TRAIL_COLOURS_RVA: Slot = Slot::at(0x922B38); // dword_922B38: 298 dwords
const TRAIL_COLOURS: i32 = 298;
const BURST_COLOURS_RVA: Slot = Slot::at(0x93A618); // dword_93A618: 0x2000 bytes
const BURST_COLOURS_LEN: i32 = 0x2000;
const UV_K_RVA: Slot = Slot::at(0x8EB969); // flt_8EB969
const DUST: [Slot; 3] = [Slot::at(0x93C680), Slot::at(0x93C684), Slot::at(0x93C688)];
const TRAIL: [Slot; 2] = [Slot::at(0x93C68C), Slot::at(0x93C690)];
/// Kind 8's and 9's: the +0x30 gain, the three dampings, the +0x3C gain.
const BURST: [[Slot; 5]; 2] = [
    [
        Slot::at(0x93C694),
        Slot::at(0x93C698),
        Slot::at(0x93C69C),
        Slot::at(0x93C6A0),
        Slot::at(0x93C6A4),
    ],
    [
        Slot::at(0x93C6A8),
        Slot::at(0x93C6AC),
        Slot::at(0x93C6B0),
        Slot::at(0x93C6B4),
        Slot::at(0x93C6B8),
    ],
];
/// The puff's: x wave (three factors, size), z wave (the same), y (add, factor, add),
/// the motion's two factors.
const PUFF_X: [Slot; 3] = [Slot::at(0x93C618), Slot::at(0x93C61C), Slot::at(0x93C620)];
const PUFF_Z: [Slot; 3] = [Slot::at(0x93C624), Slot::at(0x93C628), Slot::at(0x93C62C)];
const PUFF_Y: [Slot; 3] = [Slot::at(0x93C630), Slot::at(0x93C634), Slot::at(0x93C638)];
const PUFF_M: [Slot; 2] = [Slot::at(0x93C63C), Slot::at(0x93C640)];
const PUFF_RVA: Slot = Slot::at(0x93A600); // flt_93A600 .. flt_93A614
const BIGGER: u32 = 0x386A0;

unsafe fn get(a: u32) -> u32 {
    unsafe { (a as *const u32).read_unaligned() }
}

unsafe fn put(a: u32, v: u32) {
    unsafe { (a as *mut u32).write_unaligned(v) }
}

/// A kind's colour index (life times +0x40, `fistp`) and whether it is in its table.
unsafe fn colour_index(e: u32, life: u32) -> (i32, bool) {
    unsafe {
        let i = x87::int_times_to_int(life as i32, get(e + 0x40));
        let ok = match get(e + 8) {
            4 => (0..TRAIL_COLOURS).contains(&i),
            _ => {
                let off = (get(e + 0x48) as i32).wrapping_add(i.wrapping_mul(4));
                i >= 0 && (0..=BURST_COLOURS_LEN - 4).contains(&off)
            }
        };
        (i, ok)
    }
}

/// Why it refuses, if it does: the walk as the original makes it, nothing written.
unsafe fn refusal() -> Option<&'static str> {
    unsafe {
        let settings: u32 = crate::marks::setup::SETTINGS.pointer.get();
        if settings == 0 {
            return Some("no marks settings");
        }
        if get(settings + 4) == 0 {
            return None;
        }
        let ticks: u32 = CLOCK.frame_ticks.get();
        let mut e: u32 = game_read(RING_FREE_RVA);
        let used = game_read::<u32>(RING_USED_RVA) as i32;
        if used > 0 && (e == 0 || get(e) == 0) {
            return Some("a null particle ring entry");
        }
        for _ in 0..used.max(0) {
            if e == 0 || get(e) == 0 || get(e + 4) == 0 {
                return Some("a null particle ring entry or link");
            }
            let life = get(e + 0x44);
            let kind = get(e + 8);
            if life >= ticks && x87::compare_status(get(e + 0x10), get(e + 0x24)) != 0x100 {
                if ![0, 4, 8, 9].contains(&kind) {
                    return Some("a particle of another kind (the original's stack goes wrong)");
                }
                if kind != 0 && !colour_index(e, life - ticks).1 {
                    return Some("a particle colour index out of its table");
                }
            }
            e = get(e + 4);
        }
        None
    }
}

/// The step of the kind at e (see the header): whether the entry dies.
unsafe fn step(e: u32) -> Result<bool, Refused> {
    use crate::marks::{burst_particles, draw_particle, trail_particles};
    unsafe {
        let t: i32 = CLOCK.frame_ticks.get() as i32;
        let move_place = |with_up: bool| {
            put(e + 0xC, x87::plus_times_int(get(e + 0xC), get(e + 0x28), t));
            let y = if with_up {
                x87::plus_sum_times_int(get(e + 0x10), get(e + 0x2C), get(e + 0x30), t)
            } else {
                x87::plus_times_int(get(e + 0x10), get(e + 0x2C), t)
            };
            put(e + 0x10, y);
            put(
                e + 0x14,
                x87::plus_times_int(get(e + 0x14), get(e + 0x34), t),
            );
        };
        let gain_int = |k: Slot| {
            let v = x87::int_times_int_plus_int(game_read(k), t, get(e + 0x3C) as i32);
            put(e + 0x3C, v as u32);
        };
        let keep_last = || {
            put(e + 0x18, get(e + 0xC));
            put(e + 0x1C, get(e + 0x10));
            put(e + 0x20, get(e + 0x14));
        };
        let place = || {
            [
                extended(get(e + 0xC)),
                extended(get(e + 0x10)),
                extended(get(e + 0x14)),
            ]
        };
        match get(e + 8) {
            0 => {
                move_place(false);
                put(
                    e + 0x2C,
                    x87::plus_times_int(get(e + 0x2C), game_read(DUST[0]), t),
                );
                gain_int(DUST[1]);
                let s = x87::compare_status(game_read(DUST[2]), get(e + 0x2C));
                game_write(STATUS_RVA, s as u32);
                if s != 0x100 {
                    burst_particles::burst_particles(e)?;
                    return Ok(true);
                }
                if get(e + 0x2C) & 0x8000_0000 == 0 {
                    trail_particles::trail_particles(e)?;
                }
                keep_last();
            }
            4 => {
                move_place(false);
                put(
                    e + 0x2C,
                    x87::plus_times_int(get(e + 0x2C), game_read(TRAIL[0]), t),
                );
                gain_int(TRAIL[1]);
                let (i, _) = colour_index(e, get(e + 0x44));
                game_write(INDEX_RVA, i);
                let alpha = get(e + 0x3C) & 0xFF00_0000;
                let colour =
                    (game_read::<u32>(TRAIL_COLOURS_RVA + 4 * i as usize) & 0xFF_FFFF) | alpha;
                draw_particle::draw_particle(place(), colour, get(e + 0x38));
                keep_last();
            }
            kind => {
                let k = BURST[(kind == 9) as usize];
                move_place(true);
                put(
                    e + 0x30,
                    x87::plus_times_int(get(e + 0x30), game_read(k[0]), t),
                );
                put(
                    e + 0x28,
                    x87::less_own_times(get(e + 0x28), game_read(k[1])),
                );
                put(
                    e + 0x2C,
                    x87::less_own_times(get(e + 0x2C), game_read(k[2])),
                );
                put(
                    e + 0x34,
                    x87::less_own_times(get(e + 0x34), game_read(k[3])),
                );
                gain_int(k[4]);
                let (i, _) = colour_index(e, get(e + 0x44));
                game_write(INDEX_RVA, i);
                let tint = game_read::<u32>(
                    BURST_COLOURS_RVA + (get(e + 0x48) as usize).wrapping_add(4 * i as usize),
                ) & 0xFF_FFFF;
                if tint == 0 {
                    return Ok(true);
                }
                let colour = (get(e + 0x3C) & 0xFF00_0000) | tint;
                let mut size = get(e + 0x38);
                if kind == 9 {
                    let s = game_read::<u32>(SEED_RVA)
                        .wrapping_mul(0x9D)
                        .wrapping_add(0x47);
                    game_write(SEED_RVA, s);
                    if s.rotate_right(16) & 0x7F == 0 {
                        size = size.wrapping_add(BIGGER);
                    }
                }
                draw_particle::draw_particle(place(), colour, size);
            }
        }
    }
    Ok(false)
}

/// The Rust version (see the header).
pub unsafe fn run_particles() -> Result<(), Refused> {
    if let Some(why) = unsafe { refusal() } {
        return Err(BadInput(why).into());
    }
    unsafe {
        let settings: u32 = crate::marks::setup::SETTINGS.pointer.get();
        if get(settings + 4) == 0 {
            return Ok(());
        }
        crate::marks::set_quad_uvs::set_quad_uvs();
        let mut e: u32 = game_read(RING_FREE_RVA);
        let mut counter = game_read::<u32>(RING_USED_RVA) as i32;
        loop {
            counter = counter.wrapping_sub(1);
            if counter < 0 {
                break;
            }
            let next = get(e + 4);
            let t: u32 = CLOCK.frame_ticks.get();
            let life = get(e + 0x44);
            put(e + 0x44, life.wrapping_sub(t));
            let mut dies = life < t;
            if !dies {
                let s = x87::compare_status(get(e + 0x10), get(e + 0x24));
                game_write(STATUS_RVA, s as u32);
                dies = s == 0x100;
                if !dies {
                    dies = step(e)?;
                }
            }
            if dies {
                game_write(
                    RING_USED_RVA,
                    game_read::<u32>(RING_USED_RVA).wrapping_sub(1),
                );
                if e == game_read::<u32>(RING_FREE_RVA) {
                    game_write(RING_FREE_RVA, get(e + 4));
                } else {
                    let (before, after) = (get(e), get(e + 4));
                    put(before + 4, after);
                    put(after, before);
                    let free: u32 = game_read(RING_FREE_RVA);
                    let last = get(free);
                    put(free, e);
                    put(last + 4, e);
                    put(e, last);
                    put(e + 4, free);
                }
            }
            e = next;
        }
        if game_read::<u32>(STEPS_RVA) & 7 != 0 {
            return Ok(());
        }
        crate::sound::store::STAGE_SOUNDS.random[0].set(1);
        let seed = || {
            let s = game_read::<u32>(SEED_RVA)
                .wrapping_mul(0x9D)
                .wrapping_add(0x47);
            game_write(SEED_RVA, s);
            s & 0xFFFF
        };
        let r = seed();
        let at = MATRICES.view_place[0].addr() as u32;
        let view = |o: u32| get(at + o);
        let uv_k: u32 = game_read(UV_K_RVA);
        let wave = |k: [Slot; 3], add: u32, cosine: bool| {
            x87::int_times3_wave_plus(
                r as i32,
                [uv_k, game_read(k[0]), game_read(k[1])],
                game_read(k[2]),
                add,
                cosine,
            )
        };
        let x = wave(PUFF_X, view(0x30), false);
        let y = x87::loaded(view(0x34));
        let z = wave(PUFF_Z, view(0x38), true);
        let lift = seed();
        let up = x87::int_times_plus_times_plus(
            lift as i32,
            uv_k,
            game_read(PUFF_Y[0]),
            game_read(PUFF_Y[1]),
            game_read(PUFF_Y[2]),
        );
        let mx = x87::sub_mul(view(0x30), x, game_read(PUFF_M[0]));
        let mz = x87::sub_mul(view(0x38), z, game_read(PUFF_M[1]));
        for (n, v) in [x, y, z, mx, up, mz].into_iter().enumerate() {
            game_write(PUFF_RVA + 4 * n, v);
        }
        let six = [mz, up, mx, z, y, x].map(extended);
        crate::marks::add_drift_particle::add_drift_particle(0, six)?;
    }
    Ok(())
}
