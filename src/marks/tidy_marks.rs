//! `marks::tidy_marks` (0x4D146D): the marks' memory given back at a stage's unload. Its
//! steps, each a call (the nullsubs left out): `sub_4D06F3` (for each of the settings'
//! +0x5C cars from the first, its list at +0x2E8 freed and its +0x328 made `g_marksTaken`),
//! then `memory::free_list` of `g_marksRingList` (`sub_4CFB3D`), `stru_935620`
//! (`sub_4CB5A8`), `g_marksBMaskList` and `stru_934D76` (`sub_4C8F4D`), `g_marksPlacesList`
//! (`sub_4C38F5`) and `g_marksLinksList` (`sub_4C4E22` -> `sub_4C3EE3` -> `sub_4C5409`).
//! Each of those steps is a `lea ebx` and a call.
//!
//! In:  nothing (`g_marksSettings`)
//! Out: nothing (the stage's unload reads no register it leaves)
//!
//! `sub_4D06F3` tests `memory::free_list`'s carry (`jb`), which is always clear (its last
//! flag-setting instruction is a `test`), so its other path (a `push` left on the stack and
//! a count of 0x1FFFFFFF: section 16) is never taken. Caller: the stage's unload
//! (0x4449AE). Refuses (before anything) null settings, a cars' count below 0 or above 8
//! (the original's loop runs until it meets the count), and the game's own heap with a list
//! that holds blocks (`free_list`'s refusal).

use crate::errors::{BadInput, Refused};
use crate::marks::lines::TAKEN_RVA;
use crate::marks::store::{Slot, game_addr, game_read};
use crate::memory::free_list::free_list;
use crate::memory::{MemoryList, own_heap};

const CAR_LEN: u32 = 0x4C8;
const CAR_PART: u32 = 0x29C; // `sub_4D06F3`'s ebp: the car's +0x29C
const MOST_CARS: i32 = 8;
/// The six lists, in the order freed.
const LISTS_RVA: [Slot; 6] = [
    Slot::at(0x93A580), // g_marksRingList
    Slot::at(0x935620), // stru_935620
    Slot::at(0x934DBA), // g_marksBMaskList
    Slot::at(0x934D76), // stru_934D76
    Slot::at(0x8DB570), // g_marksPlacesList
    Slot::at(0x9206F0), // g_marksLinksList
];
/// Each car's list, worked out without writing (none for a count past the refusals).
unsafe fn car_lists(settings: u32) -> Vec<u32> {
    unsafe {
        let count = (*((settings + 0x5C) as *const i32)).clamp(0, MOST_CARS) as u32;
        (0..count)
            .map(|n| settings + n * CAR_LEN + CAR_PART + 0x4C)
            .collect()
    }
}

/// Every list it frees, the cars' first.
unsafe fn all_lists(settings: u32) -> Vec<u32> {
    unsafe {
        let mut r = car_lists(settings);
        r.extend(LISTS_RVA.map(|a| game_addr(a) as u32));
        r
    }
}

/// The Rust version (see the header).
pub unsafe fn tidy_marks() -> Result<(), Refused> {
    unsafe {
        let settings: u32 = crate::marks::setup::SETTINGS.pointer.get();
        if settings == 0 {
            return Err(BadInput("no marks settings").into());
        }
        let count = *((settings + 0x5C) as *const i32);
        if !(0..=MOST_CARS).contains(&count) {
            return Err(BadInput("a cars' count below 0 or above 8").into());
        }
        if own_heap() == 1
            && all_lists(settings)
                .iter()
                .any(|&l| (*(l as *const MemoryList)).head != 0)
        {
            return Err(
                BadInput("the game's own heap is in use with a list holding blocks").into(),
            );
        }
        for n in 0..count as u32 {
            let car = settings + n * CAR_LEN + CAR_PART;
            free_list((car + 0x4C) as *mut MemoryList)?;
            ((car + 0x8C) as *mut u32).write_unaligned(game_read(TAKEN_RVA));
        }
        for a in LISTS_RVA {
            free_list(game_addr(a) as *mut MemoryList)?;
        }
    }
    Ok(())
}
