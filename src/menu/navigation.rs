//! Each menu item names its neighbours by number at +0x24, +0x28, +0x2C and +0x30 (which key
//! goes with which is in `Direction`). A link of 0x6F means there is none, and 0xFFFFFC1A
//! means "ask the item": its callback for that direction (+0x34 .. +0x40) is called
//! instead. Every move plays the click, `sub_40F059` with eax = 1.
//! The two vertical moves go on along the same link until they reach an item whose +0x48 is
//! 0xFFFFFC1B, so items that cannot be chosen are stepped over. The two horizontal ones take
//! one step and, while `dword_7554EC` is 1, first turn a counter: `dword_75AA7C` goes round
//! 0 .. `dword_75AA78` - 1, and `dword_75A660` and `dword_75AA80` are left at 1 and 0 (right)
//! or -1 and -1 (left). They click only if `dword_7554D0` is not 1.
//!
//! The click, the callbacks and the screenshot are called directly (2026-10-24: the `Outside` trait
//! went with the screen loops' registers).

use crate::errors::Refused;
use crate::menu::flow_store::SCREEN_FLOW;
use crate::menu::item_at::item_at;
use crate::menu::loop_store::LOOP_FLAGS;
use crate::menu::state_store::MENU_STATE;

// ---- game data ----

unsafe fn dword(at: u32) -> u32 {
    unsafe { (at as *const u32).read_unaligned() }
}

/// The four moves. Which is up and which is down is a reading: the keys are 0x66 and 0x6B.
/// Left and right are keys 0x68 and 0x69, the ones that step a value down and up in the
/// service area.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Direction {
    Up,
    Down,
    Right,
    Left,
}

impl Direction {
    /// (the link's place in an item, the callback's)
    fn offsets(self) -> (u32, u32) {
        match self {
            Self::Up => (0x24, 0x34),
            Self::Down => (0x28, 0x38),
            Self::Left => (0x2C, 0x3C),
            Self::Right => (0x30, 0x40),
        }
    }
}

// ---- Rust version of the four moves ----

/// The click is `sub_40F059` with sound 1 (`sound::play_menu_sound`); the callback gets
/// the item (edi in the original, the only register its callbacks read).
pub unsafe fn move_highlight(direction: Direction) -> Result<(), Refused> {
    // Converted: `rc2k_core::menu::navigation` (`turn`, `link`, `walk`), everything worked
    // out before it is written.
    use rc2k_core::menu::navigation::{Link, Turn, link, turn, walk};
    let (link_at, callback) = direction.offsets();
    let sideways = matches!(direction, Direction::Left | Direction::Right);
    unsafe {
        let highlighted: u32 = LOOP_FLAGS.item.get();
        let item = item_at(highlighted)?;
        let turned = (sideways && SCREEN_FLOW.turning.get() == 1).then(|| {
            let t = Turn {
                count: MENU_STATE.carousel.count.get() as i32,
                value: MENU_STATE.carousel.front.get() as i32,
                way: MENU_STATE.carousel.direction.get(),
                step: MENU_STATE.carousel.angle.get(),
            };
            turn(t, direction == Direction::Right)
        });
        let next = dword(item + link_at);
        let to = match link(next) {
            Link::To(n) if !sideways => Some(walk(n, |n| {
                let it = item_at(n).map_err(|e| rc2k_core::Invalid(e.0))?;
                Ok((dword(it + link_at), dword(it + 0x48)))
            })?),
            Link::To(n) => Some(n),
            _ => None,
        };
        if let Some(t) = turned {
            MENU_STATE.carousel.front.set(t.value as u32);
            MENU_STATE.carousel.direction.set(t.way);
            MENU_STATE.carousel.angle.set(t.step);
        }
        if link(next) == Link::None {
            return Ok(());
        }
        if !sideways || SCREEN_FLOW.quiet.get() != 1 {
            crate::sound::play_menu_sound::play_menu_sound_in_game(1);
        }
        match to {
            Some(n) => LOOP_FLAGS.item.set(n),
            None => crate::menu::table_calls::call_key(dword(item + callback), item)?,
        }
    }
    Ok(())
}
