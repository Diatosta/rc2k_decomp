//! `effects::tidy_marks` (0x4D146D): the effects' memory given back at a stage's unload. Its
//! steps, each a call (the nullsubs left out): for each of the settings' +0x5C cars from the
//! first, its list at +0x2E8 freed and its +0x328 made the taken cell, then `memory::free_list`
//! of the ring list, the surface list, the two mask lists, the places list and the links list.
//!
//! Works on the effects settings.
//!
//! The cars' step tests `memory::free_list`'s carry, which is always clear (its last
//! flag-setting instruction is a `test`), so its other path (a `push` left on the stack and
//! a count of 0x1FFFFFFF: section 16) is never taken. Caller: the stage's unload. Refuses
//! (before anything) null settings, a cars' count below 0 or above 8 (the original's loop runs
//! until it meets the count), and a list that holds a block (`free_list`'s refusal; no block
//! is put on a list any more).

use crate::effects::mask_store::Masks;
use crate::effects::owner::Effects;
use crate::effects::owner::effects;
use crate::effects::ring_store::Ring;
use crate::errors::{BadInput, Refused};
use crate::memory::MemoryList;
use crate::memory::free_list::free_list;
use crate::pointers::{Store, load_value};

// The cars' loop starts from the settings' +0x29C (car 0).
const MOST_CARS: i32 = 8;
/// The six lists, in the order freed.
fn lists<'a>(ring: &'a Ring, masks: &'a Masks) -> [&'a dyn Store; 6] {
    let blocks = &effects().blocks;
    [
        &ring.pool.list,      // g_marksRingList
        &ring.work.list,      // RingWork.list
        &masks.blocks.list_b, // g_marksBMaskList
        &masks.blocks.list_a, // MaskBlocks.list_a[0]
        &blocks.places.list,  // g_marksPlacesList
        &blocks.links.list,   // g_marksLinksList
    ]
}
/// Each car's list, worked out without writing (none for a count past the refusals).
fn car_lists(effects: &Effects) -> Result<Vec<&dyn Store>, BadInput> {
    let marks = effects.settings()?;
    let count = (marks.cars_copy.get() as i32).clamp(0, MOST_CARS) as u32;
    Ok((0..count)
        .map(|n| &marks.cars[n as usize].points_list as &dyn Store)
        .collect())
}

/// Whether any list it frees holds blocks, the cars' first.
fn any_holds_blocks(effects: &Effects, ring: &Ring, masks: &Masks) -> Result<bool, Refused> {
    Ok(crate::effects::try_any(car_lists(effects)?.iter(), |&l| {
        Ok(load_value::<MemoryList>(l, 0)?.head != 0)
    })? || crate::effects::try_any(lists(ring, masks).iter(), |&l| {
        Ok(load_value::<MemoryList>(l, 0)?.head != 0)
    })?)
}

pub fn tidy_marks(effects: &Effects) -> Result<(), Refused> {
    let (paths, ring, masks) = (&effects.paths, &effects.ring, &effects.masks);
    let marks = effects.settings()?;
    let count = marks.cars_copy.get() as i32;
    if !(0..=MOST_CARS).contains(&count) {
        return Err(BadInput("a cars' count below 0 or above 8").into());
    }
    if any_holds_blocks(effects, ring, masks)? {
        return Err(BadInput("a memory list holding a block").into());
    }
    for n in 0..count as u32 {
        let car = &marks.cars[n as usize];
        free_list(&car.points_list)?;
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
