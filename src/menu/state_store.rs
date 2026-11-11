//! The menus' working variables, out of RAL.EXE's image (IDA 0x75A658 .. 0x75ACCC): the carousel, the service area's boxes, the standings' and class figures, the
//! fade, the inspection's figures and the four setup items. `#[repr(C)]` structs of atomic
//! cells with the image's layout, one static; `moved::MOVED` points the range at it.

use crate::menu::lists_store::CarList;
use crate::session::{Bytes, Dword, Flag, Float};

/// One of the service area's four setup items (gear ratio, tyre, ride height, suspension
/// stiffness; IDA 0x75AC90 on, 12 bytes each).
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct SetupItem {
    /// The value being chosen (`service::step_*_up` / `_down`).
    pub chosen: Dword,
    /// The value installed on the car (put back when the change is toggled off).
    pub installed: Dword,
    /// The value at the start of the item's edit page (`service::begin_edits`).
    pub start: Dword,
}

/// The menu carousel (IDA 0x75A658 .. 0x75AA90): the entry shown, the list, the turn,
/// the place of its entries, their kinds, and the menu's own numbers.
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct Carousel {
    /// `dword_75A658`: the kind of the entry being drawn.
    pub kind: Dword,
    /// `dword_75A65C`: the menu's list of cars (the address of the entries' records in the
    /// original), `CarList::code` (0: none); `Carousel::list`.
    pub entries: Dword,
    /// `g_carouselDirection`: 1 after right, -1 after left, 0 at rest.
    pub direction: Dword,
    /// `dword_75A664`: the x spacing of the places.
    pub spacing: Dword,
    /// `dword_75A668`: the y of the places.
    pub y: Dword,
    /// `dword_75A66C`: the z of the places.
    pub z: Dword,
    /// `dword_75A670`: the way forward (the front entry's step).
    pub forward: Dword,
    /// `dword_75A674`: the entries' kinds (room for 0x100: the carousel checks that many).
    pub kinds: [Dword; 256],
    /// `g_carouselMenu`: the menu whose carousel is up.
    pub menu: Dword,
    /// `g_carouselCount`: the entries.
    pub count: Dword,
    /// `g_carouselFront`: the entry in front.
    pub front: Dword,
    /// `g_carouselAngle`: the turn's angle (0 after right, -1 after left).
    pub angle: Dword,
    /// `dword_75AA84`: the clock at the last `menu::step_carousel`.
    pub last_clock: Dword,
    /// `dword_75AA88`: the time since, held to 100.
    pub passed: Dword,
    /// `dword_75AA8C`: set to 1 by `scene::set_up_carousel` when the carousel's set-up is done; only the carousel set-up lists (scene::carousel_set_ups) name it.
    pub set_up: Dword,
}

impl Carousel {
    /// The list the carousel shows, none before one is set up.
    pub fn list(&self) -> Result<Option<CarList>, crate::errors::BadInput> {
        Ok(CarList::from_code(self.entries.get())?)
    }

    pub fn set_list(&self, list: CarList) {
        self.entries.set(list.code());
    }
}

/// The service area's boxes and the inspection's fade (IDA 0x75AA90 .. 0x75AAA8).
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct ServiceUi {
    /// `dword_75AA90`: the boxes' see-through value (`screens::draw_service_boxes`).
    pub box_colour_a: Dword,
    /// `dword_75AA94`: the fade's full see-through (`menu::fades`).
    pub fade_full: Dword,
    /// `dword_75AA98`: the colour of the item's boxes, set from its inspection flags as each item is drawn.
    pub box_colour_b: Dword,
    /// `dword_75AA9C`: 1 when a service box is turned over.
    pub box_shown: Flag,
    /// `dword_75AAA0`: its page.
    pub box_page: Dword,
    /// `dword_75AAA4`: the result of the last attempt (`service::state`).
    pub status: Dword,
}

/// The carousel's pictures (IDA 0x75AAA8 .. 0x75AAD8): four wanted, four shown, four held.
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct CarouselPictures {
    /// `dword_75AAA8` ..: the pictures wanted (`scene::draw_car_object`).
    pub wanted: [Dword; 4],
    /// `dword_75AAB8` ..: the pictures shown.
    pub shown: [Dword; 4],
    /// `dword_75AAC8` ..: the pictures held (`menu::clear_choice_marks` sets them to -1).
    pub held: [Dword; 4],
}

