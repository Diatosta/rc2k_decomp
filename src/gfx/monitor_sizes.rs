//! The race's screen sizes beyond the game's own table: the monitor's modes, the usual sizes of
//! the common ratios, and a size set by hand. A size of this kind has a number (from 1) kept in
//! the top byte of the screen mode, with the size bits of the mode left 0; the number names a
//! place in this list, which only grows while the program runs, so a mode keeps its meaning.
//! What the options keep is the size itself (`options::model::Display::race_size`).

use rc2k_core::gfx::mode_choices::CUSTOM_SHIFT;
use std::sync::Mutex;

/// The most sizes beyond the table (the top byte's numbers).
const MOST: usize = 255;

struct Sizes {
    /// The sizes numbered so far.
    numbered: Vec<(u32, u32)>,
    /// The monitor's modes and its desktop's size, when asked.
    monitor: Vec<(u32, u32)>,
    desktop: Option<(u32, u32)>,
    /// The size set by hand.
    custom: Option<(u32, u32)>,
    /// The size the player gave the window by resizing it (or the race size the options kept
    /// beyond the game's table: it was the window's last).
    window: Option<(u32, u32)>,
    /// The window's size when the list offers it as an entry of its own (no table, monitor or
    /// usual size is that one), shown as "(window)".
    window_listed: Option<(u32, u32)>,
}

static SIZES: Mutex<Sizes> = Mutex::new(Sizes {
    numbered: Vec::new(),
    monitor: Vec::new(),
    desktop: None,
    custom: None,
    window: None,
    window_listed: None,
});

fn lock() -> std::sync::MutexGuard<'static, Sizes> {
    SIZES
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

/// The number (from 1) of a size, given one if it has none; None when the numbers are used up.
pub fn number_of(width: u32, height: u32) -> Option<u32> {
    let mut sizes = lock();
    if let Some(at) = sizes.numbered.iter().position(|&s| s == (width, height)) {
        return Some(at as u32 + 1);
    }
    if sizes.numbered.len() >= MOST {
        return None;
    }
    sizes.numbered.push((width, height));
    Some(sizes.numbered.len() as u32)
}

/// The size a screen mode names by its number, if it has one.
pub fn size_of_mode(mode: u32) -> Option<(u32, u32)> {
    let number = (mode >> CUSTOM_SHIFT) as usize;
    if number == 0 {
        return None;
    }
    lock().numbered.get(number - 1).copied()
}

/// `mode` (a mode of the table's kind) with the size `(width, height)` in place of its size.
/// `table` holds the table's sizes with their keys (the mode's low byte); a size not there gets
/// a number. None when the numbers are used up.
pub(super) fn mode_for(
    mode: u32,
    table: &[(u32, u32, u32)],
    width: u32,
    height: u32,
) -> Option<u32> {
    let base = mode & 0x00FF_FF00;
    if let Some(&(_, _, key)) = table.iter().find(|&&(w, h, _)| (w, h) == (width, height)) {
        return Some(base | key);
    }
    Some(base | (number_of(width, height)? << CUSTOM_SHIFT))
}

/// A mode as the options file keeps it: a size number we gave cleared (any other mode as it is) and a size key in its place (the
/// 800 x 600 of the game's table, which an old reader understands).
pub fn kept_mode(mode: u32) -> u32 {
    if size_of_mode(mode).is_none() {
        mode
    } else {
        (mode & 0x00FF_FF00) | 2
    }
}

/// The size set by hand (`custom_width` and `custom_height` in the options file), or none.
pub fn set_custom(size: Option<(u32, u32)>) {
    lock().custom = size.filter(|&(w, h)| (320..=16384).contains(&w) && (240..=16384).contains(&h));
}

pub fn custom() -> Option<(u32, u32)> {
    lock().custom
}

/// The size the window was given by the player's resizing, to be offered in the list; none to
/// stop offering one.
pub fn set_window(size: Option<(u32, u32)>) {
    lock().window = size.filter(|&(w, h)| (320..=16384).contains(&w) && (240..=16384).contains(&h));
}

pub fn window() -> Option<(u32, u32)> {
    lock().window
}

/// What the list made last offered as the window's own size (see `window_listed`).
pub(super) fn set_window_listed(size: Option<(u32, u32)>) {
    lock().window_listed = size;
}

/// Whether `(width, height)` is the entry the list offers for the window's size.
pub fn is_window_listed(width: u32, height: u32) -> bool {
    lock().window_listed == Some((width, height))
}

/// The race mode for the size `(width, height)`, from the mode `race` and the table's sizes in
/// `modes` (a size not in the table gets a number); None when the numbers are used up.
pub fn race_mode_for(
    modes: &crate::gfx::modes::Modes,
    race: u32,
    width: u32,
    height: u32,
) -> Option<u32> {
    let table: Vec<(u32, u32, u32)> = (0..8)
        .map(|n| {
            let size = modes.sizes[2 * n + 1].get();
            (size & 0xFFFF, size >> 16, modes.sizes[2 * n].get())
        })
        .collect();
    mode_for(race, &table, width, height)
}

/// The monitor's modes (each size once) and desktop size, as last found.
pub(super) fn set_monitor(modes: Vec<(u32, u32)>, desktop: Option<(u32, u32)>) {
    let mut sizes = lock();
    sizes.monitor = modes;
    sizes.desktop = desktop;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn numbers_are_kept_and_cleared_for_the_file() {
        let table = [(800, 600, 2), (1024, 768, 8)];
        let base = 0x32_1802;
        assert_eq!(mode_for(base, &table, 1024, 768), Some(0x32_1808));
        let m = mode_for(base, &table, 2561, 1081).unwrap();
        assert_eq!(m >> CUSTOM_SHIFT, number_of(2561, 1081).unwrap());
        assert_eq!(m & 0xFF, 0);
        assert_eq!(size_of_mode(m), Some((2561, 1081)));
        assert_eq!(kept_mode(m), 0x32_1802);
        assert_eq!(kept_mode(0x32_1808), 0x32_1808);
        assert_eq!(size_of_mode(0x32_1808), None);
    }
}
