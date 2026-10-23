//! `marks::cars_particles` (0x4C9AF6): the marks' frame for each car, when the settings'
//! +0x10 is 1: for each of the settings' +0x5C cars (from the last; each 0x4C8 bytes from
//! the settings' +0x60), `dword_93547E` its number by `dword_9354DE` (`fimul`),
//! `g_marksCar` it, its matrix's adjugate at +0x94 (`scene::adjugate_matrix`); its motion
//! history moved on (+0x58 .. +0x60 into +0x64, into +0x70, into +0x7C), the new one the
//! matrix's place less the last (+0x40 ..), the mean of the four by `flt_7651BC` at +0x4C
//! .., the place kept; the mean's length (+0x88, `fist`) and a rate from it (+0x90: 1 - 1
//! / (length by `flt_9354E2` plus `flt_9354E6`), by `flt_9354EA`); while +0x30 counts
//! down, the mean and length 0. Then the side factors: `flt_935452` .. the cross product
//! of `flt_935446` .. and the vector at +0x28, made of length 1 (`x87::normalised3`);
//! `flt_93543A` .. that crossed with the vector again, turned by the matrix transposed
//! (`x87::crossed_turned`); `flt_93545E` .. the cross product of the two; `dword_935476` /
//! `dword_93547A` from `dword_7694AA` by the car's +4; then `marks::emit_car_particles`
//! (unless +0x30 counts down, by one) and `marks::emit_particle_13`, `emit_particle_19`,
//! `emit_particle_22`.
//!
//! In:  nothing (`g_marksSettings`)
//! Out: ebp the settings when nothing is done, else as the last car leaves it (see
//!      `cars`); the rest as the last car leaves them
//!
//! Caller: `sub_4C3EEE` (0x4C4006). Refuses null settings, more than 8 cars, a car's null
//! matrix or vector, a car model (+4) of 33 or more, and the emitters' refusals; on a
//! refusal nothing stays written (all it may write is put back). Self-test: settings of
//! the test's with up to three cars on the test ring (`marks::emit_particle_6::TestEmitter`).
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::{BadInput, Refused};
use crate::framework::{Regs, game_addr, game_read, game_write, rva};
use crate::marks::emit_car_particles::{self, emit_car_particles};
use crate::marks::emitters::*;
use crate::marks::lines::SETTINGS_RVA;
use crate::marks::{emit_particle_13, emit_particle_19, emit_particle_22};
use crate::math::x87;
use crate::race::stage_prefs::left_of;
use crate::scene::adjugate_matrix::adjugate_matrix;
use crate::screens::harness::Left;
use crate::service::harness::{Region, at as at_rva, capture, restore, state_entry};

const CAR_LEN: u32 = 0x4C8;
const MOST_CARS: u32 = 8;
const TAG_K_RVA: usize = rva(0x9354DE); // dword_9354DE
const MEAN_K_RVA: usize = rva(0x7651BC); // flt_7651BC
/// flt_9354E2, flt_9354E6, flt_9354EA: the rate's factor, base and scale.
const RATE_K: [usize; 3] = [rva(0x9354E2), rva(0x9354E6), rva(0x9354EA)];
const UP_RVA: usize = rva(0x935446); // flt_935446 .. flt_93544E
const ACROSS_RVA: usize = rva(0x935452); // flt_935452 .. flt_93545A
const SIDE_B_RVA: usize = rva(0x93543A); // flt_93543A .. flt_935442
const SIDE_A_RVA: usize = rva(0x93545E); // flt_93545E .. flt_935466
/// dword_7694AA, dword_7694B2: the looks by the car's model (8 bytes apart, 33).
const LOOKS_TABLE_RVA: usize = rva(0x7694AA);
const MODELS: u32 = 33;
const LOOKS_RVA: [usize; 2] = [rva(0x935476), rva(0x93547A)];

unsafe fn put(a: u32, v: u32) {
    unsafe { (a as *mut u32).write_unaligned(v) }
}

/// The cars, worked out without writing: none when the settings' +0x10 is not 1.
unsafe fn cars(settings: u32) -> Vec<u32> {
    unsafe {
        if at(settings + 0x10) != 1 {
            return Vec::new();
        }
        let count = at(settings + 0x5C) as i32;
        (0..count.clamp(0, MOST_CARS as i32 + 1) as u32)
            .rev()
            .map(|n| settings + 0x60 + n * CAR_LEN)
            .collect()
    }
}

/// Why the Rust version refuses, if it does (its own checks), worked out without writing.
unsafe fn refusal(settings: u32) -> Option<&'static str> {
    unsafe {
        if settings == 0 {
            return Some("null marks settings");
        }
        if at(settings + 0x10) == 1 && at(settings + 0x5C) as i32 > MOST_CARS as i32 {
            return Some("more than 8 cars");
        }
        for car in cars(settings) {
            if at(car + 0x24) == 0 || at(car + 0x28) == 0 {
                return Some("a car's null matrix or vector");
            }
            if at(car + 4) >= MODELS {
                return Some("a car model past dword_7694AA's 33 entries");
            }
        }
        None
    }
}

/// The Rust version (see the header): the registers it leaves.
pub unsafe fn cars_particles(entry: &Regs) -> Result<Regs, Refused> {
    let settings: u32 = unsafe { game_read(SETTINGS_RVA) };
    if let Some(why) = unsafe { refusal(settings) } {
        return Err(BadInput(why).into());
    }
    let written = regions(settings);
    let saved = capture(&written);
    let done = unsafe { run(entry, settings) };
    if done.is_err() {
        restore(&written, &saved);
    }
    done
}