/// The menus' working variables (IDA 0x75AAD8 .. 0x75AB58): the review kind, the class
/// times `menu::calc_cpu_times` makes, the fade, the standings' switches.
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct Menus {
    /// `dword_75AAD8`: the review option screens' kind (`sensiblesaving` in `ral.cfg`).
    pub review_kind: Dword,
    /// `dword_75AADC`: 1 for "voltar" in the results' first item, else "continuar".
    pub which_title: Flag,
    /// `dword_75AAE0`: 1 = leave the service area's edit variables alone.
    pub skip_edit_values: Flag,
    /// `dword_75AAE4`: no code reads or writes it (its first value is 0).
    pub dword_75aae4: Dword,
    /// `dword_75AAE8`: the class times (`menu::calc_cpu_times`), by class.
    pub class_times: [Dword; 4],
    /// `dword_75AAF8`, `dword_75AAFC`: the track entry's +0xC and its hundredfold / 0x96A.
    pub track_length: [Dword; 2],
    /// `dword_75AB00`: the drivers per class.
    pub class_counts: [Dword; 4],
    /// `dword_75AB10`: the first driver of each group (`saves::default_best_scores`).
    pub firsts: [Dword; 4],
    /// `dword_75AB20`: 0 first, 1 when going to the rally's results.
    pub rally_done: Flag,
    /// `g_firstRowTime`: the first row's time, kept for the others' gap to it.
    pub first_row_time: Dword,
    /// `unk_75AB28`: eight bytes no code reads or writes.
    pub pair_ab28: [Dword; 2],
    /// `dword_75AB30`: bit 0 turned over by `menu::toggle_caption_choice`.
    pub caption_choice: Dword,
    /// `dword_75AB34`: 1 while the championship's standings show.
    pub standings_shown: Flag,
    /// `dword_75AB38`: set to 1 by the repairs, 0 by the service pass.
    pub service_flag: Flag,
    /// `unk_75AB3C`: no code reads or writes it.
    pub unk_75ab3c: Dword,
    /// `dword_75AB40`: 2 in event mode 0x13 (`menu::class_championship_pass`).
    pub class_mode: Dword,
    /// `dword_75AB44`: the fade's length in milliseconds.
    pub fade_length: Dword,
    /// `dword_75AB48`: the clock when the fade (or a repair phase) started.
    pub fade_start: Dword,
    /// `dword_75AB4C`: the time gone.
    pub fade_gone: Dword,
    /// `dword_75AB50`: the see-through the fade's quad is drawn with.
    pub fade_see: Dword,
    /// `dword_75AB54`: `elapsed` at the last repair step (`service::apply_repairs`).
    pub repair_last_step: Dword,
}

/// The driver figures and the service area's variables (IDA 0x75AB58 .. 0x75ACCC).
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct DriverStats {
    /// `flt_75AB58`: the steps a flat pair is divided by (`menu::driver_stats`).
    pub steps: Float,
    /// `flt_75AB5C`: the scale the wear figures use.
    pub scale: Float,
    /// `dword_75AB60`: the repair times; 14 values for `menu::driver_stats`, 16 in two
    /// columns of eight for `screens::draw_repair_times` (items 12 .. 15 are `begin_edits`' times).
    pub repair_times: [Dword; 16],
    /// `dword_75ABA0`: each part's high end.
    pub highs: [Dword; 14],
    /// `dword_75ABD8`: each part's limit (`screens::draw_inspection_status` reads
    /// the item's from 0x75ABD0, two words earlier).
    pub limits: [Dword; 14],
    /// `dword_75AC10`: each part's share of the total.
    pub weights: [Dword; 14],
    /// `dword_75AC48`: each part's penalty (`screens::draw_inspection_status` reads the
    /// item's from 0x75AC40, two words earlier).
    pub penalties: [Dword; 14],
    /// `dword_75AC80`: the inspection's score (100 or more passes).
    pub score: Dword,
    /// `dword_75AC84`: the high ends' total but part 4's.
    pub total: Dword,
    /// Unreferenced bytes (IDA 0X75AC88 .. 0X75AC8C).
    pub _gap_75ac88: Bytes<4>,
    /// `dword_75AC8C`: the time of the chosen repairs.
    pub repair_time: Dword,
    /// `dword_75AC90` ..: the four setup items: [0] gear ratio, [1] tyre (0 .. 5, its name is string
    /// 0x4B0 + it), [2] ride height (a percentage), [3] suspension stiffness (a percentage).
    pub setups: [SetupItem; 4],
    /// `dword_75ACC0`: the money (times 100 into the player's +0x288).
    pub money: Dword,
    /// `dword_75ACC4`: part 4's switch (`menu::driver_stats`).
    pub switch_a: Dword,
    /// `dword_75ACC8`: part 5's switch.
    pub switch_b: Dword,
}

/// The menus' working variables.
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct MenuState {
    /// The menu carousel (IDA 0x75A658 .. 0x75AA90): the entry shown, the list, the turn,
    pub carousel: Carousel,
    /// The service area's boxes and the inspection's fade (IDA 0x75AA90 .. 0x75AAA8).
    pub service_ui: ServiceUi,
    /// The carousel's pictures (IDA 0x75AAA8 .. 0x75AAD8): four wanted, four shown, four held.
    pub carousel_pictures: CarouselPictures,
    /// The menus' working variables (IDA 0x75AAD8 .. 0x75AB58): the review kind, the class
    pub menus: Menus,
    /// The driver figures and the service area's variables (IDA 0x75AB58 .. 0x75ACCC).
    pub driver_stats: DriverStats,
}

