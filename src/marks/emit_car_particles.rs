//! `marks::emit_car_particles` (0x4C9E2E): a car's particles for the frame. First the
//! places of its four wheel records (+0x3C .. of the records at +0xD4, +0x11C, +0x164,
//! +0x1AC): from the object's whole numbers +0xC (z, by `flt_9354EE`; front +0x118, rear
//! +0x160), +0x10 (the other z, by `flt_9354F2`: +0x1A8, +0x1F0), +0x14 (x, by
//! `flt_9354F6` plus `flt_9354FA`: right +0x158, +0x1E8, negated left +0x110, +0x1A0) and
//! +0x18 (y, by `flt_9354FE` less `flt_935502` and `flt_935506`: all four), each turned
//! and placed by the car's matrix (`scene::rotate_point`); `dword_935476` |the first x|
//! and `dword_93547A` the two z's sizes by `flt_765158`, rounded. Then
//! `marks::emit_wheel_particles` for each wheel (`dword_935416` 0 .. 3; before the rear
//! ones the side factors `flt_93545E` .. the matrix's first row, `flt_93543A` .. its third
//! negated), the factors set from `flt_93550A` .., and `marks::emit_body_particles` for
//! the record at +0x1F4.
//!
//! In:  esi the car object
//! Out: ebp the car's matrix, edi the object + 0x1F4; eax, ebx, ecx as the last emitter
//!      leaves them (from the third row's negated values on); the rest kept
//!
//! Caller: 0x4C9E0C. Refuses a null object or matrix and the emitters' refusals; on a
//! refusal nothing stays written (all it may write is put back). Self-test: a car of the
//! test's (five records) on the test ring (`marks::emit_particle_6::TestEmitter`), several
//! sizes and surfaces.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::{BadInput, Refused};
use crate::framework::{Regs, game_addr, game_read, game_write, rva};
use crate::marks::emit_body_particles::emit_body_particles;
use crate::marks::emit_wheel_particles::{WRITTEN, emit_wheel_particles};
use crate::marks::emitters::*;
use crate::marks::setup::SEED_RVA;
use crate::math::x87;
use crate::race::stage_prefs::left_of;
use crate::scene::matrix3::{extended, rounded};
use crate::scene::rotate_point::rotate_point;
use crate::screens::harness::Left;
use crate::service::harness::{Region, at as at_rva, capture, restore, state_entry};

/// flt_9354EE .. flt_935506: the places' factors and bases.
const K: [usize; 7] = [
    rva(0x9354EE),
    rva(0x9354F2),
    rva(0x9354F6),
    rva(0x9354FA),
    rva(0x9354FE),
    rva(0x935502),
    rva(0x935506),
];
const LOOKS_RVA: [usize; 2] = [rva(0x935476), rva(0x93547A)];
const SIZE_K_RVA: usize = rva(0x765158); // flt_765158
const WHICH_RVA: usize = rva(0x935416); // dword_935416
/// flt_93545E .. flt_935466 and flt_93543A .. flt_935442: the emitters' side factors.
const SIDE_A_RVA: usize = rva(0x93545E);
const SIDE_B_RVA: usize = rva(0x93543A);
/// flt_93550A .. flt_93551E: the side factors for the body record.
const BODY_SIDES_RVA: usize = rva(0x93550A);
/// The wheel records, then the body's.
const RECORDS: [u32; 5] = [0xD4, 0x11C, 0x164, 0x1AC, 0x1F4];
const RECORD_LEN: u32 = 0x48;

/// Writes a float at an address.
unsafe fn put(a: u32, v: u32) {
    unsafe { (a as *mut u32).write_unaligned(v) }
}

/// The Rust version (see the header): the registers it leaves.
pub unsafe fn emit_car_particles(entry: &Regs) -> Result<Regs, Refused> {
    let o = entry.esi;
    if o == 0 || unsafe { at(o + 0x24) } == 0 {
        return Err(BadInput("a null car object or matrix").into());
    }
    let written = regions(o);
    let saved = capture(&written);
    let done = unsafe { run(entry) };
    if done.is_err() {
        restore(&written, &saved);
    }
    done
}

