//! `marks::emit_body_particles` (0x4CA13B): the particles of the car's fifth record
//! (+0x1F4, under the body). First the object's +0x90 times `flt_765174` (rounded), the
//! record's +4 plus 0x386A0, its +0 bit 0 flipped (`dword_935416` 2, or 3 when the bit is
//! now clear), its +0xC 0; then `marks::emit_wheel_particles`' chance (its own random
//! `dword_935526`) of copying the surface's kind and flags, and its switch, but for kind 1
//! (`marks::emit_particle_18`) and kind 3 (the bursts of kind 2: section 16).
//!
//! In:  edi the record, esi the car object
//! Out: eax, ebx, ecx the record's +0x28 .. +0x30 when copied (else eax the random's low
//!      word); every other register kept
//!
//! Caller: `sub_4C9E2E`'s end. Refuses a null record or object and the emitters'
//! refusals; on a refusal nothing stays written (all it may write is put back).
//! Self-test: a ring, record and car of the test's (`marks::emit_particle_6::TestEmitter`),
//! every kind, with and without the copy.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::{BadInput, Refused};
use crate::framework::{Regs, game_read, game_write, rva};
use crate::marks::emit_wheel_particles::{Table, emit, emitter_regions};
use crate::marks::emitters::*;
use crate::math::x87;
use crate::race::stage_prefs::left_of;
use crate::screens::harness::Left;
use crate::service::harness::{Region, at as at_rva, capture, restore, state_entry};

const SIZE_K_RVA: usize = rva(0x765174); // flt_765174
const WHICH_RVA: usize = rva(0x935416); // dword_935416: the record's number
const CHANCE_RVA: usize = rva(0x9356C0); // dword_9356C0
const CHANCE_K_RVA: usize = rva(0x7651DC); // flt_7651DC
const CHANCE_RANDOM_RVA: usize = rva(0x935526); // dword_935526
const PACE: u32 = 0x386A0;

/// The first part (see the header), written; the chance's random returned.
unsafe fn prepare(g: &mut Regs) -> u32 {
    unsafe {
        let (record, object) = (g.edi, g.esi);
        let size = x87::int_times_to_int(at(object + 0x90) as i32, game_read(SIZE_K_RVA));
        ((object + 0x90) as *mut i32).write_unaligned(size);
        ((record + 4) as *mut u32).write_unaligned(at(record + 4).wrapping_add(PACE));
        game_write(WHICH_RVA, 2u32);
        let flip = at(record) ^ 1;
        (record as *mut u32).write_unaligned(flip);
        if flip & 1 == 0 {
            game_write(WHICH_RVA, 3u32);
        }
        ((record + 0xC) as *mut u32).write_unaligned(0);
        let chance = x87::mul_to_int(at(record + 0x34), game_read(CHANCE_K_RVA));
        game_write(CHANCE_RVA, chance);
        let r = next_c5(CHANCE_RANDOM_RVA);
        g.eax = r & 0xFFFF;
        if ((r & 0xFFFF) as i32) < chance {
            let surface = [at(record + 0x28), at(record + 0x2C), at(record + 0x30)];
            for (n, v) in surface.iter().enumerate() {
                ((record + 0x1C + 4 * n as u32) as *mut u32).write_unaligned(*v);
            }
            [g.eax, g.ebx, g.ecx] = surface;
        }
        r
    }
}

/// The Rust version (see the header): the registers it leaves.
pub unsafe fn emit_body_particles(entry: &Regs) -> Result<Regs, Refused> {
    let mut g = *entry;
    if g.edi == 0 || g.esi == 0 {
        return Err(BadInput("a null record or object").into());
    }
    let written = regions(g.edi, g.esi);
    let saved = capture(&written);
    let done = unsafe {
        prepare(&mut g);
        emit(Table::Body, at(g.edi + 0x1C), g.edi, g.esi)
    };
    if let Err(e) = done {
        restore(&written, &saved);
        return Err(e);
    }
    Ok(g)
}

/// The record's and the object's fields it writes, and its globals.
fn own(record: u32, object: u32) -> Vec<Region> {
    let mut r: Vec<Region> = [WHICH_RVA, CHANCE_RVA, CHANCE_RANDOM_RVA]
        .map(at_rva)
        .to_vec();
    if record != 0 && object != 0 {
        r.extend([
            (object + 0x90, 4),
            (record, 8),
            (record + 0xC, 4),
            (record + 0x1C, 12),
        ]);
    }
    r
}

/// What it may write: its own and the emitters' (worked out on the record as the first
/// part leaves it, put back at once).
fn regions(record: u32, object: u32) -> Vec<Region> {
    let mut r = own(record, object);
    if record == 0 || object == 0 {
        return r;
    }
    let saved = capture(&r);
    unsafe {
        let mut g = Regs {
            edi: record,
            esi: object,
            ..Regs::default()
        };
        prepare(&mut g);
        r.extend(emitter_regions(Table::Body, record));
    }
    restore(&own(record, object), &saved);
    r
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(left_of(&unsafe { emit_body_particles(r) }?))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
