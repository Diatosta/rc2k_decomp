//! Controllers known by name: the bindings a new profile for one of them starts with, and the
//! family of button pictures shown for one the platform does not take as a gamepad.
//!
//! Every other controller starts from the defaults of its kind (`Profile::new`) and, unless it is
//! a gamepad, shows its buttons as numbers until the player picks a style.

use crate::input::icons::Family;
use crate::input::profile::{AxisUse, Digital, HatDirection, Profile};

/// A controller known by its name, as the platform reports it.
struct Known {
    /// The name, compared whole.
    name: &'static str,
    /// The pictures of its buttons (in DirectInput's order for that family).
    family: Family,
    /// Its starting bindings, over the defaults of its kind.
    defaults: fn(&mut Profile),
}

const KNOWN: &[Known] = &[Known {
    name: "FANATEC Wheel",
    family: Family::PlayStation,
    defaults: fanatec_wheel,
}];

fn known(name: &str) -> Option<&'static Known> {
    KNOWN.iter().find(|k| k.name == name)
}

/// The family of button pictures for the controller called `name`, when it is known.
pub fn family(name: &str) -> Option<Family> {
    known(name).map(|k| k.family)
}

/// The known controller's starting bindings put into `profile` (made with its kind's defaults);
/// nothing for a controller not known.
pub fn apply_defaults(profile: &mut Profile) {
    if let Some(k) = known(&profile.device.name) {
        (k.defaults)(profile);
    }
}

/// A Fanatec wheel base with a PlayStation rim: the wheel's axis 0 over 270 degrees of its 900, the
/// throttle on axis 2 and the brake on axis 5 (both at rest high), the paddles for the gears, an
/// H shifter on buttons 12 (reverse) and 13 .. 19, the face buttons as a PlayStation pad's.
fn fanatec_wheel(profile: &mut Profile) {
    let button = |button| Some(Digital::Button { button });
    let hat = |direction| Some(Digital::Hat { hat: 0, direction });
    profile.wheel = true;
    profile.full_rotation_degrees = 900;
    profile.shifter = true;
    let steering = profile.settings_mut(0);
    steering.dead_zone = 10;
    steering.range = 300;
    profile.settings_mut(1).dead_zone = 0;
    let b = &mut profile.bindings;
    b.steering_axis = Some(AxisUse::Centred { axis: 0 });
    b.accelerate_axis = Some(AxisUse::Pedal {
        axis: 2,
        rest_high: true,
    });
    b.brake_axis = Some(AxisUse::Pedal {
        axis: 5,
        rest_high: true,
    });
    b.gear_up = button(4);
    b.gear_down = button(5);
    b.handbrake = button(2);
    b.camera = button(3);
    b.pause = button(9);
    b.lights = button(8);
    b.reset = button(21);
    b.reverse = button(12);
    b.gear_1 = button(13);
    b.gear_2 = button(14);
    b.gear_3 = button(15);
    b.gear_4 = button(16);
    b.gear_5 = button(17);
    b.gear_6 = button(18);
    b.gear_7 = button(19);
    b.menu_enter = button(1);
    b.menu_back = button(2);
    b.menu_up = hat(HatDirection::Up);
    b.menu_down = hat(HatDirection::Down);
    b.menu_third = button(0);
    b.menu_tab = button(4);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::input::device_id::DeviceId;
    use crate::input::profile::Kind;

    fn id(name: &str) -> DeviceId {
        DeviceId {
            name: name.into(),
            serial: None,
            nth: 1,
        }
    }

    #[test]
    fn the_fanatec_wheel_starts_with_its_own_bindings_and_playstation_pictures() {
        let p = Profile::new(id("FANATEC Wheel"), Kind::Wheel);
        assert!(p.wheel && p.shifter);
        assert_eq!(p.settings(0).range, 300);
        assert_eq!(p.settings(0).dead_zone, 10);
        assert_eq!(
            p.bindings.accelerate_axis,
            Some(AxisUse::Pedal {
                axis: 2,
                rest_high: true
            })
        );
        assert_eq!(p.bindings.gear_7, Some(Digital::Button { button: 19 }));
        assert_eq!(p.bindings.menu_enter, Some(Digital::Button { button: 1 }));
        assert_eq!(family("FANATEC Wheel"), Some(Family::PlayStation));
    }

    #[test]
    fn other_controllers_keep_their_kinds_defaults() {
        let wheel = Profile::new(id("Some Wheel"), Kind::Wheel);
        assert_eq!(wheel.bindings.gear_up, Some(Digital::Button { button: 0 }));
        assert_eq!(wheel.bindings.accelerate_axis, None);
        assert_eq!(family("Some Wheel"), None);
    }
}
