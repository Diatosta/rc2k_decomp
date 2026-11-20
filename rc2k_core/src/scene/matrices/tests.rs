use super::*;
use crate::scene::places::TestPlaces;

fn layout() -> Layout {
    let m = |n: u32| n * 0x40;
    Layout {
        turn_x: m(0),
        turn_y: m(1),
        turn_z: m(2),
        placed: m(3),
        turned: m(4),
        lit: m(5),
        view_turn: m(6),
        object: m(7),
        identity: m(8),
        kept_place: m(9),
        view_back: m(10),
        view: m(11),
        turn_x_work: m(12),
        turn_y_work: m(12) + 8,
        turn_z_work: m(12) + 16,
        light_direction: m(12) + 24,
        checked: m(12) + 36,
        half_turns: m(12) + 40,
        turn: m(12) + 44,
        nothing: m(12) + 48,
        squared: m(13),
        inverted: m(14),
        sines: m(16),
        cosines: m(16) + 0x4000,
    }
}

fn places() -> TestPlaces {
    let l = layout();
    let mut p = TestPlaces(vec![0; (l.cosines as usize + 0x4000) / 4]);
    for n in 0..0x1000u32 {
        let a = n as f64 * std::f64::consts::TAU / 4096.0;
        p.set_float(l.sines + 4 * n, crate::math::portable::sin(a) as f32);
        p.set_float(l.cosines + 4 * n, crate::math::portable::cos(a) as f32);
    }
    let identity = [1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0];
    for at in [l.identity, l.turn_x, l.turn_y, l.turn_z] {
        p.set_floats(at, &identity);
        p.set_float(at + 0x3C, 1.0);
    }
    p.set_float(l.half_turns, 65536.0);
    p.set_float(l.turn, (1.0 / std::f64::consts::TAU) as f32);
    p
}

#[test]
fn adjugates() {
    let mut p = TestPlaces(vec![0; 32]);
    let m = [
        2.0, 0.0, 0.0, 7.0, 0.0, 3.0, 0.0, 7.0, 0.0, 0.0, 4.0, 7.0, 1.0, -2.0, 0.0, 7.0,
    ];
    p.set_floats(0, &m);
    adjugate_matrix(&mut p, 0, 0x40);
    assert_eq!(p.floats::<3>(0x40), [12.0, 0.0, 0.0]);
    assert_eq!(p.floats::<3>(0x50), [0.0, 8.0, 0.0]);
    assert_eq!(p.floats::<3>(0x60), [0.0, 0.0, 6.0]);
    assert_eq!(p.floats::<3>(0x70), [-1.0, 2.0, -0.0]);
    assert_eq!(p.dword(0x4C), 0);
    // In place: later entries read the earlier results.
    adjugate_matrix(&mut p, 0, 0);
    assert_eq!(p.float(0), 12.0);
    assert_eq!(p.float(0x14), 12.0 * 4.0);
}

#[test]
fn multiplying_in_place() {
    let mut p = TestPlaces(vec![0; 48]);
    let a: Vec<f32> = (0..16).map(|n| n as f32 + 1.0).collect();
    p.set_floats(0, &a);
    p.set_floats(0x40, &a);
    // Apart, by a turning that is the identity with a place: the rows moved.
    let id = [1.0, 0.0, 0.0, 9.0, 0.0, 1.0, 0.0, 9.0, 0.0, 0.0, 1.0, 9.0];
    p.set_floats(0x80, &id);
    p.set_floats(0xB0, &[10.0, 20.0, 30.0, 9.0]);
    multiply(&mut p, 0, 0x80, 0x40);
    assert_eq!(p.floats::<4>(0x40), [1.0, 2.0, 3.0, 4.0]);
    assert_eq!(p.floats::<4>(0x70), [23.0, 34.0, 45.0, 16.0]);
    // Over the second: column 0 of row 0 from the column as it was, row 1 from the
    // cell just written.
    let mut s = TestPlaces(vec![0; 32]);
    s.set_floats(0, &a);
    s.set_floats(0x40, &a);
    multiply(&mut s, 0, 0x40, 0x40);
    let first = 1.0 * 1.0 + (2.0 * 5.0 + 3.0 * 9.0);
    assert_eq!(s.float(0x40), first);
    assert_eq!(s.float(0x50), 5.0 * first + (6.0 * 5.0 + 7.0 * 9.0));
    // Column 1 of row 0 reads column 1, not yet written.
    assert_eq!(s.float(0x44), 1.0 * 2.0 + (2.0 * 6.0 + 3.0 * 10.0));
    // The fourth column is left.
    assert_eq!(s.float(0x4C), 4.0);
}

