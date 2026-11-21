//! The menus' lists and tables: the carousel kinds' own places, the paths and sounds the menus
//! name, the best scores' titles. Plain arrays behind one `RefCell` (the game's thread alone reads
//! and writes them, each borrow lasting one statement). A list read past its end goes on into the
//! next field (and over the gaps), also into the lists that live elsewhere
//! (`runs_from_map_overlays`).

use crate::audio::names_store::NameList;
use crate::memory::session::{GameThread, Run};
use crate::menu::cars_store::car_handle;
use crate::menu::texts_store::{MenuText, SpeechRow};
pub use rc2k_core::menu::car_lists::CarList;
use std::cell::{Ref, RefCell, RefMut};

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

/// The dwords of a row of bytes, little-endian.
const fn dwords_of<const N: usize, const B: usize>(bytes: [u8; B]) -> [u32; N] {
    assert!(B == 4 * N);
    let mut out = [0u32; N];
    let mut n = 0;
    while n < N {
        out[n] = u32::from_le_bytes([
            bytes[4 * n],
            bytes[4 * n + 1],
            bytes[4 * n + 2],
            bytes[4 * n + 3],
        ]);
        n += 1;
    }
    out
}

/// One kind's own place in a menu scene (`scene::draw_carousel_entry`: the
/// kind's size, its y and its angle).
#[derive(Clone, Copy)]
pub struct OwnPlace {
    /// Its size.
    pub size: u32,
    /// Its y.
    pub y: u32,
    /// Its angle about x.
    pub angle: u32,
}

/// The menus' lists and tables, as values.
#[derive(Clone, Copy)]
pub struct ListTables {
    /// The carousel kinds' own places, indexed by a kind's number
    /// (`scene::draw_carousel_entry` checks it against 0x88 records, a slip: the table holds 73
    /// and the lists that follow answer the rest).
    pub own_places: [OwnPlace; 73],
    pub gap_after_own_places: [u32; 1],
    /// The pictures' paths the stage view's welcome shows, a championship's.
    pub welcome_championship: [u32; 6],
    /// The same for the other modes (the A8 game).
    pub welcome_other: [u32; 6],
    /// The map's overlays, by the stage's `event_slot` (`menu::stage_choice`).
    pub map_overlays: [u32; 36],
    pub gap_after_map_overlays: [u32; 1],
    /// The first of the four lists of paths a car page cycles through
    /// (`menu::load_car_background`), ending in a gap dword.
    pub page_pictures_a: [u32; 25],
    /// The second of the four lists.
    pub page_pictures_b: [u32; 25],
    /// The third of the four lists.
    pub page_pictures_c: [u32; 25],
    /// The fourth of the four lists.
    pub page_pictures_d: [u32; 25],
    /// What the lists above were found by; only read on into.
    pub page_picture_lists: [u32; 4],
    pub gap_after_page_picture_lists: [u32; 1],
    /// What the car pages were found by; only read on into.
    pub page_functions: [u32; 4],
    pub gap_after_page_functions: [u32; 1],
    /// The car pages' title strings, ended by -1 (`screens::car_page_title`).
    pub car_page_titles: [u32; 5],
    /// The slideshow pictures' path templates (`menu::load_slideshow_picture`).
    pub slideshow_templates: [u32; 24],
    /// The inspection's fault flags, 13 items.
    pub inspection_faults: [u32; 14],
    pub gap_after_inspection_faults: [u32; 4],
    /// A kind by car number (`scene::set_up_carousel`).
    pub car_kinds: [u32; 35],
    /// The best scores' section title string ids, by section.
    pub section_titles: [u32; 7],
    /// The best scores' page title string ids, by page.
    pub page_titles_a: [u32; 12],
    /// The best scores' second page title string ids, by page.
    pub page_titles_b: [u32; 11],
    /// The rally speeches' sounds, by section (`menu::stage_view_steps`).
    pub rally_speeches: [u32; 6],
    pub gap_after_rally_speeches: [u32; 1],
    /// The car streams' file names (`sound::play_car_stream`).
    pub car_streams: [u32; 24],
    /// The stage speeches' sounds (`menu::stage_view_steps`).
    pub stage_speeches: [u32; 36],
    pub gap_after_stage_speeches: [u32; 1],
    /// The cheat car list (`menu::cheat_car_list`).
    pub cheat_cars: [u32; 22],
    pub gap_after_cheat_cars: [u32; 12],
    /// The stage panel's durations in minutes, by entry +8
    /// (`screens::stage_panel`; the table runs on over unreferenced bytes).
    pub durations: [u32; 37],
    /// The service pictures' paths (`service::load_background`).
    pub service_pictures: [u32; 60],
    pub gap_after_service_pictures: [u32; 1],
    /// The path "files\g\frnt\back.pcx", as dwords (only read on into).
    pub front_back_path: [u32; 6],
}

