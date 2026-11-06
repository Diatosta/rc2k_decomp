//! `effects::tidy_marks` (0x4D146D): the marks' memory given back at a stage's unload. Its
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

use crate::effects::blocks::{MARK_LINKS, PLACES_BLOCK};
use crate::effects::mask_store::MASK_BLOCKS;
use crate::effects::morph_object_points::MORPH_WORK;
use crate::effects::ring_store::Ring;
use crate::errors::{BadInput, Refused};
use crate::memory::free_list::free_list;
use crate::memory::{MemoryList, own_heap};
use crate::pointers::GamePtr;

// `sub_4D06F3`'s ebp: the settings' +0x29C for car 0.
const CAR_PART: u32 = crate::race::marks_settings_store::CAR_BLOCK;
const MOST_CARS: i32 = 8;
/// The six lists, in the order freed.
fn lists(ring: &Ring) -> [GamePtr; 6] {
    [
        GamePtr::to(&ring.pool.list),     // g_marksRingList
        GamePtr::to(&ring.work.list),     // stru_935620
        GamePtr::to(&MASK_BLOCKS.list_b), // g_marksBMaskList
        GamePtr::to(&MASK_BLOCKS.list_a), // stru_934D76
        GamePtr::to(&PLACES_BLOCK.list),  // g_marksPlacesList
        GamePtr::to(&MARK_LINKS.list),    // g_marksLinksList
    ]
}
/// Each car's list, worked out without writing (none for a count past the refusals).
fn car_lists() -> Result<Vec<GamePtr>, BadInput> {
    let marks = crate::effects::setup::marks()?;
    let count = (marks.cars_copy.get() as i32).clamp(0, MOST_CARS) as u32;
    Ok((0..count)
        .map(|n| marks.cars[n as usize].at(CAR_PART + 0x4C))
        .collect())
}

/// Every list it frees, the cars' first.
fn all_lists(ring: &Ring) -> Result<Vec<GamePtr>, BadInput> {
    let mut r = car_lists()?;
    r.extend(lists(ring));
    Ok(r)
}

/// The Rust version (see the header).
pub fn tidy_marks(ring: &Ring) -> Result<(), Refused> {
    let marks = crate::effects::setup::marks()?;
    let count = marks.cars_copy.get() as i32;
    if !(0..=MOST_CARS).contains(&count) {
        return Err(BadInput("a cars' count below 0 or above 8").into());
    }
    if own_heap() == 1
        && crate::effects::try_any(all_lists(ring)?.iter(), |&l| {
            Ok(l.read::<MemoryList>("a car's block list")?.head != 0)
        })?
    {
        return Err(BadInput("the game's own heap is in use with a list holding blocks").into());
    }
    for n in 0..count as u32 {
        let car = &marks.cars[n as usize];
        free_list(car.at(CAR_PART + 0x4C))?;
        if let Some(points) = crate::effects::lines::BLOCK_POINTS.get(n as usize) {
            points.give_back();
        }
        car.put(CAR_PART + 0x8C, MORPH_WORK.taken.get())?;
    }
    for list in lists(ring) {
        free_list(list)?;
    }
    // The particles and the masks went with their lists.
    crate::effects::owner::effects().particles.ring.give_back();
    ring.entries.give_back();
    crate::effects::mask_store::MASK.give_back();
    crate::effects::mask_store::BMASK.give_back();
    Ok(())
}
