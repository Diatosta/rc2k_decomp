//! `menu::add_clio_car` (0x471C33): puts the Clio into three lists of cars.
//!
//! In:  nothing
//! Out: every register preserved
//! For the lists at `off_759DCA`, `off_75A042` and `off_759E52`: if the car description
//! `unk_7574E0` is not in the list, it is written over the list's -1. No -1 is written
//! after it: the list then ends wherever the next -1 happens to be (the original's
//! slip, kept; in the image the lists are followed by -1 up to the next one).
//! One caller, the main menu's set-up `sub_47C63F`, when `g_hasClio` is 1 (the Clio
//! update's file is there).
//! Refuses a list without its end within its room.

use crate::errors::{BadInput, Refused};
use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_write, hook_asm, rva};
use crate::menu::remove_secret_cars::{FIRST, List, read_list, region, write_list};
use crate::screens::harness::Left;
use crate::service::harness::{Region, capture, check_state, dispatch_state, restore};
use rc2k_core::menu::car_lists::place_if_missing;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("menu::add_clio_car", rva(0x471C33), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- game data ----

const CLIO_RVA: usize = rva(0x7574E0); // unk_7574E0
/// The three lists, in the original's order, with the room each has before the next
/// variable.
const LISTS: [List; 3] = [
    (rva(0x759DCA), 0x22), // off_759DCA
    FIRST,                 // off_75A042
    (rva(0x759E52), 0x3E), // off_759E52
];

// ---- Rust version ----

pub unsafe fn add_clio_car() -> Result<(), BadInput> {
    unsafe {
        let clio = game_addr(CLIO_RVA) as u32;
        let mut ends = Vec::new();
        // Converted: `rc2k_core::menu::car_lists::place_if_missing`.
        for list in LISTS {
            if let Some(n) = place_if_missing(&read_list(list)?, clio) {
                ends.push(list.0 + n * 4);
            }
        }
        for end in ends {
            game_write(end, clio);
        }
        Ok(())
    }
}

// ---- hook ----

fn regions() -> [Region; 3] {
    LISTS.map(region)
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    unsafe { add_clio_car() }?;
    Ok(Left::nothing())
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &regions(), &rust);
}

// ---- self-test: lists of the test's in the game's places, put back ----

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let regions = regions();
    let saved = capture(&regions);
    let clio = game_addr(CLIO_RVA) as u32;
    let (a, b) = (0x1111_0000u32, 0x2222_0000u32);
    let cases: [(&str, [Vec<u32>; 3]); 5] = [
        ("in none", [vec![a, b], vec![b], vec![a, b, a]]),
        ("empty lists", [vec![], vec![], vec![]]),
        ("in all", [vec![clio], vec![a, clio, b], vec![a, b, clio]]),
        ("in the second only", [vec![a], vec![clio], vec![b]]),
        (
            "full lists",
            [
                (0..0x21).map(|n| a + n).collect(),
                (0..0x27).map(|n| b + n).collect(),
                (0..0x3D).map(|n| a + n).collect(),
            ],
        ),
    ];
    for (what, lists) in cases {
        check_state(
            &mut t,
            what,
            call_original,
            Regs::SENTINEL,
            &|| unsafe {
                // What follows a list's end is -1 here too, as in the image.
                restore(&regions, &vec![0xFF; saved.len()]);
                for (list, cars) in LISTS.into_iter().zip(&lists) {
                    write_list(list, cars);
                }
            },
            &regions,
            &rust,
        );
    }
    restore(&regions, &saved);
    t.finish()
}
