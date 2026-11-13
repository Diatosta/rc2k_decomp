//! The menus' lists and tables, out of RAL.EXE's image: the last draw list, the carousel kinds' own
//! places, the paths and sounds the menus name, the car lists the menus walk, the best scores'
//! titles. `#[repr(C)]` struct of atomic cells, one static; the lists run on past their fields into
//! the next (and over the gaps), read by byte offset. Its last field is a 22-byte string: the
//! struct is padded to a multiple of 4.

use crate::audio::names_store::NameList;
use crate::menu::cars_store::car_cell;
use crate::menu::owner::Menus;
use crate::menu::texts_store::{MenuText, SpeechRow};
use crate::session::{PlainBytes, PlainDword};
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
#[derive(rc2k_derive::Cells, rc2k_derive::ByteView)]
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
#[derive(rc2k_derive::Cells, rc2k_derive::ByteView)]
pub struct MenuLists {
    /// The menus' last draw list (25 entries, -1 at start;
    /// `menu::player4_buttons_loop`'s).
    pub draw_list: [PlainDword; 25],
    /// `MenuLists.own_places[0].size`: the carousel kinds' own places, 12 bytes each, indexed by a
    /// kind's number (`scene::draw_carousel_entry` checks it against 0x88 records, a slip: the
    /// table holds 73 and the lists that follow answer the rest).
    pub own_places: [OwnPlace; 73],
    pub _gap_75987a: PlainBytes<4>,
    /// The pictures' paths the stage view's welcome shows, a championship's.
    pub welcome_championship: [PlainDword; 6],
    /// The same for the other modes (the A8 game).
    pub welcome_other: [PlainDword; 6],
    /// The map's overlays, by the stage's +0x3C (`menu::stage_choice`).
    pub map_overlays: [PlainDword; 36],
    pub _gap_75993e: PlainBytes<4>,
    /// The first of the four lists of paths a car page cycles through
    /// (`menu::load_car_background`), ending in a gap dword.
    pub page_pictures_a: [PlainDword; 25],
    /// The second of the four lists.
    pub page_pictures_b: [PlainDword; 25],
    /// The third of the four lists.
    pub page_pictures_c: [PlainDword; 25],
    /// The fourth of the four lists.
    pub page_pictures_d: [PlainDword; 25],
    /// The addresses of the four lists above; read by nothing (`page_pictures`).
    pub page_picture_lists: [PlainDword; 4],
    pub _gap_759ae2: PlainBytes<4>,
    /// Four code addresses of the car pages. Not used.
    pub page_functions: [PlainDword; 4],
    pub _gap_759af6: PlainBytes<4>,
    /// The car pages' title strings, ended by -1 (`screens::car_page_title`).
    pub car_page_titles: [PlainDword; 5],
    /// The slideshow pictures' path templates (`menu::load_slideshow_picture`).
    pub slideshow_templates: [PlainDword; 24],
    /// The service boxes' flags, the first set, by item value.
    pub inspection_flags_a: [PlainDword; 21],
    /// The service boxes' flags, the second set, by item value.
    pub inspection_flags_b: [PlainDword; 21],
    /// Pointers to each repair item's time, by item (`service::state`).
    pub repair_time_ptrs: [PlainDword; 24],
    /// The inspection's fault flags, 13 items.
    pub inspection_faults: [PlainDword; 14],
    /// The championship's car list (`menu::event_flow`).
    pub cars_championship: [PlainDword; 26],
    /// The A8 rally championship's car list (45 entries ended by -1; `menu::remove_secret_cars`
    /// edits it with the first list).
    pub a8_championship_cars: [PlainDword; 45],
    /// The car list without the Clio, the first.
    pub cars_first: [PlainDword; 34],
    /// The car list with the Clio and the extra cars.
    pub cars_with_clio: [PlainDword; 62],
    /// The cars won.
    pub secret_cars: [PlainDword; 12],
    /// The scratch list `menu::remove_secret_cars` builds.
    pub cars_scratch: [PlainDword; 50],
    /// The second car list (arcade events).
    pub cars_second: [PlainDword; 59],
    /// The list of the cars shown (`menu::arcade_steps`).
    pub shown_cars: [PlainDword; 48],
    pub _gap_75a1ee: PlainBytes<16>,
    /// A kind by car number (`scene::set_up_carousel`).
    pub car_kinds: [PlainDword; 35],
    /// The best scores' section title string ids, by section.
    pub section_titles: [PlainDword; 7],
    /// The best scores' page title string ids, by page.
    pub page_titles_a: [PlainDword; 12],
    /// The best scores' second page title string ids, by page.
    pub page_titles_b: [PlainDword; 11],
    /// The rally speeches' sounds, by section (`menu::stage_view_steps`).
    pub rally_speeches: [PlainDword; 6],
    pub _gap_75a31a: PlainBytes<4>,
    /// The car streams' file names (`sound::play_car_stream`).
    pub car_streams: [PlainDword; 24],
    /// The stage speeches' sounds (`menu::stage_view_steps`).
    pub stage_speeches: [PlainDword; 36],
    pub _gap_75a40e: PlainBytes<4>,
    /// The cheat car list (`menu::cheat_car_list`).
    pub cheat_cars: [PlainDword; 22],
    pub _gap_75a46a: PlainBytes<48>,
    /// The stage panel's durations in minutes, by entry +8
    /// (`screens::stage_panel`; the table runs on over unreferenced bytes).
    pub durations: [PlainDword; 37],
    /// The service pictures' paths (`service::load_background`).
    pub service_pictures: [PlainDword; 60],
    pub _gap_75a61e: PlainBytes<4>,
    /// The eight texture numbers of the carousel's menu (-1 none).
    pub carousel_textures: [PlainDword; 8],
    /// The path "files\g\frnt\back.pcx" (and two bytes to the record's size, which every
    /// byte of a record of cells is part of).
    pub front_back_path: PlainBytes<24>,
}

