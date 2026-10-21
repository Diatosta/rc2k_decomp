//! `saves::list_car_numbers` (0x470825): two lists of numbers from two lists of pointers,
//! the other way round from `saves::rebuild_car_lists`.
//!
//! In:  nothing
//! Out: every register preserved
//! 0x28 dwords of -1 at `unk_755258` and 0x28 at `unk_7552F8` (the second list has 0x14
//! entries in `var\ral.dat`; the rest of what is filled is whatever lies after it); then
//! for each entry of `off_75A042` up to its -1 the first dword of what it points at goes
//! to the first, and likewise from `off_75A12E` to the second.
//! Callers: `sub_47486C` after the main menu is left, and `sub_486BD3`.
//! Refuses a list of pointers without its -1 within the numbers' room, and a null entry.

use crate::errors::{BadInput, Refused};
use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_read, game_write, hook_asm, rva};
use crate::screens::harness::Left;
use crate::service::harness::{Region, capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("saves::list_car_numbers", rva(0x470825), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- game data ----

/// How many dwords of each list of numbers are filled.
const FILLED: usize = 0x28;
/// (the numbers, the pointers)
const LISTS: [(usize, usize); 2] = [
    (rva(0x755258), rva(0x75A042)), // unk_755258, off_75A042
    (rva(0x7552F8), rva(0x75A12E)), // unk_7552F8, off_75A12E
];

// ---- Rust version ----

pub unsafe fn list_car_numbers() -> Result<(), BadInput> {
    // Converted: `rc2k_core::saves::cars::numbers`, both lists worked out before either is
    // written.
    unsafe {
        let mut found = Vec::new();
        for (_, pointers) in LISTS {
            let list: Vec<u32> = (0..FILLED)
                .map(|n| game_read::<u32>(pointers + n * 4))
                .collect();
            found.push(rc2k_core::saves::cars::numbers(&list, |car| {
                (car as *const u32).read_unaligned()
            })?);
        }
        for ((numbers, _), list) in LISTS.into_iter().zip(found) {
            for (n, v) in list.into_iter().enumerate() {
                game_write(numbers + n * 4, v);
            }
        }
        Ok(())
    }
}

// ---- hook ----

fn regions() -> [Region; 2] {
    LISTS.map(|(numbers, _)| (game_addr(numbers) as u32, FILLED * 4))
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    unsafe { list_car_numbers() }?;
    Ok(Left::nothing())
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &regions(), &rust);
}

// ---- self-test: lists of pointers to numbers of the test's, in the game's places ----

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let regions = regions();
    let pointers = LISTS.map(|(_, pointers)| (game_addr(pointers) as u32, FILLED * 4));
    let saved = (capture(&regions), capture(&pointers));
    // What the test's "cars" start with.
    let numbers: Vec<u32> = (0..0x30).map(|n| 0x100 + n * 7).collect();
    for (what, first, second) in [
        ("a few", 5usize, 3usize),
        ("none", 0, 0),
        ("one and many", 1, 0x13),
        ("full", 0x27, 0x27),
    ] {
        check_state(
            &mut t,
            what,
            call_original,
            Regs::SENTINEL,
            &|| unsafe {
                restore(&regions, &vec![0xAA; saved.0.len()]);
                for ((_, pointers), count) in LISTS.into_iter().zip([first, second]) {
                    for n in 0..count {
                        // The second list takes them from the other end.
                        let from = if pointers == LISTS[0].1 { n } else { 0x2F - n };
                        game_write(pointers + n * 4, &numbers[from] as *const u32 as u32);
                    }
                    game_write(pointers + count * 4, u32::MAX);
                }
            },
            &regions,
            &rust,
        );
    }
    restore(&regions, &saved.0);
    restore(&pointers, &saved.1);
    t.finish()
}
