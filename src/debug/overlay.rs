//! The debug overlay's pages (`debug::draw_pages`, 0x4297C5, IDA 2026-10-30): `g_debugPage`
//! 0 .. 7; 1 "debug keys active", 2 the display (wear bars, profile), 3 and up the kstar
//! modes, 4 .. 7 the dumps (`notes/reference.md`). The game's code that set the page and
//! `g_debugFreezeRace` is not in RAL.EXE (`input::end_frame` sets both 0 every frame):
//! here two keys of our own, with Ctrl held (hidden from the game then), kept across frames and put back by
//! `input::end_frame` (`words`): F11 the next page, F12 the race frozen (`drive::race_step`
//! skips its physics ticks). Each page draws its title and a line in the debug font (top
//! right: the page, frozen); page 2 its wear bars (`wear_page`); the rest piece by piece.

use crate::errors::Refused;
use crate::input::store::DEVICES;
use crate::physics::tables_store::PHYSICS;
use crate::race::numbers_store::NUMBERS;
use crate::race::text_state_store::TEXT_STATE;
use crate::text::draw_text::{Canvas, draw_text_at};
use core::sync::atomic::{AtomicU32, Ordering};

/// The keys (scan codes in `g_keysNow` / `g_keysPressed`): Ctrl, F11, F12.
const CONTROL_KEY: usize = 0x1D;
const NEXT_PAGE_KEY: usize = 0x57;
const FREEZE_KEY: usize = 0x58;

/// The pages.
pub const PAGES: u32 = 8;

/// `g_debugPageTitles` (0x5FFF12): each page's title.
pub const TITLES: [&[u8]; PAGES as usize] = [
    b" ",
    b"debug keys active",
    b"debug keys and display",
    b"debug keys and kstar modes",
    b"debug keys and dumps",
    b"debug keys and dumps",
    b"debug keys and dumps",
    b"debug keys and dumps",
];

/// The page shown, and 1 while the race is frozen.
static PAGE: AtomicU32 = AtomicU32::new(0);
static FROZEN: AtomicU32 = AtomicU32::new(0);

/// The keys, once a frame (`input::update`, before the game reads the frame's keys): with
/// Ctrl down, F11 pressed the next page (after 7, none), F12 pressed the race frozen or going
/// again; and both taken out of `g_keysDown` (the bindings read it), `g_keysNow` and
/// `g_keysPressed`, so that the game does not see them (F12 is its map).
pub fn take_keys() {
    if DEVICES.keys_now.get(CONTROL_KEY) == 0 {
        return;
    }
    let pressed = |key| DEVICES.keys_pressed.get(key) & 1 != 0;
    let (next, freeze) = (pressed(NEXT_PAGE_KEY), pressed(FREEZE_KEY));
    for key in [NEXT_PAGE_KEY, FREEZE_KEY] {
        DEVICES.keys_down.set(key, 0);
        DEVICES.keys_now.set(key, 0);
        DEVICES.keys_pressed.set(key, 0);
    }
    let pressed = |key| if key == NEXT_PAGE_KEY { next } else { freeze };
    if pressed(NEXT_PAGE_KEY) {
        let page = (PAGE.load(Ordering::Relaxed) + 1) % PAGES;
        PAGE.store(page, Ordering::Relaxed);
        crate::framework::log(&format!("debug: page {page}"));
    }
    if pressed(FREEZE_KEY) {
        let frozen = FROZEN.load(Ordering::Relaxed) ^ 1;
        FROZEN.store(frozen, Ordering::Relaxed);
        crate::framework::log(&format!("debug: race frozen {frozen}"));
    }
}

/// What `input::end_frame` leaves in `g_debugPage` and `g_debugFreezeRace`: the overlay's
/// (both 0, the original's, until a key is used).
pub fn words() -> [u32; 2] {
    [PAGE.load(Ordering::Relaxed), FROZEN.load(Ordering::Relaxed)]
}

/// `debug::draw_pages` (0x4297C5), so far: on page 2 the wear bars (`wear_page`), the
/// page's title (`draw_page_title`) and the overlay's own line.
pub fn draw_pages(canvas: &mut dyn Canvas) -> Result<(), Refused> {
    let page = PHYSICS.debug[0].get();
    let frozen = PHYSICS.debug[1].get() != 0;
    if page == 0 && !frozen {
        return Ok(());
    }
    TEXT_STATE.text_mode.set(2);
    NUMBERS.debug_scale.set(2);
    let line = format!("page {page}{}\0", if frozen { " frozen" } else { "" });
    // Top right, clear of the pages' items.
    crate::debug::draw::print(0x2A0, 4, line.as_bytes())?;
    if page == 2 {
        crate::debug::wear_page::draw_wear_bars()?;
    }
    draw_page_title(page, canvas)
}

/// `debug::draw_page_title` (0x429824): but on page 0, its title (`g_debugPageTitles`) at
/// (0x190, 0x14) in the font at 0x51DE90 (built in the text buffer).
fn draw_page_title(page: u32, canvas: &mut dyn Canvas) -> Result<(), Refused> {
    let Some(&title) = TITLES.get(page as usize).filter(|_| page != 0) else {
        return Ok(());
    };
    let t = crate::strings::text_from_ids::text_at();
    t.write_bytes(&[title, b"\0"].concat(), "a debug page's title")?;
    draw_text_at(
        crate::pointers::GamePtr::to(&crate::gfx::fonts_store::FONTS.menu_fonts[3]),
        t,
        0x190,
        0x14,
        canvas,
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_titles_are_the_images() {
        let image = crate::race::store::first_values::<{ 0x5FFFCC - 0x5FFF32 }>(0x5FFF32);
        let texts: Vec<&[u8]> = image.split(|&b| b == 0).take(8).collect();
        assert_eq!(texts, TITLES);
    }
}