#[test]
fn turns() {
    let l = layout();
    let mut p = places();
    let t = set_table_turn(&mut p, &l, Axis::Y, 0x4000);
    assert_eq!(f32::from_bits(t.sine), 1.0);
    assert_eq!(p.float(l.turn_y + 8), -1.0);
    assert_eq!(p.float(l.turn_y + 0x20), 1.0);
    assert_eq!(p.dword(l.turn_y_work), t.sine);
    let t = set_fine_turn(&mut p, &l, Axis::Z, 0x4000_0000);
    assert_eq!(p.dword(l.checked), 0x4000_0000);
    assert!((f32::from_bits(t.sine) - 1.0).abs() < 1e-6);
    assert_eq!(p.dword(l.turn_z + 0x10), t.negated());
}

#[test]
fn unit_and_square() {
    let l = layout();
    let mut p = places();
    p.set_floats(0, &[3.0, 0.0, -4.0]);
    unit_vector(&mut p, &l, 0, 0x10);
    assert_eq!(p.floats::<3>(0x10), [0.6, 0.0, -0.8]);
    assert_eq!(p.float(l.checked), 5.0);
    p.set_floats(0, &[0.0, -0.0, 0.0]);
    unit_vector(&mut p, &l, 0, 0);
    assert_eq!(p.floats::<3>(0).map(f32::to_bits), [0; 3]);
    // A turning that has drifted, squared again.
    let m = [
        1.0, 0.01, 0.0, 0.0, -0.01, 1.0, 0.0, 0.0, 0.0, 0.0, 2.0, 0.0,
    ];
    p.set_floats(0x40, &m);
    square(&mut p, &l, 0x40);
    assert_eq!(p.floats::<3>(0x60), [0.0, 0.0, 1.0]);
    let r: [f32; 3] = p.floats(0x50);
    assert!((vector::length(r) - 1.0).abs() < 1e-6);
}

#[test]
fn inverting_and_the_view() {
    let l = layout();
    let mut p = places();
    let place = set_view_matrices(&mut p, &l, [0x1000_0000, 0x0800_0000, 0], [5, 6, 7]);
    assert_eq!(
        place,
        [
            5u32.wrapping_neg(),
            6u32.wrapping_neg(),
            7u32.wrapping_neg()
        ]
    );
    // The turning undone times the turning is near the identity.
    for row in 0..3 {
        for column in 0..3 {
            let r: [f32; 3] = p.floats(cell(l.view_back, row, 0));
            let c = [0, 1, 2].map(|k| p.float(cell(l.view_turn, k, column)));
            let want = if row == column { 1.0 } else { 0.0 };
            assert!((vector::dot(r, c) - want).abs() < 1e-5, "{row} {column}");
        }
    }
    assert_eq!(p.floats::<3>(cell(l.kept_place, 3, 0)), [-5.0, -6.0, -7.0]);
}

#[test]
fn object_matrix() {
    let l = layout();
    let mut p = places();
    p.set_floats(
        l.view_turn,
        &[1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0],
    );
    let o = Object {
        angles: [0, 0],
        place: [1, 2, 3],
        size: 32,
    };
    make_object_matrix(&mut p, &l, [0, 0], o);
    assert_eq!(p.floats::<3>(l.light_direction), [0.0, 0.0, 1.0]);
    assert_eq!(p.float(l.checked), 2.0);
    assert_eq!(p.float(l.object), 2.0);
    assert_eq!(p.floats::<3>(cell(l.object, 3, 0)), [1.0, 2.0, 3.0]);
    assert_eq!(sixteenths(-8), -0.5);
}

#[test]
fn crosses() {
    let mut p = TestPlaces(vec![0; 9]);
    p.set_floats(0, &[1.0, 0.0, 0.0, 0.0, 1.0, 0.0]);
    cross3(&mut p, 0, 12, 24);
    assert_eq!(p.floats::<3>(24), [0.0, 0.0, 1.0]);
    // b x c, as well.
    p.set_floats(24, &[0.0; 3]);
    cross(&mut p, 0, 12, 24);
    assert_eq!(p.floats::<3>(24), [0.0, 0.0, 1.0]);
    // Over its first input: the second part reads the first part written.
    p.set_floats(0, &[1.0, 2.0, 3.0, 4.0, 5.0, 6.0]);
    cross3(&mut p, 0, 12, 0);
    let x = 2.0 * 6.0 - 3.0 * 5.0;
    assert_eq!(
        p.floats::<3>(0),
        [
            x,
            3.0 * 4.0 - x * 6.0,
            x * 5.0 - (3.0 * 4.0 - x * 6.0) * 4.0
        ]
    );
}
