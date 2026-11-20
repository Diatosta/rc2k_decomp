use super::*;

#[test]
fn object_points() {
    assert!(check_object(0xFF).is_ok() && check_object(0x100).is_err());
    let w = walk_points(2, 0, 0x1000, 77, |_| panic!());
    assert_eq!(
        w,
        PointWalk {
            inside: false,
            number: 2,
            left: 0,
            address: 77
        }
    );
    let w = walk_points(2, 3, 0x1000, 77, |a| a == 0x1000 + 36);
    assert_eq!(
        w,
        PointWalk {
            inside: true,
            number: 3,
            left: 2,
            address: 0x1024
        }
    );
    let w = walk_points(2, 2, 0x1000, 77, |_| false);
    assert_eq!(
        (w.inside, w.number, w.left, w.address),
        (false, 4, 0, 0x1030)
    );
}

#[test]
fn contacts() {
    let n = |v: i32| v as u32;
    assert!(touches(n(-5), 0x1000, false));
    assert!(!touches(5, 0x1000, false));
    assert!(touches(5, 0x1000, true));
    assert!(!touches(0x1000, 0x1000, true));
    // The sign of the difference: far apart wraps.
    assert!(!touches(n(i32::MIN), 0x1000, true));
    assert!(check_body_point(0x3F).is_ok() && check_body_point(0x40).is_err());
    assert_eq!(normal_offsets(1), [0x1C, 0x34, 0x3C]);
    assert_eq!(normal_offsets(2), [0x18, 0x30, 0x38]);
}

#[test]
fn boxes() {
    let n = |v: i32| v as u32;
    assert!(inside_box(0, 0, 10, 20, 30));
    assert!(inside_box(n(-10), n(-30), 10, 20, 30));
    assert!(!inside_box(10, 0, 10, 20, 30));
    assert!(!inside_box(0, 20, 10, 20, 30));
    assert!(!inside_box(0, n(-31), 10, 20, 30));
    assert_eq!(point_part(5, 10, 1, 20), n(-4));
    assert!(kind_hits(6) && !kind_hits(2));
}
