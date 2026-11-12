//! The menus' lists and tables, out of RAL.EXE's image (IDA 0x7594AA .. 0x75A658): the last draw list, the carousel kinds' own places, the paths and sounds the menus
//! name, the car lists the menus walk, the best scores' titles. `#[repr(C)]` struct of atomic
//! cells with the image's layout, one static. Its
//! last field is a 22-byte string: the struct is padded to a multiple of 4.

use crate::audio::names_store::NameList;
use crate::menu::cars_store::car_cell;
use crate::menu::owner::Menus;
use crate::menu::owner::Tables;
use crate::menu::texts_store::MenuTexts;
use crate::session::{PlainBytes, PlainDword};
use core::mem::offset_of;
pub use rc2k_core::menu::car_lists::CarList;

/// A gap's first bytes: `runs` are (offset, bytes) of the non-zero ones.
const fn sparse<const N: usize>(runs: &[(usize, &[u8])]) -> [u8; N] {
    let mut out = [0u8; N];
    let mut r = 0;
    while r < runs.len() {
        let (at, bytes) = runs[r];
        let mut i = 0;
        while i < bytes.len() {
            out[at + i] = bytes[i];
            i += 1;
        }
        r += 1;
    }
    out
}

/// One kind's own place in a menu scene (`scene::draw_carousel_entry`: the
/// kind's size, its y and its angle).
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct OwnPlace {
    /// Its size.
    pub size: PlainDword,
    /// Its y.
    pub y: PlainDword,
    /// Its angle about x.
    pub angle: PlainDword,
}

/// The menus' lists and tables.
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct MenuLists {
    /// `dword_7594AA`: the menus' last draw list (25 entries, -1 at start;
    /// `menu::player4_buttons_loop`'s).
    pub draw_list: [PlainDword; 25],
    /// `MenuLists.own_places[0].size`: the carousel kinds' own places, 12 bytes each, indexed by a kind's
    /// number (`scene::draw_carousel_entry` checks it against 0x88 records, a slip: the table
    /// holds 73 and the lists that follow answer the rest).
    pub own_places: [OwnPlace; 73],
    /// Unreferenced bytes (IDA 0X75987A .. 0X75987E).
    pub _gap_75987a: PlainBytes<4>,
    /// `off_75987E`: the pictures' paths the stage view's welcome shows, a championship's.
    pub welcome_championship: [PlainDword; 6],
    /// `off_759896`: the same for the other modes (the A8 game).
    pub welcome_other: [PlainDword; 6],
    /// `off_7598AE`: the map's overlays, by the stage's +0x3C (`menu::stage_choice`).
    pub map_overlays: [PlainDword; 36],
    /// Unreferenced bytes (IDA 0X75993E .. 0X759942).
    pub _gap_75993e: PlainBytes<4>,
    /// `off_759942`: the first of the four lists of paths a car page cycles through
    /// (`menu::load_car_background`), ending in a gap dword.
    pub page_pictures_a: [PlainDword; 25],
    /// `off_7599A6`: the second of the four lists.
    pub page_pictures_b: [PlainDword; 25],
    /// `off_759A0A`: the third of the four lists.
    pub page_pictures_c: [PlainDword; 25],
    /// `off_759A6E`: the fourth of the four lists.
    pub page_pictures_d: [PlainDword; 25],
    /// `off_759AD2`: the addresses of the four lists above; read by nothing now (`page_pictures`).
    pub page_picture_lists: [PlainDword; 4],
    /// Unreferenced bytes (IDA 0X759AE2 .. 0X759AE6).
    pub _gap_759ae2: PlainBytes<4>,
    /// `funcs_47CED1`: four code addresses (0x469C40 ..) of the car pages. Not used by the
    /// standalone.
    pub page_functions: [PlainDword; 4],
    /// Unreferenced bytes (IDA 0X759AF6 .. 0X759AFA).
    pub _gap_759af6: PlainBytes<4>,
    /// `dword_759AFA`: the car pages' title strings, ended by -1 (`screens::car_page_title`).
    pub car_page_titles: [PlainDword; 5],
    /// `off_759B0E`: the slideshow pictures' path templates (`menu::load_slideshow_picture`).
    pub slideshow_templates: [PlainDword; 24],
    /// `dword_759B6E`: the service boxes' flags, the first set, by item value.
    pub inspection_flags_a: [PlainDword; 21],
    /// `dword_759BC2`: the service boxes' flags, the second set, by item value.
    pub inspection_flags_b: [PlainDword; 21],
    /// `unk_759C16`: pointers to each repair item's time, by item (`service::state`).
    pub repair_time_ptrs: [PlainDword; 24],
    /// `dword_759C76`: the inspection's fault flags, 13 items.
    pub inspection_faults: [PlainDword; 14],
    /// `g_championshipCars`: the championship's car list (`menu::event_flow`).
    pub cars_championship: [PlainDword; 26],
    /// `g_a8Cars`: the A8 rally championship's car list (45 entries ended by -1; `menu::remove_secret_cars` edits it with the first list).
    pub a8_championship_cars: [PlainDword; 45],
    /// `off_759DCA`: the car list without the Clio, the first.
    pub cars_first: [PlainDword; 34],
    /// `off_759E52`: the car list with the Clio and the extra cars.
    pub cars_with_clio: [PlainDword; 62],
    /// `g_secretCars`: the cars won.
    pub secret_cars: [PlainDword; 12],
    /// `unk_759F7A`: the scratch list `menu::remove_secret_cars` builds.
    pub cars_scratch: [PlainDword; 50],
    /// `off_75A042`: the second car list (arcade events).
    pub cars_second: [PlainDword; 59],
    /// `off_75A12E`: the list of the cars shown (`menu::arcade_steps`).
    pub shown_cars: [PlainDword; 48],
    /// Unreferenced bytes (IDA 0X75A1EE .. 0X75A1FE).
    pub _gap_75a1ee: PlainBytes<16>,
    /// `dword_75A1FE`: a kind by car number (`scene::set_up_carousel`).
    pub car_kinds: [PlainDword; 35],
    /// `dword_75A28A`: the best scores' section title string ids, by section.
    pub section_titles: [PlainDword; 7],
    /// `dword_75A2A6`: the best scores' page title string ids, by page.
    pub page_titles_a: [PlainDword; 12],
    /// `dword_75A2D6`: the best scores' second page title string ids, by page.
    pub page_titles_b: [PlainDword; 11],
    /// `off_75A302`: the rally speeches' sounds, by section (`menu::stage_view_steps`).
    pub rally_speeches: [PlainDword; 6],
    /// Unreferenced bytes (IDA 0X75A31A .. 0X75A31E).
    pub _gap_75a31a: PlainBytes<4>,
    /// `off_75A31E`: the car streams' file names (`sound::play_car_stream`).
    pub car_streams: [PlainDword; 24],
    /// `off_75A37E`: the stage speeches' sounds (`menu::stage_view_steps`).
    pub stage_speeches: [PlainDword; 36],
    /// Unreferenced bytes (IDA 0X75A40E .. 0X75A412).
    pub _gap_75a40e: PlainBytes<4>,
    /// `off_75A412`: the cheat car list (`menu::cheat_car_list`).
    pub cheat_cars: [PlainDword; 22],
    /// Unreferenced bytes (IDA 0X75A46A .. 0X75A49A).
    pub _gap_75a46a: PlainBytes<48>,
    /// `dword_75A49A`: the stage panel's durations in minutes, by entry +8
    /// (`screens::stage_panel`; the table runs on over unreferenced bytes).
    pub durations: [PlainDword; 37],
    /// `g_servicePictures`: the service pictures' paths (`service::load_background`).
    pub service_pictures: [PlainDword; 60],
    /// Unreferenced bytes (IDA 0X75A61E .. 0X75A622).
    pub _gap_75a61e: PlainBytes<4>,
    /// `dword_75A622`: the eight texture numbers of the carousel's menu (-1 none).
    pub carousel_textures: [PlainDword; 8],
    /// `aFilesGFrntBack`: a path string, ends at 0x75A658 (and two bytes to the record's
    /// size, which every byte of a record of cells is part of).
    pub front_back_path: PlainBytes<24>,
}

