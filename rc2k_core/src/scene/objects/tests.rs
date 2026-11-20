use super::*;

#[test]
fn rooms() {
    assert_eq!(object_rooms(true), (0xC_0000, 0x24_0000));
    assert_eq!(object_rooms(false).0 + object_rooms(false).1, 0x58_0000);
}

#[test]
fn measures() {
    assert_eq!(measure(&mut []), None);
    let mut points = [[10, 20, 30], [30, 0, 0xFFFF_FFF6]];
    let m = measure(&mut points).unwrap();
    assert_eq!(m.lowest, [10, 0, 0xFFFF_FFF6]);
    assert_eq!(m.highest, [30, 20, 30]);
    assert_eq!(m.half, [10, 10, 20]);
    assert_eq!(m.middle, [20, 10, 10]);
    assert_eq!(
        points,
        [[0xFFFF_FFF6, 10, 20], [10, 0xFFFF_FFF6, 0xFFFF_FFEC]]
    );
}

#[test]
fn textures() {
    // Two points, three faces: 1 vertex texture 5, 0 vertices texture 0x10005, 2
    // vertices texture 7.
    let mut file = vec![0u32; 0x2C / 4];
    file[1] = 2;
    file[2] = 3;
    file.extend([0; 6]);
    file.extend([1, 5, 0, 0, 9, 9, 9, 9]);
    file.extend([0, 0x10005, 0, 0]);
    file.extend([2, 7, 0, 0]);
    file.extend([0; 8]);
    let dword = |o: usize| {
        file.get(o / 4)
            .copied()
            .ok_or(Invalid("the object's file runs past its room"))
    };
    assert_eq!(texture_list(dword), Ok(vec![5, 7]));
    file[2] = 0;
    let dword = |o: usize| Ok(file[o / 4]);
    assert!(texture_list(dword).is_err());
    assert_eq!(texture_record(0x40, 300), Ok(None));
    assert_eq!(texture_record(1, 15), Ok(Some(0x40)));
    assert!(texture_record(0x3F, 15).is_ok());
    assert!(texture_record(0x3F, 16).is_err());
}

#[test]
fn names() {
    assert_eq!(
        [0, 0x18F, 0x190, 0x257, 0x258, 0x31F, 0x320, u32::MAX].map(folder),
        [0, 0, 1, 1, 2, 2, 3, 3]
    );
    assert_eq!(number_digits(1234), *b"1234");
    assert_eq!(number_digits(10_000)[0], b'0' + 10);
}

#[test]
fn faces() {
    assert_eq!(object_faces(5, 2, |n| n * 0x10), Ok(vec![0x50, 0x60]));
    assert!(object_faces(5, 0, |n| n).is_err());
    assert!(object_faces(FACES - 1, 2, |n| n).is_err());
    assert!(object_faces(0, 2, |n| n).is_err());
}
