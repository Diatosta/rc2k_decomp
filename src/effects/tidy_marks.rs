//! `effects::tidy_marks` (0x4D146D): the marks' memory given back at a stage's unload. Its
//! steps, each a call (the nullsubs left out): 0x4D06F3 (for each of the settings'
//! +0x5C cars from the first, its list at +0x2E8 freed and its +0x328 made `g_marksTaken`),
//! then `memory::free_list` of `g_marksRingList` (0x4CFB3D), `stru_935620`
//! (0x4CB5A8), `g_marksBMaskList` and `stru_934D76` (0x4C8F4D), `g_marksPlacesList`
//! (0x4C38F5) and `g_marksLinksList` (0x4C4E22 -> 0x4C3EE3 -> 0x4C5409).
//! Each of those steps is a `lea ebx` and a call.
//!
//! Works on `g_marksSettings`.
//!
//! 0x4D06F3 tests `memory::free_list`'s carry (`jb`), which is always clear (its last
//! flag-setting instruction is a `test`), so its other path (a `push` left on the stack and
//! a count of 0x1FFFFFFF: section 16) is never taken. Caller: the stage's unload
//! (0x4449AE). Refuses (before anything) null settings, a cars' count below 0 or above 8
//! (the original's loop runs until it meets the count), and a list that holds a block
//! (`free_list`'s refusal; none is put on a list since the own heap's removal, 2026-11-07).

use crate::effects::mask_store::Masks;
use crate::effects::owner::Effects;
use crate::effects::owner::effects;
use crate::effects::ring_store::Ring;
use crate::errors::{BadInput, Refused};
use crate::memory::MemoryList;
use crate::memory::free_list::{free_list, free_list_at};
use crate::pointers::{GamePtr, Store, load_value};

// 0x4D06F3's ebp: the settings' +0x29C for car 0.
const MOST_CARS: i32 = 8;
/// The six lists, in the order freed.
fn lists<'a>(ring: &'a Ring, masks: &'a Masks) -> [&'a dyn Store; 6] {
    let blocks = &effects().blocks;
    [
        &ring.pool.list,      // g_marksRingList
        &ring.work.list,      // stru_935620
        &masks.blocks.list_b, // g_marksBMaskList
        &masks.blocks.list_a, // stru_934D76
        &blocks.places.list,  // g_marksPlacesList
        &blocks.links.list,   // g_marksLinksList
    ]
}
/// Each car's list, worked out without writing (none for a count past the refusals).
fn car_lists() -> Result<Vec<GamePtr>, BadInput> {
    let marks = crate::effects::setup::marks()?;
    let count = (marks.cars_copy.get() as i32).clamp(0, MOST_CARS) as u32;
    Ok((0..count)
        .map(|n| GamePtr::to(&marks.cars[n as usize].points_list))
        .collect())
}

/// Whether any list it frees holds blocks, the cars' first.
fn any_holds_blocks(ring: &Ring, masks: &Masks) -> Result<bool, Refused> {
    let what = "a car's block list";
    Ok(crate::effects::try_any(car_lists()?.iter(), |&l| {
        Ok(l.read::<MemoryList>(what)?.head != 0)
    })? || crate::effects::try_any(lists(ring, masks).iter(), |&l| {
        Ok(load_value::<MemoryList>(l, 0)?.head != 0)
    })?)
}

/// The Rust version (see the header).
pub fn tidy_marks(effects: &Effects) -> Result<(), Refused> {
    let (paths, ring, masks) = (&effects.paths, &effects.ring, &effects.masks);
    let marks = crate::effects::setup::marks()?;
    let count = marks.cars_copy.get() as i32;
    if !(0..=MOST_CARS).contains(&count) {
        return Err(BadInput("a cars' count below 0 or above 8").into());
    }
    if any_holds_blocks(ring, masks)? {
        return Err(BadInput("a memory list holding a block").into());
    }
    for n in 0..count as u32 {
        let car = &marks.cars[n as usize];
        free_list_at(GamePtr::to(&car.points_list))?;
        if let Some(points) = paths.block_points.get(n as usize) {
            points.give_back();
        }
        car.point_block.set(paths.morph_work.taken.get());
    }
    for list in lists(ring, masks) {
        free_list(list)?;
    }
    // The particles and the masks went with their lists.
    effects.particles.ring.give_back();
    ring.entries.give_back();
    masks.mask.give_back();
    masks.bmask.give_back();
    Ok(())
}
