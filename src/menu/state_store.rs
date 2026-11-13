//! The menus' working variables, out of RAL.EXE's image (IDA 0x75A658 .. 0x75ACCC): the carousel, the service area's boxes, the standings' and class figures, the
//! fade, the inspection's figures and the four setup items. `#[repr(C)]` structs of atomic
//! cells, one static.

use crate::menu::lists_store::CarList;
use crate::session::{PlainDword, PlainFlag, PlainFloat};

/// One of the service area's four setup items (gear ratio, tyre, ride height, suspension
/// stiffness; IDA 0x75AC90 on, 12 bytes each).
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct SetupItem {
    /// The value being chosen (`service::step_*_up` / `_down`).
    pub chosen: PlainDword,
    /// The value installed on the car (put back when the change is toggled off).
    pub installed: PlainDword,
    /// The value at the start of the item's edit page (`service::begin_edits`).
    pub start: PlainDword,
}

/// The menu carousel (IDA 0x75A658 .. 0x75AA90): the entry shown, the list, the turn,
/// the place of its entries, their kinds, and the menu's own numbers.
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct Carousel {
    /// `MenuState.carousel.kind`: the kind of the entry being drawn.
    pub kind: PlainDword,
    /// `MenuState.carousel.entries`: the menu's list of cars (the address of the entries' records in the
    /// original), `CarList::code` (0: none); `Carousel::list`.
    pub entries: PlainDword,
    /// `g_carouselDirection`: 1 after right, -1 after left, 0 at rest.
    pub direction: PlainDword,
    /// `MenuState.carousel.spacing`: the x spacing of the places.
    pub spacing: PlainDword,
    /// `MenuState.carousel.y`: the y of the places.
    pub y: PlainDword,
    /// `MenuState.carousel.z`: the z of the places.
    pub z: PlainDword,
    /// `MenuState.carousel.forward`: the way forward (the front entry's step).
    pub forward: PlainDword,
    /// `MenuState.carousel.kinds[0]`: the entries' kinds (room for 0x100: the carousel checks that many).
    pub kinds: [PlainDword; 256],
    /// `g_carouselMenu`: the menu whose carousel is up.
    pub menu: PlainDword,
    /// `g_carouselCount`: the entries.
    pub count: PlainDword,
    /// `g_carouselFront`: the entry in front.
    pub front: PlainDword,
    /// `g_carouselAngle`: the turn's angle (0 after right, -1 after left).
    pub angle: PlainDword,
    /// `MenuState.carousel.last_clock`: the clock at the last `menu::step_carousel`.
    pub last_clock: PlainDword,
    /// `MenuState.carousel.passed`: the time since, held to 100.
    pub passed: PlainDword,
    /// `MenuState.carousel.set_up`: set to 1 by `scene::set_up_carousel` when the carousel's set-up is done; only the carousel set-up lists (scene::carousel_set_ups) name it.
    pub set_up: PlainDword,
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
    /// `MenuState.service_ui.box_colour_a`: the boxes' see-through value (`screens::draw_service_boxes`).
    pub box_colour_a: PlainDword,
    /// `MenuState.service_ui.fade_full`: the fade's full see-through (`menu::fades`).
    pub fade_full: PlainDword,
    /// `MenuState.service_ui.box_colour_b`: the colour of the item's boxes, set from its inspection flags as each item is drawn.
    pub box_colour_b: PlainDword,
    /// `MenuState.service_ui.box_shown`: 1 when a service box is turned over.
    pub box_shown: PlainFlag,
    /// `MenuState.service_ui.box_page`: its page.
    pub box_page: PlainDword,
    /// `MenuState.service_ui.status`: the result of the last attempt (`service::state`).
    pub status: PlainDword,
}

/// The carousel's pictures (IDA 0x75AAA8 .. 0x75AAD8): four wanted, four shown, four held.
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct CarouselPictures {
    /// `MenuState.carousel_pictures.wanted[0]` ..: the pictures wanted (`scene::draw_car_object`).
    pub wanted: [PlainDword; 4],
    /// `MenuState.carousel_pictures.shown[0]` ..: the pictures shown.
    pub shown: [PlainDword; 4],
    /// `MenuState.carousel_pictures.held[0]` ..: the pictures held (`menu::clear_choice_marks` sets them to -1).
    pub held: [PlainDword; 4],
}

