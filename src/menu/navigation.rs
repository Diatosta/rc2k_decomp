//! Each menu item names its neighbours by number at +0x24, +0x28, +0x2C and +0x30 (which key
//! goes with which is in `Direction`). A link of 0x6F means there is none, and 0xFFFFFC1A
//! means "ask the item": its callback for that direction (+0x34 .. +0x40) is called instead.
//! Every move plays the click, `sound::play_menu_sound` with sound 1. The two vertical moves go
//! on along the same link until they reach an item whose +0x48 is 0xFFFFFC1B, so items that
//! cannot be chosen are stepped over. The two horizontal ones take one step and, while
//! `flow.turning` is 1, first turn a counter: the object in front goes round the objects, and
//! the turn direction and the angle are left at 1 and 0 (right) or -1 and -1 (left). They click
//! only if `flow.quiet` is not 1.
//!
//! The click, the callbacks and the screenshot are called directly.

use crate::errors::Refused;
use crate::menu::item_at::item_at;
use crate::owners::Owners;

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

/// The click is `sound::play_menu_sound` with sound 1; the callback gets the item.
pub fn move_highlight(o: Owners<'static>, direction: Direction) -> Result<(), Refused> {
    let menus = o.menus;
    let screens = &menus.screens;
    // The logic is in `rc2k_core::menu::navigation` (`turn`, `link`, `walk`), everything worked
    // out before it is written.
    use rc2k_core::menu::navigation::{Link, Turn, link, turn, walk};
    let way = direction.slot();
    let sideways = matches!(direction, Direction::Left | Direction::Right);
    {
        let highlighted: u32 = screens.loop_flags.item.get();
        let item = item_at(menus, highlighted)?;
        let turned = (sideways && screens.flow.turning.get() == 1).then(|| {
            let t = Turn {
                count: screens.carousel.count.get() as i32,
                value: screens.carousel.front.get() as i32,
                way: screens.carousel.direction.get(),
                step: screens.carousel.angle.get(),
            };
            turn(t, direction == Direction::Right)
        });
        let next = item.fixed().links[way];
        let to = match link(next) {
            Link::To(n) if !sideways => Some(walk(n, |n| {
                let it = item_at(menus, n).map_err(|e| rc2k_core::Invalid(e.0))?;
                Ok((it.fixed().links[way], it.skip_mark.get()))
            })?),
            Link::To(n) => Some(n),
            _ => None,
        };
        if let Some(t) = turned {
            screens.carousel.front.set(t.value as u32);
            screens.carousel.direction.set(t.way);
            screens.carousel.angle.set(t.step);
        }
        if link(next) == Link::None {
            return Ok(());
        }
        if !sideways || screens.flow.quiet.get() != 1 {
            crate::sound::play_menu_sound::play_menu_sound_in_game(
                o.sound, o.audio, o.menus, o.system, 1,
            );
        }
        match to {
            Some(n) => screens.loop_flags.item.set(n),
            None => crate::menu::table_calls::call_key(o, item.fixed().on_key[way], item)?,
        }
    }
    Ok(())
}
