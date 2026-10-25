//! `screens::draw_graphics_settings` (0x46FDF0): the graphics page: item 2 choosable only
//! when `dword_4F50D4` + `dword_4F50D8` is 2; the settings' names (strings by the work
//! values, each kept in `dword_7559EC`) and the screen mode "w x h x bits" (`unk_4F50E8`
//! [`dword_4F50C8`], built in the shared text buffer) at x 0x1D1 from y 0xDC every 0x1B, in
//! the font at 0x51E8C4.
//!
//! In:  nothing
//! Out: as the last `draw_string_by_id` leaves them (carry, eax 0, ebx, ecx, edx)
//!
//! In a menu loop's draw list (0x758CB2). Refuses what `menu::enable_item` refuses (no current
//! menu), before anything. The menus and draw lists run its `entry` (`menu::table_calls`); the
//! registers above are the original's, for reference.

use crate::errors::Refused;
use crate::framework::{game_read, rva};
use crate::gfx::choices_store::SCREEN_CHOICES;
use crate::menu::disable_item::disable_item;
use crate::menu::enable_item::enable_item;
use crate::menu::option_work_store::OPTION_WORK;
use crate::text::draw_text::Canvas;

fn k(a: u32) -> u32 {
    unsafe { game_read(rva(a as usize)) }
}

/// (the setting, its first string, its row) for the names; row 1 is the screen mode.
type Name = (fn() -> u32, u32, i32);

const NAMES: [Name; 8] = [
    (|| k(0x4F50C4), 0x38F, 0),
    (|| k(0x4F50CC), 0x391, 2),
    (|| k(0x4F50D0), 0x56D, 3),
    (|| OPTION_WORK.setting_0.get(), 0x56D, 4),
    (|| OPTION_WORK.setting_1.get(), 0x56D, 5),
    (|| OPTION_WORK.setting_2.get(), 0x396, 6),
    (|| OPTION_WORK.setting_3.get(), 0x393, 7),
    (|| OPTION_WORK.setting_4.get(), 0x394, 8),
];

/// The Rust version (the registers the last name leaves: the menus read none of them).
pub unsafe fn draw_graphics_settings(canvas: &mut dyn Canvas) -> Result<(), Refused> {
    // Converted: `rc2k_core::screens::settings_pages::{graphics_item, graphics}`.
    use crate::screens::game_screen::SettingsScreen;
    unsafe {
        if rc2k_core::screens::settings_pages::graphics_item(
            SCREEN_CHOICES.page_a.get(),
            SCREEN_CHOICES.page_b.get(),
        ) {
            enable_item(2)?;
        } else {
            disable_item(2)?;
        }
        let mode = SCREEN_CHOICES.list.addr() as u32 + 0x10 * SCREEN_CHOICES.mode.get();
        let v = |i: u32| ((mode + 4 * i) as *const u32).read_unaligned();
        let ids = NAMES.map(|(setting, first, _)| setting().wrapping_add(first));
        rc2k_core::screens::settings_pages::graphics(
            &mut SettingsScreen::new(canvas),
            ids,
            [v(0), v(1), v(2)],
        )?;
        Ok(())
    }
}