/// The menus' lists and tables.
pub struct MenuLists(GameThread<RefCell<ListTables>>);

/// A field of `ListTables` as a run of dwords, for the reads that go on past a field's end.
#[derive(Clone, Copy)]
enum Part {
    MapOverlays,
    GapAfterMapOverlays,
    PagePicturesA,
    PagePicturesB,
    PagePicturesC,
    PagePicturesD,
    PagePictureLists,
    GapAfterPagePictureLists,
    PageFunctions,
    GapAfterPageFunctions,
    CarPageTitles,
    SlideshowTemplates,
    InspectionFaults,
    GapAfterInspectionFaults,
    CarKinds,
    SectionTitles,
    PageTitlesA,
    PageTitlesB,
    RallySpeeches,
    GapAfterRallySpeeches,
    CarStreams,
    StageSpeeches,
    GapAfterStageSpeeches,
    CheatCars,
    GapAfterCheatCars,
    Durations,
    ServicePictures,
    GapAfterServicePictures,
    FrontBackPath,
    WelcomeChampionship,
    WelcomeOther,
}

impl ListTables {
    fn part(&self, part: Part) -> &[u32] {
        match part {
            Part::MapOverlays => &self.map_overlays,
            Part::GapAfterMapOverlays => &self.gap_after_map_overlays,
            Part::PagePicturesA => &self.page_pictures_a,
            Part::PagePicturesB => &self.page_pictures_b,
            Part::PagePicturesC => &self.page_pictures_c,
            Part::PagePicturesD => &self.page_pictures_d,
            Part::PagePictureLists => &self.page_picture_lists,
            Part::GapAfterPagePictureLists => &self.gap_after_page_picture_lists,
            Part::PageFunctions => &self.page_functions,
            Part::GapAfterPageFunctions => &self.gap_after_page_functions,
            Part::CarPageTitles => &self.car_page_titles,
            Part::SlideshowTemplates => &self.slideshow_templates,
            Part::InspectionFaults => &self.inspection_faults,
            Part::GapAfterInspectionFaults => &self.gap_after_inspection_faults,
            Part::CarKinds => &self.car_kinds,
            Part::SectionTitles => &self.section_titles,
            Part::PageTitlesA => &self.page_titles_a,
            Part::PageTitlesB => &self.page_titles_b,
            Part::RallySpeeches => &self.rally_speeches,
            Part::GapAfterRallySpeeches => &self.gap_after_rally_speeches,
            Part::CarStreams => &self.car_streams,
            Part::StageSpeeches => &self.stage_speeches,
            Part::GapAfterStageSpeeches => &self.gap_after_stage_speeches,
            Part::CheatCars => &self.cheat_cars,
            Part::GapAfterCheatCars => &self.gap_after_cheat_cars,
            Part::Durations => &self.durations,
            Part::ServicePictures => &self.service_pictures,
            Part::GapAfterServicePictures => &self.gap_after_service_pictures,
            Part::FrontBackPath => &self.front_back_path,
            Part::WelcomeChampionship => &self.welcome_championship,
            Part::WelcomeOther => &self.welcome_other,
        }
    }