unsafe fn run(entry: &Regs) -> Result<Regs, Refused> {
    let mut g = *entry;
    let o = g.esi;
    unsafe {
        let k = K.map(|r| game_read::<u32>(r));
        let z1 = x87::int_times(at(o + 0xC) as i32, k[0]);
        put(o + 0x118, z1);
        put(o + 0x160, z1);
        let z2 = x87::int_times(at(o + 0x10) as i32, k[1]);
        put(o + 0x1A8, z2);
        put(o + 0x1F0, z2);
        let (x, left) = x87::int_times_plus_and_negated(at(o + 0x14) as i32, k[2], k[3]);
        put(o + 0x158, x);
        put(o + 0x1E8, x);
        put(o + 0x110, left);
        put(o + 0x1A0, left);
        let y = x87::int_times_less_less(at(o + 0x18) as i32, k[4], k[5], k[6]);
        for a in [0x15C, 0x1EC, 0x114, 0x1A4] {
            put(o + a, y);
        }
        game_write(LOOKS_RVA[0], x87::to_int(at(o + 0x110) & 0x7FFF_FFFF));
        game_write(
            LOOKS_RVA[1],
            x87::abs_sum_times_to_int(at(o + 0x118), at(o + 0x1A8), game_read(SIZE_K_RVA)),
        );
        let m = at(o + 0x24);
        for p in [0x110, 0x158, 0x1A0, 0x1E8] {
            let turned = rotate_point(
                m,
                [
                    extended(at(o + p)),
                    extended(at(o + p + 4)),
                    extended(at(o + p + 8)),
                ],
            );
            for (n, v) in turned.iter().enumerate() {
                put(o + p + 4 * n as u32, rounded(*v));
            }
        }
        for (n, &r) in RECORDS[..4].iter().enumerate() {
            if n == 2 {
                g.ebp = m;
                [g.eax, g.ebx, g.ecx] = [at(m), at(m + 4), at(m + 8)];
                game_write(SIDE_A_RVA, [g.eax, g.ebx, g.ecx]);
                [g.eax, g.ebx, g.ecx] = [0x20, 0x24, 0x28].map(|f| at(m + f) ^ 0x8000_0000);
                game_write(SIDE_B_RVA, [g.eax, g.ebx, g.ecx]);
            }
            g.edi = o + r;
            game_write(WHICH_RVA, n as u32);
            g = emit_wheel_particles(&g)?;
        }
        let sides: [u32; 6] =
            std::array::from_fn(|n| x87::loaded(game_read(BODY_SIDES_RVA + 4 * n)));
        game_write(SIDE_A_RVA, [sides[0], sides[1], sides[2]]);
        game_write(SIDE_B_RVA, [sides[3], sides[4], sides[5]]);
        g.edi = o + RECORDS[4];
        g = emit_body_particles(&g)?;
    }
    Ok(g)
}

/// The globals it and its emitters may write.
pub fn globals() -> Vec<Region> {
    let mut r: Vec<Region> = WRITTEN.map(at_rva).to_vec();
    r.extend(
        [
            rva(0x935526),
            FLAGS_RVA,
            RATE_RVA,
            SEED_RVA,
            RING_CURSOR_RVA,
            RING_COUNT_RVA,
            LOOKS_RVA[0],
            LOOKS_RVA[1],
        ]
        .map(at_rva),
    );
    for a in [SIDE_A_RVA, SIDE_B_RVA, SIDES_RVA] {
        r.push((game_addr(a) as u32, 12));
    }
    r
}

/// At most how many ring entries the car's records take (their flags' bound).
pub unsafe fn entry_bound(o: u32) -> u32 {
    unsafe {
        RECORDS
            .iter()
            .map(|&rec| {
                let flags = [0x20, 0x24, 0x2C, 0x30]
                    .map(|f| at(o + rec + f))
                    .into_iter()
                    .max();
                (flags.unwrap_or(0) >> 6)
                    .saturating_mul(8)
                    .saturating_add(2)
            })
            .fold(0u32, |a, b| a.saturating_add(b))
            .min(MOST_TAKEN + 1)
    }
}

/// What it may write: the globals, the object's +0x90 and records, and as many ring
/// entries as the records' flags allow at most.
fn regions(o: u32) -> Vec<Region> {
    let mut r = globals();
    if o == 0 {
        return r;
    }
    r.push((o + 0x90, 4));
    r.push((
        o + RECORDS[0],
        (RECORDS[4] + RECORD_LEN - RECORDS[0]) as usize,
    ));
    unsafe {
        let most = entry_bound(o);
        let mut e: u32 = game_read(RING_CURSOR_RVA);
        for _ in 0..most {
            if e == 0 {
                break;
            }
            e = at(e);
            r.push((e, ENTRY_LEN));
        }
    }
    r
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(left_of(&unsafe { emit_car_particles(r) }?))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