/// First values the image's.
/// A text of `texts_store` by its place there (by handle; the original kept its
/// address): the byte offset of the field that holds it.
const fn text_ref(offset: usize) -> PlainDword {
    PlainDword::new(offset as u32)
}

/// The place of car `n`'s row in the speech samples' names table (three dwords a car).
const fn car_speech_row(n: usize) -> usize {
    assert!(n < 24);
    offset_of!(MenuTexts, car_speech) + 12 * n
}

/// The place of stage `n`'s row in the speech samples' names table (four dwords a stage).
const fn stage_speech_row(n: usize) -> usize {
    assert!(n < 36);
    offset_of!(MenuTexts, stage_speech) + 16 * n
}

/// The `n`th rally speech name (33 bytes each) among the settings page's speech names, from
/// the one 0xB4 bytes into them, as `audio::store::name_key` has it (a name's key).
const fn name_ref(n: usize) -> PlainDword {
    assert!(n < 6);
    PlainDword::new(NameList::MenuSpeech.first_key() + 0xB4 + 33 * n as u32)
}

/// A repair time's number + 1 in `MENU_STATE`'s repair times (the original kept its address;
/// 0: none; -1 the end mark).
const fn repair_key(ida: u32) -> u32 {
    match ida {
        0 | 0xFFFF_FFFF => ida,
        _ => {
            assert!(REPAIR_TIMES_IDA <= ida && ida < REPAIR_TIMES_IDA + 16 * 4);
            (ida - REPAIR_TIMES_IDA) / 4 + 1
        }
    }
}

/// IDA's address of `MENU_STATE`'s repair times.
const REPAIR_TIMES_IDA: u32 = 0x75AB60;

const fn repair_ref(ida: u32) -> PlainDword {
    PlainDword::new(repair_key(ida))
}

/// The car pages' pictures by page (the original's `page_picture_lists`, in its order; a car
/// past a list reads on into the next, as there).
pub fn page_pictures(
    tables: &Tables,
    page: u32,
) -> Result<&'_ [PlainDword], crate::errors::BadInput> {
    Ok(match page {
        0 => &tables.lists.page_pictures_a,
        1 => &tables.lists.page_pictures_c,
        2 => &tables.lists.page_pictures_d,
        3 => &tables.lists.page_pictures_b,
        _ => return Err(crate::errors::BadInput("a car page past the four")),
    })
}

/// An item's repair time cell (`repair_time_ptrs`; None: none).
pub fn repair_time_at(
    menus: &Menus,
    key: u32,
) -> Result<Option<&PlainDword>, crate::errors::BadInput> {
    match key {
        0 => Ok(None),
        _ => menus
            .screens
            .state
            .driver_stats
            .repair_times
            .get(key.wrapping_sub(1) as usize)
            .map(Some)
            .ok_or(crate::errors::BadInput(
                "a repair time's handle off the times",
            )),
    }
}

impl MenuLists {
    /// A list of cars, from its first entry to the end of these lists: the original reads
    /// a list on to its end mark, and a list of one shown twice (menu 0xE) is read on past
    /// its field into the next.
    pub fn car_entry(&self, list: CarList, n: u32) -> Result<u32, crate::errors::BadInput> {
        crate::cells::load_value(self, self.car_entry_offset(list, n))
    }

    /// Entry `n` of a list of cars, written.
    pub fn set_car_entry(
        &self,
        list: CarList,
        n: u32,
        value: u32,
    ) -> Result<(), crate::errors::BadInput> {
        crate::cells::store_value(self, self.car_entry_offset(list, n), value)
    }

    /// Where entry `n` of a list of cars is, in these lists.
    fn car_entry_offset(&self, list: CarList, n: u32) -> usize {
        use std::mem::offset_of;
        let at = match list {
            CarList::First => offset_of!(MenuLists, cars_first),
            CarList::Second => offset_of!(MenuLists, cars_second),
            CarList::WithClio => offset_of!(MenuLists, cars_with_clio),
            CarList::Championship => offset_of!(MenuLists, cars_championship),
            CarList::A8 => offset_of!(MenuLists, a8_championship_cars),
            CarList::Cheat => offset_of!(MenuLists, cheat_cars),
            CarList::Shown => offset_of!(MenuLists, shown_cars),
        };
        at.wrapping_add((n as usize).wrapping_mul(4))
    }
}