    fn part_mut(&mut self, part: Part) -> &mut [u32] {
        match part {
            Part::MapOverlays => &mut self.map_overlays,
            Part::GapAfterMapOverlays => &mut self.gap_after_map_overlays,
            Part::PagePicturesA => &mut self.page_pictures_a,
            Part::PagePicturesB => &mut self.page_pictures_b,
            Part::PagePicturesC => &mut self.page_pictures_c,
            Part::PagePicturesD => &mut self.page_pictures_d,
            Part::PagePictureLists => &mut self.page_picture_lists,
            Part::GapAfterPagePictureLists => &mut self.gap_after_page_picture_lists,
            Part::PageFunctions => &mut self.page_functions,
            Part::GapAfterPageFunctions => &mut self.gap_after_page_functions,
            Part::CarPageTitles => &mut self.car_page_titles,
            Part::SlideshowTemplates => &mut self.slideshow_templates,
            Part::InspectionFaults => &mut self.inspection_faults,
            Part::GapAfterInspectionFaults => &mut self.gap_after_inspection_faults,
            Part::CarKinds => &mut self.car_kinds,
            Part::SectionTitles => &mut self.section_titles,
            Part::PageTitlesA => &mut self.page_titles_a,
            Part::PageTitlesB => &mut self.page_titles_b,
            Part::RallySpeeches => &mut self.rally_speeches,
            Part::GapAfterRallySpeeches => &mut self.gap_after_rally_speeches,
            Part::CarStreams => &mut self.car_streams,
            Part::StageSpeeches => &mut self.stage_speeches,
            Part::GapAfterStageSpeeches => &mut self.gap_after_stage_speeches,
            Part::CheatCars => &mut self.cheat_cars,
            Part::GapAfterCheatCars => &mut self.gap_after_cheat_cars,
            Part::Durations => &mut self.durations,
            Part::ServicePictures => &mut self.service_pictures,
            Part::GapAfterServicePictures => &mut self.gap_after_service_pictures,
            Part::FrontBackPath => &mut self.front_back_path,
            Part::WelcomeChampionship => &mut self.welcome_championship,
            Part::WelcomeOther => &mut self.welcome_other,
        }
    }
}

/// One of the lists' fields, read and written dword by dword.
struct OwnRun<'a> {
    lists: &'a MenuLists,
    part: Part,
}

impl Run for OwnRun<'_> {
    fn count(&self) -> usize {
        self.lists.read().part(self.part).len()
    }
    fn bits(&self, i: usize) -> u32 {
        self.lists.read().part(self.part)[i]
    }
    fn set_bits(&self, i: usize, v: u32) {
        self.lists.write().part_mut(self.part)[i] = v;
    }
}

/// A run of this record's, or one of the lists that live elsewhere.
enum Segment<'a> {
    Own(OwnRun<'a>),
    Foreign(&'a dyn Run),
}

impl Run for Segment<'_> {
    fn count(&self) -> usize {
        match self {
            Segment::Own(run) => run.count(),
            Segment::Foreign(run) => run.count(),
        }
    }
    fn bits(&self, i: usize) -> u32 {
        match self {
            Segment::Own(run) => run.bits(i),
            Segment::Foreign(run) => run.bits(i),
        }
    }
    fn set_bits(&self, i: usize, v: u32) {
        match self {
            Segment::Own(run) => run.set_bits(i, v),
            Segment::Foreign(run) => run.set_bits(i, v),
        }
    }
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
        let mut places = [Self {
            size: 0,
            y: 0,
            angle: 0,
        }; N];
        let mut n = 0;
        while n < N {
            places[n] = Self {
                size: rows[n][0],
                y: rows[n][1],
                angle: rows[n][2],
            };
            n += 1;
        }
        places
    }
}

/// The car numbers of the cheat car list, in list order.
const CHEAT_CARS: [usize; 22] = [
    0, 1, 3, 8, 9, 15, 16, 17, 5, 11, 12, 13, 18, 19, 2, 4, 7, 10, 14, 20, 21, 22,
];

/// A row of `texts_store`'s speech samples' names by its key.
const fn speech_ref(row: SpeechRow) -> u32 {
    row.key()
}

/// The `n`th rally speech name (33 bytes each) among the settings page's speech names, from
/// the one 0xB4 bytes into them, as `SampleName::key` has it (a name's key).
const fn name_key(n: usize) -> u32 {
    assert!(n < 6);
    NameList::MenuSpeech.first_key() + 0xB4 + 33 * n as u32
}

