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

struct Cars([u32; 7]);
impl CarLights for Cars {
    fn one_car(&self) -> bool {
        self.0[0] != 0
    }
    fn the_car(&self) -> u32 {
        self.0[1]
    }
    fn all_cars(&self) -> bool {
        self.0[2] == 1
    }
    fn near(&self) -> u32 {
        self.0[3]
    }
    fn view(&self) -> u32 {
        self.0[4]
    }
    fn drawn(&self) -> u32 {
        self.0[5]
    }
    fn viewed(&self) -> u32 {
        self.0[6]
    }
}

#[test]
fn cars() {
    let lit = |c: Cars| (0..4).filter(|&n| car_lights(&c, n)).collect::<Vec<_>>();
    assert_eq!(lit(Cars([1, 2, 0, 0, 0, 0, 0])), [2]);
    assert_eq!(lit(Cars([1, 2, 1, 0, 0, 0, 0])), [0, 1, 2, 3]);
    assert_eq!(lit(Cars([0, 0, 0, 3, 0, 1, 0])), [0, 2]);
    assert_eq!(lit(Cars([0, 0, 0, 3, 1, 9, 2])), [2]);
    assert_eq!(lit(Cars([0, 0, 1, 1, 1, 0, 2])), [1, 2, 3]);
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