impl MenuLists {
    pub const fn new() -> Self {
        MenuLists {
            draw_list: [const { PlainDword::new(0xffffffff) }; 25],
            own_places: [
                OwnPlace {
                    size: PlainDword::new(0x10),
                    y: PlainDword::new(0),
                    angle: PlainDword::new(0),
                },
                OwnPlace {
                    size: PlainDword::new(0x10),
                    y: PlainDword::new(0),
                    angle: PlainDword::new(0),
                },
                OwnPlace {
                    size: PlainDword::new(0x10),
                    y: PlainDword::new(0),
                    angle: PlainDword::new(0),
                },
                OwnPlace {
                    size: PlainDword::new(0x10),
                    y: PlainDword::new(0),
                    angle: PlainDword::new(0),
                },
                OwnPlace {
                    size: PlainDword::new(0x10),
                    y: PlainDword::new(0),
                    angle: PlainDword::new(0),
                },
                OwnPlace {
                    size: PlainDword::new(0x10),
                    y: PlainDword::new(0),
                    angle: PlainDword::new(0),
                },
                OwnPlace {
                    size: PlainDword::new(0xd),
                    y: PlainDword::new(0),
                    angle: PlainDword::new(0),
                },
                OwnPlace {
                    size: PlainDword::new(0xd),
                    y: PlainDword::new(0),
                    angle: PlainDword::new(0),
                },
                OwnPlace {
                    size: PlainDword::new(0x10),
                    y: PlainDword::new(0),
                    angle: PlainDword::new(0),
                },
                OwnPlace {
                    size: PlainDword::new(0x14),
                    y: PlainDword::new(0),
                    angle: PlainDword::new(0),
                },
                OwnPlace {
                    size: PlainDword::new(0xe),
                    y: PlainDword::new(0),
                    angle: PlainDword::new(0),
                },
                OwnPlace {
                    size: PlainDword::new(0x11),
                    y: PlainDword::new(0),
                    angle: PlainDword::new(0),
                },
                OwnPlace {
                    size: PlainDword::new(0x11),
                    y: PlainDword::new(0),
                    angle: PlainDword::new(0x5ff),
                },
                OwnPlace {
                    size: PlainDword::new(0xe),
                    y: PlainDword::new(0),
                    angle: PlainDword::new(0),
                },
                OwnPlace {
                    size: PlainDword::new(0x13),
                    y: PlainDword::new(0),
                    angle: PlainDword::new(0),
                },
                OwnPlace {
                    size: PlainDword::new(0x13),
                    y: PlainDword::new(0),
                    angle: PlainDword::new(0),
                },
                OwnPlace {
                    size: PlainDword::new(0x10),
                    y: PlainDword::new(0),
                    angle: PlainDword::new(0),
                },
                OwnPlace {
                    size: PlainDword::new(0x12),
                    y: PlainDword::new(0),
                    angle: PlainDword::new(0xcf0),
                },
                OwnPlace {
                    size: PlainDword::new(0x10),
                    y: PlainDword::new(0),
                    angle: PlainDword::new(0),
                },
                OwnPlace {
                    size: PlainDword::new(0xe),
                    y: PlainDword::new(0),
                    angle: PlainDword::new(0),
                },
                OwnPlace {
                    size: PlainDword::new(0x12),
                    y: PlainDword::new(0),
                    angle: PlainDword::new(0),
                },
                OwnPlace {
                    size: PlainDword::new(0x12),
                    y: PlainDword::new(0),
                    angle: PlainDword::new(0),
                },
                OwnPlace {
                    size: PlainDword::new(0x11),
                    y: PlainDword::new(0),
                    angle: PlainDword::new(0),
                },
                OwnPlace {
                    size: PlainDword::new(0x10),
                    y: PlainDword::new(0),
                    angle: PlainDword::new(0x3ff),
                },
                OwnPlace {
                    size: PlainDword::new(0x10),
                    y: PlainDword::new(0),
                    angle: PlainDword::new(0x3ff),
                },
                OwnPlace {
                    size: PlainDword::new(0x10),
                    y: PlainDword::new(0),
                    angle: PlainDword::new(0x3ff),
                },
                OwnPlace {
                    size: PlainDword::new(0x10),
                    y: PlainDword::new(0),
                    angle: PlainDword::new(0x3ff),
                },
                OwnPlace {
                    size: PlainDword::new(0x10),
                    y: PlainDword::new(0),
                    angle: PlainDword::new(0),
                },
                OwnPlace {
                    size: PlainDword::new(0xe),
                    y: PlainDword::new(0),
                    angle: PlainDword::new(0),
                },
                OwnPlace {
                    size: PlainDword::new(0x10),
                    y: PlainDword::new(0),
                    angle: PlainDword::new(0),
                },
                OwnPlace {
                    size: PlainDword::new(0x12),
                    y: PlainDword::new(0),
                    angle: PlainDword::new(0),
                },
                OwnPlace {
                    size: PlainDword::new(0x12),
                    y: PlainDword::new(0),
                    angle: PlainDword::new(0),
                },
                OwnPlace {
                    size: PlainDword::new(0x10),
                    y: PlainDword::new(0),
                    angle: PlainDword::new(0),
                },
                OwnPlace {
                    size: PlainDword::new(0xe),
                    y: PlainDword::new(0),
                    angle: PlainDword::new(0),
                },
                OwnPlace {
                    size: PlainDword::new(0x12),
                    y: PlainDword::new(0),
                    angle: PlainDword::new(0xcf0),
                },
                OwnPlace {
                    size: PlainDword::new(0xc),
                    y: PlainDword::new(0),
                    angle: PlainDword::new(0),
                },
                OwnPlace {
                    size: PlainDword::new(0x12),
                    y: PlainDword::new(0),
                    angle: PlainDword::new(0),
                },
                OwnPlace {
                    size: PlainDword::new(0x12),
                    y: PlainDword::new(0),
                    angle: PlainDword::new(0),
                },
                OwnPlace {
                    size: PlainDword::new(0x12),
                    y: PlainDword::new(0),
                    angle: PlainDword::new(0),
                },
                OwnPlace {
                    size: PlainDword::new(0xe),
                    y: PlainDword::new(0),
                    angle: PlainDword::new(0xcf0),
                },
                OwnPlace {
                    size: PlainDword::new(5),
                    y: PlainDword::new(0),
                    angle: PlainDword::new(0xcf0),
                },
                OwnPlace {
                    size: PlainDword::new(5),
                    y: PlainDword::new(0),
                    angle: PlainDword::new(0xcf0),
                },
                OwnPlace {
                    size: PlainDword::new(5),
                    y: PlainDword::new(0),
                    angle: PlainDword::new(0xcf0),
                },
                OwnPlace {
                    size: PlainDword::new(5),
                    y: PlainDword::new(0),
                    angle: PlainDword::new(0xcf0),
                },
                OwnPlace {
                    size: PlainDword::new(5),
                    y: PlainDword::new(0),
                    angle: PlainDword::new(0xcf0),
                },
                OwnPlace {
                    size: PlainDword::new(5),
                    y: PlainDword::new(0),
                    angle: PlainDword::new(0xcf0),
                },
                OwnPlace {
                    size: PlainDword::new(5),
                    y: PlainDword::new(0),
                    angle: PlainDword::new(0xcf0),
                },
                OwnPlace {
                    size: PlainDword::new(5),
                    y: PlainDword::new(0),
                    angle: PlainDword::new(0xcf0),
                },
                OwnPlace {
                    size: PlainDword::new(5),
                    y: PlainDword::new(0),
                    angle: PlainDword::new(0xcf0),
                },
                OwnPlace {
                    size: PlainDword::new(5),
                    y: PlainDword::new(0),
                    angle: PlainDword::new(0xcf0),
                },
                OwnPlace {
                    size: PlainDword::new(5),
                    y: PlainDword::new(0),
                    angle: PlainDword::new(0xcf0),
                },
                OwnPlace {
                    size: PlainDword::new(5),
                    y: PlainDword::new(0),
                    angle: PlainDword::new(0xcf0),
                },
                OwnPlace {
                    size: PlainDword::new(5),
                    y: PlainDword::new(0),
                    angle: PlainDword::new(0xcf0),
                },
                OwnPlace {
                    size: PlainDword::new(5),
                    y: PlainDword::new(0),
                    angle: PlainDword::new(0xcf0),
                },
                OwnPlace {
                    size: PlainDword::new(5),
                    y: PlainDword::new(0),
                    angle: PlainDword::new(0xcf0),
                },
                OwnPlace {
                    size: PlainDword::new(5),
                    y: PlainDword::new(0),
                    angle: PlainDword::new(0xcf0),
                },
                OwnPlace {
                    size: PlainDword::new(5),
                    y: PlainDword::new(0),
                    angle: PlainDword::new(0xcf0),
                },
                OwnPlace {
                    size: PlainDword::new(5),
                    y: PlainDword::new(0),
                    angle: PlainDword::new(0xcf0),
                },
                OwnPlace {
                    size: PlainDword::new(5),
                    y: PlainDword::new(0),
                    angle: PlainDword::new(0xcf0),
                },
                OwnPlace {
                    size: PlainDword::new(5),
                    y: PlainDword::new(0),
                    angle: PlainDword::new(0xcf0),
                },
                OwnPlace {
                    size: PlainDword::new(5),
                    y: PlainDword::new(0),
                    angle: PlainDword::new(0xcf0),
                },
                OwnPlace {
                    size: PlainDword::new(5),
                    y: PlainDword::new(0),
                    angle: PlainDword::new(0xcf0),
                },
                OwnPlace {
                    size: PlainDword::new(5),
                    y: PlainDword::new(0),
                    angle: PlainDword::new(0xcf0),
                },
                OwnPlace {
                    size: PlainDword::new(5),
                    y: PlainDword::new(0),
                    angle: PlainDword::new(0xcf0),
                },
                OwnPlace {
                    size: PlainDword::new(5),
                    y: PlainDword::new(0),
                    angle: PlainDword::new(0xcf0),
                },
                OwnPlace {
                    size: PlainDword::new(5),
                    y: PlainDword::new(0),
                    angle: PlainDword::new(0xcf0),
                },
                OwnPlace {
                    size: PlainDword::new(5),
                    y: PlainDword::new(0),
                    angle: PlainDword::new(0xcf0),
                },
                OwnPlace {
                    size: PlainDword::new(5),
                    y: PlainDword::new(0),
                    angle: PlainDword::new(0xcf0),
                },
                OwnPlace {
                    size: PlainDword::new(5),
                    y: PlainDword::new(0),
                    angle: PlainDword::new(0xcf0),
                },
                OwnPlace {
                    size: PlainDword::new(5),
                    y: PlainDword::new(0),
                    angle: PlainDword::new(0xcf0),
                },
                OwnPlace {
                    size: PlainDword::new(5),
                    y: PlainDword::new(0),
                    angle: PlainDword::new(0xcf0),
                },
                OwnPlace {
                    size: PlainDword::new(5),
                    y: PlainDword::new(0),
                    angle: PlainDword::new(0xcf0),
                },
                OwnPlace {
                    size: PlainDword::new(5),
                    y: PlainDword::new(0),
                    angle: PlainDword::new(0xcf0),
                },
            ],
            _gap_75987a: PlainBytes::new([0xff; 4]),
            welcome_championship: [
                text_ref(offset_of!(MenuTexts, welvx_png)),
                text_ref(offset_of!(MenuTexts, welpir_png)),
                text_ref(offset_of!(MenuTexts, welsct_png)),
                text_ref(offset_of!(MenuTexts, weljc_png)),
                text_ref(offset_of!(MenuTexts, welul_png)),
                text_ref(offset_of!(MenuTexts, welmnx_png)),
            ],
            welcome_other: [
                text_ref(offset_of!(MenuTexts, a8_1_png)),
                text_ref(offset_of!(MenuTexts, a8_2_png)),
                text_ref(offset_of!(MenuTexts, a8_3_png)),
                text_ref(offset_of!(MenuTexts, a8_4_png)),
                text_ref(offset_of!(MenuTexts, a8_5_png)),
                text_ref(offset_of!(MenuTexts, a8_6_png)),
            ],
            map_overlays: [
                text_ref(offset_of!(MenuTexts, vrw_001_png)),
                text_ref(offset_of!(MenuTexts, vrw_002_png)),
                text_ref(offset_of!(MenuTexts, vrw_003_png)),
                text_ref(offset_of!(MenuTexts, vrw_004_png)),
                text_ref(offset_of!(MenuTexts, vrw_005_png)),
                text_ref(offset_of!(MenuTexts, vrw_006_png)),
                text_ref(offset_of!(MenuTexts, pir_001_png)),
                text_ref(offset_of!(MenuTexts, pir_002_png)),
                text_ref(offset_of!(MenuTexts, pir_003_png)),
                text_ref(offset_of!(MenuTexts, pir_004_png)),
                text_ref(offset_of!(MenuTexts, pir_005_png)),
                text_ref(offset_of!(MenuTexts, pir_006_png)),
                text_ref(offset_of!(MenuTexts, sc_001_png)),
                text_ref(offset_of!(MenuTexts, sc_002_png)),
                text_ref(offset_of!(MenuTexts, sc_003_png)),
                text_ref(offset_of!(MenuTexts, sc_004_png)),
                text_ref(offset_of!(MenuTexts, sc_005_png)),
                text_ref(offset_of!(MenuTexts, sc_006_png)),
                text_ref(offset_of!(MenuTexts, jc_001_png)),
                text_ref(offset_of!(MenuTexts, jc_002_png)),
                text_ref(offset_of!(MenuTexts, jc_003_png)),
                text_ref(offset_of!(MenuTexts, jc_004_png)),
                text_ref(offset_of!(MenuTexts, jc_005_png)),
                text_ref(offset_of!(MenuTexts, jc_006_png)),
                text_ref(offset_of!(MenuTexts, ul_001_png)),
                text_ref(offset_of!(MenuTexts, ul_002_png)),
                text_ref(offset_of!(MenuTexts, ul_003_png)),
                text_ref(offset_of!(MenuTexts, ul_004_png)),
                text_ref(offset_of!(MenuTexts, ul_005_png)),
                text_ref(offset_of!(MenuTexts, ul_006_png)),
                text_ref(offset_of!(MenuTexts, mnx_001_png)),
                text_ref(offset_of!(MenuTexts, mnx_002_png)),
                text_ref(offset_of!(MenuTexts, mnx_003_png)),
                text_ref(offset_of!(MenuTexts, mnx_004_png)),
                text_ref(offset_of!(MenuTexts, mnx_005_png)),
                text_ref(offset_of!(MenuTexts, mnx_006_png)),
            ],
            _gap_75993e: PlainBytes::new([0xff; 4]),
            page_pictures_a: [
                text_ref(offset_of!(MenuTexts, mgnvc_png)),
                text_ref(offset_of!(MenuTexts, vxavc_png)),
                text_ref(offset_of!(MenuTexts, stcvc_png)),
                text_ref(offset_of!(MenuTexts, hycvc_png)),
                text_ref(offset_of!(MenuTexts, sbivc_png)),
                text_ref(offset_of!(MenuTexts, pgovc_png)),
                text_ref(offset_of!(MenuTexts, mxpvc_png)),
                text_ref(offset_of!(MenuTexts, mtlvc_png)),
                text_ref(offset_of!(MenuTexts, vwgvc_png)),
                text_ref(offset_of!(MenuTexts, nsavc_png)),
                text_ref(offset_of!(MenuTexts, ptwvc_png)),
                text_ref(offset_of!(MenuTexts, ctsvc_png)),
                text_ref(offset_of!(MenuTexts, hncvc_png)),
                text_ref(offset_of!(MenuTexts, nsmvc_png)),
                text_ref(offset_of!(MenuTexts, pgtvc_png)),
                text_ref(offset_of!(MenuTexts, frtvc_png)),
                text_ref(offset_of!(MenuTexts, sdovc_png)),
                text_ref(offset_of!(MenuTexts, stivc_png)),
                text_ref(offset_of!(MenuTexts, sdfvc_png)),
                text_ref(offset_of!(MenuTexts, ptcvc_png)),
                text_ref(offset_of!(MenuTexts, fmxvc_png)),
                text_ref(offset_of!(MenuTexts, fpmvc_png)),
                text_ref(offset_of!(MenuTexts, oscvc_png)),
                text_ref(offset_of!(MenuTexts, rclvc_png)),
                PlainDword::new(0xffffffff),
            ],
            page_pictures_b: [
                text_ref(offset_of!(MenuTexts, mgndm_png)),
                text_ref(offset_of!(MenuTexts, vxadm_png)),
                text_ref(offset_of!(MenuTexts, stcdm_png)),
                text_ref(offset_of!(MenuTexts, hycdm_png)),
                text_ref(offset_of!(MenuTexts, sbidm_png)),
                text_ref(offset_of!(MenuTexts, pgodm_png)),
                text_ref(offset_of!(MenuTexts, mxpdm_png)),
                text_ref(offset_of!(MenuTexts, mtldm_png)),
                text_ref(offset_of!(MenuTexts, vwgdm_png)),
                text_ref(offset_of!(MenuTexts, nsadm_png)),
                text_ref(offset_of!(MenuTexts, ptwdm_png)),
                text_ref(offset_of!(MenuTexts, ctsdm_png)),
                text_ref(offset_of!(MenuTexts, hncdm_png)),
                text_ref(offset_of!(MenuTexts, nsmdm_png)),
                text_ref(offset_of!(MenuTexts, pgtdm_png)),
                text_ref(offset_of!(MenuTexts, frtdm_png)),
                text_ref(offset_of!(MenuTexts, sdodm_png)),
                text_ref(offset_of!(MenuTexts, stidm_png)),
                text_ref(offset_of!(MenuTexts, sdfdm_png)),
                text_ref(offset_of!(MenuTexts, ptcdm_png)),
                text_ref(offset_of!(MenuTexts, fmxdm_png)),
                text_ref(offset_of!(MenuTexts, fpmdm_png)),
                text_ref(offset_of!(MenuTexts, oscdm_png)),
                text_ref(offset_of!(MenuTexts, rcldm_png)),
                PlainDword::new(0xffffffff),
            ],
            page_pictures_c: [
                text_ref(offset_of!(MenuTexts, mgnen_png)),
                text_ref(offset_of!(MenuTexts, vxaen_png)),
                text_ref(offset_of!(MenuTexts, stcen_png)),
                text_ref(offset_of!(MenuTexts, hycen_png)),
                text_ref(offset_of!(MenuTexts, sbien_png)),
                text_ref(offset_of!(MenuTexts, pgoen_png)),
                text_ref(offset_of!(MenuTexts, mxpen_png)),
                text_ref(offset_of!(MenuTexts, mtlen_png)),
                text_ref(offset_of!(MenuTexts, vwgen_png)),
                text_ref(offset_of!(MenuTexts, nsaen_png)),
                text_ref(offset_of!(MenuTexts, ptwen_png)),
                text_ref(offset_of!(MenuTexts, ctsen_png)),
                text_ref(offset_of!(MenuTexts, hncen_png)),
                text_ref(offset_of!(MenuTexts, nsmen_png)),
                text_ref(offset_of!(MenuTexts, pgten_png)),
                text_ref(offset_of!(MenuTexts, frten_png)),
                text_ref(offset_of!(MenuTexts, sdoen_png)),
                text_ref(offset_of!(MenuTexts, stien_png)),
                text_ref(offset_of!(MenuTexts, sdfen_png)),
                text_ref(offset_of!(MenuTexts, ptcen_png)),
                text_ref(offset_of!(MenuTexts, fmxen_png)),
                text_ref(offset_of!(MenuTexts, fpmen_png)),
                text_ref(offset_of!(MenuTexts, oscen_png)),
                text_ref(offset_of!(MenuTexts, rclen_png)),
                PlainDword::new(0xffffffff),
            ],
            page_pictures_d: [
                text_ref(offset_of!(MenuTexts, mgntc_png)),
                text_ref(offset_of!(MenuTexts, vxatc_png)),
                text_ref(offset_of!(MenuTexts, stctc_png)),
                text_ref(offset_of!(MenuTexts, hyctc_png)),
                text_ref(offset_of!(MenuTexts, sbitc_png)),
                text_ref(offset_of!(MenuTexts, pgotc_png)),
                text_ref(offset_of!(MenuTexts, mxptc_png)),
                text_ref(offset_of!(MenuTexts, mtltc_png)),
                text_ref(offset_of!(MenuTexts, vwgtc_png)),
                text_ref(offset_of!(MenuTexts, nsatc_png)),
                text_ref(offset_of!(MenuTexts, ptwtc_png)),
                text_ref(offset_of!(MenuTexts, ctstc_png)),
                text_ref(offset_of!(MenuTexts, hnctc_png)),
                text_ref(offset_of!(MenuTexts, nsmtc_png)),
                text_ref(offset_of!(MenuTexts, pgttc_png)),
                text_ref(offset_of!(MenuTexts, frttc_png)),
                text_ref(offset_of!(MenuTexts, sdotc_png)),
                text_ref(offset_of!(MenuTexts, stitc_png)),
                text_ref(offset_of!(MenuTexts, sdftc_png)),
                text_ref(offset_of!(MenuTexts, ptctc_png)),
                text_ref(offset_of!(MenuTexts, fmxtc_png)),
                text_ref(offset_of!(MenuTexts, fpmtc_png)),
                text_ref(offset_of!(MenuTexts, osctc_png)),
                text_ref(offset_of!(MenuTexts, rcltc_png)),
                PlainDword::new(0xffffffff),
            ],
            page_picture_lists: [
                PlainDword::new(0x759942),
                PlainDword::new(0x759a0a),
                PlainDword::new(0x759a6e),
                PlainDword::new(0x7599a6),
            ],
            _gap_759ae2: PlainBytes::new([0xff; 4]),
            page_functions: [
                PlainDword::new(0x469c40),
                PlainDword::new(0x469f12),
                PlainDword::new(0x46a193),
                PlainDword::new(0x469ca0),
            ],
            _gap_759af6: PlainBytes::new([0xff; 4]),
            car_page_titles: [
                PlainDword::new(0x24c),
                PlainDword::new(0x249),
                PlainDword::new(0x24b),
                PlainDword::new(0x24a),
                PlainDword::new(0xffffffff),
            ],
            slideshow_templates: [
                text_ref(offset_of!(MenuTexts, mgnsl00_png)),
                text_ref(offset_of!(MenuTexts, vxasl00_png)),
                text_ref(offset_of!(MenuTexts, stcsl00_png)),
                text_ref(offset_of!(MenuTexts, hycsl00_png)),
                text_ref(offset_of!(MenuTexts, sbisl00_png)),
                text_ref(offset_of!(MenuTexts, pgosl00_png)),
                text_ref(offset_of!(MenuTexts, mxpsl00_png)),
                text_ref(offset_of!(MenuTexts, mtlsl00_png)),
                text_ref(offset_of!(MenuTexts, vwgsl00_png)),
                text_ref(offset_of!(MenuTexts, nsasl00_png)),
                text_ref(offset_of!(MenuTexts, ptwsl00_png)),
                text_ref(offset_of!(MenuTexts, ctssl00_png)),
                text_ref(offset_of!(MenuTexts, hncsl00_png)),
                text_ref(offset_of!(MenuTexts, nsmsl00_png)),
                text_ref(offset_of!(MenuTexts, pgtsl00_png)),
                text_ref(offset_of!(MenuTexts, frtsl00_png)),
                text_ref(offset_of!(MenuTexts, sdosl00_png)),
                text_ref(offset_of!(MenuTexts, stisl00_png)),
                text_ref(offset_of!(MenuTexts, sdfsl00_png)),
                text_ref(offset_of!(MenuTexts, ptcsl00_png)),
                text_ref(offset_of!(MenuTexts, fmxsl00_png)),
                text_ref(offset_of!(MenuTexts, fpmsl00_png)),
                text_ref(offset_of!(MenuTexts, oscsl00_png)),
                text_ref(offset_of!(MenuTexts, rclsl00_png)),
            ],
            inspection_flags_a: [
                PlainDword::new(0),
                PlainDword::new(0),
                PlainDword::new(0),
                PlainDword::new(0),
                PlainDword::new(0),
                PlainDword::new(0),
                PlainDword::new(0),
                PlainDword::new(0),
                PlainDword::new(0),
                PlainDword::new(0),
                PlainDword::new(0),
                PlainDword::new(0),
                PlainDword::new(0),
                PlainDword::new(0),
                PlainDword::new(0),
                PlainDword::new(0),
                PlainDword::new(0),
                PlainDword::new(0),
                PlainDword::new(0),
                PlainDword::new(0),
                PlainDword::new(0xffffffff),
            ],
            inspection_flags_b: [
                PlainDword::new(0),
                PlainDword::new(0),
                PlainDword::new(0),
                PlainDword::new(0),
                PlainDword::new(0),
                PlainDword::new(0),
                PlainDword::new(0),
                PlainDword::new(0),
                PlainDword::new(0),
                PlainDword::new(0),
                PlainDword::new(0),
                PlainDword::new(0),
                PlainDword::new(0),
                PlainDword::new(0),
                PlainDword::new(0),
                PlainDword::new(0),
                PlainDword::new(0),
                PlainDword::new(0),
                PlainDword::new(0),
                PlainDword::new(0),
                PlainDword::new(0xffffffff),
            ],
            repair_time_ptrs: [
                repair_ref(0),
                repair_ref(0),
                repair_ref(0x75ab60),
                repair_ref(0x75ab64),
                repair_ref(0x75ab68),
                repair_ref(0x75ab6c),
                repair_ref(0x75ab70),
                repair_ref(0x75ab74),
                repair_ref(0x75ab78),
                repair_ref(0x75ab7c),
                repair_ref(0x75ab80),
                repair_ref(0x75ab84),
                repair_ref(0x75ab88),
                repair_ref(0x75ab8c),
                repair_ref(0x75ab90),
                repair_ref(0x75ab94),
                repair_ref(0),
                repair_ref(0),
                repair_ref(0x75ab98),
                repair_ref(0x75ab9c),
                repair_ref(0),
                repair_ref(0),
                repair_ref(0),
                repair_ref(0xffffffff),
            ],
            inspection_faults: [
                PlainDword::new(0),
                PlainDword::new(0),
                PlainDword::new(0),
                PlainDword::new(0),
                PlainDword::new(0),
                PlainDword::new(0),
                PlainDword::new(0),
                PlainDword::new(0),
                PlainDword::new(0),
                PlainDword::new(0),
                PlainDword::new(0),
                PlainDword::new(0),
                PlainDword::new(0),
                PlainDword::new(0xffffffff),
            ],
            cars_championship: [
                car_cell(0x756d60),
                car_cell(0x756dd8),
                car_cell(0x756e50),
                car_cell(0x756b08),
                car_cell(0x756a90),
                car_cell(0x7567c0),
                car_cell(0x7569a0),
                car_cell(0x756c70),
                car_cell(0x756ce8),
                car_cell(0x756ec8),
                car_cell(0x7565e0),
                car_cell(0x7566d0),
                car_cell(0x756928),
                car_cell(0x756568),
                car_cell(0x756b80),
                car_cell(0x756f40),
                car_cell(0xffffffff),
                car_cell(0xffffffff),
                car_cell(0xffffffff),
                car_cell(0xffffffff),
                car_cell(0xffffffff),
                car_cell(0xffffffff),
                car_cell(0xffffffff),
                car_cell(0xffffffff),
                car_cell(0xffffffff),
                car_cell(0xffffffff),
            ],
            a8_championship_cars: [
                car_cell(0x756bf8),
                car_cell(0x756748),
                car_cell(0x7568b0),
                car_cell(0x756658),
                car_cell(0x756a18),
                car_cell(0x756fb8),
                car_cell(0xffffffff),
                car_cell(0xffffffff),
                car_cell(0xffffffff),
                car_cell(0xffffffff),
                car_cell(0xffffffff),
                car_cell(0xffffffff),
                car_cell(0xffffffff),
                car_cell(0xffffffff),
                car_cell(0xffffffff),
                car_cell(0x756d60),
                car_cell(0x7568b0),
                car_cell(0xffffffff),
                car_cell(0x756568),
                car_cell(0x7565e0),
                car_cell(0x756658),
                car_cell(0x7566d0),
                car_cell(0x756748),
                car_cell(0x7567c0),
                car_cell(0x756838),
                car_cell(0x7568b0),
                car_cell(0x756928),
                car_cell(0x7569a0),
                car_cell(0x756a18),
                car_cell(0x756a90),
                car_cell(0x756b08),
                car_cell(0x756b80),
                car_cell(0x756bf8),
                car_cell(0x756c70),
                car_cell(0x756ce8),
                car_cell(0x756d60),
                car_cell(0x756dd8),
                car_cell(0x756e50),
                car_cell(0x756ec8),
                car_cell(0x756f40),
                car_cell(0x756fb8),
                car_cell(0xffffffff),
                car_cell(0xffffffff),
                car_cell(0xffffffff),
                car_cell(0xffffffff),
            ],
            cars_first: [
                car_cell(0x756b80),
                car_cell(0x756f40),
                car_cell(0x756dd8),
                car_cell(0x756e50),
                car_cell(0x756b08),
                car_cell(0x756a90),
                car_cell(0x7567c0),
                car_cell(0x7569a0),
                car_cell(0x756c70),
                car_cell(0x756ce8),
                car_cell(0x756ec8),
                car_cell(0x7565e0),
                car_cell(0x7566d0),
                car_cell(0x756928),
                car_cell(0x756568),
                car_cell(0x756d60),
                car_cell(0x756fb8),
                car_cell(0x756a18),
                car_cell(0x756658),
                car_cell(0x7568b0),
                car_cell(0x756748),
                car_cell(0x756bf8),
                car_cell(0xffffffff),
                car_cell(0xffffffff),
                car_cell(0xffffffff),
                car_cell(0xffffffff),
                car_cell(0xffffffff),
                car_cell(0xffffffff),
                car_cell(0xffffffff),
                car_cell(0xffffffff),
                car_cell(0xffffffff),
                car_cell(0xffffffff),
                car_cell(0xffffffff),
                car_cell(0xffffffff),
            ],
            cars_with_clio: [
                car_cell(0x756b80),
                car_cell(0x756f40),
                car_cell(0x756dd8),
                car_cell(0x756e50),
                car_cell(0x756b08),
                car_cell(0x756a90),
                car_cell(0x7567c0),
                car_cell(0x7569a0),
                car_cell(0x756c70),
                car_cell(0x756ce8),
                car_cell(0x756ec8),
                car_cell(0x7565e0),
                car_cell(0x7566d0),
                car_cell(0x756928),
                car_cell(0x756568),
                car_cell(0x756d60),
                car_cell(0x756fb8),
                car_cell(0x756a18),
                car_cell(0x756658),
                car_cell(0x7568b0),
                car_cell(0x756748),
                car_cell(0x756bf8),
                car_cell(0x757030),
                car_cell(0x7570a8),
                car_cell(0x757120),
                car_cell(0x757198),
                car_cell(0x757210),
                car_cell(0x757288),
                car_cell(0x757300),
                car_cell(0x7573f0),
                car_cell(0x757468),
                car_cell(0xffffffff),
                car_cell(0xffffffff),
                car_cell(0xffffffff),
                car_cell(0xffffffff),
                car_cell(0xffffffff),
                car_cell(0xffffffff),
                car_cell(0xffffffff),
                car_cell(0xffffffff),
                car_cell(0xffffffff),
                car_cell(0xffffffff),
                car_cell(0xffffffff),
                car_cell(0xffffffff),
                car_cell(0xffffffff),
                car_cell(0xffffffff),
                car_cell(0xffffffff),
                car_cell(0xffffffff),
                car_cell(0xffffffff),
                car_cell(0xffffffff),
                car_cell(0xffffffff),
                car_cell(0xffffffff),
                car_cell(0xffffffff),
                car_cell(0xffffffff),
                car_cell(0xffffffff),
                car_cell(0xffffffff),
                car_cell(0xffffffff),
                car_cell(0xffffffff),
                car_cell(0xffffffff),
                car_cell(0xffffffff),
                car_cell(0xffffffff),
                car_cell(0xffffffff),
                car_cell(0xffffffff),
            ],
            secret_cars: [
                car_cell(0x757030),
                car_cell(0x7570a8),
                car_cell(0x757120),
                car_cell(0x757198),
                car_cell(0x757210),
                car_cell(0x757288),
                car_cell(0x757300),
                car_cell(0x757378),
                car_cell(0x7573f0),
                car_cell(0x757468),
                car_cell(0x756838),
                car_cell(0xffffffff),
            ],
            cars_scratch: [const { car_cell(0xffffffff) }; 50],
            cars_second: [
                car_cell(0x756b80),
                car_cell(0x756f40),
                car_cell(0x756dd8),
                car_cell(0x756e50),
                car_cell(0x756b08),
                car_cell(0x756a90),
                car_cell(0x7567c0),
                car_cell(0x7569a0),
                car_cell(0x756c70),
                car_cell(0x756ce8),
                car_cell(0x756ec8),
                car_cell(0x7565e0),
                car_cell(0x7566d0),
                car_cell(0x756928),
                car_cell(0x756568),
                car_cell(0x756d60),
                car_cell(0xffffffff),
                car_cell(0xffffffff),
                car_cell(0xffffffff),
                car_cell(0xffffffff),
                car_cell(0xffffffff),
                car_cell(0xffffffff),
                car_cell(0xffffffff),
                car_cell(0xffffffff),
                car_cell(0xffffffff),
                car_cell(0xffffffff),
                car_cell(0xffffffff),
                car_cell(0xffffffff),
                car_cell(0xffffffff),
                car_cell(0xffffffff),
                car_cell(0xffffffff),
                car_cell(0xffffffff),
                car_cell(0xffffffff),
                car_cell(0xffffffff),
                car_cell(0xffffffff),
                car_cell(0xffffffff),
                car_cell(0xffffffff),
                car_cell(0xffffffff),
                car_cell(0xffffffff),
                car_cell(0xffffffff),
                car_cell(0xffffffff),
                car_cell(0xffffffff),
                car_cell(0xffffffff),
                car_cell(0xffffffff),
                car_cell(0xffffffff),
                car_cell(0xffffffff),
                car_cell(0xffffffff),
                car_cell(0xffffffff),
                car_cell(0xffffffff),
                car_cell(0xffffffff),
                car_cell(0xffffffff),
                car_cell(0xffffffff),
                car_cell(0xffffffff),
                car_cell(0xffffffff),
                car_cell(0xffffffff),
                car_cell(0xffffffff),
                car_cell(0xffffffff),
                car_cell(0xffffffff),
                car_cell(0xffffffff),
            ],
            shown_cars: [
                car_cell(0x756658),
                car_cell(0x756748),
                car_cell(0x7568b0),
                car_cell(0x756a18),
                car_cell(0x756bf8),
                car_cell(0x756fb8),
                car_cell(0xffffffff),
                car_cell(0xffffffff),
                car_cell(0xffffffff),
                car_cell(0xffffffff),
                car_cell(0xffffffff),
                car_cell(0xffffffff),
                car_cell(0xffffffff),
                car_cell(0xffffffff),
                car_cell(0xffffffff),
                car_cell(0xffffffff),
                car_cell(0xffffffff),
                car_cell(0xffffffff),
                car_cell(0xffffffff),
                car_cell(0xffffffff),
                car_cell(0xffffffff),
                car_cell(0xffffffff),
                car_cell(0xffffffff),
                car_cell(0xffffffff),
                car_cell(0xffffffff),
                car_cell(0xffffffff),
                car_cell(0xffffffff),
                car_cell(0xffffffff),
                car_cell(0xffffffff),
                car_cell(0xffffffff),
                car_cell(0xffffffff),
                car_cell(0xffffffff),
                car_cell(0xffffffff),
                car_cell(0xffffffff),
                car_cell(0xffffffff),
                car_cell(0xffffffff),
                car_cell(0xffffffff),
                car_cell(0xffffffff),
                car_cell(0xffffffff),
                car_cell(0xffffffff),
                car_cell(0xffffffff),
                car_cell(0xffffffff),
                car_cell(0xffffffff),
                car_cell(0xffffffff),
                car_cell(0xffffffff),
                car_cell(0xffffffff),
                car_cell(0xffffffff),
                car_cell(0xffffffff),
            ],
            _gap_75a1ee: PlainBytes::new([0xff; 16]),
            car_kinds: [
                PlainDword::new(0x191),
                PlainDword::new(0x193),
                PlainDword::new(0x195),
                PlainDword::new(0x197),
                PlainDword::new(0x199),
                PlainDword::new(0x19b),
                PlainDword::new(0x1c3),
                PlainDword::new(0x19f),
                PlainDword::new(0x1a1),
                PlainDword::new(0x1a3),
                PlainDword::new(0x1a5),
                PlainDword::new(0x1a7),
                PlainDword::new(0x1a9),
                PlainDword::new(0x1ab),
                PlainDword::new(0x1ad),
                PlainDword::new(0x1af),
                PlainDword::new(0x1b1),
                PlainDword::new(0x1b3),
                PlainDword::new(0x1b5),
                PlainDword::new(0x1b7),
                PlainDword::new(0x1b9),
                PlainDword::new(0x1bb),
                PlainDword::new(0x1bd),
                PlainDword::new(0x1bf),
                PlainDword::new(0x19d),
                PlainDword::new(0x1c1),
                PlainDword::new(0x1c5),
                PlainDword::new(0x1c7),
                PlainDword::new(0x1c9),
                PlainDword::new(0x1cb),
                PlainDword::new(0x1cd),
                PlainDword::new(0x1cf),
                PlainDword::new(0x1d1),
                PlainDword::new(0x23a),
                PlainDword::new(0xffffffff),
            ],
            section_titles: [
                PlainDword::new(0x3d6),
                PlainDword::new(0x3d7),
                PlainDword::new(0x3d8),
                PlainDword::new(0x3d9),
                PlainDword::new(0x3da),
                PlainDword::new(0x3db),
                PlainDword::new(0xffffffff),
            ],
            page_titles_a: [
                PlainDword::new(0x361),
                PlainDword::new(0x360),
                PlainDword::new(0x35f),
                PlainDword::new(0x362),
                PlainDword::new(0x35e),
                PlainDword::new(0x363),
                PlainDword::new(0x364),
                PlainDword::new(0x365),
                PlainDword::new(0x366),
                PlainDword::new(0x367),
                PlainDword::new(0x368),
                PlainDword::new(0xffffffff),
            ],
            page_titles_b: [
                PlainDword::new(3),
                PlainDword::new(2),
                PlainDword::new(1),
                PlainDword::new(4),
                PlainDword::new(0),
                PlainDword::new(0),
                PlainDword::new(1),
                PlainDword::new(2),
                PlainDword::new(3),
                PlainDword::new(4),
                PlainDword::new(5),
            ],
            rally_speeches: [
                name_ref(0),
                name_ref(1),
                name_ref(2),
                name_ref(3),
                name_ref(4),
                name_ref(5),
            ],
            _gap_75a31a: PlainBytes::new([0xff; 4]),
            car_streams: [
                text_ref(offset_of!(MenuTexts, car_speech)),
                text_ref(car_speech_row(1)),
                text_ref(car_speech_row(2)),
                text_ref(car_speech_row(3)),
                text_ref(car_speech_row(4)),
                text_ref(car_speech_row(5)),
                text_ref(car_speech_row(6)),
                text_ref(car_speech_row(7)),
                text_ref(car_speech_row(8)),
                text_ref(car_speech_row(9)),
                text_ref(car_speech_row(10)),
                text_ref(car_speech_row(11)),
                text_ref(car_speech_row(12)),
                text_ref(car_speech_row(13)),
                text_ref(car_speech_row(14)),
                text_ref(car_speech_row(15)),
                text_ref(car_speech_row(16)),
                text_ref(car_speech_row(17)),
                text_ref(car_speech_row(18)),
                text_ref(car_speech_row(19)),
                text_ref(car_speech_row(20)),
                text_ref(car_speech_row(21)),
                text_ref(car_speech_row(22)),
                text_ref(car_speech_row(23)),
            ],
            stage_speeches: [
                text_ref(offset_of!(MenuTexts, stage_speech)),
                text_ref(stage_speech_row(1)),
                text_ref(stage_speech_row(2)),
                text_ref(stage_speech_row(3)),
                text_ref(stage_speech_row(4)),
                text_ref(stage_speech_row(5)),
                text_ref(stage_speech_row(6)),
                text_ref(stage_speech_row(7)),
                text_ref(stage_speech_row(8)),
                text_ref(stage_speech_row(9)),
                text_ref(stage_speech_row(10)),
                text_ref(stage_speech_row(11)),
                text_ref(stage_speech_row(12)),
                text_ref(stage_speech_row(13)),
                text_ref(stage_speech_row(14)),
                text_ref(stage_speech_row(15)),
                text_ref(stage_speech_row(16)),
                text_ref(stage_speech_row(17)),
                text_ref(stage_speech_row(18)),
                text_ref(stage_speech_row(19)),
                text_ref(stage_speech_row(20)),
                text_ref(stage_speech_row(21)),
                text_ref(stage_speech_row(22)),
                text_ref(stage_speech_row(23)),
                text_ref(stage_speech_row(24)),
                text_ref(stage_speech_row(25)),
                text_ref(stage_speech_row(26)),
                text_ref(stage_speech_row(27)),
                text_ref(stage_speech_row(28)),
                text_ref(stage_speech_row(29)),
                text_ref(stage_speech_row(30)),
                text_ref(stage_speech_row(31)),
                text_ref(stage_speech_row(32)),
                text_ref(stage_speech_row(33)),
                text_ref(stage_speech_row(34)),
                text_ref(stage_speech_row(35)),
            ],
            _gap_75a40e: PlainBytes::new([0xff; 4]),
            cheat_cars: [
                car_cell(0x756568),
                car_cell(0x7565e0),
                car_cell(0x7566d0),
                car_cell(0x756928),
                car_cell(0x7569a0),
                car_cell(0x756c70),
                car_cell(0x756ce8),
                car_cell(0x756d60),
                car_cell(0x7567c0),
                car_cell(0x756a90),
                car_cell(0x756b08),
                car_cell(0x756b80),
                car_cell(0x756dd8),
                car_cell(0x756e50),
                car_cell(0x756658),
                car_cell(0x756748),
                car_cell(0x7568b0),
                car_cell(0x756a18),
                car_cell(0x756bf8),
                car_cell(0x756ec8),
                car_cell(0x756f40),
                car_cell(0x756fb8),
            ],
            _gap_75a46a: PlainBytes::new([0xff; 48]),
            durations: [
                PlainDword::new(0x11),
                PlainDword::new(0xe),
                PlainDword::new(0x14),
                PlainDword::new(0x15),
                PlainDword::new(0x17),
                PlainDword::new(0x14),
                PlainDword::new(0x1f),
                PlainDword::new(0x1b),
                PlainDword::new(0x17),
                PlainDword::new(0x1e),
                PlainDword::new(0xd),
                PlainDword::new(0x19),
                PlainDword::new(0xd),
                PlainDword::new(8),
                PlainDword::new(8),
                PlainDword::new(0xa),
                PlainDword::new(0x1a),
                PlainDword::new(0x1d),
                PlainDword::new(5),
                PlainDword::new(9),
                PlainDword::new(8),
                PlainDword::new(9),
                PlainDword::new(3),
                PlainDword::new(9),
                PlainDword::new(0xe),
                PlainDword::new(0x11),
                PlainDword::new(0xf),
                PlainDword::new(0x11),
                PlainDword::new(0xd),
                PlainDword::new(0xd),
                PlainDword::new(7),
                PlainDword::new(0xb),
                PlainDword::new(0xf),
                PlainDword::new(6),
                PlainDword::new(0xe),
                PlainDword::new(0x10),
                PlainDword::new(0xffffffff),
            ],
            service_pictures: [
                text_ref(offset_of!(MenuTexts, serv00_png_4)),
                text_ref(offset_of!(MenuTexts, serv01_png)),
                text_ref(offset_of!(MenuTexts, serv02_png)),
                text_ref(offset_of!(MenuTexts, serv03_png)),
                text_ref(offset_of!(MenuTexts, serv04_png)),
                text_ref(offset_of!(MenuTexts, serv05_png)),
                text_ref(offset_of!(MenuTexts, serv06_png)),
                text_ref(offset_of!(MenuTexts, serv07_png)),
                text_ref(offset_of!(MenuTexts, serv08_png)),
                text_ref(offset_of!(MenuTexts, serv09_png)),
                text_ref(offset_of!(MenuTexts, serv10_png)),
                text_ref(offset_of!(MenuTexts, serv11_png)),
                text_ref(offset_of!(MenuTexts, serv12_png)),
                text_ref(offset_of!(MenuTexts, serv13_png)),
                text_ref(offset_of!(MenuTexts, serv14_png)),
                text_ref(offset_of!(MenuTexts, serv15_png)),
                text_ref(offset_of!(MenuTexts, serv16_png)),
                text_ref(offset_of!(MenuTexts, serv17_png)),
                text_ref(offset_of!(MenuTexts, serv18_png)),
                text_ref(offset_of!(MenuTexts, serv19_png)),
                text_ref(offset_of!(MenuTexts, serv20_png)),
                text_ref(offset_of!(MenuTexts, serv21_png)),
                text_ref(offset_of!(MenuTexts, serv22_png)),
                text_ref(offset_of!(MenuTexts, serv33_png)),
                text_ref(offset_of!(MenuTexts, serv22_png)),
                text_ref(offset_of!(MenuTexts, serv22_png)),
                text_ref(offset_of!(MenuTexts, serv22_png)),
                text_ref(offset_of!(MenuTexts, serv22_png)),
                text_ref(offset_of!(MenuTexts, serv22_png)),
                text_ref(offset_of!(MenuTexts, serv22_png)),
                text_ref(offset_of!(MenuTexts, serv22_png)),
                text_ref(offset_of!(MenuTexts, serv22_png)),
                text_ref(offset_of!(MenuTexts, serv22_png)),
                text_ref(offset_of!(MenuTexts, serv22_png)),
                text_ref(offset_of!(MenuTexts, serv22_png)),
                text_ref(offset_of!(MenuTexts, serv22_png)),
                text_ref(offset_of!(MenuTexts, serv22_png)),
                text_ref(offset_of!(MenuTexts, serv22_png)),
                text_ref(offset_of!(MenuTexts, serv22_png)),
                text_ref(offset_of!(MenuTexts, serv22_png)),
                text_ref(offset_of!(MenuTexts, serv22_png)),
                text_ref(offset_of!(MenuTexts, serv22_png)),
                text_ref(offset_of!(MenuTexts, serv22_png)),
                text_ref(offset_of!(MenuTexts, serv22_png)),
                text_ref(offset_of!(MenuTexts, serv22_png)),
                text_ref(offset_of!(MenuTexts, serv22_png)),
                text_ref(offset_of!(MenuTexts, serv22_png)),
                text_ref(offset_of!(MenuTexts, serv22_png)),
                text_ref(offset_of!(MenuTexts, serv22_png)),
                text_ref(offset_of!(MenuTexts, serv22_png)),
                text_ref(offset_of!(MenuTexts, serv22_png)),
                text_ref(offset_of!(MenuTexts, serv22_png)),
                text_ref(offset_of!(MenuTexts, serv22_png)),
                text_ref(offset_of!(MenuTexts, serv22_png)),
                text_ref(offset_of!(MenuTexts, serv22_png)),
                text_ref(offset_of!(MenuTexts, serv22_png)),
                text_ref(offset_of!(MenuTexts, serv22_png)),
                text_ref(offset_of!(MenuTexts, serv22_png)),
                text_ref(offset_of!(MenuTexts, serv22_png)),
                text_ref(offset_of!(MenuTexts, serv22_png)),
            ],
            _gap_75a61e: PlainBytes::new([0xff; 4]),
            carousel_textures: [const { PlainDword::new(0) }; 8],
            front_back_path: PlainBytes::new(sparse::<24>(&[(
                0x0,
                &[
                    0x66, 0x69, 0x6c, 0x65, 0x73, 0x5c, 0x67, 0x5c, 0x66, 0x72, 0x6e, 0x74, 0x5c,
                    0x62, 0x61, 0x63, 0x6b, 0x2e, 0x70, 0x63, 0x78,
                ],
            )])),
        }
    }
}

