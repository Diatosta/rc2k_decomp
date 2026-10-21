//! Which texture slots a stage wants at which level (converted 2026-10-21 from
//! `scene::init_texture_levels` 0x424001, `race::loadlevel_init`'s "init ed textures"):
//! each car's runs of slots, each kind's slots by the stage prefs, then fixed runs, into
//! `LevelTables`. With the other levels on (`dword_5CDADC`, which the original sets to 0
//! first and nothing else sets) every level is the run's other one.

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

/// The runs after the kinds' slots.
pub const AFTER_KINDS: [Run; 15] = [
    run(1, 0x87, 0, 0, FIXED_TAG),
    run(2, 0xC0, 0, 0, FIXED_TAG),
    run(1, 0xC2, 0, 1, FIXED_TAG),
    run(1, 0x257, 0, 0, FIXED_TAG),
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
/// The runs with `dword_621514` set (the framed layout).
pub const FRAMED: [Run; 3] = [
    run(0x10, 0x200, 2, 2, FIXED_TAG),
    run(1, 0x218, 0, 0, FIXED_TAG),
    run(8, 0x210, 0, 0, FIXED_TAG),
];
/// The most cars.
pub const MOST_CARS: u32 = 8;

/// The race's cars held to 1 .. 8, by the signs of the differences (as the original).
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

/// The cars' runs: the first car's two (slots 0x100, 0x104; the second's level 0 when
/// `second_first` (`dword_63DB94` 1), else 1), and each further car's two (0x108 + 8 a
/// car, tag 0x28 + the car).
pub fn car_runs(cars: u32, second_first: bool) -> Vec<Run> {
    let mut runs = vec![
        run(4, 0x100, 0, 1, 0x29),
        run(4, 0x104, if second_first { 0 } else { 1 }, 1, 0x29),
    ];
    for car in 2..=MOST_CARS {
        if cars == car - 1 {
            break;
        }
        let first = 0x108 + 8 * (car - 2);
        runs.push(run(4, first, 0, 2, 0x28 + car));
        runs.push(run(4, first + 4, 1, 2, 0x28 + car));
    }
    runs
}

/// A kind's block of slots: the first, its tag, levels, and what the kinds' slot count
/// (`dword_5DC978`) gets for each slot taken.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct KindSlots {
    pub first: u32,
    pub tag: u32,
    pub level: u32,
    pub other_level: u32,
    pub step: u32,
}

impl LevelTables<'_> {
    /// A run, by `set_texture_levels`.
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

/// What `init_texture_levels` leaves: the last run and the kinds' slot count's growth.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LevelsSet {
    pub last: Run,
    pub kind_slots_added: u32,
}

/// `init_texture_levels` after its calls: the cars' runs, the kinds' slots (each block
/// with its prefs entries), the runs after them, the slots of level one, the last run and
/// with `framed` three more. Refuses a run past the tables (none of the fixed ones is).
pub fn set_levels(
    t: &mut LevelTables,
    cars: u32,
    second_first: bool,
    kinds: &[(KindSlots, Vec<u32>)],
    framed: bool,
    other: bool,
) -> Result<LevelsSet, Invalid> {
    for r in car_runs(cars, second_first) {
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
    let mut last = LAST;
    t.apply(&LAST, other)?;
    if framed {
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
mod tests {
    use super::*;

    #[test]
    fn cars() {
        assert_eq!(
            [0, 1, 5, 8, 9, 0x8000_0000, 0x7FFF_FFFF].map(held_cars),
            [1, 1, 5, 8, 8, 8, 8]
        );
        assert_eq!(car_runs(1, true).len(), 2);
        assert_eq!(car_runs(1, true)[1].level, 0);
        let r = car_runs(8, false);
        assert_eq!(r.len(), 16);
        assert_eq!(
            (r[15].first, r[15].tag, r[1].level),
            (0x108 + 8 * 6 + 4, 0x30, 1)
        );
    }

    #[test]
    fn levels() {
        let (mut tags, mut system, mut video) =
            (vec![0u32; 0x400], vec![9u32; 0x400], vec![9u32; 0x400]);
        let mut t = LevelTables {
            tags: &mut tags,
            system: &mut system,
            video: &mut video,
        };
        let kind = KindSlots {
            first: 0x40,
            tag: 0x20,
            level: 1,
            other_level: 2,
            step: 1,
        };
        let set = set_levels(
            &mut t,
            2,
            false,
            &[(kind, vec![0, 5, u32::MAX, 3])],
            true,
            false,
        )
        .unwrap();
        assert_eq!(set.kind_slots_added, 2);
        assert_eq!(set.last, FRAMED[2]);
        assert_eq!(
            (tags[0x41], system[0x41], tags[0x40], system[0x40]),
            (0x20, 1, 0, 9)
        );
        assert_eq!(
            (system[0xE1], video[0xE1], video[0xDD], tags[0xE1]),
            (1, 9, 1, FIXED_TAG)
        );
        assert_eq!((system[0x10C], tags[0x10C], system[0x110]), (1, 0x2A, 9));
        assert_eq!(system[0x200], 2);
    }
}
