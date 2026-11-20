use super::*;

fn put(file: &mut Vec<u8>, words: &[u32]) {
    for w in words {
        file.extend_from_slice(&w.to_le_bytes());
    }
}

/// Two faces (groups 1 and 2) over three points, one normal.
fn sample() -> Vec<u8> {
    let mut f = vec![0u8; 4];
    put(&mut f, &[3, 2, 1]);
    f.resize(0x2C, 0);
    put(&mut f, &[10, 20, 30, 40, 50, 60, 70, 80, 90]);
    // Face 0: group 1, three vertices on points 0, 1, 2.
    put(&mut f, &[3, 5 | (1 << 24), 0, 0]);
    for p in 0..3 {
        put(&mut f, &[p, 1, 2, 0]);
    }
    // Face 1: group 2, one vertex on point 2.
    put(&mut f, &[1, 6 | (2 << 24), 0, 0]);
    put(&mut f, &[2, 3, 4, 0]);
    put(&mut f, &[100, 200, 300]);
    f.resize(0x40000, 0);
    f
}

fn request(mask: u32) -> ShapeRequest {
    ShapeRequest {
        group_mask: mask,
        all_points: false,
        turned: false,
        ground_drop: 0,
    }
}

#[test]
fn a_mask_takes_its_faces_and_their_points() {
    let Parsed::Shape(shape) = parse_shape(&sample(), request(1 << 2)).unwrap() else {
        panic!("a shape");
    };
    assert_eq!(shape.faces.len(), 1);
    assert_eq!(shape.points, vec![[70, 80, 90]]);
    assert_eq!(shape.normals, vec![[100, 200, 300]]);
    assert_eq!(shape.point_marks[2], 0);
    assert_eq!(shape.point_marks[0], u32::MAX);
    assert!(!shape.face_taken[0] && shape.face_taken[1]);
    let bytes = shape.face_bytes(0, 10, 7);
    let words: Vec<u32> = bytes
        .chunks(4)
        .map(|c| u32::from_le_bytes(c.try_into().unwrap()))
        .collect();
    assert_eq!(words, vec![1, 6 | (2 << 24), 0, 7, 10, 3, 4, 7]);
    assert_eq!(shape.added().face_bytes, 0x20);
}

#[test]
fn all_points_and_the_turn_apply() {
    let mut req = request(1 << 2);
    req.all_points = true;
    req.turned = true;
    req.ground_drop = 5;
    let Parsed::Shape(mut shape) = parse_shape(&sample(), req).unwrap() else {
        panic!("a shape");
    };
    assert_eq!(shape.points[0], [30, 15, 10u32.wrapping_neg()]);
    shape.measure_reach(|a, b| a.abs().max(b.abs()));
    assert_eq!(shape.radius, 90);
    assert_eq!(shape.last_reach, Some(90));
}

#[test]
fn a_file_without_normals_gives_normal_zero() {
    let mut file = sample();
    file[0xC..0x10].copy_from_slice(&0u32.to_le_bytes());
    let Parsed::Shape(shape) = parse_shape(&file, request(1 << 1)).unwrap() else {
        panic!("a shape");
    };
    assert_eq!(shape.missing_normals, 4);
    assert_eq!(shape.face_bytes(0, 0, 99)[12..16], [0; 4]);
}

#[test]
fn empty_files_and_bad_files() {
    let mut file = sample();
    file[8..12].copy_from_slice(&0u32.to_le_bytes());
    assert_eq!(
        parse_shape(&file, request(1)).unwrap(),
        Parsed::Empty { points: Some(3) }
    );
    file[4..8].copy_from_slice(&0u32.to_le_bytes());
    assert_eq!(
        parse_shape(&file, request(1)).unwrap(),
        Parsed::Empty { points: None }
    );
    assert!(parse_shape(&[0; 6], request(1)).is_err());
}

#[test]
fn car_masks_have_one_bit_each_and_a_rest() {
    let mut groups = [u32::MAX; PARTS];
    groups[0] = 0;
    groups[1] = 3;
    groups[5] = 1;
    let masks = car_masks(groups, 4);
    assert_eq!(masks.parts[0], 1);
    assert_eq!(masks.parts[1], 8);
    assert_eq!(masks.parts[2], 0);
    assert_eq!(masks.parts[5], 2 | 0x0600_0000);
    assert_eq!(masks.rest, u32::MAX - 9);
    let slots = first_part_slots(&masks);
    assert_eq!(slots[0].mask, masks.rest);
    assert!(slots[5..].iter().all(|s| s.measured));
    assert_eq!(slots[12].mask, masks.parts[5]);
}
