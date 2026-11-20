//! Which texture slots a stage wants at which level: each car's runs of slots, each kind's
//! slots by the stage prefs, then fixed runs, into `LevelTables`. With the other levels on
//! (`ObjectTables.other_levels`, which nothing sets) every level is the run's other one.

use crate::Invalid;
use crate::scene::textures::LevelTables;

/// A run of slots: the first, how many, the level, the other level, the tag.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Run {
    pub first: u32,
    pub count: u32,
    pub level: u32,
    pub other_level: u32,
    pub tag: u32,
}

const fn run(count: u32, first: u32, level: u32, other_level: u32, tag: u32) -> Run {
    Run {
        first,
        count,
        level,
        other_level,
        tag,
    }
}

/// The fixed runs' tag.
pub const FIXED_TAG: u32 = 0x63;

/// The texture the tyre change's title is drawn into: a texture of its own, since the title
/// (in Portuguese) is wider than the tyre change texture's strips (a number no stage's object
/// names).
pub const TYRE_TITLE_TEXTURE: u32 = 0x3F0;

/// The textures of the third and fourth players' map pictures in a split screen of three or four:
/// textures of their own beside the two the first and second players' maps use, the same size
/// and format (numbers no stage's object names). They are made only for such a race.
pub const FURTHER_MAP_TEXTURES: [u32; 2] = [0x3F1, 0x3F2];

/// The runs after the kinds' slots.
pub const AFTER_KINDS: [Run; 16] = [
    run(1, 0x87, 0, 0, FIXED_TAG),
    run(2, 0xC0, 0, 0, FIXED_TAG),
    run(1, 0xC2, 0, 1, FIXED_TAG),
    run(1, 0x257, 0, 0, FIXED_TAG),
    run(1, TYRE_TITLE_TEXTURE, 0, 0, FIXED_TAG),
    run(1, 0xCA, 0, 0, FIXED_TAG),
    run(1, 0xC8, 3, 3, FIXED_TAG),
    run(1, 0xD3, 0, 0, FIXED_TAG),
    run(1, 0xD4, 0, 0, FIXED_TAG),
    run(1, 0xC3, 0, 0, FIXED_TAG),
    run(1, 0xD2, 0, 0, FIXED_TAG),
    run(1, 0xC6, 1, 1, FIXED_TAG),
    run(1, 0xC7, 1, 1, FIXED_TAG),
    run(1, 0xC4, 1, 1, FIXED_TAG),
    run(1, 0xC5, 1, 1, FIXED_TAG),
    run(1, 0xC9, 1, 1, FIXED_TAG),
];
/// The slots from 0xDC level 1 in one memory each, whatever the other levels: six in
/// system memory, two in video memory.
pub const LEVEL_ONE_FROM: u32 = 0xDC;
pub const LEVEL_ONE_SYSTEM: u32 = 6;
pub const LEVEL_ONE_VIDEO: u32 = 2;
/// The last run.
pub const LAST: Run = run(2, 0x21C, 0, 0, FIXED_TAG);
/// The runs with `FrameSwitches.editor_mode` set (the framed layout).
pub const FRAMED: [Run; 3] = [
    run(0x10, 0x200, 2, 2, FIXED_TAG),
    run(1, 0x218, 0, 0, FIXED_TAG),
    run(8, 0x210, 0, 0, FIXED_TAG),
];
/// The runs of the further players' map textures, level 1 as the first two maps are.
pub fn further_map_runs() -> [Run; 2] {
    FURTHER_MAP_TEXTURES.map(|texture| run(1, texture, 1, 1, FIXED_TAG))
}
/// The most cars.
pub const MOST_CARS: u32 = 8;

/// The race's cars held to 1 ..= 8, by the signs of the differences.
pub fn held_cars(cars: u32) -> u32 {
    let mut cars = cars;
    if (cars.wrapping_sub(1) as i32) < 0 {
        cars = 1;
    }
    if (cars.wrapping_sub(MOST_CARS) as i32) >= 0 {
        cars = MOST_CARS;
    }
    cars
}

