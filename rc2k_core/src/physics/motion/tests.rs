use super::*;

#[test]
fn sums_and_moves() {
    let f = [
        1.0f32, 2.0, 3.0, 4.0, 1.0, 1.0, 1.0, 1.0, 2.0, 2.0, 2.0, 2.0, 2.0,
    ];
    assert_eq!(sum_forces(&f, 2.0, 0.5), [1.0, 2.0, 7.0, 2.0, 5.0]);
    let mut place = [10u32, 20, 30];
    assert_eq!(
        move_place([1.0, -2.0, 0.5], &mut place, 65536.0, 0.25),
        -32768
    );
    assert_eq!(place, [10 + 16384, 20u32.wrapping_sub(32768), 30 + 8192]);
    let mut turns = [0u32, 5];
    assert_eq!(spin([2.0, -1.0], &mut turns, 2.0, 1.0, 4.0, 0.5), -1);
    assert_eq!(turns, [2, 4]);
}

#[test]
fn drag_and_holds() {
    assert_eq!(air_drag([2.0, -2.0, 0.0], 3.0, 4.0, 0.0), [-3.0, 3.0, -0.0]);
    assert!(air_drag([f32::NAN, 1.0, 1.0], 1.0, 1.0, 0.0)[0].is_nan());
    let (mut l, mut x, mut z) = (2.0f32, 3.0f32, -0.5f32);
    assert_eq!(hold_spin(&mut l, &mut x, &mut z), 1);
    assert_eq!((l, x, z), (2.0, 2.0, -0.5));
    let (mut l, mut x, mut z) = (2.0f32, -1.0f32, -5.0f32);
    assert_eq!(hold_spin(&mut l, &mut x, &mut z), 1);
    assert_eq!(z, -2.0);
    let (mut l, mut x, mut z) = (2.0f32, 1.0f32, 0.5f32);
    assert_eq!(hold_spin(&mut l, &mut x, &mut z), 0);
}

#[test]
fn blends_and_marks() {
    let b = LoadBlend {
        figure: 5.0,
        signs: [-1.0, 1.0, -1.0],
        threshold: 1.0,
        divisor: 10.0,
        zero: 0.0,
        one: 1.0,
    };
    let mut slots = [0.0f32; 8];
    assert_eq!(blend_load(&b, &mut slots), Some((5, 0.5)));
    assert_eq!(slots[5], 0.5);
    let held = LoadBlend { figure: 50.0, ..b };
    assert_eq!(blend_load(&held, &mut slots), Some((5, 1.0)));
    assert_eq!(slots[5], 1.0);
    assert_eq!(
        blend_load(
            &LoadBlend {
                figure: f32::NAN,
                ..b
            },
            &mut slots
        ),
        None
    );
}

#[test]
fn lowest() {
    let depths = [0u32, 0, 0, 0, 5, 3, 0xFFFF_FFF0, 7];
    let (mut l, mut n) = (0, 0);
    assert_eq!(
        lowest_point(8, |k| depths[k as usize], &mut l, &mut n),
        Ok(Some((7, 8)))
    );
    assert_eq!((l, n), (0xFFFF_FFF0, 6));
    assert_eq!(lowest_point(4, |_| 0, &mut l, &mut n), Ok(None));
    assert!(lowest_point(0x41, |_| 0, &mut l, &mut n).is_err());
    assert_eq!(lowest_share(0x8_0000, 65536.0), 8.0);
}

#[test]
fn stops() {
    let mut c = [0u32, 299, 5];
    assert_eq!(stop_timers(&mut c, 0.5, 1.0), Some(u32::MAX));
    assert_eq!(c, [0, 300, 5]);
    let mut c = [0u32, 0, 300];
    assert_eq!(stop_timers(&mut c, 2.0, 1.0), Some(2));
    let mut c = [3u32, 0, 7];
    assert_eq!(stop_timers(&mut c, 2.0, 1.0), None);
    assert_eq!(c, [3, 0, 7]);
    assert_eq!(stop_timers(&mut c, f32::NAN, 1.0), None);
    assert_eq!(c, [4, 0, 8]);
}

#[test]
fn counters() {
    let mut c = StepCounters {
        first: [1, 0],
        standing: 4,
        running: 2,
        later: [0, 3, 1],
        highest: 6.0,
    };
    let f = StepFacts {
        steps: 2000,
        mode: 1,
        bits: 0x10,
        figure: 7.5,
        ..Default::default()
    };
    assert_eq!(count_down(&mut c, &f), Some(0x10));
    assert_eq!(c.first, [0, 0]);
    assert_eq!(
        (c.standing, c.running, c.later, c.highest),
        (5, 3, [0, 2, 0], 7.5)
    );
    assert_eq!(count_down(&mut c, &StepFacts { steps: 10, ..f }), None);
    assert_eq!(c.standing, 0);
    assert_eq!(count_down(&mut c, &StepFacts { mode: 2, ..f }), None);
    assert_eq!(c.standing, 0);
}
