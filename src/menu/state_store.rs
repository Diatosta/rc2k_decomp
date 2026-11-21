//! The menus' working variables: the carousel, the service area's boxes, the standings' and
//! class figures, the fade, the inspection's figures and the four setup items. Structs of
//! plain fields behind one `RefCell`: the game's thread alone reads and writes them, each borrow
//! lasting one statement.

use crate::memory::session::GameThread;
use std::cell::{RefCell, RefMut};

/// The service area's boxes and the inspection's fade.
#[derive(Clone, Copy)]
pub struct ServiceUi {
    /// The boxes' see-through value (`screens::draw_service_boxes`).
    pub box_colour_a: u32,
    /// The fade's full see-through (`menu::fades`).
    pub fade_full: u32,
    /// The colour of the item's boxes, set from its inspection flags as each item is drawn.
    pub box_colour_b: u32,
    /// 1 when a service box is turned over.
    pub box_shown: bool,
    /// Its page.
    pub box_page: u32,
    /// The result of the last attempt (`service::state`).
    pub status: u32,
}

/// The menus' working variables: the review kind, the class
/// times `menu::calc_cpu_times` makes, the fade, the standings' switches.
#[derive(Clone, Copy)]
pub struct MenuVariables {
    /// The review option screens' kind (`sensiblesaving` in `ral.cfg`).
    pub review_kind: u32,
    /// 1 for "back" in the results' first item, else "continue".
    pub which_title: bool,
    /// 1 = leave the service area's edit variables alone.
    pub skip_edit_values: bool,
    /// The class times (`menu::calc_cpu_times`), by class.
    pub class_times: [u32; 4],
    /// The track entry's length and its hundredfold / 0x96A.
    pub track_length: [u32; 2],
    /// 0 first, 1 when going to the rally's results.
    pub rally_done: bool,
    /// The first row's time, kept for the others' gap to it.
    pub first_row_time: u32,
    /// Eight bytes no code reads or writes.
    pub pair_ab28: [u32; 2],
    /// Bit 0 turned over by `menu::toggle_caption_choice`.
    pub caption_choice: u32,
    /// 1 while the championship's standings show.
    pub standings_shown: bool,
    /// Set to 1 by the repairs, 0 by the service pass.
    pub service_flag: bool,
    /// 2 in event mode 0x13 (`menu::class_championship_pass`).
    pub class_mode: u32,
    /// The fade's length in milliseconds.
    pub fade_length: u32,
    /// The clock when the fade (or a repair phase) started.
    pub fade_start: u32,
    /// The time gone.
    pub fade_gone: u32,
    /// The see-through the fade's quad is drawn with.
    pub fade_see: u32,
}

/// The driver figures and the service area's variables.
#[derive(Clone, Copy)]
pub struct DriverStats {
    /// The steps a flat pair is divided by (`menu::driver_stats`).
    pub steps: f32,
    /// The scale the wear figures use.
    pub scale: f32,
    /// Each part's high end.
    pub highs: [u32; 14],
    /// Each part's limit (`screens::draw_inspection_status` reads the item's from two words
    /// before this array).
    pub limits: [u32; 14],
    /// Each part's share of the total.
    pub weights: [u32; 14],
    /// Each part's penalty (`screens::draw_inspection_status` reads the item's from two words
    /// before this array).
    pub penalties: [u32; 14],
    /// The inspection's score (100 or more passes).
    pub score: u32,
    /// The high ends' total but part 4's.
    pub total: u32,
    /// The money (times 100 into the player's `service_spent`).
    pub money: u32,
    /// Part 4's switch (`menu::driver_stats`).
    pub switch_a: u32,
    /// Part 5's switch.
    pub switch_b: u32,
}

/// The menus' working variables.
#[derive(Clone, Copy)]
pub struct MenuValues {
    /// The service area's boxes and the inspection's fade.
    pub service_ui: ServiceUi,
    /// The menus' working variables: the review kind, the class times, the fade, the
    /// standings' switches.
    pub menus: MenuVariables,
    /// The driver figures and the service area's variables.
    pub driver_stats: DriverStats,
}

/// The menus' working variables.
pub struct MenuState(GameThread<RefCell<MenuValues>>);

impl MenuState {
    pub const fn new() -> Self {
        MenuState(GameThread::new(RefCell::new(MenuValues {
            service_ui: ServiceUi {
                box_colour_a: 0xba,
                fade_full: 0xee,
                box_colour_b: 0,
                box_shown: false,
                box_page: 1,
                status: 0,
            },
            menus: MenuVariables {
                review_kind: 0,
                which_title: false,
                skip_edit_values: false,
                class_times: [0; 4],
                track_length: [0, 0],
                rally_done: false,
                first_row_time: 0,
                pair_ab28: [0, 0],
                caption_choice: 0,
                standings_shown: false,
                service_flag: false,
                class_mode: 2,
                fade_length: 0x1f4,
                fade_start: 0x7d0,
                fade_gone: 0,
                fade_see: 0,
            },
            driver_stats: DriverStats {
                steps: f32::from_bits(0x42c60000),
                scale: f32::from_bits(0x42c80000),
                highs: [0; 14],
                limits: [0; 14],
                weights: [0; 14],
                penalties: [0; 14],
                score: 0x64,
                total: 0,
                money: 0,
                switch_a: 0,
                switch_b: 0,
            },
        })))
    }

    /// A copy of the values: no borrow outlives the statement that reads them.
    pub fn read(&self) -> MenuValues {
        *self.0.get().borrow()
    }

    /// The values, to write.
    pub fn write(&self) -> RefMut<'_, MenuValues> {
        self.0.get().borrow_mut()
    }
}
