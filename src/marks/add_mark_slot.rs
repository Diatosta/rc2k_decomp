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

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_read, game_write, hook_asm, rva};
use crate::marks::setup::{SLOT_LEN, SLOTS, SLOTS_RVA};
use crate::math::x87;
use crate::screens::harness::Left;
use crate::service::floats::{
    FLOAT, Floats, Tolerance, at_test_precisions, check_state_floats, dispatch_state_floats,
};
use crate::service::harness::{Region, at, capture, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new("marks::add_mark_slot", rva(0x4C99F4), thunk, &ORIGINAL)
    .private_copy(0x79)
    .manual_patch()
    .self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

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

/// The slots' floats.
fn float_places() -> Vec<(Region, Tolerance)> {
    (0..SLOTS)
        .map(|n| {
            (
                (game_addr(SLOTS_RVA) as u32 + SLOT_LEN * n + 0x10, 4),
                FLOAT,
            )
        })
        .collect()
}

fn regions() -> Vec<Region> {
    vec![
        (game_addr(SLOTS_RVA) as u32, (SLOTS * SLOT_LEN) as usize),
        at(RANDOM_RVA),
    ]
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    unsafe { add_mark_slot_plain(r.eax, r.esi) };
    Ok(Left::nothing())
}

extern "C" fn dispatch(r: &mut Regs) {
    let places = float_places();
    let floats = Floats {
        memory: &places,
        regs: &[],
    };
    dispatch_state_floats(&HOOK, r, call_original, &regions(), &floats, &rust);
}

/// A slot's first dword by its number.
type Filling = fn(usize) -> i32;

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let regions = regions();
    let saved = capture(&regions);
    let base: u32 = unsafe { game_read(BASE_RVA) };
    // Each slot's first dword by case: 1 busy, else free (0 or less).
    let fillings: [(&str, Filling); 5] = [
        ("all free", |_| 0),
        ("all busy", |_| 1),
        ("the last busy", |n| (n == 0x1F) as i32),
        ("only 3 free (-5)", |n| if n == 3 { -5 } else { 0x3C }),
        ("odd ones busy", |n| (n % 2) as i32 * 7),
    ];
    let places = float_places();
    let tolerant = Floats {
        memory: &places,
        regs: &[],
    };
    at_test_precisions(&tolerant, &mut |word, held| {
        for (case, (what, busy)) in fillings.into_iter().enumerate() {
            for random in [0u32, 0x1234_5678, 0xFFFF_FFFF] {
                check_state_floats(
                    &mut t,
                    "marks::add_mark_slot",
                    &format!("control word {word:#x}, {what}, random {random:#x}, base {base:#x}"),
                    call_original,
                    Regs {
                        eax: 0xA000 + case as u32,
                        esi: 0xB000_0000 + case as u32,
                        ..Regs::SENTINEL
                    },
                    &|| unsafe {
                        restore(&regions, &saved);
                        for n in 0..SLOTS as usize {
                            let slot = SLOTS_RVA + SLOT_LEN as usize * n;
                            game_write(
                                slot,
                                [
                                    busy(n) as u32,
                                    0xAAAA_AAAA,
                                    0xAAAA_AAAA,
                                    0xAAAA_AAAA,
                                    0xAAAA_AAAA,
                                ],
                            );
                        }
                        game_write(RANDOM_RVA, random);
                    },
                    &regions,
                    held,
                    &rust,
                );
            }
        }
    });
    restore(&regions, &saved);
    t.finish()
}
