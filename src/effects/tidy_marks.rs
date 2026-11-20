//! `effects::tidy_marks`: the effects' memory given back at a stage's unload. Its steps: for each
//! of the settings' `cars_copy` cars from the first, its `points_list` freed and its `point_block`
//! made the taken cell, then `memory::free_list` of the ring list, the surface list, the two mask lists, the
//! places list and the links list.
//!
//! Works on the effects settings.
//!
//! Refuses (before anything) null settings, a cars' count below 0 or above 8, and a list that holds
//! a block (`free_list`'s refusal; no block is put on a list any more).

use crate::effects::mask_store::MaskBlockList;
use crate::effects::owner::Effects;
use crate::errors::{BadInput, Refused};
use crate::memory::free_list::free_list;
use crate::memory::{HeldList, ListCells};

// The cars' loop starts from the first car's record.
const MOST_CARS: i32 = 8;
/// The six lists, in the order freed.
fn with_lists<R>(effects: &Effects, f: impl FnOnce([&dyn HeldList; 6]) -> R) -> R {
    let (ring, masks, blocks) = (&effects.ring, &effects.masks, &effects.blocks);
    let (second, first) = (
        MaskBlockList::new(&masks.blocks, true),
        MaskBlockList::new(&masks.blocks, false),
    );
    f([
        &ring.pool.list,
        &ring.work,
        &second,
        &first,
        &blocks.places.list,
        &blocks.links.list,
    ])
}
/// Each car's list, worked out without writing (none for a count past the refusals).
fn car_lists(effects: &Effects) -> Result<Vec<&ListCells>, BadInput> {
    let marks = effects.settings()?;
    let count = (marks.cars_copy.get() as i32).clamp(0, MOST_CARS) as u32;
    Ok((0..count)
        .map(|n| &marks.cars[n as usize].points_list)
        .collect())
}

/// Whether any list it frees holds blocks, the cars' first.
fn any_holds_blocks(effects: &Effects) -> Result<bool, Refused> {
    Ok(
        crate::effects::try_any(car_lists(effects)?.iter(), |&l| Ok(l.get().head != 0))?
            || with_lists(effects, |lists| {
                crate::effects::try_any(lists.iter(), |&l| Ok(l.get().head != 0))
            })?,
    )
}

pub fn tidy_marks(effects: &Effects) -> Result<(), Refused> {
    let (paths, ring, masks) = (&effects.paths, &effects.ring, &effects.masks);
    let marks = effects.settings()?;
    let count = marks.cars_copy.get() as i32;
    if !(0..=MOST_CARS).contains(&count) {
        return Err(BadInput("a cars' count below 0 or above 8").into());
    }
    if any_holds_blocks(effects)? {
        return Err(BadInput("a memory list holding a block").into());
    }
    for n in 0..count as u32 {
        let car = &marks.cars[n as usize];
        free_list(&car.points_list)?;
        if let Some(points) = paths.block_points.get(n as usize) {
            points.give_back();
        }
        car.point_block.set(paths.morph_taken.get());
    }
    with_lists(effects, |lists| {
        for list in lists {
            free_list(list)?;
        }
        Ok::<(), BadInput>(())
    })?;
    // The particles and the masks went with their lists.
    effects.particles.ring.give_back();
    ring.entries.give_back();
    masks.mask.give_back();
    masks.bmask.give_back();
    Ok(())
}