/// IDA 0X7594AA, the first byte of the piece.
#[cfg(test)]
pub const START: usize = 0x7594AA;
/// IDA 0X75A658, the end of the piece.
#[cfg(test)]
pub const END: usize = 0x75A658;

/// The tables of text handles (IDA start, entries).
#[cfg(test)]
const TEXT_TABLES: [(usize, usize); 6] = [
    (0x75987E, 12 + 36), // the welcome pictures, the map's overlays
    (0x759942, 4 * 25),  // the car pages' pictures
    (0x759B0E, 24),      // the slideshow's templates
    (0x75A31E, 24),      // the car streams
    (0x75A37E, 36),      // the stage speeches
    (0x75A52E, 60),      // the service pictures
];
/// The rally speeches (IDA start, entries).
#[cfg(test)]
const NAME_TABLE: (usize, usize) = (0x75A302, 6);
/// The repair times' handles (IDA start, entries).
#[cfg(test)]
const REPAIR_TABLE: (usize, usize) = (0x759C16, 24);

/// The car lists (IDA's address, entries): their entries are car handles
/// (`cars_store::car_ref`) where the original kept descriptions' addresses.
#[cfg(test)]
const CAR_LISTS: [(usize, usize); 9] = {
    use core::mem::offset_of;
    [
        (START + offset_of!(MenuLists, cars_championship), 26),
        (START + offset_of!(MenuLists, a8_championship_cars), 45),
        (START + offset_of!(MenuLists, cars_first), 34),
        (START + offset_of!(MenuLists, cars_with_clio), 62),
        (START + offset_of!(MenuLists, secret_cars), 12),
        (START + offset_of!(MenuLists, cars_scratch), 50),
        (START + offset_of!(MenuLists, cars_second), 59),
        (START + offset_of!(MenuLists, shown_cars), 48),
        (START + offset_of!(MenuLists, cheat_cars), 22),
    ]
};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::menu::owner::menus;
    use core::mem::{offset_of, size_of};

    #[test]
    fn menu_lists_starts_as_the_games() {
        let size = size_of::<MenuLists>();
        assert!(END - START <= size && size <= (END - START).next_multiple_of(4));
        let mine = unsafe {
            std::slice::from_raw_parts((&raw const menus().tables.lists).cast::<u8>(), END - START)
        };
        // The car lists' entries are handles: the image's addresses made handles to compare.
        let mut image =
            crate::test_support::first_values::bytes("menu.owner.MENUS.tables.lists/4526").to_vec();
        for (at, len) in CAR_LISTS {
            for n in 0..len {
                let o = at - START + 4 * n;
                let v = u32::from_le_bytes(image[o..o + 4].try_into().unwrap());
                image[o..o + 4].copy_from_slice(&crate::menu::cars_store::car_ref(v).to_le_bytes());
            }
        }
        // So are the texts, names and repair times.
        let mut made = |(at, len): (usize, usize), key: &dyn Fn(u32) -> u32| {
            for n in 0..len {
                let o = at - START + 4 * n;
                let v = u32::from_le_bytes(image[o..o + 4].try_into().unwrap());
                image[o..o + 4].copy_from_slice(&key(v).to_le_bytes());
            }
        };
        for t in TEXT_TABLES {
            made(t, &crate::menu::texts_store::text_key);
        }
        made(NAME_TABLE, &crate::audio::store::name_key);
        made(REPAIR_TABLE, &repair_key);
        assert_eq!(mine, &image[..]);
        assert_eq!(offset_of!(MenuLists, secret_cars), 0xaa0);
        assert_eq!(offset_of!(MenuLists, carousel_textures), 0x1178);
    }
}