/// The menus' working variables (IDA 0x75AAD8 .. 0x75AB58): the review kind, the class
/// times `menu::calc_cpu_times` makes, the fade, the standings' switches.
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct Menus {
    /// `MenuState.menus.review_kind`: the review option screens' kind (`sensiblesaving` in `ral.cfg`).
    pub review_kind: PlainDword,
    /// `MenuState.menus.which_title`: 1 for "voltar" in the results' first item, else "continuar".
    pub which_title: PlainFlag,
    /// `MenuState.menus.skip_edit_values`: 1 = leave the service area's edit variables alone.
    pub skip_edit_values: PlainFlag,
    /// `MenuState.menus.dword_75aae4`: no code reads or writes it (its first value is 0).
    pub dword_75aae4: PlainDword,
    /// `MenuState.menus.class_times[0]`: the class times (`menu::calc_cpu_times`), by class.
    pub class_times: [PlainDword; 4],
    /// `MenuState.menus.track_length[0]`, `MenuState.menus.track_length[1]`: the track entry's +0xC and its hundredfold / 0x96A.
    pub track_length: [PlainDword; 2],
    /// `MenuState.menus.class_counts[0]`: the drivers per class.
    pub class_counts: [PlainDword; 4],
    /// `MenuState.menus.firsts[0]`: the first driver of each group (`saves::default_best_scores`).
    pub firsts: [PlainDword; 4],
    /// `MenuState.menus.rally_done`: 0 first, 1 when going to the rally's results.
    pub rally_done: PlainFlag,
    /// `g_firstRowTime`: the first row's time, kept for the others' gap to it.
    pub first_row_time: PlainDword,
    /// `MenuState.menus.pair_ab28[0]`: eight bytes no code reads or writes.
    pub pair_ab28: [PlainDword; 2],
    /// `MenuState.menus.caption_choice`: bit 0 turned over by `menu::toggle_caption_choice`.
    pub caption_choice: PlainDword,
    /// `MenuState.menus.standings_shown`: 1 while the championship's standings show.
    pub standings_shown: PlainFlag,
    /// `MenuState.menus.service_flag`: set to 1 by the repairs, 0 by the service pass.
    pub service_flag: PlainFlag,
    /// `MenuState.menus.unk_75ab3c`: no code reads or writes it.
    pub unk_75ab3c: PlainDword,
    /// `MenuState.menus.class_mode`: 2 in event mode 0x13 (`menu::class_championship_pass`).
    pub class_mode: PlainDword,
    /// `MenuState.menus.fade_length`: the fade's length in milliseconds.
    pub fade_length: PlainDword,
    /// `MenuState.menus.fade_start`: the clock when the fade (or a repair phase) started.
    pub fade_start: PlainDword,
    /// `MenuState.menus.fade_gone`: the time gone.
    pub fade_gone: PlainDword,
    /// `MenuState.menus.fade_see`: the see-through the fade's quad is drawn with.
    pub fade_see: PlainDword,
    /// `MenuState.menus.repair_last_step`: `elapsed` at the last repair step (`service::apply_repairs`).
    pub repair_last_step: PlainDword,
}