/// The cars' runs, two each: the own car's second at level 0 when `second_first`, else 1,
/// each other car's second at level 1 (half the size); firsts at level 0; the other levels
/// 1 for the own car, 2 for the others. Car 0's slots are 0x100 and 0x104, a further car's
/// 0x108 + 8 a car; the tag 0x29 for car 0, else 0x29 + the car. `own` tells the cars this
/// machine's cockpit may draw from: its own and any of the same model (a fixed bug: "A network
/// client's dashboard is blurred" in `wiki/Fixed-bugs.md`).
pub fn car_runs(cars: u32, second_first: bool, own: impl Fn(u32) -> bool) -> Vec<Run> {
    (0..MOST_CARS)
        .take_while(|&car| car == 0 || cars != car)
        .flat_map(|car| {
            let first = if car == 0 {
                0x100
            } else {
                0x108 + 8 * (car - 1)
            };
            let tag = if car == 0 { 0x29 } else { 0x29 + car };
            if own(car) {
                let second = u32::from(!second_first);
                [run(4, first, 0, 1, tag), run(4, first + 4, second, 1, tag)]
            } else {
                [run(4, first, 0, 2, tag), run(4, first + 4, 1, 2, tag)]
            }
        })
        .collect()
}

/// A kind's block of slots: the first, its tag, levels, and what the kinds' slot count gets
/// for each slot taken.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct KindSlots {
    pub first: u32,
    pub tag: u32,
    pub level: u32,
    pub other_level: u32,
    pub step: u32,
}

impl LevelTables<'_> {
    /// A run applied: `other` picks its other level.
    pub fn apply(&mut self, r: &Run, other: bool) -> Result<(), Invalid> {
        let level = if other { r.other_level } else { r.level };
        self.set_run(r.first, r.count, level, r.tag)
    }

    /// A kind's slots: each whose prefs entry is not 0 or -1 gets the tag and the level;
    /// gives what the kinds' slot count gets (wrapping). Refuses slots past the tables,
    /// before writing.
    pub fn kind_slots(
        &mut self,
        k: &KindSlots,
        prefs: &[u32],
        other: bool,
    ) -> Result<u32, Invalid> {
        if k.first as usize + prefs.len() > self.tags.len() {
            return Err(Invalid("a kind's texture slots past 0x400"));
        }
        let mut added = 0u32;
        let level = if other { k.other_level } else { k.level };
        for (n, &p) in prefs.iter().enumerate() {
            if p != 0 && p != u32::MAX {
                added = added.wrapping_add(k.step);
                let slot = k.first as usize + n;
                self.tags[slot] = k.tag;
                self.system[slot] = level;
                self.video[slot] = level;
            }
        }
        Ok(added)
    }
}

/// What `set_levels` leaves: the last run and the kinds' slot count's growth.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LevelsSet {
    pub last: Run,
    pub kind_slots_added: u32,
}

/// Which runs the stage has besides the ones every stage has.
#[derive(Clone, Copy, Debug, Default)]
pub struct ExtraRuns {
    /// The framed layout's runs.
    pub framed: bool,
    /// The third and fourth players' map textures.
    pub further_maps: bool,
}

/// The cars' runs, the kinds' slots (each block with its prefs entries), the runs after
/// them, the slots of level one, the last run and with `framed` three more. Refuses a run
/// past the tables (none of the fixed ones is).
pub fn set_levels(
    t: &mut LevelTables,
    cars: u32,
    second_first: bool,
    kinds: &[(KindSlots, Vec<u32>)],
    extra: ExtraRuns,
    other: bool,
    own: impl Fn(u32) -> bool,
) -> Result<LevelsSet, Invalid> {
    for r in car_runs(cars, second_first, &own) {
        t.apply(&r, other)?;
    }
    let mut added = 0u32;
    for (k, prefs) in kinds {
        added = added.wrapping_add(t.kind_slots(k, prefs, other)?);
    }
    for r in &AFTER_KINDS {
        t.apply(r, other)?;
    }
    let from = LEVEL_ONE_FROM as usize;
    t.system[from..from + LEVEL_ONE_SYSTEM as usize].fill(1);
    t.tags[from..from + LEVEL_ONE_SYSTEM as usize].fill(FIXED_TAG);
    t.video[from..from + LEVEL_ONE_VIDEO as usize].fill(1);
    if extra.further_maps {
        for r in &further_map_runs() {
            t.apply(r, other)?;
        }
    }
    let mut last = LAST;
    t.apply(&LAST, other)?;
    if extra.framed {
        for r in &FRAMED {
            t.apply(r, other)?;
            last = *r;
        }
    }
    Ok(LevelsSet {
        last,
        kind_slots_added: added,
    })
}

#[cfg(test)]
mod tests;
