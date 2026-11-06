//! `menu::step_carousel` (0x46989B): moves a menu's turning objects on by the time passed.
//!
//! In:  nothing
//! Out: every register preserved
//! The time since the last call (the clock `dword_613830` less `dword_75AA84`, which becomes the
//! clock) is kept, held to 100, in `dword_75AA88`. Then the turn (`sub_49044F`, inlined): while
//! `dword_75A660` is not 0 the angle `dword_75AA80` is moved by ((angle / 8) + 0x4000000) x the
//! time / 20 (`sub_4904C7`), down for a negative direction, up (from the angle negated) for a
//! positive one. When it passes zero it is 0 and the direction 0; after a turn upwards the object
//! in front, `dword_75AA7C`, is the next one, the first again after the last (`dword_75AA78`).
//! `menu::move_highlight_left` and `_right` start a turn. In 17 lists of the screens with turning
//! objects (the main menu's among them). Refuses a time kept above 0x71C71C6, with which the
//! division would trap; it cannot come, the time being held to 100. The menus and draw lists run
//! its `entry` (`menu::table_calls`); the registers above are the original's, for reference.

use crate::errors::BadInput;
use crate::menu::owner::Screens;
use crate::race::owner::race;

// ---- game data ----

// ---- Rust version ----

pub fn step_carousel(screens: &Screens) -> Result<(), BadInput> {
    // Converted: `rc2k_core::menu::navigation` (`passed`, `spin`), worked out before
    // anything is written.
    use rc2k_core::menu::navigation::{Turn, passed, spin};
    let clock: u32 = race().shared.physics_tables.scene_clock.bits();
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