impl MenuLists {
    pub const fn new() -> Self {
        MenuLists(GameThread::new(RefCell::new(ListTables {
            own_places: OwnPlace::rows(OWN_PLACES),
            gap_after_own_places: [0xffff_ffff],
            welcome_championship: WELCOME_CHAMPIONSHIP,
            welcome_other: WELCOME_OTHER,
            map_overlays: MAP_OVERLAYS,
            gap_after_map_overlays: [0xffff_ffff],
            page_pictures_a: PAGE_PICTURES_A,
            page_pictures_b: PAGE_PICTURES_B,
            page_pictures_c: PAGE_PICTURES_C,
            page_pictures_d: PAGE_PICTURES_D,
            page_picture_lists: PAGE_PICTURE_LISTS,
            gap_after_page_picture_lists: [0xffff_ffff],
            page_functions: PAGE_FUNCTIONS,
            gap_after_page_functions: [0xffff_ffff],
            car_page_titles: CAR_PAGE_TITLES,
            slideshow_templates: SLIDESHOW_TEMPLATES,
            inspection_faults: INSPECTION_FAULTS,
            gap_after_inspection_faults: [0xffff_ffff; 4],
            car_kinds: CAR_KINDS,
            section_titles: SECTION_TITLES,
            page_titles_a: PAGE_TITLES_A,
            page_titles_b: PAGE_TITLES_B,
            rally_speeches: RALLY_SPEECHES,
            gap_after_rally_speeches: [0xffff_ffff],
            car_streams: {
                let mut row = [0; 24];
                let mut n = 0;
                while n < 24 {
                    row[n] = speech_ref(SpeechRow::Car(n));
                    n += 1;
                }
                row
            },
            stage_speeches: {
                let mut row = [0; 36];
                let mut n = 0;
                while n < 36 {
                    row[n] = speech_ref(SpeechRow::Stage(n));
                    n += 1;
                }
                row
            },
            gap_after_stage_speeches: [0xffff_ffff],
            cheat_cars: {
                let mut row = [0; 22];
                let mut n = 0;
                while n < 22 {
                    row[n] = car_handle(CHEAT_CARS[n]);
                    n += 1;
                }
                row
            },
            gap_after_cheat_cars: [0xffff_ffff; 12],
            durations: DURATIONS,
            service_pictures: SERVICE_PICTURES,
            gap_after_service_pictures: [0xffff_ffff],
            front_back_path: dwords_of(sparse::<24>(&[(
                0x0,
                &[
                    0x66, 0x69, 0x6c, 0x65, 0x73, 0x5c, 0x67, 0x5c, 0x66, 0x72, 0x6e, 0x74, 0x5c,
                    0x62, 0x61, 0x63, 0x6b, 0x2e, 0x70, 0x63, 0x78,
                ],
            )])),
        })))
    }

