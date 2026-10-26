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
use crate::marks::blocks::{MARK_LINKS, PLACES_BLOCK, RING_POOL};
use crate::marks::mask_store::MASK_BLOCKS;
use crate::marks::morph_object_points::MORPH_WORK;
use crate::memory::free_list::free_list;
use crate::memory::{MemoryList, own_heap};
use crate::pointers::GamePtr;

const CAR_LEN: u32 = 0x4C8;
const CAR_PART: u32 = 0x29C; // `sub_4D06F3`'s ebp: the car's +0x29C
const MOST_CARS: i32 = 8;
/// The six lists, in the order freed.
fn lists() -> [u32; 6] {
    [
        RING_POOL.list[0].addr(),                        // g_marksRingList
        crate::marks::ring_store::RING_WORK.list.addr(), // stru_935620
        MASK_BLOCKS.list_b.addr(),                       // g_marksBMaskList
        MASK_BLOCKS.list_a.addr(),                       // stru_934D76
        PLACES_BLOCK.list.addr(),                        // g_marksPlacesList
        MARK_LINKS.list.addr(),                          // g_marksLinksList
    ]
}
/// Each car's list, worked out without writing (none for a count past the refusals).
fn car_lists(settings: u32) -> Result<Vec<u32>, BadInput> {
    let count = GamePtr::new(settings + 0x5C)
        .read::<i32>("the cars' count")?
        .clamp(0, MOST_CARS) as u32;
    Ok((0..count)
        .map(|n| settings + n * CAR_LEN + CAR_PART + 0x4C)
        .collect())
}

/// Every list it frees, the cars' first.
fn all_lists(settings: u32) -> Result<Vec<u32>, BadInput> {
    let mut r = car_lists(settings)?;
    r.extend(lists());
    Ok(r)
}

/// The Rust version (see the header).
pub fn tidy_marks() -> Result<(), Refused> {
    let settings: u32 = crate::marks::setup::SETTINGS.pointer.get();
    if settings == 0 {
        return Err(BadInput("no marks settings").into());
    }
    let count = GamePtr::new(settings + 0x5C).read::<i32>("the cars' count")?;
    if !(0..=MOST_CARS).contains(&count) {
        return Err(BadInput("a cars' count below 0 or above 8").into());
    }
    if own_heap() == 1
        && crate::marks::try_any(all_lists(settings)?.iter(), |&l| {
            Ok(GamePtr::new(l)
                .read::<MemoryList>("a car's block list")?
                .head
                != 0)
        })?
    {
        return Err(BadInput("the game's own heap is in use with a list holding blocks").into());
    }
    for n in 0..count as u32 {
        let car = settings + n * CAR_LEN + CAR_PART;
        free_list(crate::pointers::host(car + 0x4C) as *mut MemoryList)?;
        GamePtr::new(car + 0x8C).write::<u32>(MORPH_WORK.taken.get(), "a mark's field")?;
    }
    for a in lists() {
        free_list(crate::pointers::host(a) as *mut MemoryList)?;
    }
    Ok(())
}
