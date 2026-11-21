//! `menu::step_carousel`: moves a menu's turning objects on by the time passed.
//!
//! The time since the last call (the scene clock less the clock kept at the last call, which
//! becomes the clock) is kept, held to 100, as the time since the last step. Then the turn
//! (`menu::turn_carousel`, inlined): while the turn direction is not 0 the angle is moved by
//! ((angle / 8) + 2^26) x the time / 20, down for a negative direction, up (from the angle
//! negated) for a positive one. When it passes zero it is 0 and the direction 0; after a turn
//! upwards the object in front is the next one, the first again after the last.
//! `menu::move_highlight_left` and `_right` start a turn. In 17 lists of the screens with turning
//! objects (the main menu's among them). Refuses a time kept so large that
//! the step would not fit 32 bits; it cannot come, the time being held to 100.

use crate::errors::BadInput;
use crate::menu::owner::Screens;
use crate::race::owner::Race;

pub fn step_carousel(screens: &Screens, race: &Race) -> Result<(), BadInput> {
    // The logic is in `rc2k_core::menu::navigation` (`passed`, `spin`), worked out before
    // anything is written.
    use rc2k_core::menu::navigation::{Turn, passed, spin};
    let clock: u32 = race.shared.run().scene_clock;
    let passed = passed(clock, screens.carousel.read().last_clock);
    let t = Turn {
        count: screens.carousel.read().count as i32,
        value: screens.carousel.read().front as i32,
        way: screens.carousel.read().direction,
        step: screens.carousel.read().angle,
    };
    let turned = spin(t, passed)?;
    screens.carousel.write().last_clock = clock;
    screens.carousel.write().passed = passed;
    if t.way != 0 {
        screens.carousel.write().angle = turned.step;
        screens.carousel.write().direction = turned.way;
        if turned.value != t.value {
            screens.carousel.write().front = turned.value as u32;
        }
    }
    Ok(())
}