unsafe fn run(entry: &Regs, settings: u32) -> Result<Regs, Refused> {
    let mut g = *entry;
    g.ebp = settings;
    let count = unsafe { cars(settings) }.len() as u32;
    unsafe {
        for n in (0..count).rev() {
            g.ebp = settings;
            g.esi = n;
            game_write(
                TAG_RVA,
                x87::int_times_int_to_int(n as i32, game_read(TAG_K_RVA)),
            );
            let car = settings + 0x60 + n * CAR_LEN;
            g.esi = car;
            game_write(OBJECT_RVA, car);
            let m = at(car + 0x24);
            adjugate_matrix(m, car + 0x94);
            for (to, from) in [(0x7C, 0x70), (0x70, 0x64), (0x64, 0x58)] {
                let v = [at(car + from), at(car + from + 4), at(car + from + 8)];
                for (k, x) in v.iter().enumerate() {
                    put(car + to + 4 * k as u32, *x);
                }
                [g.eax, g.ebx, g.ecx] = v;
            }
            g.ebp = m;
            for k in 0..3 {
                let k4 = 4 * k;
                put(
                    car + 0x58 + k4,
                    x87::sub(at(m + 0x30 + k4), at(car + 0x40 + k4)),
                );
            }
            let mean_k: u32 = game_read(MEAN_K_RVA);
            for k in 0..3 {
                let k4 = 4 * k;
                put(
                    car + 0x4C + k4,
                    x87::sum4_times(
                        at(car + 0x58 + k4),
                        at(car + 0x64 + k4),
                        at(car + 0x70 + k4),
                        at(car + 0x7C + k4),
                        mean_k,
                    ),
                );
            }
            [g.eax, g.ebx, g.ecx] = [at(m + 0x30), at(m + 0x34), at(m + 0x38)];
            put(car + 0x40, g.eax);
            put(car + 0x44, g.ebx);
            put(car + 0x48, g.ecx);
            let [k1, k2, k3] = RATE_K.map(|r| game_read::<u32>(r));
            let one: u32 = game_read(ONE_RVA);
            let (length, rate) = x87::length_and_rate(
                [at(car + 0x4C), at(car + 0x50), at(car + 0x54)],
                k1,
                k2,
                one,
                k3,
            );
            put(car + 0x88, length as u32);
            put(car + 0x90, rate as u32);
            if at(car + 0x30) != 0 {
                for o in [0x4C, 0x50, 0x54, 0x88] {
                    put(car + o, 0);
                }
            }
            let w = at(car + 0x28);
            g.edi = w;
            let up: [u32; 3] = game_read(UP_RVA);
            let across = [
                x87::product_difference(up[1], at(w + 8), up[2], at(w + 4)),
                x87::product_difference(up[2], at(w), up[0], at(w + 8)),
                x87::product_difference(up[0], at(w + 4), up[1], at(w)),
            ];
            let across = x87::normalised3(across, one);
            game_write(ACROSS_RVA, across);
            let b = x87::crossed_turned(across, w, m);
            game_write(SIDE_B_RVA, b);
            let a = [
                x87::product_difference(b[1], across[2], b[2], across[1]),
                x87::product_difference(b[2], across[0], b[0], across[2]),
                x87::product_difference(b[0], across[1], b[1], across[0]),
            ];
            game_write(SIDE_A_RVA, a);
            let model = at(car + 4);
            g.eax = model;
            g.ebx = game_read(LOOKS_TABLE_RVA + 8 * model as usize);
            g.ecx = game_read(LOOKS_TABLE_RVA + 8 + 8 * model as usize);
            game_write(LOOKS_RVA[0], g.ebx);
            game_write(LOOKS_RVA[1], g.ecx);
            if at(car + 0x30) != 0 {
                put(car + 0x30, at(car + 0x30).wrapping_sub(1));
            } else {
                g = emit_car_particles(&g)?;
            }
            emit_particle_13::emit_particle_13(car)?;
            emit_particle_19::emit_particle_19(car)?;
            emit_particle_22::emit_particle_22(car)?;
        }
    }
    Ok(g)
}

/// The globals it and its emitters may write (no ring entry: the cursor may be 0 outside
/// a stage, so nothing here reads through it).
fn globals() -> Vec<Region> {
    let mut r = emit_car_particles::globals();
    r.extend(emit_particle_13::globals());
    r.extend(emit_particle_19::globals(&emit_particle_19::NINETEENTH));
    r.extend(emit_particle_19::globals(&emit_particle_22::TWENTY_SECOND));
    r.extend([TAG_RVA, OBJECT_RVA].map(at_rva));
    r.push((game_addr(ACROSS_RVA) as u32, 12));
    r
}

/// What it may write: the globals, the cars' fields and records, and as many ring entries
/// as all the cars' records (and three a car) take at most.
pub fn regions(settings: u32) -> Vec<Region> {
    let mut r = globals();
    if settings == 0 {
        return r;
    }
    let cars = unsafe { cars(settings) };
    let mut most = 0u32;
    for &car in &cars {
        r.push((car + 0x30, 4));
        r.push((car + 0x40, 0xD4 - 0x40));
        r.push((car + 0x90, 4));
        r.push((car + 0xD4, 0x23C - 0xD4));
        most = most.saturating_add(unsafe { emit_car_particles::entry_bound(car) } + 3);
    }
    let mut e: u32 = unsafe { game_read(RING_CURSOR_RVA) };
    for _ in 0..most.min(MOST_TAKEN + 1) {
        if e == 0 {
            break;
        }
        e = unsafe { at(e) };
        r.push((e, ENTRY_LEN));
    }
    r
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(left_of(&unsafe { cars_particles(r) }?))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
