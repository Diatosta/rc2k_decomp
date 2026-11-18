//! The part of the screen the HUD is being drawn into. A split screen of three or four players
//! draws each player's HUD as if the part were a screen of its own: the pieces keep to the part's
//! corners and middle by its shape, and are put where the part is (`hud::add_polygon`).

use rc2k_core::race::view::PlayerPart;
use std::sync::atomic::{AtomicU32, Ordering::Relaxed};

/// x, y, width, height of the part; a width of 0 is no part.
static PART: [AtomicU32; 4] = [const { AtomicU32::new(0) }; 4];

/// The HUD goes into `part` from now on (`None`: the whole screen).
pub fn set(part: Option<PlayerPart>) {
    let [x, y, width, height] = part.map_or([0; 4], |p| [p.x, p.y, p.width, p.height]);
    for (cell, value) in PART.iter().zip([x, y, width, height]) {
        cell.store(value, Relaxed);
    }
}

/// The part the HUD goes into, if one is set.
pub fn get() -> Option<PlayerPart> {
    let [x, y, width, height] = std::array::from_fn(|n| PART[n].load(Relaxed));
    (width != 0 && height != 0).then_some(PlayerPart {
        x,
        y,
        width,
        height,
    })
}

/// The player whose half-size car icon the HUD draws (`None`: the first full-size icon).
static ICON: AtomicU32 = AtomicU32::new(0);

/// The car icon the HUD draws from now on is player `player`'s half-size one.
pub fn set_icon(player: Option<usize>) {
    ICON.store(player.map_or(0, |n| n as u32 + 1), Relaxed);
}

/// The player whose half-size icon is drawn, if one is set.
pub fn icon() -> Option<usize> {
    (ICON.load(Relaxed) as usize).checked_sub(1)
}
