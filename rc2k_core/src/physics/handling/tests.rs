use super::*;
use std::collections::HashMap;

#[derive(Default)]
struct Map(HashMap<At, u32>);

impl Figures for Map {
    fn get(&self, at: At) -> u32 {
        *self.0.get(&at).unwrap_or(&0)
    }
    fn put(&mut self, at: At, v: u32) {
        self.0.insert(at, v);
    }
}

#[test]
fn ramps() {
    let t = [1.0, 10.0, 3.0, 20.0];
    assert_eq!(ramp(0.5, t, 0.0), 10.0);
    assert_eq!(ramp(f32::NAN, t, 0.0), 10.0);
    assert_eq!(ramp(3.0, t, 0.0), 20.0);
    assert_eq!(ramp(2.0, t, 0.0), 15.0);
    // A flat table: t1.
    assert_eq!(ramp(1.0, [1.0, 7.0, 1.0, 9.0], 0.0), 9.0);
    assert_eq!(
        one_less_products(&[[0.5, 2.0, 0.25], [1.0, 1.0, 0.5]]),
        0.25
    );
    assert_eq!(int_lerp(128, 256.0, 3.0, 1.0), 2.0);
}

#[test]
fn rates() {
    let mut m = Map::default();
    m.put_float(K, 1.0);
    m.put_float(LOW, 0.0);
    m.put_float(HIGH, 1.0);
    m.put_float(Car(0x2B8), 0.25);
    m.put_float(Grades(0x354), 1.0);
    m.put_float(CHANCE, 100.0);
    // A chance figure of 1: 100 against the random's low word 0x50 turned: below.
    m.put_float(Grades(0x1D8), 0.0);
    m.put_float(Grades(0x1E0), 0.0);
    m.put_float(Grades(0x1DC), 1.0);
    m.put_float(Grades(0x1E4), 1.0);
    m.put_float(Car(0x2C8), 5.0);
    m.put(RANDOM, 0x50 << 0xB);
    let bits = rate_damage(&mut m);
    assert_eq!(
        m.float(FIRST),
        0.0,
        "the first grade cleared: +0x310 not below 0"
    );
    assert_eq!(m.float(Car(0x3F8)), 1.0, "1 held to the ceiling");
    assert_eq!(m.get(SCRATCH), 0, "the last chance's figure");
    assert_eq!(m.get(Car(0x42C)), 1);
    assert_eq!(m.get(Car(0x29C)), 1);
    assert_eq!(bits, 0x1FF, "every gear's figure is 0");
    assert_eq!(m.get(GEAR_BITS), bits);
}

#[test]
fn handling() {
    let mut m = Map::default();
    m.put(DRIVE_KIND, 2);
    assert!(set_handling(&mut m, |_| ()).is_err());
    assert!(m.0.len() == 1, "nothing written");
    m.put(DRIVE_KIND, 1);
    m.put_float(At::Scale, 256.0);
    m.put(Car(0x30), 64);
    m.put_float(Handling(0x158), 5.0);
    m.put_float(Handling(0x150), 1.0);
    m.put_float(Handling(0xE18), 2.0);
    let (graded, kind) = set_handling(&mut m, |m| m.get(DRIVE_KIND) + 7).unwrap();
    assert_eq!((graded, kind), (8, 1));
    assert_eq!(m.float(Handling(0xDFC)), 4.0);
}
