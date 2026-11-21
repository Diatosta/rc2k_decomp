//! The screens' flow variables: the sections an event starts at, the yes / no screens' switches,
//! the lists a screen puts in place (the unread, draw, overlay and scene lists) and a few switches
//! of the item keys. One static. The list pointers hold the lists by handle (`menu::fn_list`).

use crate::memory::session::GameThread;
use crate::menu::fn_list::ListCell;
use std::cell::{RefCell, RefMut};

/// The screens' flow variables. Plain values behind one `RefCell`: the game's thread alone reads
/// and writes them, each borrow lasting one statement. The lists are handles of their own.
pub struct ScreenFlow {
    values: GameThread<RefCell<FlowValues>>,
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

/// The screens' flow values.
#[derive(Clone, Copy)]
pub struct FlowValues {
    /// The start section: the rally of the Mobil One championship to start at, 0 .. 6 (0: from
    /// the first, with the welcome; n: section n - 1, `rc2k_core::menu::championship`). Chosen
    /// on the options page by `menu::next_colour_a` / `previous_colour_a` (nothing here is a
    /// colour; the pictures are string 0x1D9, the championship's name, then the six rallies'
    /// names, 0x3DC .. 0x3E1). `saves::gather_savegame` keeps it in player 0's `saved_start_section` and
    /// `load_savegame` puts it back.
    pub start_section: u32,
    /// The start section of the A8 rally championship, 0 .. 6 (the pictures are string 0x1DA,
    /// "A8 Rally Championship", then "round one" .. "round six", 0x3D6 .. 0x3DB; chosen by
    /// `menu::next_colour_b` / `previous_colour_b`). Kept in player 0's `saved_start_section_a8` by the saved games.
    pub start_section_a8: u32,
    /// Kept by `menu::run_event_races`.
    pub kept: u32,
    /// True on a yes / no screen (`menu::leaving`).
    pub asking: bool,
    /// 1 = left and right do not click (`menu::navigation`).
    pub quiet: u32,
    /// 1 picks the first list of cars (`menu::event_flow`).
    pub car_list_kind: u32,
    /// 0 on the service area's screen, 1 after it (`draw_background_band`).
    pub left_service: u32,
    /// The menu sounds' second volume (`sound::play_menu_sound`).
    pub menu_sounds_volume: u32,
    /// 1 = left and right turn the counter (`menu::navigation`).
    pub turning: u32,
    /// The gearing shown (`screens::gearing_text`).
    pub shown_gearing: u32,
}

impl ScreenFlow {
    pub const fn new() -> Self {
        ScreenFlow {
            values: GameThread::new(RefCell::new(FlowValues {
                start_section: 0x0,
                start_section_a8: 0x0,
                kept: 0x1,
                asking: false,
                quiet: 0x0,
                car_list_kind: 0x0,
                left_service: 0x1,
                menu_sounds_volume: 0x0,
                turning: 0x1,
                shown_gearing: 0x1,
            })),
            unread_list: ListCell::new(),
            draw_list: ListCell::new(),
            overlay_list: ListCell::new(),
            setup_list: ListCell::new(),
            scene_list: ListCell::new(),
        }
    }

    /// A copy of the values: no borrow outlives the statement that reads them.
    pub fn read(&self) -> FlowValues {
        *self.values.get().borrow()
    }

    /// The values, to write.
    pub fn write(&self) -> RefMut<'_, FlowValues> {
        self.values.get().borrow_mut()
    }
}
