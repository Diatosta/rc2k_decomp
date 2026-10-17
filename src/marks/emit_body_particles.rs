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

use crate::errors::{BadInput, Refused};
use crate::framework::{RawHook, Regs, SelfTest, game_read, game_write, hook_asm, rva};
use crate::marks::emit_particle_6::{TEST_ENTRIES, TestEmitter};
use crate::marks::emit_wheel_particles::{Table, WRITTEN, emit, emitter_regions};
use crate::marks::emitters::*;
use crate::marks::setup::SEED_RVA;
use crate::math::x87;
use crate::race::stage_prefs::left_of;
use crate::screens::harness::Left;
use crate::service::harness::{
    Region, at as at_rva, capture, check_state, dispatch_state, restore,
};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "marks::emit_body_particles",
    rva(0x4CA13B),
    thunk,
    &ORIGINAL,
)
.self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

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

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &regions(r.edi, r.esi), &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let mut extra = WRITTEN.to_vec();
    extra.push(CHANCE_RANDOM_RVA);
    let mut test = TestEmitter::new(&extra);
    let mut regions = test.regions();
    regions.push((test.object.as_ptr() as u32, 4 * test.object.len()));
    let mut case = 0u32;
    // (the record's +0x1C, +0x28, the chance's +0x34, its +0)
    for (now, surface, chance, bit) in [
        (0u32, 0u32, 0.0f32, 0u32),
        (0, 1, 1e9, 1),
        (1, 0, 0.0, 0),
        (2, 2, 0.0, 1),
        (3, 0, 0.0, 0),
        (0, 3, 1e9, 0),
        (5, 5, 0.0, 1),
        (6, 7, 1e9, 0),
        (8, 9, 0.5, 3),
        (0xA, 0xB, 1e9, 0),
        (0xB, 0xC, 0.0, 1),
        (0xC, 0xD, 1e9, 2),
    ] {
        case += 1;
        let ptrs = (
            test.wheel.as_mut_ptr(),
            test.object.as_mut_ptr(),
            test.matrix.as_mut_ptr(),
        );
        let seed = case.wrapping_mul(0x2545_F491);
        test.prepare(case % 3);
        check_state(
            &mut t,
            &format!("kind {now:#x}, surface {surface:#x}, chance {chance}, +0 {bit}"),
            call_original,
            Regs {
                edi: ptrs.0 as u32,
                esi: ptrs.1 as u32,
                ..Regs::SENTINEL
            },
            &|| unsafe {
                restore(&regions, &vec![0xAA; regions.iter().map(|r| r.1).sum()]);
                let base = test.ring.as_ptr() as u32;
                for n in 0..TEST_ENTRIES {
                    let e = (base + n * ENTRY_LEN as u32) as *mut u32;
                    e.write_unaligned(base + ((n + 1) % TEST_ENTRIES) * ENTRY_LEN as u32);
                }
                game_write(RING_CURSOR_RVA, base + 3 * ENTRY_LEN as u32);
                game_write(RING_COUNT_RVA, case % 3);
                game_write(OBJECT_RVA, ptrs.1 as u32);
                game_write(SEED_RVA, seed);
                for (n, r) in extra.iter().enumerate() {
                    game_write(*r, seed ^ (n as u32).wrapping_mul(0x0101_0101));
                }
                for r in [rva(0x93545E), rva(0x93543A), rva(0x935442)] {
                    game_write(r, 0.5f32.to_bits());
                }
                let w = ptrs.0;
                for n in 0..0x20 {
                    *w.add(n) = (n as f32 * 0.75 - 3.0).to_bits();
                }
                *w = bit;
                *w.add(1) = 0x5000;
                *w.add(3) = 3.0f32.to_bits();
                *w.add(4) = 50.0f32.to_bits();
                *w.add(6) = (-20.0f32).to_bits();
                *w.add(7) = now;
                *w.add(8) = 0xC5;
                *w.add(9) = 0x85;
                *w.add(10) = surface;
                *w.add(11) = 0x47;
                *w.add(12) = 0xC3;
                *w.add(13) = chance.to_bits();
                *w.add(14) = 0.5f32.to_bits();
                let o = ptrs.1;
                for n in 0..0x30 {
                    *o.add(n) = (n as f32 * 1.25 + 0.5).to_bits();
                }
                *o.add(0x1C / 4) = 0x1234_5678;
                *o.add(0x24 / 4) = ptrs.2 as u32;
                *o.add(0x88 / 4) = 0x77;
                *o.add(0x90 / 4) = 0x2000;
                for n in 0..0x10 {
                    *ptrs.2.add(n) = ((n as f32 - 5.0) * 0.3).to_bits();
                }
            },
            &regions,
            &rust,
        );
    }
    drop(test);
    t.finish()
}
