use super::*;

const IDENTITY_FAR: [f32; 16] = [
    1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 9000.0, 0.0,
];

#[test]
fn offsets() {
    assert_eq!(record_offset(3, 6, 8), Ok(0x60));
    assert_eq!(record_offset(0x47, 6, 8), Ok(0x47 << 5));
    assert_eq!(record_offset(0x48, 6, 8), Err(TooHigh::Scene));
    assert_eq!(record_offset(-5i32 as u32, 6, 8), Ok((0x1005) << 5));
    assert_eq!(record_offset(-6i32 as u32, 6, 8), Err(TooHigh::Object));
    // 0x80000000 is an object's number past any count.
    assert_eq!(record_offset(0x8000_0000, 6, 8), Err(TooHigh::Object));
}

#[test]
fn records() {
    let s = Screen {
        scale: 640.0,
        centre: [400.0, 300.5],
        flat: None,
    };
    let m = make_record([100, -50, 1000], &IDENTITY_FAR, || s, false);
    assert_eq!(m.view, [100.0, -50.0, 10000.0]);
    assert_eq!(m.screen, [406.4, 303.7]);
    assert_eq!(m.checked, None);
    // Too near: not on the screen; lowered.
    let m = make_record([1, 2, -6000], &IDENTITY_FAR, || unreachable!(), true);
    assert_eq!((m.view[2], m.screen), (3000.0 - 262144.0, [0.0, 0.0]));
    assert_eq!(m.checked, Some(LOWERED_BY));
    // Flat: the depth divided by is 7999 + 1, the real one checked.
    let flat = Screen {
        flat: Some(7999),
        ..s
    };
    let m = make_record([80, 0, 0], &IDENTITY_FAR, || flat, false);
    assert_eq!((m.screen, m.checked), ([406.4, 300.5], Some(9000.0)));
    // The order of the sums: x' = m0 x + (m4 y + m8 z) + m12.
    let mut r = [0.0f32; 16];
    (r[0], r[4], r[8], r[12]) = (1.0, 1.0, -1.0, 0.5);
    assert_eq!(rotate([0x4000_0000, 1, -0x4000_0000], &r)[0], 2147483648.0);
    assert_eq!(turn_normal([0.5, 2.0, 1.0], &r)[0], 1.5);
    assert_eq!(
        turn_normal([0.6, -0.8, 0.0], &IDENTITY_FAR),
        [0.6, -0.8, 0.0]
    );
}

#[test]
fn depths() {
    assert!(outside_depths(100.0, 9000.0));
    assert!(!outside_depths(4096.0, 9000.0));
    assert!(outside_depths(9000.0, 9000.0));
    assert!(outside_depths(f32::NAN, 9000.0));
    assert!(!outside_depths(5000.0, f32::NAN));
}