/// The driver figures and the service area's variables (IDA 0x75AB58 .. 0x75ACCC).
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct DriverStats {
    /// `MenuState.driver_stats.steps`: the steps a flat pair is divided by (`menu::driver_stats`).
    pub steps: PlainFloat,
    /// `MenuState.driver_stats.scale`: the scale the wear figures use.
    pub scale: PlainFloat,
    /// `MenuState.driver_stats.repair_times[0]`: the repair times; 14 values for `menu::driver_stats`, 16 in two
    /// columns of eight for `screens::draw_repair_times` (items 12 .. 15 are `begin_edits`' times).
    pub repair_times: [PlainDword; 16],
    /// `MenuState.driver_stats.highs[0]`: each part's high end.
    pub highs: [PlainDword; 14],
    /// `MenuState.driver_stats.limits[0]`: each part's limit (`screens::draw_inspection_status` reads
    /// the item's from 0x75ABD0, two words earlier).
    pub limits: [PlainDword; 14],
    /// `MenuState.driver_stats.weights[0]`: each part's share of the total.
    pub weights: [PlainDword; 14],
    /// `MenuState.driver_stats.penalties[0]`: each part's penalty (`screens::draw_inspection_status` reads the
    /// item's from 0x75AC40, two words earlier).
    pub penalties: [PlainDword; 14],
    /// `MenuState.driver_stats.score`: the inspection's score (100 or more passes).
    pub score: PlainDword,
    /// `MenuState.driver_stats.total`: the high ends' total but part 4's.
    pub total: PlainDword,
    /// `MenuState.driver_stats.repair_time`: the time of the chosen repairs.
    pub repair_time: PlainDword,
    /// `MenuState.driver_stats.setups[0].chosen` ..: the four setup items: [0] gear ratio, [1] tyre (0 .. 5, its name is string
    /// 0x4B0 + it), [2] ride height (a percentage), [3] suspension stiffness (a percentage).
    pub setups: [SetupItem; 4],
    /// `MenuState.driver_stats.money`: the money (times 100 into the player's +0x288).
    pub money: PlainDword,
    /// `MenuState.driver_stats.switch_a`: part 4's switch (`menu::driver_stats`).
    pub switch_a: PlainDword,
    /// `MenuState.driver_stats.switch_b`: part 5's switch.
    pub switch_b: PlainDword,
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
    pub const fn new() -> Self {
        MenuState {
            carousel: Carousel {
                kind: PlainDword::new(0),
                entries: PlainDword::new(0),
                direction: PlainDword::new(0),
                spacing: PlainDword::new(0),
                y: PlainDword::new(0),
                z: PlainDword::new(0),
                forward: PlainDword::new(0),
                kinds: [const { PlainDword::new(0) }; 256],
                menu: PlainDword::new(0),
                count: PlainDword::new(4),
                front: PlainDword::new(3),
                angle: PlainDword::new(0),
                last_clock: PlainDword::new(0),
                passed: PlainDword::new(0),
                set_up: PlainDword::new(1),
            },
            service_ui: ServiceUi {
                box_colour_a: PlainDword::new(0xba),
                fade_full: PlainDword::new(0xee),
                box_colour_b: PlainDword::new(0),
                box_shown: PlainFlag::new(false),
                box_page: PlainDword::new(1),
                status: PlainDword::new(0),
            },
            carousel_pictures: CarouselPictures {
                wanted: [const { PlainDword::new(0) }; 4],
                shown: [const { PlainDword::new(0) }; 4],
                held: [const { PlainDword::new(0) }; 4],
            },
            menus: Menus {
                review_kind: PlainDword::new(0),
                which_title: PlainFlag::new(false),
                skip_edit_values: PlainFlag::new(false),
                dword_75aae4: PlainDword::new(0),
                class_times: [const { PlainDword::new(0) }; 4],
                track_length: [PlainDword::new(0), PlainDword::new(0)],
                class_counts: [
                    PlainDword::new(0x5a),
                    PlainDword::new(0x32),
                    PlainDword::new(0x1e),
                    PlainDword::new(0xa),
                ],
                firsts: [
                    PlainDword::new(0),
                    PlainDword::new(0x5a),
                    PlainDword::new(0x8c),
                    PlainDword::new(0xaa),
                ],
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
                repair_last_step: PlainDword::new(0),
            },
            driver_stats: DriverStats {
                steps: PlainFloat::from_bits(0x42c60000),
                scale: PlainFloat::from_bits(0x42c80000),
                repair_times: [const { PlainDword::new(0) }; 16],
                highs: [const { PlainDword::new(0) }; 14],
                limits: [const { PlainDword::new(0) }; 14],
                weights: [const { PlainDword::new(0) }; 14],
                penalties: [const { PlainDword::new(0) }; 14],
                score: PlainDword::new(0x64),
                total: PlainDword::new(0),
                repair_time: PlainDword::new(0),
                setups: [
                    SetupItem {
                        chosen: PlainDword::new(2),
                        installed: PlainDword::new(2),
                        start: PlainDword::new(0),
                    },
                    SetupItem {
                        chosen: PlainDword::new(3),
                        installed: PlainDword::new(3),
                        start: PlainDword::new(0),
                    },
                    SetupItem {
                        chosen: PlainDword::new(0x32),
                        installed: PlainDword::new(0x32),
                        start: PlainDword::new(0),
                    },
                    SetupItem {
                        chosen: PlainDword::new(0x32),
                        installed: PlainDword::new(0x32),
                        start: PlainDword::new(0),
                    },
                ],
                money: PlainDword::new(0),
                switch_a: PlainDword::new(0),
                switch_b: PlainDword::new(0),
            },
        }
    }
}
