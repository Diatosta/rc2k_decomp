use super::*;

#[test]
fn car_state() {
    let v = voices(3.0, 2.0, 7.0, false);
    assert_eq!(
        v,
        Voices {
            share: 1.5,
            full: 1.0,
            motion: 7.0
        }
    );
    assert_eq!(voices(3.0, 2.0, 7.0, true).full, 0.0);
    assert_eq!(pitch(1.5, 256.0), 384.0);
    assert_eq!(volume(0x200), 0x100);
    assert_eq!(volume(u32::MAX), 0);
    assert_eq!(volume(0x80), 0x80);
}

#[test]
fn places() {
    let mut view = [0.0; 15];
    view[0] = 1.0;
    view[5] = 1.0;
    view[10] = 1.0;
    let p = place(
        [0x30000, 5, -0x10000],
        [0x10000, 5, 0],
        None,
        &view,
        1.0 / 65536.0,
    );
    assert_eq!(p, [2.0, 0.0, -1.0]);
    let p = place(
        [i32::MAX, 0, 0],
        [i32::MIN, 0, 0],
        Some([i32::MAX, 0, 0]),
        &view,
        1.0,
    );
    assert_eq!(
        p[0],
        (3.0 * i32::MAX as f64 + 1.0) as f32,
        "no wrap: whole numbers"
    );
}

#[test]
fn object_sounds() {
    assert!(object_sound(0x100, |_| 0, |_| 0, |_| 0).is_err());
    assert!(object_sound(3, |_| 0x4B0, |_| 0, |_| 0).is_err());
    assert_eq!(
        object_sound(3, |t| t + 1, |k| k as u8 | 0xF0, |s| s * 10),
        Ok(40)
    );
}
