use super::*;

#[test]
fn revs() {
    assert_eq!(ease_revs(3000, 3512), (3100, 100));
    assert_eq!(ease_revs(3000, 0), (2414, (-586i32) as u32));
    assert_eq!(ease_revs(2000, 0).0, REVS_LOW);
    assert_eq!(ease_revs(11990, 20000).0, REVS_HIGH);
}

#[test]
fn torque() {
    assert_eq!(fistp_chop(2.9), 2);
    assert_eq!(fistp_chop(-2.9), -2);
    assert_eq!(fistp_chop(f64::NAN), i32::MIN);
    let e = Engine {
        speed: 3000.0,
        top: 6000.0,
        zero: 0.0,
        one: 1.0,
        steps: 8.0,
        quarter: 0.25,
        factor: 2.0,
    };
    let curve = |i: u32| {
        assert_eq!(i, 4);
        ([0.5, 0.625], [100.0, 200.0])
    };
    let t = engine_torque(&e, curve, || panic!("not asked"));
    assert_eq!(
        t,
        Torque {
            share: 0.5,
            entry: Some(4),
            revs: 0.125,
            torque: 200.0
        }
    );
    let past = Engine { speed: 6000.0, ..e };
    let t = engine_torque(&past, |_| panic!("not asked"), || 7.0);
    assert_eq!((t.entry, t.torque), (None, 14.0));
    assert_eq!(torque_cut(6000.0, 6000.0, 0), Some(5));
    // Not a number counts as below: no cut.
    assert_eq!(torque_cut(f32::NAN, 6000.0, 0), None);
    assert_eq!(torque_cut(100.0, 6000.0, 3), Some(2));
    assert_eq!(torque_cut(100.0, 6000.0, 0), None);
}

#[test]
fn shares() {
    assert!(shares_figures(2.0, 1.0));
    assert!(!shares_figures(1.0, 1.0));
    assert!(!shares_figures(0.5, 1.0));
    assert!(!shares_figures(f32::NAN, 1.0));
    assert!(!shares_figures(2.0, f32::NAN));
}

#[test]
fn drives() {
    let d = Drive {
        torque: 6.0,
        base: 2.0,
        gearing: 64,
        k256: 256.0,
        factor: 3.0,
        ratio: 0.5,
        k10000: 1.0,
        cut: false,
    };
    assert_eq!(drive(&d), (0.0, 0.0));
    assert_eq!(drive(&Drive { torque: 14.0, ..d }), (2.0, 12.0));
    assert_eq!(
        drive(&Drive {
            torque: 14.0,
            cut: true,
            ..d
        }),
        (2.0, 0.0)
    );
}

#[test]
fn revving() {
    let f = FreeRev {
        drive: 10.0,
        drive_factor: 2.0,
        ratio: 0.5,
        k10000: 10.0,
        gearing: 3,
        k256: 2.0,
        step: 0.5,
        engine: 100.0,
        drag_factor: 0.0,
        drag_base: 1.0,
    };
    // push 10 x 2 / 0.5 / 10 x 3 / 2 = 6; engine 100 + 6 x 0.5 / 0.5 = 106; less 1.
    assert_eq!(free_rev(&f), (6.0, 105.0, 1.0));
    // -7 + 6 = -1, brought up by 2: not past 0.
    let back = FreeRev {
        engine: -7.0,
        drag_base: 2.0,
        ..f
    };
    assert_eq!(free_rev(&back), (6.0, 0.0, 2.0));
    // -9 + 6 = -3, up by 2; -6 + 6 = 0 is left alone.
    let down = FreeRev {
        engine: -9.0,
        drag_base: 2.0,
        ..f
    };
    assert_eq!(free_rev(&down).1, -1.0);
    let still = FreeRev {
        engine: -6.0,
        drag_base: 2.0,
        ..f
    };
    assert_eq!(free_rev(&still).1, 0.0);
}
