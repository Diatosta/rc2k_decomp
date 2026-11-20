use super::*;

#[test]
fn faces() {
    let corners = |c: [u32; 4]| move |o: u32| c[(o / 4 - 1) as usize];
    let all_in = |_: u32, _: u32| Some(false);
    assert_eq!(
        face_under_point(0x10, || 0, 0, corners([0; 4]), all_in),
        FaceTest::Kind
    );
    assert_eq!(
        face_under_point(1, || 0x0140, 0x100, corners([0; 4]), all_in),
        FaceTest::Flags(0x100)
    );
    let tri = corners([1, 2, 3, 3]);
    assert_eq!(
        face_under_point(1, || 0, 0x100, tri, all_in),
        FaceTest::Inside {
            half: 0,
            corners: [4, 8, 0xC]
        }
    );
    let mut tried = Vec::new();
    let out = face_under_point(
        1,
        || 0,
        0,
        tri,
        |a, b| {
            tried.push((a, b));
            Some(a == 0xC)
        },
    );
    assert_eq!(out, FaceTest::Edge);
    assert_eq!(tried, [(4, 8), (8, 0xC), (0xC, 4)]);
    // A quad: the diagonal's outer side takes the second half.
    let quad = corners([1, 2, 3, 4]);
    let mut tried = Vec::new();
    let out = face_under_point(
        1,
        || 0,
        0,
        quad,
        |a, b| {
            tried.push((a, b));
            Some(a == 8 && b == 0xC)
        },
    );
    assert_eq!(
        out,
        FaceTest::Inside {
            half: 1,
            corners: [0x10, 0xC, 8]
        }
    );
    assert_eq!(tried, [(8, 0xC), (0x10, 0xC), (8, 0x10)]);
    assert_eq!(
        face_under_point(1, || 0, 0, quad, |_, _| None),
        FaceTest::Edge
    );
}

#[test]
fn searches() {
    assert_eq!(search_faces::<()>(5, 3, |_| Ok(true)), Ok((false, 5)));
    assert_eq!(
        search_faces::<()>(NO_FACE, NO_FACE, |_| Ok(true)),
        Ok((false, NO_FACE))
    );
    assert_eq!(search_faces::<()>(3, 6, |f| Ok(f == 5)), Ok((true, 5)));
    assert_eq!(search_faces::<()>(3, 6, |_| Ok(false)), Ok((false, 7)));
    assert!(check_record_faces(3, 9, 9).is_err());
    assert!(check_record_faces(9, 3, 9).is_ok());
    assert!(check_record_faces(NO_FACE, NO_FACE, 9).is_ok());
    assert_eq!(cell_key(0x0060_0000, 0x0040_0000), (0x1003, 0x1000));
    assert_eq!(last_face(NO_FACE, 10), None);
    assert_eq!(last_face(10, 10), None);
    assert_eq!(last_face(9, 10), Some(9));
    let mut cells = Vec::new();
    let found = ground_search::<()>(
        || Ok(false),
        || {},
        || (0x0100_0000, 0x0100_0000),
        |x, z| {
            cells.push((x, z));
            Ok(cells.len() == 3)
        },
    );
    assert_eq!(found, Ok(true));
    assert_eq!(cells[1], (0x00E0_0000, 0x00E0_0000));
    assert_eq!(cells[2], (0x0100_0000, 0x00E0_0000));
    let mut n = 0;
    cells_around::<()>(
        || (0, 0),
        |_, _| {
            n += 1;
            Ok(())
        },
    )
    .unwrap();
    assert_eq!(n, 9);
    let mut walked = Vec::new();
    let result = each_face::<Invalid>(3, 5, 9, |f| {
        walked.push(f);
        Ok(())
    });
    assert_eq!((result, walked), (Ok(6), vec![3, 4, 5]));
    assert_eq!(each_face::<Invalid>(5, 3, 9, |_| Ok(())), Ok(5));
    assert!(each_face::<Invalid>(3, NO_FACE, 9, |_| Ok(())).is_err());
    assert!(each_face::<Invalid>(3, 9, 9, |_| Ok(())).is_err());
}