    /// The tables, to read: no borrow outlives the statement that reads them.
    pub fn read(&self) -> Ref<'_, ListTables> {
        self.0.get().borrow()
    }

    /// The tables, to write.
    pub fn write(&self) -> RefMut<'_, ListTables> {
        self.0.get().borrow_mut()
    }

    /// Dword `n` of the fields `parts` laid end to end.
    fn read_parts<const N: usize>(
        &self,
        parts: [Part; N],
        n: usize,
        what: &str,
    ) -> Result<u32, crate::errors::BadInput> {
        let own = parts.map(|part| OwnRun { lists: self, part });
        let runs: [&dyn Run; N] = std::array::from_fn(|i| &own[i] as &dyn Run);
        crate::memory::session::read_runs(&runs, n, what)
    }

    /// The picture of car `car` on car page `page`, from the page's list (`page_picture_lists`,
    /// in its order). The cars of the 34 descriptions run to 0x20, past a list's 25: a car past a
    /// list reads on into the lists and tables after it.
    pub(super) fn page_picture(
        &self,
        page: u32,
        car: usize,
    ) -> Result<u32, crate::errors::BadInput> {
        let own = match page {
            0 => self.read().page_pictures_a.get(car).copied(),
            1 => self.read().page_pictures_c.get(car).copied(),
            2 => self.read().page_pictures_d.get(car).copied(),
            3 => self.read().page_pictures_b.get(car).copied(),
            _ => return Err(crate::errors::BadInput("a car page past the four")),
        };
        if let Some(picture) = own {
            return Ok(picture);
        }
        let own = [
            OwnRun {
                lists: self,
                part: Part::PagePicturesA,
            },
            OwnRun {
                lists: self,
                part: Part::PagePicturesB,
            },
            OwnRun {
                lists: self,
                part: Part::PagePicturesC,
            },
            OwnRun {
                lists: self,
                part: Part::PagePicturesD,
            },
            OwnRun {
                lists: self,
                part: Part::PagePictureLists,
            },
            OwnRun {
                lists: self,
                part: Part::GapAfterPagePictureLists,
            },
            OwnRun {
                lists: self,
                part: Part::PageFunctions,
            },
            OwnRun {
                lists: self,
                part: Part::GapAfterPageFunctions,
            },
            OwnRun {
                lists: self,
                part: Part::CarPageTitles,
            },
            OwnRun {
                lists: self,
                part: Part::SlideshowTemplates,
            },
        ];
        let lists: [&dyn Run; 10] = std::array::from_fn(|i| &own[i] as &dyn Run);
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
        let segments = self.runs_from_map_overlays(CarListParts::of(menus));
        crate::memory::session::read_runs(&runs_of(&segments), slot, "a map overlay")
    }

    /// The rally speech of section `section` (0 ..= 0x20, held by `small_section`), past the six
    /// on into the fields after them.
    pub fn rally_speech(&self, section: usize) -> Result<u32, crate::errors::BadInput> {
        if let Some(speech) = self.read().rally_speeches.get(section) {
            return Ok(*speech);
        }
        self.read_parts(
            [
                Part::RallySpeeches,
                Part::GapAfterRallySpeeches,
                Part::CarStreams,
                Part::StageSpeeches,
            ],
            section,
            "a rally speech",
        )
    }

    /// The stage speech `speech` (0 .. 0x40, held by `stage_speech`), past the 36 on into the
    /// fields after them.
    pub fn stage_speech(&self, speech: usize) -> Result<u32, crate::errors::BadInput> {
        if let Some(entry) = self.read().stage_speeches.get(speech) {
            return Ok(*entry);
        }
        self.read_parts(
            [
                Part::StageSpeeches,
                Part::GapAfterStageSpeeches,
                Part::CheatCars,
                Part::GapAfterCheatCars,
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
            self.read().welcome_championship.get(section).copied()
        } else {
            self.read().welcome_other.get(section).copied()
        };
        if let Some(picture) = own {
            return Ok(picture);
        }
        if championship {
            self.read_parts(
                [
                    Part::WelcomeChampionship,
                    Part::WelcomeOther,
                    Part::MapOverlays,
                ],
                section,
                "a welcome picture",
            )
        } else {
            self.read_parts(
                [Part::WelcomeOther, Part::MapOverlays],
                section,
                "a welcome picture",
            )
        }
    }
}

/// The position in `runs_from_map_overlays` of the first run held outside this record.
#[cfg(test)]
const FIRST_MOVED_RUN: usize = 12;

/// The runs as trait objects, for `read_runs` and `write_runs`.
fn runs_of<'s, 'a: 's>(segments: &'s [Segment<'a>; 41]) -> [&'s dyn Run; 41] {
    std::array::from_fn(|i| &segments[i] as &dyn Run)
}

