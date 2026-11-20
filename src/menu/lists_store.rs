//! The menus' lists and tables: the last draw list, the carousel kinds' own places, the paths and
//! sounds the menus name, the best scores' titles. `#[repr(C)]` struct of atomic cells, held by
//! `Menus::lists`; a list read past its end goes on into the next field (and over the gaps), also into
//! the lists that live elsewhere (`runs_from_map_overlays`). Its last field is a 22-byte string:
//! the struct is padded to a multiple of 4.

use crate::audio::names_store::NameList;
use crate::memory::session::{PlainBytes, PlainDword};
use crate::menu::cars_store::car_cell;
use crate::menu::texts_store::{MenuText, SpeechRow};
pub use rc2k_core::menu::car_lists::CarList;

/// What the lists of cars are read through: the repair flags and the carousel's textures sit after
/// them, so a list read past its end goes on into those.
#[derive(Clone, Copy)]
pub struct CarListParts<'a> {
    pub service: &'a crate::service::owner::Service,
    pub cars: &'a crate::menu::owner::Cars,
    pub carousel: &'a crate::menu::carousel_store::Carousel,
    pub lists: &'a MenuLists,
}

impl<'a> CarListParts<'a> {
    pub fn of(menus: &'a crate::menu::owner::Menus) -> Self {
        CarListParts {
            service: &menus.service,
            cars: &menus.cars,
            carousel: &menus.screens.carousel,
            lists: &menus.lists,
        }
    }
}

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
    /// The menus' last draw list (25 entries, -1 at start; the fourth
    /// player's button page in `menu::player_pages` is given it).
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
    /// The map's overlays, by the stage's `event_slot` (`menu::stage_choice`).
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
    /// The inspection's fault flags, 13 items.
    pub inspection_faults: [PlainDword; 14],
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
    /// The path "files\g\frnt\back.pcx" (and two bytes to the record's size, which every
    /// byte of a record of cells is part of).
    pub front_back_path: PlainBytes<24>,
}

/// The starting values of `own_places`: size, y and angle of each kind.
#[rustfmt::skip]
const OWN_PLACES: [[u32; 3]; 73] = [
    [16, 0, 0],
    [16, 0, 0],
    [16, 0, 0],
    [16, 0, 0],
    [16, 0, 0],
    [16, 0, 0],
    [13, 0, 0],
    [13, 0, 0],
    [16, 0, 0],
    [20, 0, 0],
    [14, 0, 0],
    [17, 0, 0],
    [17, 0, 0x5ff],
    [14, 0, 0],
    [19, 0, 0],
    [19, 0, 0],
    [16, 0, 0],
    [18, 0, 0xcf0],
    [16, 0, 0],
    [14, 0, 0],
    [18, 0, 0],
    [18, 0, 0],
    [17, 0, 0],
    [16, 0, 0x3ff],
    [16, 0, 0x3ff],
    [16, 0, 0x3ff],
    [16, 0, 0x3ff],
    [16, 0, 0],
    [14, 0, 0],
    [16, 0, 0],
    [18, 0, 0],
    [18, 0, 0],
    [16, 0, 0],
    [14, 0, 0],
    [18, 0, 0xcf0],
    [12, 0, 0],
    [18, 0, 0],
    [18, 0, 0],
    [18, 0, 0],
    [14, 0, 0xcf0],
    [5, 0, 0xcf0],
    [5, 0, 0xcf0],
    [5, 0, 0xcf0],
    [5, 0, 0xcf0],
    [5, 0, 0xcf0],
    [5, 0, 0xcf0],
    [5, 0, 0xcf0],
    [5, 0, 0xcf0],
    [5, 0, 0xcf0],
    [5, 0, 0xcf0],
    [5, 0, 0xcf0],
    [5, 0, 0xcf0],
    [5, 0, 0xcf0],
    [5, 0, 0xcf0],
    [5, 0, 0xcf0],
    [5, 0, 0xcf0],
    [5, 0, 0xcf0],
    [5, 0, 0xcf0],
    [5, 0, 0xcf0],
    [5, 0, 0xcf0],
    [5, 0, 0xcf0],
    [5, 0, 0xcf0],
    [5, 0, 0xcf0],
    [5, 0, 0xcf0],
    [5, 0, 0xcf0],
    [5, 0, 0xcf0],
    [5, 0, 0xcf0],
    [5, 0, 0xcf0],
    [5, 0, 0xcf0],
    [5, 0, 0xcf0],
    [5, 0, 0xcf0],
    [5, 0, 0xcf0],
    [5, 0, 0xcf0],
];

