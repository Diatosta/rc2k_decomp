use super::*;

#[test]
fn bits_and_searches() {
    let b = GearBits(0b1_0110);
    assert_eq!(
        b.test(1),
        Ok(GearTest {
            bit: 2,
            barred: true
        })
    );
    assert_eq!(
        b.test(-1),
        Ok(GearTest {
            bit: 0,
            barred: false
        })
    );
    assert!(b.test(31).is_err());
    // Bits 1, 2, 4 set: gears 0, 1, 3 barred.
    assert_eq!(
        b.higher_free(0, 5),
        Ok(Search {
            gear: 2,
            bit: 3,
            found: true
        })
    );
    assert_eq!(
        GearBits(u32::MAX >> 1).higher_free(1, 4),
        Ok(Search {
            gear: 5,
            bit: 5,
            found: false
        })
    );
    assert_eq!(
        b.lower_free(4, 9),
        Ok(Search {
            gear: 2,
            bit: 3,
            found: true
        })
    );
    assert_eq!(
        b.lower_free(1, 9),
        Ok(Search {
            gear: 0,
            bit: 9,
            found: false
        })
    );
    assert!(GearBits(u32::MAX).lower_free(0, 0).is_err());
}

#[test]
fn allowed() {
    let bits = || GearBits(0b100);
    assert_eq!(allowed_gear(true, 3, bits, || 1), Ok(0));
    assert_eq!(allowed_gear(false, 1, bits, || 1), Ok(0));
    assert_eq!(allowed_gear(false, 2, bits, || 1), Ok(2));
    assert_eq!(allowed_gear(false, 2, bits, || 0), Ok(0));
    assert_eq!(allowed_gear(false, 0, || unreachable!(), || 1), Ok(0));
    assert_eq!(allowed_gear(false, -1, || unreachable!(), || 1), Ok(-1));
}

#[test]
fn changes() {
    assert_eq!(gear_change(1), Some(GearChange::Up));
    assert_eq!(gear_change(-1), Some(GearChange::Down));
    assert_eq!(gear_change(2), None);
}

#[test]
fn input_rules() {
    let car = GearCar::default();
    let mut i = GearInputs {
        first: 3,
        second: 7,
        hold: 1,
        ..Default::default()
    };
    let mut gear = 0;
    let both = GearCar {
        box_mode: 2,
        timer: 0x320,
        ..car
    };
    assert_eq!(i.apply_rules(&mut gear, &both), Some(7));
    assert_eq!((i.first, i.second, i.joined, i.hold), (7, 0, u32::MAX, 1));
    let mut gear = -1;
    let mut j = GearInputs {
        first: 3,
        second: 7,
        ..Default::default()
    };
    assert_eq!(j.apply_rules(&mut gear, &both), None);
    assert_eq!((j.first, j.second), (7, 3));
    let mut gear = 4;
    let mut k = GearInputs {
        w90: 1,
        first: 2,
        second: 3,
        wa0: 4,
        wa4: 5,
        ..Default::default()
    };
    k.apply_rules(&mut gear, &car);
    assert_eq!(
        k,
        GearInputs {
            first: 2,
            hold: HELD,
            ..Default::default()
        }
    );
    let off = GearCar { state: 5, ..car };
    let mut gear = 4;
    k.apply_rules(&mut gear, &off);
    assert_eq!(gear, 0);
}

#[test]
fn turns() {
    let t = |f, m, tm, g, i, k| gearbox_turn(f, || m, || tm, || g, || i, || k);
    assert_eq!(t(1, 1, 0, 0, 0, 0), GearboxTurn::Idle);
    assert_eq!(t(0, 0, 0x31F, 0, 0, 0), GearboxTurn::Idle);
    assert_eq!(t(0, 1, 0, 2, 0, 0), GearboxTurn::Manual);
    assert_eq!(t(0, 0, 0x320, 0, 0x300, 0), GearboxTurn::Asked);
    assert_eq!(t(0, 0, 0x320, 0, 0xE00, 2), GearboxTurn::Auto);
    assert_eq!(t(0, 0, 0x320, 0, 0xE00, 1), GearboxTurn::Manual);
    assert_eq!(asked_gear(0xF00), -1);
    assert_eq!(asked_gear(0x700), 7);
    assert_eq!(asked_gear(0x800), -8);
}
