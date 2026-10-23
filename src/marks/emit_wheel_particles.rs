//! `marks::emit_wheel_particles` (0x4CA085): one wheel's particles. With the chance
//! the wheel's +0x34 by `flt_7651DC` (rounded, into `dword_9356C0`) out of 0x10000 (a
//! random's low word, `dword_935522`, under it) the wheel's +0x28 .. +0x30 (the kind and
//! two flags of the surface under it) become its +0x1C .. +0x24; then by the kind (+0x1C):
//! 1 `marks::emit_kind_1`; 2, 4, 6 .. 9, 0xC `marks::emit_bursts` and
//! `marks::emit_particle_6`; 3 `marks::emit_bursts_2` and `emit_particle_6`; 5
//! `marks::emit_particle_5`; 0xB `marks::emit_particle_11`; 0 and the rest nothing.
//!
//! In:  edi the wheel record (the car object's +0xD4, +0x11C, +0x164, +0x1AC), esi the car
//!      object (`dword_935416` the wheel's number, set by the caller)
//! Out: eax, ebx, ecx the wheel's +0x28 .. +0x30 when copied (else eax the random's low
//!      word); every other register kept
//!
//! Callers: `sub_4C9E2E`, four times. The original EXE the owner had patched turned this
//! into a `ret` (section 16, the burst loops' ring-full slip). Refuses a null wheel and
//! the emitters' refusals; on a refusal nothing stays written (all it may write is put
//! back). Self-test: a ring, wheel and car of the test's
//! (`marks::emit_particle_6::TestEmitter`), every kind, with and without the copy.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::{BadInput, Refused};
use crate::framework::{Regs, game_read, game_write, rva};
use crate::marks::emitters::*;
use crate::marks::{
    emit_bursts, emit_bursts_2, emit_kind_1, emit_particle_5, emit_particle_6, emit_particle_11,
    emit_particle_18,
};
use crate::math::x87;
use crate::race::stage_prefs::left_of;
use crate::screens::harness::Left;
use crate::service::harness::{Region, at as at_rva, capture, restore, state_entry};

const CHANCE_RVA: usize = rva(0x9356C0); // dword_9356C0
const CHANCE_K_RVA: usize = rva(0x7651DC); // flt_7651DC
const CHANCE_RANDOM_RVA: usize = rva(0x935522); // dword_935522

/// The chance and the random it would be met by, worked out without writing.
unsafe fn chance(wheel: u32) -> (i32, u32) {
    unsafe {
        let chance = x87::mul_to_int(at(wheel + 0x34), game_read(CHANCE_K_RVA));
        let r = game_read::<u32>(CHANCE_RANDOM_RVA)
            .wrapping_mul(0xC5)
            .wrapping_add(0x5D);
        (chance, r)
    }
}

/// Whether the copy is made.
fn copies(chance: i32, r: u32) -> bool {
    ((r & 0xFFFF) as i32) < chance
}

/// Whose switch: this one's, or `marks::emit_body_particles`' (its kind 1
/// `marks::emit_particle_18`, its kind 3 the bursts of kind 2: section 16).
#[derive(Clone, Copy, PartialEq)]
pub enum Table {
    Wheel,
    Body,
}

/// The emitters for a kind.
pub unsafe fn emit(table: Table, kind: u32, wheel: u32, object: u32) -> Result<(), Refused> {
    unsafe {
        match kind {
            1 if table == Table::Body => emit_particle_18::emit_particle_18(wheel, object),
            1 => emit_kind_1::emit_kind_1(wheel, object),
            3 if table == Table::Wheel => {
                emit_bursts_2::emit_bursts_2(wheel, object)?;
                emit_particle_6::emit_particle_6(wheel, object)
            }
            2..=4 | 6..=9 | 0xC => {
                emit_bursts::emit_bursts(wheel, object)?;
                emit_particle_6::emit_particle_6(wheel, object)
            }
            5 => emit_particle_5::emit_particle_5(wheel, object),
            0xB => emit_particle_11::emit_particle_11(wheel, object),
            _ => Ok(()),
        }
    }
}

