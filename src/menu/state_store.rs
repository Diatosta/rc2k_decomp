//! The menus' working variables: the carousel, the service area's boxes, the standings' and
//! class figures, the fade, the inspection's figures and the four setup items. Structs of
//! atomic cells, one static.

use crate::memory::session::{PlainDword, PlainFlag, PlainFloat};

/// The service area's boxes and the inspection's fade.
#[derive(rc2k_derive::Cells)]
pub struct ServiceUi {
    /// The boxes' see-through value (`screens::draw_service_boxes`).
    pub box_colour_a: PlainDword,
    /// The fade's full see-through (`menu::fades`).
    pub fade_full: PlainDword,
    /// The colour of the item's boxes, set from its inspection flags as each item is drawn.
    pub box_colour_b: PlainDword,
    /// 1 when a service box is turned over.
    pub box_shown: PlainFlag,
    /// Its page.
    pub box_page: PlainDword,
    /// The result of the last attempt (`service::state`).
    pub status: PlainDword,
}

/// The menus' working variables: the review kind, the class
/// times `menu::calc_cpu_times` makes, the fade, the standings' switches.
#[derive(rc2k_derive::Cells)]
pub struct MenuVariables {
    /// The review option screens' kind (`sensiblesaving` in `ral.cfg`).
    pub review_kind: PlainDword,
    /// 1 for "voltar" in the results' first item, else "continuar".
    pub which_title: PlainFlag,
    /// 1 = leave the service area's edit variables alone.
    pub skip_edit_values: PlainFlag,
    /// No code reads or writes it (its first value is 0).
    pub dword_75aae4: PlainDword,
    /// The class times (`menu::calc_cpu_times`), by class.
    pub class_times: [PlainDword; 4],
    /// The track entry's +0xC and its hundredfold / 0x96A.
    pub track_length: [PlainDword; 2],
    /// 0 first, 1 when going to the rally's results.
    pub rally_done: PlainFlag,
    /// The first row's time, kept for the others' gap to it.
    pub first_row_time: PlainDword,
    /// Eight bytes no code reads or writes.
    pub pair_ab28: [PlainDword; 2],
    /// Bit 0 turned over by `menu::toggle_caption_choice`.
    pub caption_choice: PlainDword,
    /// 1 while the championship's standings show.
    pub standings_shown: PlainFlag,
    /// Set to 1 by the repairs, 0 by the service pass.
    pub service_flag: PlainFlag,
    /// No code reads or writes it.
    pub unk_75ab3c: PlainDword,
    /// 2 in event mode 0x13 (`menu::class_championship_pass`).
    pub class_mode: PlainDword,
    /// The fade's length in milliseconds.
    pub fade_length: PlainDword,
    /// The clock when the fade (or a repair phase) started.
    pub fade_start: PlainDword,
    /// The time gone.
    pub fade_gone: PlainDword,
    /// The see-through the fade's quad is drawn with.
    pub fade_see: PlainDword,
}

/// The driver figures and the service area's variables.
#[derive(rc2k_derive::Cells)]
pub struct DriverStats {
    /// The steps a flat pair is divided by (`menu::driver_stats`).
    pub steps: PlainFloat,
    /// The scale the wear figures use.
    pub scale: PlainFloat,
    /// Each part's high end.
    pub highs: [PlainDword; 14],
    /// Each part's limit (`screens::draw_inspection_status` reads the item's from two words
    /// before this array).
    pub limits: [PlainDword; 14],
    /// Each part's share of the total.
    pub weights: [PlainDword; 14],
    /// Each part's penalty (`screens::draw_inspection_status` reads the item's from two words
    /// before this array).
    pub penalties: [PlainDword; 14],
    /// The inspection's score (100 or more passes).
    pub score: PlainDword,
    /// The high ends' total but part 4's.
    pub total: PlainDword,
    /// The money (times 100 into the player's +0x288).
    pub money: PlainDword,
    /// Part 4's switch (`menu::driver_stats`).
    pub switch_a: PlainDword,
    /// Part 5's switch.
    pub switch_b: PlainDword,
}

/// The menus' working variables.
#[derive(rc2k_derive::Cells)]
pub struct MenuState {
    /// The service area's boxes and the inspection's fade.
    pub service_ui: ServiceUi,
    /// The menus' working variables: the review kind, the class times, the fade, the
    /// standings' switches.
    pub menus: MenuVariables,
    /// The driver figures and the service area's variables.
    pub driver_stats: DriverStats,
}

impl MenuState {
    pub const fn new() -> Self {
        MenuState {
            service_ui: ServiceUi {
                box_colour_a: PlainDword::new(0xba),
                fade_full: PlainDword::new(0xee),
                box_colour_b: PlainDword::new(0),
                box_shown: PlainFlag::new(false),
                box_page: PlainDword::new(1),
                status: PlainDword::new(0),
            },
            menus: MenuVariables {
                review_kind: PlainDword::new(0),
                which_title: PlainFlag::new(false),
                skip_edit_values: PlainFlag::new(false),
                dword_75aae4: PlainDword::new(0),
                class_times: [const { PlainDword::new(0) }; 4],
                track_length: [PlainDword::new(0), PlainDword::new(0)],
                rally_done: PlainFlag::new(false),
                first_row_time: PlainDword::new(0),
                pair_ab28: [PlainDword::new(0), PlainDword::new(0)],
                caption_choice: PlainDword::new(0),
                standings_shown: PlainFlag::new(false),
                service_flag: PlainFlag::new(false),
                unk_75ab3c: PlainDword::new(0),
                class_mode: PlainDword::new(2),
                fade_length: PlainDword::new(0x1f4),
                fade_start: PlainDword::new(0x7d0),
                fade_gone: PlainDword::new(0),
                fade_see: PlainDword::new(0),
            },
            driver_stats: DriverStats {
                steps: PlainFloat::from_bits(0x42c60000),
                scale: PlainFloat::from_bits(0x42c80000),
                highs: [const { PlainDword::new(0) }; 14],
                limits: [const { PlainDword::new(0) }; 14],
                weights: [const { PlainDword::new(0) }; 14],
                penalties: [const { PlainDword::new(0) }; 14],
                score: PlainDword::new(0x64),
                total: PlainDword::new(0),
                money: PlainDword::new(0),
                switch_a: PlainDword::new(0),
                switch_b: PlainDword::new(0),
            },
        }
    }
}
