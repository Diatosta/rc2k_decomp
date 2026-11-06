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
use crate::menu::item_at::item_at;
use crate::menu::owner::menus;

// ---- game data ----

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
    /// The item's link (+0x24 ..) and key function (+0x34 ..) this move reads.
    fn slot(self) -> usize {
        match self {
            Self::Up => 0,
            Self::Down => 1,
            Self::Left => 2,
            Self::Right => 3,
        }
    }
}

// ---- Rust version of the four moves ----

/// The click is `sub_40F059` with sound 1 (`sound::play_menu_sound`); the callback gets
/// the item (edi in the original, the only register its callbacks read).
pub fn move_highlight(direction: Direction) -> Result<(), Refused> {
    let screens = &menus().screens;
    // Converted: `rc2k_core::menu::navigation` (`turn`, `link`, `walk`), everything worked
    // out before it is written.
    use rc2k_core::menu::navigation::{Link, Turn, link, turn, walk};
    let way = direction.slot();
    let sideways = matches!(direction, Direction::Left | Direction::Right);
    {
        let highlighted: u32 = screens.loop_flags.item.get();
        let item = item_at(highlighted)?;
        let turned = (sideways && screens.flow.turning.get() == 1).then(|| {
            let t = Turn {
                count: screens.state.carousel.count.get() as i32,
                value: screens.state.carousel.front.get() as i32,
                way: screens.state.carousel.direction.get(),
                step: screens.state.carousel.angle.get(),
            };
            turn(t, direction == Direction::Right)
        });
        let next = item.links[way].get();
        let to = match link(next) {
            Link::To(n) if !sideways => Some(walk(n, |n| {
                let it = item_at(n).map_err(|e| rc2k_core::Invalid(e.0))?;
                Ok((it.links[way].get(), it.skip_mark.get()))
            })?),
            Link::To(n) => Some(n),
            _ => None,
        };
        if let Some(t) = turned {
            screens.state.carousel.front.set(t.value as u32);
            screens.state.carousel.direction.set(t.way);
            screens.state.carousel.angle.set(t.step);
        }
        if link(next) == Link::None {
            return Ok(());
        }
        if !sideways || screens.flow.quiet.get() != 1 {
            crate::sound::play_menu_sound::play_menu_sound_in_game(1);
        }
        match to {
            Some(n) => screens.loop_flags.item.set(n),
            None => crate::menu::table_calls::call_key(item.on_key[way].get(), item)?,
        }
    }
    Ok(())
}
