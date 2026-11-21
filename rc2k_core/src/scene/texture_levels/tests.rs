use super::*;

#[test]
fn the_own_car_has_the_better_levels_whichever_it_is() {
    // Car 0 is the own car.
    let first = car_runs(2, true, |car| car == 0);
    assert_eq!(first[0], run(4, 0x100, 0, 1, 0x29));
    assert_eq!(first[1], run(4, 0x104, 0, 1, 0x29));
    assert_eq!(first[2], run(4, 0x108, 0, 2, 0x2A));
    assert_eq!(first[3], run(4, 0x10C, 1, 2, 0x2A));
    // A network client's: the levels change places, the slots and tags stay.
    let second = car_runs(2, true, |car| car == 1);
    assert_eq!(second[0], run(4, 0x100, 0, 2, 0x29));
    assert_eq!(second[1], run(4, 0x104, 1, 2, 0x29));
    assert_eq!(second[2], run(4, 0x108, 0, 1, 0x2A));
    assert_eq!(second[3], run(4, 0x10C, 0, 1, 0x2A));
    // Two cars of the own model: both.
    let both = car_runs(2, true, |_| true);
    assert_eq!(both[0], run(4, 0x100, 0, 1, 0x29));
    assert_eq!(both[1], run(4, 0x104, 0, 1, 0x29));
    assert_eq!(both[3], run(4, 0x10C, 0, 1, 0x2A));
}

#[test]
fn cars() {
    assert_eq!(
        [0, 1, 5, 8, 9, 0x8000_0000, 0x7FFF_FFFF].map(held_cars),
        [1, 1, 5, 8, 8, 8, 8]
    );
    assert_eq!(car_runs(1, true, |car| car == 0).len(), 2);
    assert_eq!(car_runs(1, true, |car| car == 0)[1].level, 0);
    let r = car_runs(8, false, |car| car == 0);
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
        ExtraRuns {
            framed: true,
            further_maps: true,
        },
        false,
        |car| car == 0,
    )
    .unwrap();
    assert_eq!(set.kind_slots_added, 2);
    assert_eq!(
        (system[0x3F1], video[0x3F2], tags[0x3F2], system[0x3F3]),
        (1, 1, FIXED_TAG, 9)
    );
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
