//! `marks::emit_particle_18` (0x4CDF3A, a chunk of `sub_4CA13B` entered by `jz` from
//! 0x4CA1C5 for the record's kind 1, left by `popa; retn`): ring entries of kind 0x12.
//!
//! In:  edi the record (the car object's +0x1F4), esi the car object
//! Out: every register kept
//!
//! `g_marksWheelFlags` the record's +0x20; the count (`dword_9356C0`): the record's +0x38
//! by `flt_7651D0` (held to 1, `fld1`, unless under `flt_76516C`) times four times the
//! flags >> 6, rounded; that many entries (`dec; jns`: none for 0 or less, but for
//! 0x80000000), until the ring is full. Each: its motion `flt_93546A` .. moved by
//! `marks::jitter_point` (spread `flt_765198`), y made positive; its place the record's
//! (+0x3C ..), less `flt_7651DC` in y at +0x28; +0x1C .. +0x24 0; a life of 300 (+0x78
//! `flt_7651A4` over it); +0x68 `g_marksCar`. The loop reuses esi for the entry: only the
//! first entry's tag (+0x90) is the object's +0x1C, the later ones the previous entry's
//! (0; section 16). Refuses (before anything) a null record, and a null object when an
//! entry is made. Self-test: a ring, record and car of the test's
//! (`marks::emit_particle_6::TestEmitter`), several sizes, flags and fillings.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::{BadInput, Refused};
use crate::framework::{Regs, game_read, game_write, rva};
use crate::marks::emitters::*;
use crate::marks::jitter_point::jitter_point;
use crate::math::x87;
use crate::scene::matrix3::{extended, rounded};
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

const COUNT_RVA: usize = rva(0x9356C0); // dword_9356C0
const SIZE_K_RVA: usize = rva(0x7651D0); // flt_7651D0
/// flt_93546A, flt_93546E, flt_935472: the motion (x, y, z).
const MOTION_RVA: [usize; 3] = [rva(0x93546A), rva(0x93546E), rva(0x935472)];
const SPREAD_RVA: usize = rva(0x765198); // flt_765198
const DROP_RVA: usize = rva(0x7651DC); // flt_7651DC
const LIFE_SCALE_RVA: usize = rva(0x7651A4); // flt_7651A4
const LIFE: u32 = 0x12C;
const ONE: u32 = 0x3F80_0000; // fld1

/// The count (and the flags' part it is scaled by), worked out without writing.
unsafe fn count(record: u32) -> (i32, i32) {
    unsafe {
        let n = ((at(record + 0x20) >> 6) << 2) as i32;
        let (size, k) = (at(record + 0x38), game_read::<u32>(SIZE_K_RVA));
        let under = x87::times_compare(size, k, game_read(ONE_RVA)) & 0x700 == 0x100;
        let count = if under {
            x87::times_int_to_int(size, k, n)
        } else {
            x87::float_times_int_to_int(ONE, n)
        };
        (count, n)
    }
}

/// How many entries are made, worked out without writing.
unsafe fn made(record: u32) -> u32 {
    unsafe {
        let (count, _) = count(record);
        let wanted = if count == i32::MIN {
            1 << 31
        } else {
            count.max(0) as u32
        };
        wanted.min(MOST_TAKEN.saturating_sub(game_read(RING_COUNT_RVA)))
    }
}

/// The Rust version (see the header).
pub unsafe fn emit_particle_18(record: u32, object: u32) -> Result<(), Refused> {
    unsafe {
        if record == 0 {
            return Err(BadInput("a null record").into());
        }
        if object == 0 && made(record) > 0 {
            return Err(BadInput("a null object").into());
        }
        let flags = at(record + 0x20);
        game_write(FLAGS_RVA, flags);
        let (count, n) = count(record);
        game_write(COUNT_RVA, n);
        game_write(COUNT_RVA, count);
        let mut left = count;
        let mut tag_from = object;
        loop {
            left = left.wrapping_sub(1);
            if left < 0 || next_entry().is_none() {
                break;
            }
            let e = take_entry(at(tag_from + 0x1C) & 0xFF_FFFF, 0x12);
            tag_from = e;
            let put = |o: u32, v: u32| ((e + o) as *mut u32).write_unaligned(v);
            let m = MOTION_RVA.map(|r| extended(game_read::<u32>(r)));
            let (moved, _) = jitter_point([extended(game_read(SPREAD_RVA)), m[2], m[1], m[0]]);
            put(0x34, rounded(moved[0]));
            put(0x30, rounded(moved[1]) & 0x7FFF_FFFF);
            put(0x2C, rounded(moved[2]));
            put(0x10, x87::loaded(at(record + 0x3C)));
            put(0x14, x87::loaded(at(record + 0x40)));
            put(0x28, x87::sub(at(record + 0x40), game_read(DROP_RVA)));
            put(0x18, x87::loaded(at(record + 0x44)));
            for o in [0x1C, 0x20, 0x24] {
                put(o, 0);
            }
            put(0x84, LIFE);
            put(0x68, game_read(OBJECT_RVA));
            put(0x78, x87::over_int(game_read(LIFE_SCALE_RVA), LIFE as i32));
        }
    }
    Ok(())
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    unsafe { emit_particle_18(r.edi, r.esi) }?;
    Ok(Left::nothing())
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