/// A text of `texts_store` by its key (`MenuText::key`).
const fn text_ref(text: MenuText) -> PlainDword {
    PlainDword::new(text.key())
}

/// A row of `texts_store`'s speech samples' names by its key.
const fn speech_ref(row: SpeechRow) -> PlainDword {
    PlainDword::new(row.key())
}

/// The `n`th rally speech name (33 bytes each) among the settings page's speech names, from
/// the one 0xB4 bytes into them, as `audio::store::name_key` has it (a name's key).
const fn name_ref(n: usize) -> PlainDword {
    assert!(n < 6);
    PlainDword::new(NameList::MenuSpeech.first_key() + 0xB4 + 33 * n as u32)
}

/// The handle of repair time `n` among `MENU_STATE`'s repair times: its number + 1 (0 means
/// none, -1 ends the list).
const fn repair_time(n: u32) -> PlainDword {
    assert!(n < 16);
    PlainDword::new(n + 1)
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
    /// The picture of car `car` on car page `page`, from the page's list (`page_picture_lists`,
    /// in its order). The cars of the 34 descriptions run to 0x20, past a list's 25: a car past a
    /// list reads on into the lists and tables after it.
    pub fn page_picture(&self, page: u32, car: usize) -> Result<u32, crate::errors::BadInput> {
        let own = match page {
            0 => &self.page_pictures_a,
            1 => &self.page_pictures_c,
            2 => &self.page_pictures_d,
            3 => &self.page_pictures_b,
            _ => return Err(crate::errors::BadInput("a car page past the four")),
        };
        if let Some(picture) = own.get(car) {
            return Ok(picture.get());
        }
        let lists: [&dyn crate::session::Run; 10] = [
            &self.page_pictures_a,
            &self.page_pictures_b,
            &self.page_pictures_c,
            &self.page_pictures_d,
            &self.page_picture_lists,
            &self._gap_759ae2,
            &self.page_functions,
            &self._gap_759af6,
            &self.car_page_titles,
            &self.slideshow_templates,
        ];
        let first = match page {
            0 => 0,
            3 => 1,
            1 => 2,
            _ => 3,
        };
        crate::session::read_runs(&lists[first..], car, "a car page's picture")
    }

    /// The map overlay of event slot `slot` (0 ..= 35 in the image's tables; the records can
    /// be loaded from a file, so a slot past the list reads on into the fields after it).
    pub fn map_overlay(&self, slot: usize) -> Result<u32, crate::errors::BadInput> {
        crate::session::read_runs(&self.runs_from_map_overlays(), slot, "a map overlay")
    }

    /// The rally speech of section `section` (0 ..= 0x20, held by `small_section`), past the six
    /// on into the fields after them.
    pub fn rally_speech(&self, section: usize) -> Result<u32, crate::errors::BadInput> {
        if let Some(speech) = self.rally_speeches.get(section) {
            return Ok(speech.get());
        }
        crate::session::read_runs(
            &[
                &self.rally_speeches,
                &self._gap_75a31a,
                &self.car_streams,
                &self.stage_speeches,
            ],
            section,
            "a rally speech",
        )
    }

    /// The stage speech `speech` (0 .. 0x40, held by `stage_speech`), past the 36 on into the
    /// fields after them.
    pub fn stage_speech(&self, speech: usize) -> Result<u32, crate::errors::BadInput> {
        if let Some(entry) = self.stage_speeches.get(speech) {
            return Ok(entry.get());
        }
        crate::session::read_runs(
            &[
                &self.stage_speeches,
                &self._gap_75a40e,
                &self.cheat_cars,
                &self._gap_75a46a,
            ],
            speech,
            "a stage speech",
        )
    }

    /// The welcome picture of section `section` (0 ..= 0x20, held by `small_section`), in the
    /// championship's list or the others', past the six on into the fields after them.
    pub fn welcome_picture(
        &self,
        championship: bool,
        section: usize,
    ) -> Result<u32, crate::errors::BadInput> {
        let own = if championship {
            &self.welcome_championship
        } else {
            &self.welcome_other
        };
        if let Some(picture) = own.get(section) {
            return Ok(picture.get());
        }
        let lists: [&dyn crate::session::Run; 3] = [
            &self.welcome_championship,
            &self.welcome_other,
            &self.map_overlays,
        ];
        let first = if championship { 0 } else { 1 };
        crate::session::read_runs(&lists[first..], section, "a welcome picture")
    }
}

