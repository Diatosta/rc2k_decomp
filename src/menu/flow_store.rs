//! The screens' flow variables: the sections an event starts at, the yes / no screens' switches,
//! the lists a screen puts in place (the unread, draw, overlay and scene lists) and a few switches
//! of the item keys. One static. The list pointers hold the lists by handle (`menu::fn_list`).

use crate::memory::session::{PlainDword, PlainFlag};
use crate::menu::fn_list::ListCell;

/// The screens' flow variables.
#[derive(rc2k_derive::Cells)]
pub struct ScreenFlow {
    /// The start section: the rally of the Mobil One championship to start at, 0 .. 6 (0: from
    /// the first, with the welcome; n: section n - 1, `rc2k_core::menu::championship`). Chosen
    /// on the options page by `menu::next_colour_a` / `previous_colour_a` (nothing here is a
    /// colour; the pictures are string 0x1D9, the championship's name, then the six rallies'
    /// names, 0x3DC .. 0x3E1). `saves::gather_savegame` keeps it in car 0 at +0x14 and
    /// `load_savegame` puts it back.
    pub start_section: PlainDword,
    /// The start section of the A8 rally championship, 0 .. 6 (the pictures are string 0x1DA,
    /// "Campeonato de Rali A8", then "ronda um" .. "ronda seis", 0x3D6 .. 0x3DB; chosen by
    /// `menu::next_colour_b` / `previous_colour_b`). Kept in car 0 at +0x274 by the saved games.
    pub start_section_a8: PlainDword,
    /// Kept by `menu::run_event_races`.
    pub kept: PlainDword,
    /// 1 on a yes / no screen (`menu::leaving`).
    pub asking: PlainFlag,
    /// 1 = left and right do not click (`menu::navigation`).
    pub quiet: PlainDword,
    /// Enter acts as key 4 (`menu::handle_keys`, `menu::begin_player_name`).
    pub enter_skips_name_entry: PlainDword,
    /// 1 picks the first list of cars (`menu::event_flow`).
    pub car_list_kind: PlainDword,
    /// 0 on the service area's screen, 1 after it (`draw_background_band`).
    pub left_service: PlainDword,
    /// The menu sounds' second volume (`sound::play_menu_sound`).
    pub menu_sounds_volume: PlainDword,
    /// 1 = left and right turn the counter (`menu::navigation`).
    pub turning: PlainDword,
    /// The gearing shown (`screens::gearing_text`).
    pub shown_gearing: PlainDword,
    /// The screen's first list (`menu::fn_list`: by handle, not the list's address).
    pub unread_list: ListCell,
    /// The screen's draw list.
    pub draw_list: ListCell,
    /// The screen's overlay list.
    pub overlay_list: ListCell,
    /// The screen's set-up list: run once at set-up (`menu::run_list`) and not put in place
    /// again.
    pub setup_list: ListCell,
    /// The screen's 3D list.
    pub scene_list: ListCell,
}

impl ScreenFlow {
    pub const fn new() -> Self {
        ScreenFlow {
            start_section: PlainDword::new(0x0),
            start_section_a8: PlainDword::new(0x0),
            kept: PlainDword::new(0x1),
            asking: PlainFlag::new(false),
            quiet: PlainDword::new(0x0),
            enter_skips_name_entry: PlainDword::new(0x0),
            car_list_kind: PlainDword::new(0x0),
            left_service: PlainDword::new(0x1),
            menu_sounds_volume: PlainDword::new(0x0),
            turning: PlainDword::new(0x1),
            shown_gearing: PlainDword::new(0x1),
            unread_list: ListCell::new(),
            draw_list: ListCell::new(),
            overlay_list: ListCell::new(),
            setup_list: ListCell::new(),
            scene_list: ListCell::new(),
        }
    }
}
