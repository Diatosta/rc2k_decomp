use super::*;

#[test]
fn sums() {
    assert_eq!(brighter(0xF0F0_F0F0, 0x2020_2020), 0xFFFF_FFFF);
    assert_eq!(amount(0x20_0000, 5), 0);
    assert_eq!(amount(0, 0x100), 0xFF);
    // The product wraps.
    assert_eq!(amount(0, 0x2_0000), 0);
    assert_eq!(amount(0x40_0000, 1), 0);
    assert_eq!(amount(0x1F_0000, 0x100), 0x80);
    assert_eq!(tinted(0x80, 0x00FF_8000), 0x007F_4000);
}

#[test]
fn point_lights() {
    let lamps = [
        Lamp {
            point: 1,
            strength: 0x100,
            colour: 0x0000_00FF,
        },
        Lamp {
            point: 2,
            strength: 0,
            colour: 7,
        },
    ];
    let out = add_point_lights(lamps, [0; 3], |p| [p, 0, 0], 0x10);
    assert_eq!(
        out,
        PointLights {
            strength: Some(0),
            colour: Some(0xFF),
            distance: Some(0),
            light: Some(0xFF),
        }
    );
    let none = add_point_lights([], [0; 3], |_| unreachable!(), 0);
    assert_eq!(none, PointLights::default());
}

#[test]
fn plain() {
    assert!(plain_light(0, || 0) && !plain_light(1, || unreachable!()));
}

#[test]
fn object_lights() {
    assert_eq!(object_light(5, || unreachable!()), ObjectLight::White);
    assert_eq!(
        object_light(2, || true),
        ObjectLight::Gathered {
            clear_strength: true,
            own_car: false
        }
    );
    assert_eq!(
        object_light(2, || false),
        ObjectLight::Gathered {
            clear_strength: false,
            own_car: false
        }
    );
    assert_eq!(
        object_light(3, || unreachable!()),
        ObjectLight::Gathered {
            clear_strength: false,
            own_car: true
        }
    );
    assert_eq!(object_light(0, || unreachable!()), ObjectLight::Nothing);
    assert_eq!(object_light(4, || unreachable!()), ObjectLight::Nothing);
}
