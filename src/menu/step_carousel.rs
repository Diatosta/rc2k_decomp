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
use crate::framework::{game_read, game_write, rva};

// ---- game data ----

const CLOCK_RVA: usize = rva(0x613830); // dword_613830
const LAST_RVA: usize = rva(0x75AA84); // dword_75AA84
const PASSED_RVA: usize = rva(0x75AA88); // dword_75AA88
pub const DIRECTION_RVA: usize = rva(0x75A660); // dword_75A660
pub const ANGLE_RVA: usize = rva(0x75AA80); // dword_75AA80
pub const FRONT_RVA: usize = rva(0x75AA7C); // dword_75AA7C
pub const COUNT_RVA: usize = rva(0x75AA78); // dword_75AA78

// ---- Rust version ----

pub unsafe fn step_carousel() -> Result<(), BadInput> {
    // Converted: `rc2k_core::menu::navigation` (`passed`, `spin`), worked out before
    // anything is written.
    use rc2k_core::menu::navigation::{Turn, passed, spin};
    unsafe {
        let clock: u32 = game_read(CLOCK_RVA);
        let passed = passed(clock, game_read(LAST_RVA));
        let t = Turn {
            count: game_read(COUNT_RVA),
            value: game_read(FRONT_RVA),
            way: game_read(DIRECTION_RVA),
            step: game_read(ANGLE_RVA),
        };
        let turned = spin(t, passed)?;
        game_write(LAST_RVA, clock);
        game_write(PASSED_RVA, passed);
        if t.way != 0 {
            game_write(ANGLE_RVA, turned.step);
            game_write(DIRECTION_RVA, turned.way);
            if turned.value != t.value {
                game_write(FRONT_RVA, turned.value);
            }
        }
        Ok(())
    }
}