impl MenuState {
    /// First values the image's.
    pub const fn new() -> Self {
        MenuState {
            carousel: Carousel {
                kind: Dword::new(0),
                entries: Dword::new(0),
                direction: Dword::new(0),
                spacing: Dword::new(0),
                y: Dword::new(0),
                z: Dword::new(0),
                forward: Dword::new(0),
                kinds: [const { Dword::new(0) }; 256],
                menu: Dword::new(0),
                count: Dword::new(4),
                front: Dword::new(3),
                angle: Dword::new(0),
                last_clock: Dword::new(0),
                passed: Dword::new(0),
                set_up: Dword::new(1),
            },
            service_ui: ServiceUi {
                box_colour_a: Dword::new(0xba),
                fade_full: Dword::new(0xee),
                box_colour_b: Dword::new(0),
                box_shown: Flag::new(false),
                box_page: Dword::new(1),
                status: Dword::new(0),
            },
            carousel_pictures: CarouselPictures {
                wanted: [const { Dword::new(0) }; 4],
                shown: [const { Dword::new(0) }; 4],
                held: [const { Dword::new(0) }; 4],
            },
            menus: Menus {
                review_kind: Dword::new(0),
                which_title: Flag::new(false),
                skip_edit_values: Flag::new(false),
                dword_75aae4: Dword::new(0),
                class_times: [const { Dword::new(0) }; 4],
                track_length: [Dword::new(0), Dword::new(0)],
                class_counts: [
                    Dword::new(0x5a),
                    Dword::new(0x32),
                    Dword::new(0x1e),
                    Dword::new(0xa),
                ],
                firsts: [
                    Dword::new(0),
                    Dword::new(0x5a),
                    Dword::new(0x8c),
                    Dword::new(0xaa),
                ],
                rally_done: Flag::new(false),
                first_row_time: Dword::new(0),
                pair_ab28: [Dword::new(0), Dword::new(0)],
                caption_choice: Dword::new(0),
                standings_shown: Flag::new(false),
                service_flag: Flag::new(false),
                unk_75ab3c: Dword::new(0),
                class_mode: Dword::new(2),
                fade_length: Dword::new(0x1f4),
                fade_start: Dword::new(0x7d0),
                fade_gone: Dword::new(0),
                fade_see: Dword::new(0),
                repair_last_step: Dword::new(0),
            },
            driver_stats: DriverStats {
                steps: Float::from_bits(0x42c60000),
                scale: Float::from_bits(0x42c80000),
                repair_times: [const { Dword::new(0) }; 16],
                highs: [const { Dword::new(0) }; 14],
                limits: [const { Dword::new(0) }; 14],
                weights: [const { Dword::new(0) }; 14],
                penalties: [const { Dword::new(0) }; 14],
                score: Dword::new(0x64),
                total: Dword::new(0),
                _gap_75ac88: Bytes::new([0; 4]),
                repair_time: Dword::new(0),
                setups: [
                    SetupItem {
                        chosen: Dword::new(2),
                        installed: Dword::new(2),
                        start: Dword::new(0),
                    },
                    SetupItem {
                        chosen: Dword::new(3),
                        installed: Dword::new(3),
                        start: Dword::new(0),
                    },
                    SetupItem {
                        chosen: Dword::new(0x32),
                        installed: Dword::new(0x32),
                        start: Dword::new(0),
                    },
                    SetupItem {
                        chosen: Dword::new(0x32),
                        installed: Dword::new(0x32),
                        start: Dword::new(0),
                    },
                ],
                money: Dword::new(0),
                switch_a: Dword::new(0),
                switch_b: Dword::new(0),
            },
        }
    }
}

/// IDA 0X75A658, the first byte of the piece.
pub const START: usize = 0x75A658;
/// IDA 0X75ACCC, the end of the piece.
pub const END: usize = 0x75ACCC;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::moved::image_bytes;
    use core::mem::{offset_of, size_of};

    #[test]
    fn menu_state_starts_as_the_games() {
        let size = size_of::<MenuState>();
        assert!(END - START <= size && size <= (END - START).next_multiple_of(4));
        let mine = unsafe {
            std::slice::from_raw_parts(
                (&raw const crate::menu::owner::MENUS.screens.state).cast::<u8>(),
                END - START,
            )
        };
        assert_eq!(mine, &image_bytes(START, END - START)[..]);
        let at = |field: usize, ida: usize| assert_eq!(field, ida - START, "{ida:#X}");
        at(offset_of!(MenuState, carousel), 0x75A658);
        at(offset_of!(MenuState, service_ui), 0x75AA90);
        at(offset_of!(MenuState, carousel_pictures), 0x75AAA8);
        at(offset_of!(MenuState, menus), 0x75AAD8);
        at(offset_of!(MenuState, driver_stats), 0x75AB58);
    }
}
