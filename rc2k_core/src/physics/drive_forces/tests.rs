use super::*;

#[test]
fn low_speeds_and_reverse() {
    assert_eq!(low_speed_factor(-10.0, 20.0, 0.5, 2.0), Some(2.0));
    assert_eq!(low_speed_factor(20.0, 20.0, 0.5, 2.0), None);
    assert_eq!(
        low_speed_factor(f32::NAN, 20.0, 0.5, 2.0).map(f32::is_nan),
        Some(true)
    );
    assert_eq!(eased([2.0, -3.0], 0.5), [1.0, -1.5]);
    assert_eq!(cap_reverse(-1, 1.0, 0.0, [3.0, 1.0], 2.0), Some([2.0, 1.0]));
    assert_eq!(cap_reverse(-1, -1.0, 0.0, [3.0, 1.0], 2.0), None);
    assert_eq!(cap_reverse(1, 1.0, 0.0, [3.0, 1.0], 2.0), None);
}

#[test]
fn downforces() {
    assert_eq!(downforce(2.0, 0.5, 1.0, -2.0), (-6.0, 6.0));
    let (load, minus) = downforce(f32::NAN, 1.0, 0.0, 1.0);
    assert_eq!(minus.to_bits(), load.to_bits() ^ 0x8000_0000);
}

#[test]
fn forces() {
    let f = add_drive_forces(
        1,
        [2.0, 0.5],
        [1.0, 2.0, 3.0, 4.0],
        [1.0, 2.0],
        2.0,
        [1.0, 1.0],
    );
    assert_eq!(
        f,
        DriveForces {
            kept: Some(2.0),
            drive: 5.0,
            forces: [11.0, 21.0]
        }
    );
    assert_eq!(
        add_drive_forces(0, [2.0, 0.5], [1.0; 4], [0.0; 2], 1.0, [0.0; 2]).drive,
        8.0
    );
    assert_eq!(
        cap_drive_speed(10.0, -4.0, 1.0, 1.0, 0.0, 9.0),
        (1.0, -3.0, -3.0)
    );
    assert_eq!(
        cap_drive_speed(1.0, 9.0, 1.0, 1.0, 0.0, 2.0),
        (0.0, 9.0, 2.0)
    );
    let s = torque_share(1, [8, 32], -16.0, 32, 1.0, 4, 2.0);
    assert_eq!(
        (s.n, s.speed_share, s.less, s.drive_share),
        (8, 0.5, 4, 0.5)
    );
    assert_eq!((s.share, s.drive), (0.75, 1.5));
    assert_eq!(
        torque_share(0, [8, 32], 64.0, 32, 1.0, 0, 2.0).speed_share,
        1.0
    );
}
