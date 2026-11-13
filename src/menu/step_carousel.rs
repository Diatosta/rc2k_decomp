//! `menu::step_carousel`: moves a menu's turning objects on by the time passed.
//!
//! The time since the last call (the scene clock less the clock kept at the last call, which
//! becomes the clock) is kept, held to 100, as the time since the last step. Then the turn
//! (`menu::turn_carousel`, inlined): while the turn direction is not 0 the angle is moved by
//! ((angle / 8) + 0x4000000) x the time / 20, down for a negative direction, up (from the angle
//! negated) for a positive one. When it passes zero it is 0 and the direction 0; after a turn
//! upwards the object in front is the next one, the first again after the last.
//! `menu::move_highlight_left` and `_right` start a turn. In 17 lists of the screens with turning
//! objects (the main menu's among them). Refuses a time kept above 0x71C71C6, with which the
//! division would trap; it cannot come, the time being held to 100. The menus and draw lists run
//! its `entry` (`menu::table_calls`).

use crate::errors::BadInput;
use crate::menu::owner::Screens;
use crate::race::owner::Race;

pub fn step_carousel(screens: &Screens, race: &Race) -> Result<(), BadInput> {
    // The logic is in `rc2k_core::menu::navigation` (`passed`, `spin`), worked out before
    // anything is written.
    use rc2k_core::menu::navigation::{Turn, passed, spin};
    let clock: u32 = race.shared.physics_tables.get().scene_clock.get();
    let passed = passed(clock, screens.state.carousel.last_clock.get());
    let t = Turn {
        count: screens.state.carousel.count.get() as i32,
        value: screens.state.carousel.front.get() as i32,
        way: screens.state.carousel.direction.get(),
        step: screens.state.carousel.angle.get(),
    };
    let turned = spin(t, passed)?;
    screens.state.carousel.last_clock.set(clock);
    screens.state.carousel.passed.set(passed);
    if t.way != 0 {
        screens.state.carousel.angle.set(turned.step);
        screens.state.carousel.direction.set(turned.way);
        if turned.value != t.value {
            screens.state.carousel.front.set(turned.value as u32);
        }
    }
    Ok(())
}