/// The Rust version (see the header): the registers it leaves.
pub unsafe fn emit_wheel_particles(entry: &Regs) -> Result<Regs, Refused> {
    let mut g = *entry;
    let wheel = g.edi;
    if wheel == 0 {
        return Err(BadInput("a null wheel").into());
    }
    let written = regions(wheel);
    let saved = capture(&written);
    let done = unsafe {
        let (chance, _) = chance(wheel);
        game_write(CHANCE_RVA, chance);
        let r = next_c5(CHANCE_RANDOM_RVA);
        g.eax = r & 0xFFFF;
        if copies(chance, r) {
            let surface = [at(wheel + 0x28), at(wheel + 0x2C), at(wheel + 0x30)];
            for (n, v) in surface.iter().enumerate() {
                ((wheel + 0x1C + 4 * n as u32) as *mut u32).write_unaligned(*v);
            }
            [g.eax, g.ebx, g.ecx] = surface;
        }
        emit(Table::Wheel, at(wheel + 0x1C), wheel, g.esi)
    };
    if let Err(e) = done {
        restore(&written, &saved);
        return Err(e);
    }
    Ok(g)
}

/// What it may write: its globals, the wheel's +0x1C .. +0x24, and the emitters' (worked
/// out on the wheel as the copy would leave it, put back at once).
fn regions(wheel: u32) -> Vec<Region> {
    let mut r: Vec<Region> = [CHANCE_RVA, CHANCE_RANDOM_RVA].map(at_rva).to_vec();
    if wheel == 0 {
        return r;
    }
    r.push((wheel + 0x1C, 12));
    unsafe {
        let kept = [at(wheel + 0x1C), at(wheel + 0x20), at(wheel + 0x24)];
        let (chance, random) = chance(wheel);
        let put = |v: [u32; 3]| {
            for (n, v) in v.iter().enumerate() {
                ((wheel + 0x1C + 4 * n as u32) as *mut u32).write_unaligned(*v);
            }
        };
        if copies(chance, random) {
            put([at(wheel + 0x28), at(wheel + 0x2C), at(wheel + 0x30)]);
        }
        r.extend(emitter_regions(Table::Wheel, wheel));
        put(kept);
    }
    r
}

/// What the emitters for the record's kind (+0x1C) may write, the record as it is now.
pub unsafe fn emitter_regions(table: Table, wheel: u32) -> Vec<Region> {
    let mut r = Vec::new();
    unsafe {
        match at(wheel + 0x1C) {
            1 if table == Table::Body => r.extend(emit_particle_18::regions(wheel)),
            1 => r.extend(emit_kind_1::regions(wheel)),
            3 if table == Table::Wheel => {
                r.extend(emit_bursts::regions(&emit_bursts_2::SECOND, wheel));
                r.extend(emit_particle_6::regions(wheel));
            }
            2..=4 | 6..=9 | 0xC => {
                r.extend(emit_bursts::regions(&emit_bursts::FIRST, wheel));
                r.extend(emit_particle_6::regions(wheel));
            }
            5 => r.extend(emit_particle_5::regions(&emit_particle_5::FIFTH)),
            0xB => r.extend(emit_particle_5::regions(&emit_particle_11::ELEVENTH)),
            _ => {}
        }
        // The bursts and `emit_particle_6` after them take entries in turn: as many as
        // the larger burst flags allow (`(flags >> 6) * 4` a half, two halves), and one.
        let most = [at(wheel + 0x24), at(wheel + 0x30)]
            .map(|f| (f >> 6).saturating_mul(8))
            .into_iter()
            .max()
            .unwrap_or(0)
            .saturating_add(2)
            .min(MOST_TAKEN + 1);
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
    Ok(left_of(&unsafe { emit_wheel_particles(r) }?))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}

/// Every game variable the emitters write.
pub const WRITTEN: [usize; 39] = [
    CHANCE_RVA,
    CHANCE_RANDOM_RVA,
    rva(0x93A42A),
    rva(0x93A456),
    rva(0x939BAA),
    rva(0x765138),
    rva(0x923004),
    rva(0x935688),
    rva(0x93568C),
    rva(0x935690),
    rva(0x935694),
    rva(0x935698),
    rva(0x93A3D6),
    rva(0x93A3DA),
    rva(0x93A3DE),
    rva(0x93A3EA),
    rva(0x93A3EE),
    rva(0x93A3F2),
    rva(0x93A3FA),
    rva(0x93A3FE),
    rva(0x93A402),
    rva(0x93A40E),
    rva(0x93A412),
    rva(0x93A416),
    rva(0x93555E),
    rva(0x935432),
    rva(0x93A23A),
    rva(0x93A262),
    rva(0x93A1D6),
    rva(0x93567C),
    rva(0x935678),
    rva(0x93A39A),
    rva(0x93A3CA),
    rva(0x93A322),
    rva(0x93A34E),
    rva(0x935416),
    rva(0x93545E),
    rva(0x93543A),
    rva(0x935442),
];
