use super::*;

#[test]
fn patches() {
    let mut p = Patches {
        switch: 1,
        settings: [8, 0, 50, 4321, 200, 100, 100, 100, 100, 100, 100, 100],
        ..Default::default()
    };
    apply_patches(&mut p, 100.0);
    assert_eq!(p.applied, 1);
    assert_eq!(p.settings[3], 4321);
    assert_eq!(f32::from_bits(p.factors[0]), 0.5);
    assert_eq!(f32::from_bits(p.factors[1]), 43.21);
    assert_eq!(f32::from_bits(p.gravity), 9.8f32 * 0.5);
    // Record 0: by factor 6 (setting 8, 100%), then its third by factor 2 (200%).
    assert_eq!(f32::from_bits(p.records[0][0]), 1.0);
    assert_eq!(p.records[0][2], (1.7f32 * 2.0).to_bits());
    assert_eq!(p.tens, [0; 2]);
    let before = p;
    apply_patches(&mut p, 100.0);
    assert_eq!(p, before);
    // Held to the ranges, switched off: only the plain values.
    let mut q = Patches {
        settings: [0, u32::MAX, 9, 2, 0, 1, 2, 3, 4, 5, 6, 0x8000_0000],
        ..Default::default()
    };
    apply_patches(&mut q, 100.0);
    assert_eq!(q.settings[0], 4);
    assert_eq!(q.settings[1], 0);
    // 0x80000000 is the most negative figure: held to the bottom (the difference's sign
    // alone held it to the top).
    assert_eq!(q.settings[11], 10);
    let mut r = Patches {
        settings: [
            0x7FFF_FFFF,
            0x8000_0000,
            0x8000_0009,
            0x8000_0000,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0xFFFF_FFFF,
        ],
        ..Default::default()
    };
    apply_patches(&mut r, 100.0);
    assert_eq!(r.settings[0], 0x10);
    assert_eq!(r.settings[1], 0);
    assert_eq!(r.settings[2], 10);
    assert_eq!(r.settings[3], 0);
    assert_eq!(r.settings[11], 10);
    assert_eq!(q.factors, [0; 10]);
    assert_eq!(q.records[3], [PLAIN[0], PLAIN[1], OTHER_THIRD.1]);
    assert_eq!(q.tens, [TEN; 2]);
}
