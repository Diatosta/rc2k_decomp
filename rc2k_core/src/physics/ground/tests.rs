use super::*;

#[test]
fn probes() {
    assert_eq!(wheel_lifted(0, 1), (true, 6));
    assert_eq!(wheel_lifted(1 << 6, 1), (true, 2));
    assert_eq!(wheel_lifted(1 << 6 | 1 << 2, 1), (false, 2));
    let c = noise_cell(0x3F << 14, 0x3F << 14);
    assert_eq!(c.row, 0x3F);
    assert_eq!(c.corners, [0xFFF, 0xFC0, 0x000, 0x03F]);
    assert_eq!(noise_lerp(0, 0x100, 0x2000), (0x80, 0));
    assert_eq!(bump_noise([0, 0x100, 0x100, 0], 0x2000, 0).2, 0x80);
}

#[test]
fn noise() {
    assert_eq!(noise_level(0x8000, 0x10, 0x30), (0x20, 0x20, 0));
    assert_eq!(noise_level(0x10000, 0, 0x200), (0xFF, 0x200, 0));
    assert_eq!(noise_level(0xFFFF_0000, 0, 0x10), (0, 0x10, u32::MAX));
    assert_eq!(rough_floor(3, 5), 5);
    assert_eq!(rough_floor(7, 5), 7);
}

#[test]
fn plane() {
    assert_eq!(plane_height(10, 20, 0, 0.5, 0.25), 10);
    assert_eq!(
        plane_height(i32::MAX, i32::MAX, i32::MIN, 2.0, 2.0),
        i32::MIN
    );
    assert_eq!(plane_height(3, 0, 0, 0.5, 0.0), 2);
}

#[test]
fn surface() {
    assert_eq!(
        (0..5).map(surface_flag).collect::<Vec<_>>(),
        [0, 0, 1, 1, 0]
    );
}

#[test]
fn skid() {
    let mut s = SkidStart {
        pending: 5,
        ..Default::default()
    };
    assert_eq!(s.start([0; 4], || unreachable!()), None);
    assert_eq!(s.start([0, 0, 0, 1], || 77), Some(77));
    assert_eq!(
        s,
        SkidStart {
            from: 4,
            pending: 0,
            step: 77,
            on: 1,
            switch_length: 10,
            switch_share: 0
        }
    );
    assert_eq!(s.start([1, 0, 0, 0], || unreachable!()), Some(0));
}

#[test]
fn cells() {
    assert_eq!(flip_cell_index(0, 1, 0), (0, 1, 1));
    assert_eq!(flip_cell_index(9, 1, 0), (1, 0, 0));
    assert_eq!(flip_cell_index(4, 1, 0), (4, 0, 0));
    assert_eq!(flip_cell_index(6, 1, 0), (6, 0, 2));
    assert_eq!(flip_cell_index(7, 1, 1), (7, 0, 2));
    assert_eq!(flip_cell_index(3, 0, 0), (3, 1, 3));
}