impl OwnPlace {
    /// The places of `rows` (size, y, angle), in order.
    const fn rows<const N: usize>(rows: [[u32; 3]; N]) -> [Self; N] {
        let mut places: [Self; N] = [const { Self::new(0, 0, 0) }; N];
        let mut n = 0;
        while n < N {
            places[n] = Self::new(rows[n][0], rows[n][1], rows[n][2]);
            n += 1;
        }
        places
    }

    const fn new(size: u32, y: u32, angle: u32) -> Self {
        Self {
            size: PlainDword::new(size),
            y: PlainDword::new(y),
            angle: PlainDword::new(angle),
        }
    }
}

/// The car numbers of the cheat car list, in list order.
const CHEAT_CARS: [usize; 22] = [
    0, 1, 3, 8, 9, 15, 16, 17, 5, 11, 12, 13, 18, 19, 2, 4, 7, 10, 14, 20, 21, 22,
];

/// A row of `texts_store`'s speech samples' names by its key.
const fn speech_ref(row: SpeechRow) -> PlainDword {
    PlainDword::new(row.key())
}

/// The `n`th rally speech name (33 bytes each) among the settings page's speech names, from
/// the one 0xB4 bytes into them, as `SampleName::key` has it (a name's key).
const fn name_key(n: usize) -> u32 {
    assert!(n < 6);
    NameList::MenuSpeech.first_key() + 0xB4 + 33 * n as u32
}