impl MenuLists {
    /// The record's fields from `map_overlays` to its end, in order (`run_on_tests` checks them
    /// against the layout): a list read on past its field goes on into the next, over the gaps.
    fn runs_from_map_overlays(&self) -> [&dyn crate::session::Run; 41] {
        [
            &self.map_overlays,
            &self._gap_75993e,
            &self.page_pictures_a,
            &self.page_pictures_b,
            &self.page_pictures_c,
            &self.page_pictures_d,
            &self.page_picture_lists,
            &self._gap_759ae2,
            &self.page_functions,
            &self._gap_759af6,
            &self.car_page_titles,
            &self.slideshow_templates,
            &self.inspection_flags_a,
            &self.inspection_flags_b,
            &self.repair_time_ptrs,
            &self.inspection_faults,
            &self.cars_championship,
            &self.a8_championship_cars,
            &self.cars_first,
            &self.cars_with_clio,
            &self.secret_cars,
            &self.cars_scratch,
            &self.cars_second,
            &self.shown_cars,
            &self._gap_75a1ee,
            &self.car_kinds,
            &self.section_titles,
            &self.page_titles_a,
            &self.page_titles_b,
            &self.rally_speeches,
            &self._gap_75a31a,
            &self.car_streams,
            &self.stage_speeches,
            &self._gap_75a40e,
            &self.cheat_cars,
            &self._gap_75a46a,
            &self.durations,
            &self.service_pictures,
            &self._gap_75a61e,
            &self.carousel_textures,
            &self.front_back_path,
        ]
    }
}

impl MenuLists {
    /// A list of cars, from its first entry to the end of these lists: a list is read on to its
    /// end mark, and a list of one shown twice (menu 0xE) is read on past its field into the
    /// next.
    pub fn car_entry(&self, list: CarList, n: u32) -> Result<u32, crate::errors::BadInput> {
        let all = self.runs_from_map_overlays();
        crate::session::read_runs(
            &all[Self::car_list_start(list)..],
            n as usize,
            "an entry of a list of cars",
        )
    }

