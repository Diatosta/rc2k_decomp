//! The screens' flow variables, out of RAL.EXE's image (IDA 0x7554C0 .. 0x755525; phase 7, typed): the
//! sections an event starts at, the yes / no screens' switches, the lists a screen puts in place
//! (`g_unreadList`, `g_drawList`, `g_overlayList`, `g_sceneList`) and a few switches of the item
//! keys. A `#[repr(C)]` struct with the image's layout, one static; `moved::MOVED` points the range
//! at it. The list pointers hold the lists by handle (`menu::fn_list`, 2026-10-28). (Split 2026-10-25
//! from the front end's block `menu::store`.)

use crate::menu::fn_list::ListCell;
use crate::session::{Bytes, Dword, with_runs};

/// The screens' flow variables (IDA 0x7554C0 .. 0x755525).
#[repr(C)]
pub struct ScreenFlow {
    /// `g_startSection`: The section an event starts at (`menu::run_championship`); the same variable is the first car colour `menu::next_colour_a` chooses, which `saves::gather_savegame` puts in car 0 at +0x14.
    pub start_section: Dword,
    /// `g_startSectionA8`: The A8 game's start section; also the second car colour (`menu::next_colour_b`), car 0 at +0x274.
    pub start_section_a8: Dword,
    /// `dword_7554C8`: Kept by `menu::run_event_races`.
    pub kept: Dword,
    /// `g_askingYesNo`: 1 on a yes / no screen (`menu::leaving`).
    pub asking: Dword,
    /// `dword_7554D0`: 1 = left and right do not click (`menu::navigation`).
    pub quiet: Dword,
    /// `g_enterSkipsNameEntry`: Enter acts as key 4 (`menu::handle_keys`, `menu::begin_player_name`).
    pub enter_skips_name_entry: Dword,
    /// `dword_7554D8`: 1 picks the first list of cars (`menu::event_flow`).
    pub car_list_kind: Dword,
    /// Unreferenced bytes (IDA 0x7554DC .. 0x7554E4).
    pub _gap_7554dc: Bytes<8>,
    /// `dword_7554E4`: 0 on the service area's screen, 1 after it (`draw_background_band`).
    pub left_service: Dword,
    /// `dword_7554E8`: The menu sounds' second volume (`sound::play_menu_sound`).
    pub menu_sounds_volume: Dword,
    /// `dword_7554EC`: 1 = left and right turn the counter (`menu::navigation`).
    pub turning: Dword,
    /// `g_shownGearing`: The gearing shown (`screens::gearing_text`).
    pub shown_gearing: Dword,
    /// `g_unreadList`: The screen's first list (`menu::fn_list`: by handle, not the list's address).
    pub unread_list: ListCell,
    /// `g_drawList`: The screen's draw list.
    pub draw_list: ListCell,
    /// `g_overlayList`: The screen's overlay list.
    pub overlay_list: ListCell,
    /// `dword_755500`: The screen's set-up list: run once at set-up (`menu::run_list`) and not put in place again.
    pub setup_list: ListCell,
    /// `g_sceneList`: The screen's 3D list.
    pub scene_list: ListCell,
    /// Unreferenced bytes (IDA 0x755508 .. 0x755525).
    pub _gap_755508: Bytes<29>,
}

/// First values the image's (`tests::screen_flow_starts_as_the_games`).
pub static SCREEN_FLOW: ScreenFlow = ScreenFlow {
    start_section: Dword::new(0x0),
    start_section_a8: Dword::new(0x0),
    kept: Dword::new(0x1),
    asking: Dword::new(0x0),
    quiet: Dword::new(0x0),
    enter_skips_name_entry: Dword::new(0x0),
    car_list_kind: Dword::new(0x0),
    _gap_7554dc: Bytes::new([0; 8]),
    left_service: Dword::new(0x1),
    menu_sounds_volume: Dword::new(0x0),
    turning: Dword::new(0x1),
    shown_gearing: Dword::new(0x1),
    unread_list: ListCell::new(),
    draw_list: ListCell::new(),
    overlay_list: ListCell::new(),
    setup_list: ListCell::new(),
    scene_list: ListCell::new(),
    _gap_755508: Bytes::new(with_runs::<29>(&[(0x0, &[0xff, 0xff, 0xff, 0xff])])),
};

/// IDA 0x7554C0, the first byte of the piece.
pub const START: usize = 0x7554C0;
/// IDA 0x755525, the end of the piece.
pub const END: usize = 0x755525;

/// Where IDA 0x7554C0 is now (`moved::MOVED`'s home).
pub fn screen_flow_addr() -> u32 {
    crate::pointers::game_address(&raw const SCREEN_FLOW)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::moved::image_bytes;
    use core::mem::{offset_of, size_of};

    #[test]
    fn screen_flow_starts_as_the_games() {
        assert!(
            END - START <= size_of::<ScreenFlow>()
                && size_of::<ScreenFlow>() <= (END - START).next_multiple_of(4)
        );
        let mine = unsafe {
            std::slice::from_raw_parts(
                crate::pointers::host(screen_flow_addr()).cast_const(),
                END - START,
            )
        };
        assert_eq!(mine, &image_bytes(START, END - START)[..]);
        let at = |field: usize, ida: usize| assert_eq!(field, ida - START, "{ida:#X}");
        at(offset_of!(ScreenFlow, start_section), 0x7554C0);
        at(offset_of!(ScreenFlow, start_section_a8), 0x7554C4);
        at(offset_of!(ScreenFlow, kept), 0x7554C8);
        at(offset_of!(ScreenFlow, asking), 0x7554CC);
        at(offset_of!(ScreenFlow, quiet), 0x7554D0);
        at(offset_of!(ScreenFlow, enter_skips_name_entry), 0x7554D4);
        at(offset_of!(ScreenFlow, car_list_kind), 0x7554D8);
        at(offset_of!(ScreenFlow, left_service), 0x7554E4);
        at(offset_of!(ScreenFlow, menu_sounds_volume), 0x7554E8);
        at(offset_of!(ScreenFlow, turning), 0x7554EC);
        at(offset_of!(ScreenFlow, shown_gearing), 0x7554F0);
        at(offset_of!(ScreenFlow, unread_list), 0x7554F4);
        at(offset_of!(ScreenFlow, draw_list), 0x7554F8);
        at(offset_of!(ScreenFlow, overlay_list), 0x7554FC);
        at(offset_of!(ScreenFlow, setup_list), 0x755500);
        at(offset_of!(ScreenFlow, scene_list), 0x755504);
    }
}