impl MenuLists {
    /// The fields from `map_overlays` to the end, in order, with the lists that moved out of this
    /// record in their places: a list read on past its field goes on into the next, over the
    /// gaps.
    fn runs_from_map_overlays<'a>(&'a self, parts: CarListParts<'a>) -> [Segment<'a>; 41] {
        let service = parts.service;
        let cars = &parts.cars.lists;
        let carousel = parts.carousel;
        let own = |part| Segment::Own(OwnRun { lists: self, part });
        [
            own(Part::MapOverlays),
            own(Part::GapAfterMapOverlays),
            own(Part::PagePicturesA),
            own(Part::PagePicturesB),
            own(Part::PagePicturesC),
            own(Part::PagePicturesD),
            own(Part::PagePictureLists),
            own(Part::GapAfterPagePictureLists),
            own(Part::PageFunctions),
            own(Part::GapAfterPageFunctions),
            own(Part::CarPageTitles),
            own(Part::SlideshowTemplates),
            Segment::Foreign(&service.repair_flags),
            Segment::Foreign(&service.repair_marks),
            Segment::Foreign(&service.repair_time_handles),
            own(Part::InspectionFaults),
            Segment::Foreign(&cars.cars_championship),
            Segment::Foreign(&cars.a8_championship_cars),
            Segment::Foreign(&cars.cars_first),
            Segment::Foreign(&cars.cars_with_clio),
            Segment::Foreign(&cars.secret_cars),
            Segment::Foreign(&cars.cars_scratch),
            Segment::Foreign(&cars.cars_second),
            Segment::Foreign(&cars.shown_cars),
            own(Part::GapAfterInspectionFaults),
            own(Part::CarKinds),
            own(Part::SectionTitles),
            own(Part::PageTitlesA),
            own(Part::PageTitlesB),
            own(Part::RallySpeeches),
            own(Part::GapAfterRallySpeeches),
            own(Part::CarStreams),
            own(Part::StageSpeeches),
            own(Part::GapAfterStageSpeeches),
            own(Part::CheatCars),
            own(Part::GapAfterCheatCars),
            own(Part::Durations),
            own(Part::ServicePictures),
            own(Part::GapAfterServicePictures),
            Segment::Foreign(&carousel.textures),
            own(Part::FrontBackPath),
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
        let segments = self.runs_from_map_overlays(parts);
        let all = runs_of(&segments);
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
        let segments = self.runs_from_map_overlays(parts);
        let all = runs_of(&segments);
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

#[cfg(test)]
mod run_on_tests {
    use super::*;

    /// The own parts in the order of the record, from `welcome_championship` on.
    const IN_ORDER: [Part; 31] = [
        Part::WelcomeChampionship,
        Part::WelcomeOther,
        Part::MapOverlays,
        Part::GapAfterMapOverlays,
        Part::PagePicturesA,
        Part::PagePicturesB,
        Part::PagePicturesC,
        Part::PagePicturesD,
        Part::PagePictureLists,
        Part::GapAfterPagePictureLists,
        Part::PageFunctions,
        Part::GapAfterPageFunctions,
        Part::CarPageTitles,
        Part::SlideshowTemplates,
        Part::InspectionFaults,
        Part::GapAfterInspectionFaults,
        Part::CarKinds,
        Part::SectionTitles,
        Part::PageTitlesA,
        Part::PageTitlesB,
        Part::RallySpeeches,
        Part::GapAfterRallySpeeches,
        Part::CarStreams,
        Part::StageSpeeches,
        Part::GapAfterStageSpeeches,
        Part::CheatCars,
        Part::GapAfterCheatCars,
        Part::Durations,
        Part::ServicePictures,
        Part::GapAfterServicePictures,
        Part::FrontBackPath,
    ];

    /// The lists' whole bytes, as a checksum: a wrong dword in a row shows here. The dwords are
    /// the image's, in its order (the menus' draw list, nothing's, first: 25 times -1).
    #[test]
    fn the_menu_lists_keep_the_bytes_of_the_image() {
        let lists = MenuLists::new();
        let tables = lists.read();
        let mut dwords = vec![0xffff_ffff_u32; 25];
        for place in &tables.own_places {
            dwords.extend([place.size, place.y, place.angle]);
        }
        dwords.extend(tables.gap_after_own_places);
        for part in IN_ORDER {
            dwords.extend(tables.part(part));
        }
        let mut hash = 0xcbf2_9ce4_8422_2325_u64;
        let mut length = 0;
        for byte in dwords.iter().flat_map(|d| d.to_le_bytes()) {
            hash = (hash ^ u64::from(byte)).wrapping_mul(0x100_0000_01b3);
            length += 1;
        }
        assert_eq!(length, 2888);
        assert_eq!(hash, 0x119b_0f69_279b_ce27);
    }

    /// The lists, every dword from `welcome_championship` to the end of the record numbered.
    fn numbered() -> MenuLists {
        let lists = MenuLists::new();
        let mut next = 0x7000;
        for part in IN_ORDER {
            for dword in lists.write().part_mut(part) {
                *dword = next;
                next += 1;
            }
        }
        lists
    }

    /// Dword `n` of the record from `from` on, the parts side by side.
    fn old(lists: &MenuLists, from: Part, n: usize) -> Option<u32> {
        let first = IN_ORDER
            .iter()
            .position(|p| std::mem::discriminant(p) == std::mem::discriminant(&from))
            .unwrap();
        let tables = lists.read();
        let mut rest = n;
        for part in &IN_ORDER[first..] {
            let row = tables.part(*part);
            if rest < row.len() {
                return Some(row[rest]);
            }
            rest -= row.len();
        }
        None
    }

    /// Each reader gives what reading the record's dwords in a row gave, for every index the
    /// game's checks let through.
    #[test]
    fn list_readers_run_on_as_the_old_read() {
        let l = numbered();
        for n in 0..0x21 {
            assert_eq!(
                l.rally_speech(n).ok(),
                old(&l, Part::RallySpeeches, n),
                "rally {n}"
            );
            assert_eq!(
                l.welcome_picture(true, n).ok(),
                old(&l, Part::WelcomeChampionship, n),
                "welcome {n}"
            );
            assert_eq!(
                l.welcome_picture(false, n).ok(),
                old(&l, Part::WelcomeOther, n),
                "other welcome {n}"
            );
        }
        for n in 0..0x40 {
            assert_eq!(
                l.stage_speech(n).ok(),
                old(&l, Part::StageSpeeches, n),
                "speech {n}"
            );
        }
        for (page, part) in [
            (0, Part::PagePicturesA),
            (1, Part::PagePicturesC),
            (2, Part::PagePicturesD),
            (3, Part::PagePicturesB),
        ] {
            for car in 0..64 {
                assert_eq!(
                    l.page_picture(page, car).ok(),
                    old(&l, part, car),
                    "page {page} car {car}"
                );
            }
        }
        assert!(l.page_picture(4, 0).is_err());
        // Up to the first run that lives outside this record (the service's repair flags).
        let menus = crate::test_support::own::menus();
        let room: usize = l.runs_from_map_overlays(CarListParts::of(menus))[..FIRST_MOVED_RUN]
            .iter()
            .map(Run::count)
            .sum();
        for n in 0..room {
            assert_eq!(
                l.map_overlay(menus, n).ok(),
                old(&l, Part::MapOverlays, n),
                "map overlay {n}"
            );
        }
    }

    /// Each car list's place in `runs_from_map_overlays` is its own field's, and a list read on
    /// runs into what follows it, gaps included, to the end of the runs.
    #[test]
    fn car_lists_start_at_their_runs() {
        let l = MenuLists::new();
        let menus = crate::test_support::own::menus();
        let cars = &menus.cars.lists;
        let segments = l.runs_from_map_overlays(CarListParts::of(menus));
        let all = runs_of(&segments);
        let total: usize = all.iter().map(|r| r.count()).sum();
        let foreign = |at: usize| match &segments[at] {
            Segment::Foreign(run) => Some(std::ptr::from_ref(*run).cast::<u8>()),
            Segment::Own(_) => None,
        };
        for (list, table) in [
            (CarList::First, &cars.cars_first[..]),
            (CarList::Second, &cars.cars_second[..]),
            (CarList::WithClio, &cars.cars_with_clio[..]),
            (CarList::Championship, &cars.cars_championship[..]),
            (CarList::A8, &cars.a8_championship_cars[..]),
            (CarList::Shown, &cars.shown_cars[..]),
        ] {
            let start = MenuLists::car_list_start(list);
            assert_eq!(
                foreign(start),
                Some(table.as_ptr().cast::<u8>()),
                "{list:?}"
            );
            let left: usize = all[start..].iter().map(|r| r.count()).sum();
            assert!(left <= total && left > table.len());
            assert!(
                l.car_entry(CarListParts::of(menus), list, (left - 1) as u32)
                    .is_ok()
            );
            assert!(
                l.car_entry(CarListParts::of(menus), list, left as u32)
                    .is_err()
            );
        }
        let start = MenuLists::car_list_start(CarList::Cheat);
        assert!(matches!(
            &segments[start],
            Segment::Own(OwnRun {
                part: Part::CheatCars,
                ..
            })
        ));
        let left: usize = all[start..].iter().map(|r| r.count()).sum();
        assert!(left > 22);
        assert!(
            l.car_entry(CarListParts::of(menus), CarList::Cheat, (left - 1) as u32)
                .is_ok()
        );
        assert!(
            l.car_entry(CarListParts::of(menus), CarList::Cheat, left as u32)
                .is_err()
        );
    }
}