    /// Entry `n` of a list of cars, written.
    pub fn set_car_entry(
        &self,
        list: CarList,
        n: u32,
        value: u32,
    ) -> Result<(), crate::errors::BadInput> {
        let all = self.runs_from_map_overlays();
        crate::session::write_runs(
            &all[Self::car_list_start(list)..],
            n as usize,
            value,
            "an entry of a list of cars",
        )
    }

    /// Where a list of cars is among `runs_from_map_overlays`.
    fn car_list_start(list: CarList) -> usize {
        match list {
            CarList::First => 18,
            CarList::Second => 22,
            CarList::WithClio => 19,
            CarList::Championship => 16,
            CarList::A8 => 17,
            CarList::Cheat => 34,
            CarList::Shown => 23,
        }
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
                text_ref(MenuText::welvx_png),
                text_ref(MenuText::welpir_png),
                text_ref(MenuText::welsct_png),
                text_ref(MenuText::weljc_png),
                text_ref(MenuText::welul_png),
                text_ref(MenuText::welmnx_png),
            ],
            welcome_other: [
                text_ref(MenuText::a8_1_png),
                text_ref(MenuText::a8_2_png),
                text_ref(MenuText::a8_3_png),
                text_ref(MenuText::a8_4_png),
                text_ref(MenuText::a8_5_png),
                text_ref(MenuText::a8_6_png),
            ],
            map_overlays: [
                text_ref(MenuText::vrw_001_png),
                text_ref(MenuText::vrw_002_png),
                text_ref(MenuText::vrw_003_png),
                text_ref(MenuText::vrw_004_png),
                text_ref(MenuText::vrw_005_png),
                text_ref(MenuText::vrw_006_png),
                text_ref(MenuText::pir_001_png),
                text_ref(MenuText::pir_002_png),
                text_ref(MenuText::pir_003_png),
                text_ref(MenuText::pir_004_png),
                text_ref(MenuText::pir_005_png),
                text_ref(MenuText::pir_006_png),
                text_ref(MenuText::sc_001_png),
                text_ref(MenuText::sc_002_png),
                text_ref(MenuText::sc_003_png),
                text_ref(MenuText::sc_004_png),
                text_ref(MenuText::sc_005_png),
                text_ref(MenuText::sc_006_png),
                text_ref(MenuText::jc_001_png),
                text_ref(MenuText::jc_002_png),
                text_ref(MenuText::jc_003_png),
                text_ref(MenuText::jc_004_png),
                text_ref(MenuText::jc_005_png),
                text_ref(MenuText::jc_006_png),
                text_ref(MenuText::ul_001_png),
                text_ref(MenuText::ul_002_png),
                text_ref(MenuText::ul_003_png),
                text_ref(MenuText::ul_004_png),
                text_ref(MenuText::ul_005_png),
                text_ref(MenuText::ul_006_png),
                text_ref(MenuText::mnx_001_png),
                text_ref(MenuText::mnx_002_png),
                text_ref(MenuText::mnx_003_png),
                text_ref(MenuText::mnx_004_png),
                text_ref(MenuText::mnx_005_png),
                text_ref(MenuText::mnx_006_png),
            ],
            _gap_75993e: PlainBytes::new([0xff; 4]),
            page_pictures_a: [
                text_ref(MenuText::mgnvc_png),
                text_ref(MenuText::vxavc_png),
                text_ref(MenuText::stcvc_png),
                text_ref(MenuText::hycvc_png),
                text_ref(MenuText::sbivc_png),
                text_ref(MenuText::pgovc_png),
                text_ref(MenuText::mxpvc_png),
                text_ref(MenuText::mtlvc_png),
                text_ref(MenuText::vwgvc_png),
                text_ref(MenuText::nsavc_png),
                text_ref(MenuText::ptwvc_png),
                text_ref(MenuText::ctsvc_png),
                text_ref(MenuText::hncvc_png),
                text_ref(MenuText::nsmvc_png),
                text_ref(MenuText::pgtvc_png),
                text_ref(MenuText::frtvc_png),
                text_ref(MenuText::sdovc_png),
                text_ref(MenuText::stivc_png),
                text_ref(MenuText::sdfvc_png),
                text_ref(MenuText::ptcvc_png),
                text_ref(MenuText::fmxvc_png),
                text_ref(MenuText::fpmvc_png),
                text_ref(MenuText::oscvc_png),
                text_ref(MenuText::rclvc_png),
                PlainDword::new(0xffffffff),
            ],
            page_pictures_b: [
                text_ref(MenuText::mgndm_png),
                text_ref(MenuText::vxadm_png),
                text_ref(MenuText::stcdm_png),
                text_ref(MenuText::hycdm_png),
                text_ref(MenuText::sbidm_png),
                text_ref(MenuText::pgodm_png),
                text_ref(MenuText::mxpdm_png),
                text_ref(MenuText::mtldm_png),
                text_ref(MenuText::vwgdm_png),
                text_ref(MenuText::nsadm_png),
                text_ref(MenuText::ptwdm_png),
                text_ref(MenuText::ctsdm_png),
                text_ref(MenuText::hncdm_png),
                text_ref(MenuText::nsmdm_png),
                text_ref(MenuText::pgtdm_png),
                text_ref(MenuText::frtdm_png),
                text_ref(MenuText::sdodm_png),
                text_ref(MenuText::stidm_png),
                text_ref(MenuText::sdfdm_png),
                text_ref(MenuText::ptcdm_png),
                text_ref(MenuText::fmxdm_png),
                text_ref(MenuText::fpmdm_png),
                text_ref(MenuText::oscdm_png),
                text_ref(MenuText::rcldm_png),
                PlainDword::new(0xffffffff),
            ],
            page_pictures_c: [
                text_ref(MenuText::mgnen_png),
                text_ref(MenuText::vxaen_png),
                text_ref(MenuText::stcen_png),
                text_ref(MenuText::hycen_png),
                text_ref(MenuText::sbien_png),
                text_ref(MenuText::pgoen_png),
                text_ref(MenuText::mxpen_png),
                text_ref(MenuText::mtlen_png),
                text_ref(MenuText::vwgen_png),
                text_ref(MenuText::nsaen_png),
                text_ref(MenuText::ptwen_png),
                text_ref(MenuText::ctsen_png),
                text_ref(MenuText::hncen_png),
                text_ref(MenuText::nsmen_png),
                text_ref(MenuText::pgten_png),
                text_ref(MenuText::frten_png),
                text_ref(MenuText::sdoen_png),
                text_ref(MenuText::stien_png),
                text_ref(MenuText::sdfen_png),
                text_ref(MenuText::ptcen_png),
                text_ref(MenuText::fmxen_png),
                text_ref(MenuText::fpmen_png),
                text_ref(MenuText::oscen_png),
                text_ref(MenuText::rclen_png),
                PlainDword::new(0xffffffff),
            ],
            page_pictures_d: [
                text_ref(MenuText::mgntc_png),
                text_ref(MenuText::vxatc_png),
                text_ref(MenuText::stctc_png),
                text_ref(MenuText::hyctc_png),
                text_ref(MenuText::sbitc_png),
                text_ref(MenuText::pgotc_png),
                text_ref(MenuText::mxptc_png),
                text_ref(MenuText::mtltc_png),
                text_ref(MenuText::vwgtc_png),
                text_ref(MenuText::nsatc_png),
                text_ref(MenuText::ptwtc_png),
                text_ref(MenuText::ctstc_png),
                text_ref(MenuText::hnctc_png),
                text_ref(MenuText::nsmtc_png),
                text_ref(MenuText::pgttc_png),
                text_ref(MenuText::frttc_png),
                text_ref(MenuText::sdotc_png),
                text_ref(MenuText::stitc_png),
                text_ref(MenuText::sdftc_png),
                text_ref(MenuText::ptctc_png),
                text_ref(MenuText::fmxtc_png),
                text_ref(MenuText::fpmtc_png),
                text_ref(MenuText::osctc_png),
                text_ref(MenuText::rcltc_png),
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
                text_ref(MenuText::mgnsl00_png),
                text_ref(MenuText::vxasl00_png),
                text_ref(MenuText::stcsl00_png),
                text_ref(MenuText::hycsl00_png),
                text_ref(MenuText::sbisl00_png),
                text_ref(MenuText::pgosl00_png),
                text_ref(MenuText::mxpsl00_png),
                text_ref(MenuText::mtlsl00_png),
                text_ref(MenuText::vwgsl00_png),
                text_ref(MenuText::nsasl00_png),
                text_ref(MenuText::ptwsl00_png),
                text_ref(MenuText::ctssl00_png),
                text_ref(MenuText::hncsl00_png),
                text_ref(MenuText::nsmsl00_png),
                text_ref(MenuText::pgtsl00_png),
                text_ref(MenuText::frtsl00_png),
                text_ref(MenuText::sdosl00_png),
                text_ref(MenuText::stisl00_png),
                text_ref(MenuText::sdfsl00_png),
                text_ref(MenuText::ptcsl00_png),
                text_ref(MenuText::fmxsl00_png),
                text_ref(MenuText::fpmsl00_png),
                text_ref(MenuText::oscsl00_png),
                text_ref(MenuText::rclsl00_png),
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
                PlainDword::new(0),
                PlainDword::new(0),
                repair_time(0),
                repair_time(1),
                repair_time(2),
                repair_time(3),
                repair_time(4),
                repair_time(5),
                repair_time(6),
                repair_time(7),
                repair_time(8),
                repair_time(9),
                repair_time(10),
                repair_time(11),
                repair_time(12),
                repair_time(13),
                PlainDword::new(0),
                PlainDword::new(0),
                repair_time(14),
                repair_time(15),
                PlainDword::new(0),
                PlainDword::new(0),
                PlainDword::new(0),
                PlainDword::new(0xffffffff),
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
                speech_ref(SpeechRow::Car(0)),
                speech_ref(SpeechRow::Car(1)),
                speech_ref(SpeechRow::Car(2)),
                speech_ref(SpeechRow::Car(3)),
                speech_ref(SpeechRow::Car(4)),
                speech_ref(SpeechRow::Car(5)),
                speech_ref(SpeechRow::Car(6)),
                speech_ref(SpeechRow::Car(7)),
                speech_ref(SpeechRow::Car(8)),
                speech_ref(SpeechRow::Car(9)),
                speech_ref(SpeechRow::Car(10)),
                speech_ref(SpeechRow::Car(11)),
                speech_ref(SpeechRow::Car(12)),
                speech_ref(SpeechRow::Car(13)),
                speech_ref(SpeechRow::Car(14)),
                speech_ref(SpeechRow::Car(15)),
                speech_ref(SpeechRow::Car(16)),
                speech_ref(SpeechRow::Car(17)),
                speech_ref(SpeechRow::Car(18)),
                speech_ref(SpeechRow::Car(19)),
                speech_ref(SpeechRow::Car(20)),
                speech_ref(SpeechRow::Car(21)),
                speech_ref(SpeechRow::Car(22)),
                speech_ref(SpeechRow::Car(23)),
            ],
            stage_speeches: [
                speech_ref(SpeechRow::Stage(0)),
                speech_ref(SpeechRow::Stage(1)),
                speech_ref(SpeechRow::Stage(2)),
                speech_ref(SpeechRow::Stage(3)),
                speech_ref(SpeechRow::Stage(4)),
                speech_ref(SpeechRow::Stage(5)),
                speech_ref(SpeechRow::Stage(6)),
                speech_ref(SpeechRow::Stage(7)),
                speech_ref(SpeechRow::Stage(8)),
                speech_ref(SpeechRow::Stage(9)),
                speech_ref(SpeechRow::Stage(10)),
                speech_ref(SpeechRow::Stage(11)),
                speech_ref(SpeechRow::Stage(12)),
                speech_ref(SpeechRow::Stage(13)),
                speech_ref(SpeechRow::Stage(14)),
                speech_ref(SpeechRow::Stage(15)),
                speech_ref(SpeechRow::Stage(16)),
                speech_ref(SpeechRow::Stage(17)),
                speech_ref(SpeechRow::Stage(18)),
                speech_ref(SpeechRow::Stage(19)),
                speech_ref(SpeechRow::Stage(20)),
                speech_ref(SpeechRow::Stage(21)),
                speech_ref(SpeechRow::Stage(22)),
                speech_ref(SpeechRow::Stage(23)),
                speech_ref(SpeechRow::Stage(24)),
                speech_ref(SpeechRow::Stage(25)),
                speech_ref(SpeechRow::Stage(26)),
                speech_ref(SpeechRow::Stage(27)),
                speech_ref(SpeechRow::Stage(28)),
                speech_ref(SpeechRow::Stage(29)),
                speech_ref(SpeechRow::Stage(30)),
                speech_ref(SpeechRow::Stage(31)),
                speech_ref(SpeechRow::Stage(32)),
                speech_ref(SpeechRow::Stage(33)),
                speech_ref(SpeechRow::Stage(34)),
                speech_ref(SpeechRow::Stage(35)),
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
                text_ref(MenuText::serv00_png_4),
                text_ref(MenuText::serv01_png),
                text_ref(MenuText::serv02_png),
                text_ref(MenuText::serv03_png),
                text_ref(MenuText::serv04_png),
                text_ref(MenuText::serv05_png),
                text_ref(MenuText::serv06_png),
                text_ref(MenuText::serv07_png),
                text_ref(MenuText::serv08_png),
                text_ref(MenuText::serv09_png),
                text_ref(MenuText::serv10_png),
                text_ref(MenuText::serv11_png),
                text_ref(MenuText::serv12_png),
                text_ref(MenuText::serv13_png),
                text_ref(MenuText::serv14_png),
                text_ref(MenuText::serv15_png),
                text_ref(MenuText::serv16_png),
                text_ref(MenuText::serv17_png),
                text_ref(MenuText::serv18_png),
                text_ref(MenuText::serv19_png),
                text_ref(MenuText::serv20_png),
                text_ref(MenuText::serv21_png),
                text_ref(MenuText::serv22_png),
                text_ref(MenuText::serv33_png),
                text_ref(MenuText::serv22_png),
                text_ref(MenuText::serv22_png),
                text_ref(MenuText::serv22_png),
                text_ref(MenuText::serv22_png),
                text_ref(MenuText::serv22_png),
                text_ref(MenuText::serv22_png),
                text_ref(MenuText::serv22_png),
                text_ref(MenuText::serv22_png),
                text_ref(MenuText::serv22_png),
                text_ref(MenuText::serv22_png),
                text_ref(MenuText::serv22_png),
                text_ref(MenuText::serv22_png),
                text_ref(MenuText::serv22_png),
                text_ref(MenuText::serv22_png),
                text_ref(MenuText::serv22_png),
                text_ref(MenuText::serv22_png),
                text_ref(MenuText::serv22_png),
                text_ref(MenuText::serv22_png),
                text_ref(MenuText::serv22_png),
                text_ref(MenuText::serv22_png),
                text_ref(MenuText::serv22_png),
                text_ref(MenuText::serv22_png),
                text_ref(MenuText::serv22_png),
                text_ref(MenuText::serv22_png),
                text_ref(MenuText::serv22_png),
                text_ref(MenuText::serv22_png),
                text_ref(MenuText::serv22_png),
                text_ref(MenuText::serv22_png),
                text_ref(MenuText::serv22_png),
                text_ref(MenuText::serv22_png),
                text_ref(MenuText::serv22_png),
                text_ref(MenuText::serv22_png),
                text_ref(MenuText::serv22_png),
                text_ref(MenuText::serv22_png),
                text_ref(MenuText::serv22_png),
                text_ref(MenuText::serv22_png),
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

#[cfg(test)]
mod run_on_tests {
    use super::*;

    /// The lists, every dword from `welcome_championship` to the end of the record numbered.
    fn numbered() -> Box<MenuLists> {
        let lists = Box::new(MenuLists::new());
        let room = (core::mem::size_of::<MenuLists>()
            - core::mem::offset_of!(MenuLists, welcome_championship))
            / 4;
        for n in 0..room {
            crate::session::write_on_in(
                &*lists,
                &lists.welcome_championship,
                n,
                0x7000 + n as u32,
                "t",
            )
            .unwrap();
        }
        lists
    }

    /// Each reader gives what the byte-offset read gave, for every index the game's checks let
    /// through.
    #[test]
    fn list_readers_run_on_as_the_old_read() {
        let l = numbered();
        let old = |table: &[PlainDword], n: usize| {
            crate::session::read_on_in(&*l, table, n, "t").unwrap()
        };
        let old_result =
            |table: &[PlainDword], n: usize| crate::session::read_on_in(&*l, table, n, "t").ok();
        for n in 0..0x21 {
            assert_eq!(
                l.rally_speech(n).unwrap(),
                old(&l.rally_speeches, n),
                "rally {n}"
            );
            assert_eq!(
                l.welcome_picture(true, n).unwrap(),
                old(&l.welcome_championship, n),
                "welcome {n}"
            );
            assert_eq!(
                l.welcome_picture(false, n).unwrap(),
                old(&l.welcome_other, n),
                "other welcome {n}"
            );
        }
        for n in 0..0x40 {
            assert_eq!(
                l.stage_speech(n).unwrap(),
                old(&l.stage_speeches, n),
                "speech {n}"
            );
        }
        for (page, table) in [
            (0, &l.page_pictures_a),
            (1, &l.page_pictures_c),
            (2, &l.page_pictures_d),
            (3, &l.page_pictures_b),
        ] {
            for car in 0..64 {
                assert_eq!(
                    l.page_picture(page, car).unwrap(),
                    old(table, car),
                    "page {page} car {car}"
                );
            }
        }
        assert!(l.page_picture(4, 0).is_err());
        let room = (core::mem::size_of::<MenuLists>()
            - core::mem::offset_of!(MenuLists, map_overlays))
            / 4;
        for n in 0..room + 4 {
            assert_eq!(
                l.map_overlay(n).ok(),
                old_result(&l.map_overlays, n),
                "map overlay {n}"
            );
        }
    }

    /// The fields' order in `runs_from_map_overlays`, and each car list's place in it, are the
    /// layout's: every dword of the record from `map_overlays` on reads the same both ways,
    /// and each list of cars runs on into what follows it, gaps included.
    #[test]
    fn runs_follow_the_layout() {
        let l = numbered();
        let room = (core::mem::size_of::<MenuLists>()
            - core::mem::offset_of!(MenuLists, map_overlays))
            / 4;
        let all = l.runs_from_map_overlays();
        assert_eq!(all.iter().map(|r| r.count()).sum::<usize>(), room);
        for (list, table) in [
            (CarList::First, &l.cars_first[..]),
            (CarList::Second, &l.cars_second[..]),
            (CarList::WithClio, &l.cars_with_clio[..]),
            (CarList::Championship, &l.cars_championship[..]),
            (CarList::A8, &l.a8_championship_cars[..]),
            (CarList::Cheat, &l.cheat_cars[..]),
            (CarList::Shown, &l.shown_cars[..]),
        ] {
            let left = (core::mem::size_of::<MenuLists>()
                - (table.as_ptr() as usize - std::ptr::from_ref::<MenuLists>(&l) as usize))
                / 4;
            for n in 0..left + 2 {
                assert_eq!(
                    l.car_entry(list, n as u32).ok(),
                    crate::session::read_on_in(&*l, table, n, "t").ok(),
                    "{list:?} {n}"
                );
            }
            let n = table.len() + 1;
            l.set_car_entry(list, n as u32, 0x1234).unwrap();
            assert_eq!(
                crate::session::read_on_in(&*l, table, n, "t").unwrap(),
                0x1234
            );
            assert!(l.set_car_entry(list, left as u32, 1).is_err());
        }
    }
}