impl MenuLists {
    /// The picture of car `car` on car page `page`, from the page's list (`page_picture_lists`,
    /// in its order). The cars of the 34 descriptions run to 0x20, past a list's 25: a car past a
    /// list reads on into the lists and tables after it.
    pub(super) fn page_picture(
        &self,
        page: u32,
        car: usize,
    ) -> Result<u32, crate::errors::BadInput> {
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
        let lists: [&dyn crate::memory::session::Run; 10] = [
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
        crate::memory::session::read_runs(&lists[first..], car, "a car page's picture")
    }

    /// The map overlay of event slot `slot` (0 ..= 35 in the image's tables; the records can
    /// be loaded from a file, so a slot past the list reads on into the fields after it).
    pub(super) fn map_overlay(
        &self,
        menus: &crate::menu::owner::Menus,
        slot: usize,
    ) -> Result<u32, crate::errors::BadInput> {
        crate::memory::session::read_runs(
            &self.runs_from_map_overlays(CarListParts::of(menus)),
            slot,
            "a map overlay",
        )
    }

    /// The rally speech of section `section` (0 ..= 0x20, held by `small_section`), past the six
    /// on into the fields after them.
    pub fn rally_speech(&self, section: usize) -> Result<u32, crate::errors::BadInput> {
        if let Some(speech) = self.rally_speeches.get(section) {
            return Ok(speech.get());
        }
        crate::memory::session::read_runs(
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
        crate::memory::session::read_runs(
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
    pub(super) fn welcome_picture(
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
        let lists: [&dyn crate::memory::session::Run; 3] = [
            &self.welcome_championship,
            &self.welcome_other,
            &self.map_overlays,
        ];
        let first = if championship { 0 } else { 1 };
        crate::memory::session::read_runs(&lists[first..], section, "a welcome picture")
    }
}

/// The position in `runs_from_map_overlays` of the first run held outside this record.
#[cfg(test)]
const FIRST_MOVED_RUN: usize = 12;

impl MenuLists {
    /// The fields from `map_overlays` to the end, in order, with the lists that moved out of this
    /// record in their places: a list read on past its field goes on into the next, over the
    /// gaps.
    fn runs_from_map_overlays<'a>(
        &'a self,
        parts: CarListParts<'a>,
    ) -> [&'a dyn crate::memory::session::Run; 41] {
        let service = parts.service;
        let cars = &parts.cars.lists;
        let carousel = parts.carousel;
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
            &service.repair_flags,
            &service.repair_marks,
            &service.repair_time_handles,
            &self.inspection_faults,
            &cars.cars_championship,
            &cars.a8_championship_cars,
            &cars.cars_first,
            &cars.cars_with_clio,
            &cars.secret_cars,
            &cars.cars_scratch,
            &cars.cars_second,
            &cars.shown_cars,
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
            &carousel.textures,
            &self.front_back_path,
        ]
    }
}

impl MenuLists {
    /// A list of cars, from its first entry to the end of these lists: a list is read on to its
    /// end mark, and a list of one shown twice (menu 0xE) is read on past its field into the
    /// next.
    pub fn car_entry(
        &self,
        parts: CarListParts,
        list: CarList,
        n: u32,
    ) -> Result<u32, crate::errors::BadInput> {
        let all = self.runs_from_map_overlays(parts);
        crate::memory::session::read_runs(
            &all[Self::car_list_start(list)..],
            n as usize,
            "an entry of a list of cars",
        )
    }

    /// Entry `n` of a list of cars, written.
    pub fn set_car_entry(
        &self,
        parts: CarListParts,
        list: CarList,
        n: u32,
        value: u32,
    ) -> Result<(), crate::errors::BadInput> {
        let all = self.runs_from_map_overlays(parts);
        crate::memory::session::write_runs(
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

/// The starting values of `welcome_championship`.
#[rustfmt::skip]
const WELCOME_CHAMPIONSHIP: [u32; 6] = [
    MenuText::welvx_png.key(), MenuText::welpir_png.key(), MenuText::welsct_png.key(),
    MenuText::weljc_png.key(), MenuText::welul_png.key(), MenuText::welmnx_png.key(),
];

/// The starting values of `welcome_other`.
#[rustfmt::skip]
const WELCOME_OTHER: [u32; 6] = [
    MenuText::a8_1_png.key(), MenuText::a8_2_png.key(), MenuText::a8_3_png.key(),
    MenuText::a8_4_png.key(), MenuText::a8_5_png.key(), MenuText::a8_6_png.key(),
];

/// The starting values of `map_overlays`.
#[rustfmt::skip]
const MAP_OVERLAYS: [u32; 36] = [
    MenuText::vrw_001_png.key(), MenuText::vrw_002_png.key(), MenuText::vrw_003_png.key(),
    MenuText::vrw_004_png.key(), MenuText::vrw_005_png.key(), MenuText::vrw_006_png.key(),
    MenuText::pir_001_png.key(), MenuText::pir_002_png.key(), MenuText::pir_003_png.key(),
    MenuText::pir_004_png.key(), MenuText::pir_005_png.key(), MenuText::pir_006_png.key(),
    MenuText::sc_001_png.key(), MenuText::sc_002_png.key(), MenuText::sc_003_png.key(),
    MenuText::sc_004_png.key(), MenuText::sc_005_png.key(), MenuText::sc_006_png.key(),
    MenuText::jc_001_png.key(), MenuText::jc_002_png.key(), MenuText::jc_003_png.key(),
    MenuText::jc_004_png.key(), MenuText::jc_005_png.key(), MenuText::jc_006_png.key(),
    MenuText::ul_001_png.key(), MenuText::ul_002_png.key(), MenuText::ul_003_png.key(),
    MenuText::ul_004_png.key(), MenuText::ul_005_png.key(), MenuText::ul_006_png.key(),
    MenuText::mnx_001_png.key(), MenuText::mnx_002_png.key(), MenuText::mnx_003_png.key(),
    MenuText::mnx_004_png.key(), MenuText::mnx_005_png.key(), MenuText::mnx_006_png.key(),
];

/// The starting values of `page_pictures_a`.
#[rustfmt::skip]
const PAGE_PICTURES_A: [u32; 25] = [
    MenuText::mgnvc_png.key(), MenuText::vxavc_png.key(), MenuText::stcvc_png.key(),
    MenuText::hycvc_png.key(), MenuText::sbivc_png.key(), MenuText::pgovc_png.key(),
    MenuText::mxpvc_png.key(), MenuText::mtlvc_png.key(), MenuText::vwgvc_png.key(),
    MenuText::nsavc_png.key(), MenuText::ptwvc_png.key(), MenuText::ctsvc_png.key(),
    MenuText::hncvc_png.key(), MenuText::nsmvc_png.key(), MenuText::pgtvc_png.key(),
    MenuText::frtvc_png.key(), MenuText::sdovc_png.key(), MenuText::stivc_png.key(),
    MenuText::sdfvc_png.key(), MenuText::ptcvc_png.key(), MenuText::fmxvc_png.key(),
    MenuText::fpmvc_png.key(), MenuText::oscvc_png.key(), MenuText::rclvc_png.key(), 0xFFFFFFFF,
];

/// The starting values of `page_pictures_b`.
#[rustfmt::skip]
const PAGE_PICTURES_B: [u32; 25] = [
    MenuText::mgndm_png.key(), MenuText::vxadm_png.key(), MenuText::stcdm_png.key(),
    MenuText::hycdm_png.key(), MenuText::sbidm_png.key(), MenuText::pgodm_png.key(),
    MenuText::mxpdm_png.key(), MenuText::mtldm_png.key(), MenuText::vwgdm_png.key(),
    MenuText::nsadm_png.key(), MenuText::ptwdm_png.key(), MenuText::ctsdm_png.key(),
    MenuText::hncdm_png.key(), MenuText::nsmdm_png.key(), MenuText::pgtdm_png.key(),
    MenuText::frtdm_png.key(), MenuText::sdodm_png.key(), MenuText::stidm_png.key(),
    MenuText::sdfdm_png.key(), MenuText::ptcdm_png.key(), MenuText::fmxdm_png.key(),
    MenuText::fpmdm_png.key(), MenuText::oscdm_png.key(), MenuText::rcldm_png.key(), 0xFFFFFFFF,
];

/// The starting values of `page_pictures_c`.
#[rustfmt::skip]
const PAGE_PICTURES_C: [u32; 25] = [
    MenuText::mgnen_png.key(), MenuText::vxaen_png.key(), MenuText::stcen_png.key(),
    MenuText::hycen_png.key(), MenuText::sbien_png.key(), MenuText::pgoen_png.key(),
    MenuText::mxpen_png.key(), MenuText::mtlen_png.key(), MenuText::vwgen_png.key(),
    MenuText::nsaen_png.key(), MenuText::ptwen_png.key(), MenuText::ctsen_png.key(),
    MenuText::hncen_png.key(), MenuText::nsmen_png.key(), MenuText::pgten_png.key(),
    MenuText::frten_png.key(), MenuText::sdoen_png.key(), MenuText::stien_png.key(),
    MenuText::sdfen_png.key(), MenuText::ptcen_png.key(), MenuText::fmxen_png.key(),
    MenuText::fpmen_png.key(), MenuText::oscen_png.key(), MenuText::rclen_png.key(), 0xFFFFFFFF,
];

/// The starting values of `page_pictures_d`.
#[rustfmt::skip]
const PAGE_PICTURES_D: [u32; 25] = [
    MenuText::mgntc_png.key(), MenuText::vxatc_png.key(), MenuText::stctc_png.key(),
    MenuText::hyctc_png.key(), MenuText::sbitc_png.key(), MenuText::pgotc_png.key(),
    MenuText::mxptc_png.key(), MenuText::mtltc_png.key(), MenuText::vwgtc_png.key(),
    MenuText::nsatc_png.key(), MenuText::ptwtc_png.key(), MenuText::ctstc_png.key(),
    MenuText::hnctc_png.key(), MenuText::nsmtc_png.key(), MenuText::pgttc_png.key(),
    MenuText::frttc_png.key(), MenuText::sdotc_png.key(), MenuText::stitc_png.key(),
    MenuText::sdftc_png.key(), MenuText::ptctc_png.key(), MenuText::fmxtc_png.key(),
    MenuText::fpmtc_png.key(), MenuText::osctc_png.key(), MenuText::rcltc_png.key(), 0xFFFFFFFF,
];

/// The starting values of `page_picture_lists`.
#[rustfmt::skip]
const PAGE_PICTURE_LISTS: [u32; 4] = [
    0x759942, 0x759A0A, 0x759A6E, 0x7599A6,
];

/// The starting values of `page_functions`.
#[rustfmt::skip]
const PAGE_FUNCTIONS: [u32; 4] = [
    0x469C40, 0x469F12, 0x46A193, 0x469CA0,
];

/// The starting values of `car_page_titles`.
#[rustfmt::skip]
const CAR_PAGE_TITLES: [u32; 5] = [
    0x24C, 0x249, 0x24B, 0x24A, 0xFFFFFFFF,
];

/// The starting values of `slideshow_templates`.
#[rustfmt::skip]
const SLIDESHOW_TEMPLATES: [u32; 24] = [
    MenuText::mgnsl00_png.key(), MenuText::vxasl00_png.key(), MenuText::stcsl00_png.key(),
    MenuText::hycsl00_png.key(), MenuText::sbisl00_png.key(), MenuText::pgosl00_png.key(),
    MenuText::mxpsl00_png.key(), MenuText::mtlsl00_png.key(), MenuText::vwgsl00_png.key(),
    MenuText::nsasl00_png.key(), MenuText::ptwsl00_png.key(), MenuText::ctssl00_png.key(),
    MenuText::hncsl00_png.key(), MenuText::nsmsl00_png.key(), MenuText::pgtsl00_png.key(),
    MenuText::frtsl00_png.key(), MenuText::sdosl00_png.key(), MenuText::stisl00_png.key(),
    MenuText::sdfsl00_png.key(), MenuText::ptcsl00_png.key(), MenuText::fmxsl00_png.key(),
    MenuText::fpmsl00_png.key(), MenuText::oscsl00_png.key(), MenuText::rclsl00_png.key(),
];

/// The starting values of `inspection_faults`.
#[rustfmt::skip]
const INSPECTION_FAULTS: [u32; 14] = [
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0xFFFFFFFF,
];

/// The starting values of `car_kinds`.
#[rustfmt::skip]
const CAR_KINDS: [u32; 35] = [
    0x191, 0x193, 0x195, 0x197, 0x199, 0x19B, 0x1C3, 0x19F, 0x1A1, 0x1A3, 0x1A5, 0x1A7, 0x1A9,
    0x1AB, 0x1AD, 0x1AF, 0x1B1, 0x1B3, 0x1B5, 0x1B7, 0x1B9, 0x1BB, 0x1BD, 0x1BF, 0x19D, 0x1C1,
    0x1C5, 0x1C7, 0x1C9, 0x1CB, 0x1CD, 0x1CF, 0x1D1, 0x23A, 0xFFFFFFFF,
];

/// The starting values of `section_titles`.
#[rustfmt::skip]
const SECTION_TITLES: [u32; 7] = [
    0x3D6, 0x3D7, 0x3D8, 0x3D9, 0x3DA, 0x3DB, 0xFFFFFFFF,
];

/// The starting values of `page_titles_a`.
#[rustfmt::skip]
const PAGE_TITLES_A: [u32; 12] = [
    0x361, 0x360, 0x35F, 0x362, 0x35E, 0x363, 0x364, 0x365, 0x366, 0x367, 0x368, 0xFFFFFFFF,
];

/// The starting values of `page_titles_b`.
#[rustfmt::skip]
const PAGE_TITLES_B: [u32; 11] = [
    3, 2, 1, 4, 0, 0, 1, 2, 3, 4, 5,
];

/// The starting values of `rally_speeches`.
#[rustfmt::skip]
const RALLY_SPEECHES: [u32; 6] = [
    name_key(0), name_key(1), name_key(2), name_key(3), name_key(4), name_key(5),
];

/// The starting values of `durations`.
#[rustfmt::skip]
const DURATIONS: [u32; 37] = [
    17, 14, 20, 21, 23, 20, 31, 27, 23, 30, 13, 25, 13, 8, 8, 10, 26, 29, 5, 9, 8, 9, 3, 9, 14, 17,
    15, 17, 13, 13, 7, 11, 15, 6, 14, 16, 0xFFFFFFFF,
];

/// The starting values of `service_pictures`.
#[rustfmt::skip]
const SERVICE_PICTURES: [u32; 60] = [
    MenuText::serv00_png_4.key(), MenuText::serv01_png.key(), MenuText::serv02_png.key(),
    MenuText::serv03_png.key(), MenuText::serv04_png.key(), MenuText::serv05_png.key(),
    MenuText::serv06_png.key(), MenuText::serv07_png.key(), MenuText::serv08_png.key(),
    MenuText::serv09_png.key(), MenuText::serv10_png.key(), MenuText::serv11_png.key(),
    MenuText::serv12_png.key(), MenuText::serv13_png.key(), MenuText::serv14_png.key(),
    MenuText::serv15_png.key(), MenuText::serv16_png.key(), MenuText::serv17_png.key(),
    MenuText::serv18_png.key(), MenuText::serv19_png.key(), MenuText::serv20_png.key(),
    MenuText::serv21_png.key(), MenuText::serv22_png.key(), MenuText::serv33_png.key(),
    MenuText::serv22_png.key(), MenuText::serv22_png.key(), MenuText::serv22_png.key(),
    MenuText::serv22_png.key(), MenuText::serv22_png.key(), MenuText::serv22_png.key(),
    MenuText::serv22_png.key(), MenuText::serv22_png.key(), MenuText::serv22_png.key(),
    MenuText::serv22_png.key(), MenuText::serv22_png.key(), MenuText::serv22_png.key(),
    MenuText::serv22_png.key(), MenuText::serv22_png.key(), MenuText::serv22_png.key(),
    MenuText::serv22_png.key(), MenuText::serv22_png.key(), MenuText::serv22_png.key(),
    MenuText::serv22_png.key(), MenuText::serv22_png.key(), MenuText::serv22_png.key(),
    MenuText::serv22_png.key(), MenuText::serv22_png.key(), MenuText::serv22_png.key(),
    MenuText::serv22_png.key(), MenuText::serv22_png.key(), MenuText::serv22_png.key(),
    MenuText::serv22_png.key(), MenuText::serv22_png.key(), MenuText::serv22_png.key(),
    MenuText::serv22_png.key(), MenuText::serv22_png.key(), MenuText::serv22_png.key(),
    MenuText::serv22_png.key(), MenuText::serv22_png.key(), MenuText::serv22_png.key(),
];

impl MenuLists {
    pub const fn new() -> Self {
        MenuLists {
            draw_list: [const { PlainDword::new(0xffffffff) }; 25],
            own_places: OwnPlace::rows(OWN_PLACES),
            _gap_75987a: PlainBytes::new([0xff; 4]),
            welcome_championship: PlainDword::array(WELCOME_CHAMPIONSHIP),
            welcome_other: PlainDword::array(WELCOME_OTHER),
            map_overlays: PlainDword::array(MAP_OVERLAYS),
            _gap_75993e: PlainBytes::new([0xff; 4]),
            page_pictures_a: PlainDword::array(PAGE_PICTURES_A),
            page_pictures_b: PlainDword::array(PAGE_PICTURES_B),
            page_pictures_c: PlainDword::array(PAGE_PICTURES_C),
            page_pictures_d: PlainDword::array(PAGE_PICTURES_D),
            page_picture_lists: PlainDword::array(PAGE_PICTURE_LISTS),
            _gap_759ae2: PlainBytes::new([0xff; 4]),
            page_functions: PlainDword::array(PAGE_FUNCTIONS),
            _gap_759af6: PlainBytes::new([0xff; 4]),
            car_page_titles: PlainDword::array(CAR_PAGE_TITLES),
            slideshow_templates: PlainDword::array(SLIDESHOW_TEMPLATES),
            inspection_faults: PlainDword::array(INSPECTION_FAULTS),
            _gap_75a1ee: PlainBytes::new([0xff; 16]),
            car_kinds: PlainDword::array(CAR_KINDS),
            section_titles: PlainDword::array(SECTION_TITLES),
            page_titles_a: PlainDword::array(PAGE_TITLES_A),
            page_titles_b: PlainDword::array(PAGE_TITLES_B),
            rally_speeches: PlainDword::array(RALLY_SPEECHES),
            _gap_75a31a: PlainBytes::new([0xff; 4]),
            car_streams: {
                let mut cells = [const { PlainDword::new(0) }; 24];
                let mut n = 0;
                while n < 24 {
                    cells[n] = speech_ref(SpeechRow::Car(n));
                    n += 1;
                }
                cells
            },
            stage_speeches: {
                let mut cells = [const { PlainDword::new(0) }; 36];
                let mut n = 0;
                while n < 36 {
                    cells[n] = speech_ref(SpeechRow::Stage(n));
                    n += 1;
                }
                cells
            },
            _gap_75a40e: PlainBytes::new([0xff; 4]),
            cheat_cars: {
                let mut cells = [const { PlainDword::new(0) }; 22];
                let mut n = 0;
                while n < 22 {
                    cells[n] = car_cell(CHEAT_CARS[n]);
                    n += 1;
                }
                cells
            },
            _gap_75a46a: PlainBytes::new([0xff; 48]),
            durations: PlainDword::array(DURATIONS),
            service_pictures: PlainDword::array(SERVICE_PICTURES),
            _gap_75a61e: PlainBytes::new([0xff; 4]),
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

    /// The lists' whole bytes, as a checksum: a wrong dword in a row shows here.
    #[test]
    fn the_menu_lists_keep_the_bytes_of_the_image() {
        let bytes = crate::memory::cells::flatten(&MenuLists::new());
        let mut hash = 0xcbf2_9ce4_8422_2325_u64;
        for byte in &bytes {
            hash = (hash ^ u64::from(*byte)).wrapping_mul(0x100_0000_01b3);
        }
        assert_eq!(bytes.len(), 2888);
        assert_eq!(hash, 0x119b_0f69_279b_ce27);
    }

    /// The lists, every dword from `welcome_championship` to the end of the record numbered.
    fn numbered() -> Box<MenuLists> {
        let lists = Box::new(MenuLists::new());
        let room = (core::mem::size_of::<MenuLists>()
            - core::mem::offset_of!(MenuLists, welcome_championship))
            / 4;
        for n in 0..room {
            crate::memory::session::write_on_in(
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
            crate::memory::session::read_on_in(&*l, table, n, "t").unwrap()
        };
        let old_result = |table: &[PlainDword], n: usize| {
            crate::memory::session::read_on_in(&*l, table, n, "t").ok()
        };
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
        // Up to the first run that lives outside this record (the service's repair flags).
        let room: usize = l
            .runs_from_map_overlays(CarListParts::of(crate::test_support::own::menus()))
            [..FIRST_MOVED_RUN]
            .iter()
            .map(|run| run.count())
            .sum();
        for n in 0..room + 4 {
            if n >= room {
                continue;
            }
            assert_eq!(
                l.map_overlay(crate::test_support::own::menus(), n).ok(),
                old_result(&l.map_overlays, n),
                "map overlay {n}"
            );
        }
    }

    /// Each car list's place in `runs_from_map_overlays` is its own field's, and a list read on
    /// runs into what follows it, gaps included, to the end of the runs.
    #[test]
    fn car_lists_start_at_their_runs() {
        let l = MenuLists::new();
        let cars = &crate::test_support::own::menus().cars.lists;
        let all = l.runs_from_map_overlays(CarListParts::of(crate::test_support::own::menus()));
        let total: usize = all.iter().map(|r| r.count()).sum();
        for (list, table) in [
            (CarList::First, &cars.cars_first[..]),
            (CarList::Second, &cars.cars_second[..]),
            (CarList::WithClio, &cars.cars_with_clio[..]),
            (CarList::Championship, &cars.cars_championship[..]),
            (CarList::A8, &cars.a8_championship_cars[..]),
            (CarList::Cheat, &l.cheat_cars[..]),
            (CarList::Shown, &cars.shown_cars[..]),
        ] {
            let start = MenuLists::car_list_start(list);
            let run: *const u8 = std::ptr::from_ref(all[start]).cast();
            assert_eq!(run, table.as_ptr().cast::<u8>(), "{list:?}");
            let left: usize = all[start..].iter().map(|r| r.count()).sum();
            assert!(left <= total && left > table.len());
            assert!(
                l.car_entry(
                    CarListParts::of(crate::test_support::own::menus()),
                    list,
                    (left - 1) as u32
                )
                .is_ok(),
                "{list:?}"
            );
            assert!(
                l.car_entry(
                    CarListParts::of(crate::test_support::own::menus()),
                    list,
                    left as u32
                )
                .is_err(),
                "{list:?}"
            );
        }
    }
}
