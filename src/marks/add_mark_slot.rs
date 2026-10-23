//! `marks::add_mark_slot` (0x4C99F4): the last free slot of `g_markSlots` (from 0x1F down,
//! free when its first dword is 0 or less) taken: 0x3C, eax, esi, 0, and a float of the
//! slot's own random (`dword_93540A` times 0xC5 plus 0x5D, its low word, plus
//! `flt_93540E`). Nothing when none is free.
//!
//! In:  eax, esi what the slot holds
//! Out: every register kept
//!
//! One caller, `sub_4C969A` (the marks' frame). The float first gets `flt_935406`, then
//! the random at once: only the second is kept. Manual patch with a private copy (its
//! first five bytes end with a jump). Self-test: slots of several fillings, the original
//! and the Rust version, at the test precisions; put back afterwards.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{Regs, game_read, game_write, rva};
use crate::marks::setup::{SLOT_LEN, SLOTS, SLOTS_RVA};
use crate::math::x87;
use crate::screens::harness::Left;
use crate::service::floats::floats_entry;

const RANDOM_RVA: usize = rva(0x93540A); // g_marksSlotRandom
const BASE_RVA: usize = rva(0x93540E); // flt_93540E
const TIME: u32 = 0x3C;

/// The Rust version (see the header).
pub unsafe fn add_mark_slot(what: u32, at_esi: u32) {
    unsafe {
        for n in (0..SLOTS as usize).rev() {
            let slot = SLOTS_RVA + SLOT_LEN as usize * n;
            if game_read::<i32>(slot) > 0 {
                continue;
            }
            let random = game_read::<u32>(RANDOM_RVA)
                .wrapping_mul(0xC5)
                .wrapping_add(0x5D);
            game_write(RANDOM_RVA, random);
            let float = x87::int_plus((random & 0xFFFF) as i32, game_read(BASE_RVA));
            game_write(slot, [TIME, what, at_esi, 0, float]);
            return;
        }
    }
}

/// The converted `add_mark_slot` (plain floats): the hook's. The x87 version above stays
/// for `marks::draw_mark_slots` (unconverted).
// Converted: `rc2k_core::marks::slots`
pub unsafe fn add_mark_slot_plain(what: u32, at_esi: u32) {
    use rc2k_core::marks::slots;
    unsafe {
        let first = |n: usize| game_read::<i32>(SLOTS_RVA + SLOT_LEN as usize * n);
        let Some(n) = slots::free_slot(first) else {
            return;
        };
        let random = slots::next_random(game_read(RANDOM_RVA));
        game_write(RANDOM_RVA, random);
        let base = f32::from_bits(game_read(BASE_RVA));
        game_write(
            SLOTS_RVA + SLOT_LEN as usize * n,
            slots::new_slot(what, at_esi, random, base),
        );
    }
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    unsafe { add_mark_slot_plain(r.eax, r.esi) };
    Ok(Left::nothing())
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    floats_entry(r, &rust)
}
