//! The stages' tables: the track pack (its head and the 36 tracks' entries), the lists that
//! pick a record table (by stage group, by arcade level, and the results'), the championship's
//! records (66 and the -1 that ends them), the twelve events' tables (seven records and a -1
//! each), then 0xB2 bytes nothing names.
//!
//! The records, the pack and the tail are cells of the values they hold, by name. The lists are
//! constants of handles (`Listed`, `StageList`, `RecordTable`, `TimesVariable`). The old
//! layout's order survives as one explicit chain (`Piece::ORDER`): a read past a piece's end
//! goes on into the next piece (a track number from the player's files, `records::trakpak_entry`;
//! the record kind scan, `menu::next_record_kind`), each piece read as the dwords it held there
//! (a list's handle as the address it stood for). Past the last piece a read is refused. The old
//! addresses are kept only in the tests, as the oracle of the chain's order.

use crate::errors::BadInput;
use crate::memory::session::{PlainBytes, PlainDword, PlainInt, Run};
use crate::menu::champ_store::TimesVariable;
use crate::screens::game_screen::Held;

/// A record (0xA4 bytes in the options file): a stage, a service area, a rally's end or a
/// welcome (`tag`), with its two record holders.
#[derive(rc2k_derive::Cells)]
pub struct Record {
    /// 0 a stage, 1 a service area, 3 or 4 a rally's end, 5 a welcome, -1 a table's end.
    pub tag: PlainDword,
    /// The track's entry in the track pack (`records::trakpak_entry`).
    pub track: PlainDword,
    /// The track pack's id.
    pub pack_id: PlainDword,
    /// What a service area hands its step (`Var::ServiceRecordC`); 1 on a stage.
    pub service_figure: PlainDword,
    /// The time of day it starts at (hhmm).
    pub hhmm: PlainDword,
    /// A service area's time (`Var::ServiceTime`); a small number on a stage.
    pub service_time: PlainDword,
    /// The four classes' times (`menu::calc_cpu_times`; the made-up times
    /// read the first and last, `saves::default_times`).
    pub class_times: [PlainDword; 4],
    /// Read by no code by name (a read on through the tables reaches it).
    pub unread_28: PlainDword,
    /// The time trial's record holder's top speed (or figure).
    pub time_trial_speed: PlainDword,
    /// The fastest record holder's speed (or figure, signed: `menu::bests::takes_record`).
    pub fastest_speed: PlainDword,
    /// The race's stage (`RACE_STATE.settings.stage`), the time a bonus counts from.
    pub race_stage: PlainDword,
    /// The stage's name string.
    pub name: PlainDword,
    /// Which ten entries of a stage table are the stage's best times.
    pub event_slot: PlainDword,
    /// The stage's number.
    pub number: PlainDword,
    /// The two points on the map its markers go at, (x, y) each
    /// (`gfx::blit_marker_pair`).
    pub markers: [[PlainInt; 2]; 2],
    /// The fastest record holder's name (no NUL when it fills its bytes).
    pub fastest_name: PlainBytes<HOLDER_NAME_LEN>,
    /// The fastest record holder's nationality.
    pub fastest_nation: PlainDword,
    /// The fastest record holder's car.
    pub fastest_car: PlainDword,
    /// The time trial's record holder's name (set by `menu::keep_stage_best`).
    pub time_trial_name: PlainBytes<HOLDER_NAME_LEN>,
    /// The time trial's record holder's nationality.
    pub time_trial_nation: PlainDword,
    /// The time trial's record holder's car.
    pub time_trial_car: PlainDword,
}

/// A record's two record holders.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Holder {
    /// The stage's fastest holder (arcade, championship and stage panels).
    Fastest,
    /// The time trial's holder: the top speed (figure) `menu::keep_stage_best` keeps.
    TimeTrial,
}

impl Holder {
    /// Where its name (0x20 bytes) is in the record's bytes (`RecordRef::holder_text` reads on
    /// from there).
    pub const fn name_at(self) -> u32 {
        match self {
            Holder::Fastest => 0x54,
            Holder::TimeTrial => 0x7C,
        }
    }
}

/// The bytes a record holder's name has.
pub const HOLDER_NAME_LEN: usize = 0x20;

/// A record holder's numbers.
pub struct HolderRow {
    pub nation: u32,
    pub car: u32,
    pub speed: u32,
}

/// One dword of a record in the old layout's order: a cell, or four bytes of a name.
enum RecordWord<'a> {
    Dword(&'a PlainDword),
    Int(&'a PlainInt),
    Name(&'a PlainBytes<HOLDER_NAME_LEN>, usize),
}

impl RecordWord<'_> {
    fn get(&self) -> u32 {
        match self {
            RecordWord::Dword(cell) => cell.get(),
            RecordWord::Int(cell) => cell.get() as u32,
            RecordWord::Name(name, at) => name.get_u32(*at),
        }
    }

    fn set(&self, value: u32) {
        match self {
            RecordWord::Dword(cell) => cell.set(value),
            RecordWord::Int(cell) => cell.set(value as i32),
            RecordWord::Name(name, at) => name.set_u32(*at, value),
        }
    }
}

impl Record {
    /// The dwords a record has.
    pub const WORDS: usize = 41;

    /// The tag.
    pub fn tag(&self) -> u32 {
        self.tag.get()
    }

    /// The track's entry in the track pack.
    pub fn track(&self) -> u32 {
        self.track.get()
    }

    /// The track pack's id.
    pub fn pack_id(&self) -> u32 {
        self.pack_id.get()
    }

    /// The time of day it starts at (hhmm).
    pub fn hhmm(&self) -> u32 {
        self.hhmm.get()
    }

    /// The four classes' times.
    pub fn class_times(&self) -> [u32; 4] {
        self.class_times.each_ref().map(PlainDword::get)
    }

    /// The race's stage.
    pub fn race_stage(&self) -> u32 {
        self.race_stage.get()
    }

    /// The stage's name string.
    pub fn name(&self) -> u32 {
        self.name.get()
    }

    /// Which ten entries of a stage table are the stage's best times.
    pub fn event_slot(&self) -> u32 {
        self.event_slot.get()
    }

    /// The stage's number.
    pub fn number(&self) -> u32 {
        self.number.get()
    }

    /// Holder `h`'s cells: nationality, car, speed and name.
    fn holder_cells(
        &self,
        h: Holder,
    ) -> (
        &PlainDword,
        &PlainDword,
        &PlainDword,
        &PlainBytes<HOLDER_NAME_LEN>,
    ) {
        match h {
            Holder::Fastest => (
                &self.fastest_nation,
                &self.fastest_car,
                &self.fastest_speed,
                &self.fastest_name,
            ),
            Holder::TimeTrial => (
                &self.time_trial_nation,
                &self.time_trial_car,
                &self.time_trial_speed,
                &self.time_trial_name,
            ),
        }
    }

    /// One of its two record holders: nationality, car and speed (or figure, signed:
    /// `menu::bests::takes_record`).
    pub fn holder(&self, h: Holder) -> HolderRow {
        let (nation, car, speed, _) = self.holder_cells(h);
        HolderRow {
            nation: nation.get(),
            car: car.get(),
            speed: speed.get(),
        }
    }

    /// A record holder's name (the bytes as kept: no NUL when it fills them).
    pub(super) fn holder_name(&self, h: Holder) -> [u8; HOLDER_NAME_LEN] {
        let (.., name) = self.holder_cells(h);
        std::array::from_fn(|k| name.get(k))
    }

    /// Makes `row` and `name` (at most `HOLDER_NAME_LEN` bytes; written without a NUL, the
    /// bytes after it kept) its holder `h`.
    pub fn set_holder(&self, h: Holder, row: &HolderRow, name: &[u8]) {
        let (nation, car, speed, name_cells) = self.holder_cells(h);
        nation.set(row.nation);
        car.set(row.car);
        speed.set(row.speed);
        for (k, &b) in name.iter().enumerate() {
            name_cells.set(k, b);
        }
    }

    /// The two points on the map its markers go at (`gfx::blit_marker_pair`).
    pub fn markers(&self) -> [(i32, i32); 2] {
        self.markers.each_ref().map(|[x, y]| (x.get(), y.get()))
    }

    /// Its dwords in the options file's order: the one place
    /// that knows the record's byte layout.
    fn words(&self) -> [RecordWord<'_>; Record::WORDS] {
        use RecordWord::{Dword as D, Int as I, Name as N};
        let [[x0, y0], [x1, y1]] = &self.markers;
        let [t0, t1, t2, t3] = &self.class_times;
        let (fastest, time_trial) = (&self.fastest_name, &self.time_trial_name);
        [
            D(&self.tag),
            D(&self.track),
            D(&self.pack_id),
            D(&self.service_figure),
            D(&self.hhmm),
            D(&self.service_time),
            D(t0),
            D(t1),
            D(t2),
            D(t3),
            D(&self.unread_28),
            D(&self.time_trial_speed),
            D(&self.fastest_speed),
            D(&self.race_stage),
            D(&self.name),
            D(&self.event_slot),
            D(&self.number),
            I(x0),
            I(y0),
            I(x1),
            I(y1),
            N(fastest, 0),
            N(fastest, 4),
            N(fastest, 8),
            N(fastest, 12),
            N(fastest, 16),
            N(fastest, 20),
            N(fastest, 24),
            N(fastest, 28),
            D(&self.fastest_nation),
            D(&self.fastest_car),
            N(time_trial, 0),
            N(time_trial, 4),
            N(time_trial, 8),
            N(time_trial, 12),
            N(time_trial, 16),
            N(time_trial, 20),
            N(time_trial, 24),
            N(time_trial, 28),
            D(&self.time_trial_nation),
            D(&self.time_trial_car),
        ]
    }

    /// Its bytes as the old layout and the options file hold them.
    fn image_bytes(&self) -> impl Iterator<Item = u8> {
        self.words().into_iter().flat_map(|w| w.get().to_le_bytes())
    }

    /// The dword at byte `at` of the record, for callers that name a field by its offset
    /// (the flows' `Var::Record`; refused off a dword or past the record).
    pub fn word_at(&self, at: u32) -> Result<u32, BadInput> {
        if !at.is_multiple_of(4) {
            return Err(BadInput("a record's field off a dword"));
        }
        self.words()
            .get(at as usize / 4)
            .map(RecordWord::get)
            .ok_or(BadInput("a record's field past its end"))
    }

    /// The record of the options file's dwords `v`.
    const fn new(v: [u32; Record::WORDS]) -> Self {
        const fn name(v: &[u32; Record::WORDS], first: usize) -> PlainBytes<HOLDER_NAME_LEN> {
            let mut bytes = [0; HOLDER_NAME_LEN];
            let mut k = 0;
            while k < HOLDER_NAME_LEN {
                bytes[k] = v[first + k / 4].to_le_bytes()[k % 4];
                k += 1;
            }
            PlainBytes::new(bytes)
        }
        const fn d(v: &[u32; Record::WORDS], k: usize) -> PlainDword {
            PlainDword::new(v[k])
        }
        const fn i(v: &[u32; Record::WORDS], k: usize) -> PlainInt {
            PlainInt::new(v[k] as i32)
        }
        Self {
            tag: d(&v, 0),
            track: d(&v, 1),
            pack_id: d(&v, 2),
            service_figure: d(&v, 3),
            hhmm: d(&v, 4),
            service_time: d(&v, 5),
            class_times: [d(&v, 6), d(&v, 7), d(&v, 8), d(&v, 9)],
            unread_28: d(&v, 10),
            time_trial_speed: d(&v, 11),
            fastest_speed: d(&v, 12),
            race_stage: d(&v, 13),
            name: d(&v, 14),
            event_slot: d(&v, 15),
            number: d(&v, 16),
            markers: [[i(&v, 17), i(&v, 18)], [i(&v, 19), i(&v, 20)]],
            fastest_name: name(&v, 21),
            fastest_nation: d(&v, 29),
            fastest_car: d(&v, 30),
            time_trial_name: name(&v, 31),
            time_trial_nation: d(&v, 39),
            time_trial_car: d(&v, 40),
        }
    }
}

/// A record as the old layout's 41 dwords (the options file's walk, the read on).
impl Run for Record {
    fn count(&self) -> usize {
        Record::WORDS
    }
    fn bits(&self, i: usize) -> u32 {
        self.words()[i].get()
    }
    fn set_bits(&self, i: usize, v: u32) {
        self.words()[i].set(v);
    }
}

/// A record's bytes.
pub const RECORD_LEN: u32 = 0xA4;

const _: () = assert!(Record::WORDS * 4 == RECORD_LEN as usize);

/// An event's table: seven records and the -1 that ends them.
#[derive(rc2k_derive::Cells)]
pub struct EventRecords {
    pub records: [Record; 7],
    pub end: PlainDword,
}

/// An event's table as the old layout's dwords: the records', then the end mark.
impl Run for EventRecords {
    fn count(&self) -> usize {
        self.records.count() + 1
    }
    fn bits(&self, i: usize) -> u32 {
        match self.records.get(i / Record::WORDS) {
            Some(record) => record.bits(i % Record::WORDS),
            None => self.end.get(),
        }
    }
    fn set_bits(&self, i: usize, v: u32) {
        match self.records.get(i / Record::WORDS) {
            Some(record) => record.set_bits(i % Record::WORDS, v),
            None => self.end.set(v),
        }
    }
}

/// A track's entry in the track pack (0x40 bytes in the old layout; fields read from
/// `screens::stage_panel`, `menu::calc_cpu_times`, `menu::run_event_races` and
/// `race::single_rally`).
#[derive(rc2k_derive::Cells)]
pub struct TrackEntry {
    /// The track's name (a string id; also the first word `race::single_rally`
    /// checks for the pack's closing -1).
    pub name: PlainDword,
    /// 0 in all 36 entries and read by no code (probably a spare word between the
    /// name and the duration).
    pub unread_zero_04: PlainDword,
    /// The index into `MenuLists::durations`.
    pub duration: PlainDword,
    /// The stage's length.
    pub length: PlainDword,
    /// The description's string id.
    pub description: PlainDword,
    /// The weather.
    pub weather: PlainDword,
    /// 1 in all 36 entries and read by no code (probably a spare word after the
    /// weather).
    pub unread_one_18: PlainDword,
    /// The temperature (signed on the panel, a plain dword in the race's settings).
    pub temperature: PlainDword,
    /// The wind.
    pub wind: PlainDword,
    /// The four surfaces.
    pub surfaces: [PlainDword; 4],
    /// The stage number (what `race::single_rally` looks the entry up by).
    pub stage: PlainDword,
    /// The stage's number on the panel.
    pub number: PlainDword,
    /// 1 when the number's label is the singular.
    pub number_label_one: PlainDword,
}

/// A field of a track's entry, by name (`records::trakpak_entry::Track::get`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TrackField {
    Name,
    Duration,
    Length,
    Description,
    Weather,
    Temperature,
    Wind,
    Surface0,
    Surface1,
    Surface2,
    Surface3,
    Stage,
    Number,
    NumberLabelOne,
}

impl TrackField {
    /// The four surfaces, in order.
    pub const SURFACES: [TrackField; 4] = [
        TrackField::Surface0,
        TrackField::Surface1,
        TrackField::Surface2,
        TrackField::Surface3,
    ];

    /// Its dword in the entry's file layout (the name first).
    pub const fn word(self) -> usize {
        match self {
            TrackField::Name => 0,
            TrackField::Duration => 2,
            TrackField::Length => 3,
            TrackField::Description => 4,
            TrackField::Weather => 5,
            TrackField::Temperature => 7,
            TrackField::Wind => 8,
            TrackField::Surface0 => 9,
            TrackField::Surface1 => 10,
            TrackField::Surface2 => 11,
            TrackField::Surface3 => 12,
            TrackField::Stage => 13,
            TrackField::Number => 14,
            TrackField::NumberLabelOne => 15,
        }
    }
}

impl TrackEntry {
    /// The dwords an entry has.
    pub const WORDS: usize = 16;

    fn words(&self) -> [&PlainDword; TrackEntry::WORDS] {
        let [s0, s1, s2, s3] = &self.surfaces;
        [
            &self.name,
            &self.unread_zero_04,
            &self.duration,
            &self.length,
            &self.description,
            &self.weather,
            &self.unread_one_18,
            &self.temperature,
            &self.wind,
            s0,
            s1,
            s2,
            s3,
            &self.stage,
            &self.number,
            &self.number_label_one,
        ]
    }

    pub fn get(&self, field: TrackField) -> u32 {
        self.words()[field.word()].get()
    }
}

/// A track's entry as the old layout's 16 dwords (the read on past the pack).
impl Run for TrackEntry {
    fn count(&self) -> usize {
        TrackEntry::WORDS
    }
    fn bits(&self, i: usize) -> u32 {
        self.words()[i].get()
    }
    fn set_bits(&self, i: usize, v: u32) {
        self.words()[i].set(v);
    }
}

const fn track(v: [u32; 16]) -> TrackEntry {
    TrackEntry {
        name: PlainDword::new(v[0]),
        unread_zero_04: PlainDword::new(v[1]),
        duration: PlainDword::new(v[2]),
        length: PlainDword::new(v[3]),
        description: PlainDword::new(v[4]),
        weather: PlainDword::new(v[5]),
        unread_one_18: PlainDword::new(v[6]),
        temperature: PlainDword::new(v[7]),
        wind: PlainDword::new(v[8]),
        surfaces: [
            PlainDword::new(v[9]),
            PlainDword::new(v[10]),
            PlainDword::new(v[11]),
            PlainDword::new(v[12]),
        ],
        stage: PlainDword::new(v[13]),
        number: PlainDword::new(v[14]),
        number_label_one: PlainDword::new(v[15]),
    }
}

/// The track pack: the link, the pack's id, its name, the count, the entries
/// (`records::trakpak_entry`, `race::single_rally`) and a -1.
#[derive(rc2k_derive::Cells)]
pub struct TrakPak {
    pub link: PlainDword,
    pub pack_id: PlainDword,
    /// "Track Pack 1".
    pub name: PlainBytes<0x28>,
    pub count: PlainDword,
    pub tracks: [TrackEntry; 36],
    pub end: PlainDword,
}

/// The stages' tables' cells (the lists are constants: `GROUP_LISTS` and the others).
#[derive(rc2k_derive::Cells)]
pub struct StageTables {
    /// The track pack.
    pub trak_pak: TrakPak,
    /// The championship's records.
    pub championship: [Record; 66],
    pub championship_end: PlainDword,
    /// The events' tables.
    pub events: [EventRecords; 12],
    /// Bytes nothing names.
    pub tail: PlainBytes<TAIL_LEN>,
}

/// The tail's bytes.
const TAIL_LEN: usize = 0xB2;

/// An entry of a built-in list: a thing, or the -1 that ends the list.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Listed<T> {
    Entry(T),
    End,
}

/// A thing a built-in list names, as the address it stood for in the old layout (what a read on
/// through the tables sees in the list's dword).
pub trait ImageName: Copy {
    fn image_name(self) -> u32;
}

impl<T: ImageName> Listed<T> {
    /// The dword the old layout held: the thing's address, or -1.
    fn image_word(self) -> u32 {
        match self {
            Listed::Entry(thing) => thing.image_name(),
            Listed::End => u32::MAX,
        }
    }
}

impl ImageName for TimesVariable {
    fn image_name(self) -> u32 {
        TimesVariable::image_name(self)
    }
}

impl ImageName for RecordTable {
    fn image_name(self) -> u32 {
        match self {
            RecordTable::Championship => 0x735576,
            RecordTable::Event(k) => 0x737FC2 + 0x480 * k as u32,
        }
    }
}

/// One of the three lists of event tables, one after the other: the two stage groups' and the
/// arcade levels'; a list read past its -1 reads on into the next (`group_event_table`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum StageList {
    GroupA,
    GroupB,
    Levels,
}

impl ImageName for StageList {
    fn image_name(self) -> u32 {
        match self {
            StageList::GroupA => 0x735522,
            StageList::GroupB => 0x73553E,
            StageList::Levels => 0x73555A,
        }
    }
}

impl StageList {
    /// The three lists in the old layout's order.
    const ORDER: [StageList; 3] = [StageList::GroupA, StageList::GroupB, StageList::Levels];

    const fn entries(self) -> &'static [Listed<RecordTable>; 7] {
        match self {
            StageList::GroupA => &GROUP_LIST_A,
            StageList::GroupB => &GROUP_LIST_B,
            StageList::Levels => &LEVEL_TABLES,
        }
    }

    /// Entry `n` from this list's first on, reading on into the lists after it; `None` past
    /// the last (refused, not read on into the championship's records).
    fn entry_on(self, n: usize) -> Option<Listed<RecordTable>> {
        StageList::ORDER
            .into_iter()
            .skip_while(|&list| list != self)
            .flat_map(|list| list.entries().iter().copied())
            .nth(n)
    }
}

/// The stage results' four best-times tables and a -1 (`screens::stage_results`).
pub const RESULT_TABLES: [Listed<TimesVariable>; 5] = [
    Listed::Entry(TimesVariable::StageTimes),
    Listed::Entry(TimesVariable::CourseTimes),
    Listed::Entry(TimesVariable::TimeTrialTimes),
    Listed::Entry(TimesVariable::StageTimesAlt),
    Listed::End,
];

/// Per stage group, its list of event tables.
const GROUP_LISTS: [StageList; 4] = [
    StageList::GroupB,
    StageList::GroupA,
    StageList::GroupB,
    StageList::GroupA,
];

/// The event tables `Event(5)` down to `Event(0)` and a -1.
const EVENTS_DOWN: [Listed<RecordTable>; 7] = [
    Listed::Entry(RecordTable::Event(5)),
    Listed::Entry(RecordTable::Event(4)),
    Listed::Entry(RecordTable::Event(3)),
    Listed::Entry(RecordTable::Event(2)),
    Listed::Entry(RecordTable::Event(1)),
    Listed::Entry(RecordTable::Event(0)),
    Listed::End,
];

/// The first stage group's six event tables and a -1.
const GROUP_LIST_A: [Listed<RecordTable>; 7] = EVENTS_DOWN;

/// The second stage group's six event tables and a -1.
const GROUP_LIST_B: [Listed<RecordTable>; 7] = [
    Listed::Entry(RecordTable::Event(6)),
    Listed::Entry(RecordTable::Event(7)),
    Listed::Entry(RecordTable::Event(8)),
    Listed::Entry(RecordTable::Event(9)),
    Listed::Entry(RecordTable::Event(10)),
    Listed::Entry(RecordTable::Event(11)),
    Listed::End,
];

/// Per arcade level (and the championship's stages), an event table, and a -1.
const LEVEL_TABLES: [Listed<RecordTable>; 7] = EVENTS_DOWN;

/// The tables' pieces in the old layout's order: the chain a read on goes along.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Piece {
    /// The pack's link, id, name (10 dwords) and count.
    PackHead,
    Tracks,
    PackEnd,
    ResultTables,
    GroupLists,
    GroupListA,
    GroupListB,
    LevelTables,
    Championship,
    ChampionshipEnd,
    Events,
    /// 0xB2 bytes: 44 whole dwords and two bytes no dword read reaches whole.
    Tail,
}

impl Piece {
    /// The chain, in the old layout's order (`chain_tests` checks it against the old addresses).
    pub const ORDER: [Piece; 12] = [
        Piece::PackHead,
        Piece::Tracks,
        Piece::PackEnd,
        Piece::ResultTables,
        Piece::GroupLists,
        Piece::GroupListA,
        Piece::GroupListB,
        Piece::LevelTables,
        Piece::Championship,
        Piece::ChampionshipEnd,
        Piece::Events,
        Piece::Tail,
    ];

    fn count(self, t: &StageTables) -> usize {
        match self {
            Piece::PackHead => 3 + t.trak_pak.name.count(),
            Piece::Tracks => t.trak_pak.tracks.count(),
            Piece::PackEnd | Piece::ChampionshipEnd => 1,
            Piece::ResultTables => RESULT_TABLES.len(),
            Piece::GroupLists => GROUP_LISTS.len(),
            Piece::GroupListA => GROUP_LIST_A.len(),
            Piece::GroupListB => GROUP_LIST_B.len(),
            Piece::LevelTables => LEVEL_TABLES.len(),
            Piece::Championship => t.championship.count(),
            Piece::Events => t.events.count(),
            Piece::Tail => t.tail.count(),
        }
    }

    fn len(self, t: &StageTables) -> usize {
        match self {
            Piece::Tail => TAIL_LEN,
            _ => 4 * self.count(t),
        }
    }

    /// Its dword `i` (below `count`), as the old layout held it.
    fn word(self, t: &StageTables, i: usize) -> u32 {
        let pack = &t.trak_pak;
        match self {
            Piece::PackHead => match i {
                0 => pack.link.get(),
                1 => pack.pack_id.get(),
                _ if i - 2 < pack.name.count() => pack.name.get_u32(4 * (i - 2)),
                _ => pack.count.get(),
            },
            Piece::Tracks => pack.tracks.bits(i),
            Piece::PackEnd => pack.end.get(),
            Piece::ResultTables => RESULT_TABLES[i].image_word(),
            Piece::GroupLists => GROUP_LISTS[i].image_name(),
            Piece::GroupListA => GROUP_LIST_A[i].image_word(),
            Piece::GroupListB => GROUP_LIST_B[i].image_word(),
            Piece::LevelTables => LEVEL_TABLES[i].image_word(),
            Piece::Championship => t.championship.bits(i),
            Piece::ChampionshipEnd => t.championship_end.get(),
            Piece::Events => t.events.bits(i),
            Piece::Tail => t.tail.get_u32(4 * i),
        }
    }

    fn byte(self, t: &StageTables, k: usize) -> u8 {
        match self {
            Piece::Tail => t.tail.get(k),
            _ => self.word(t, k / 4).to_le_bytes()[k % 4],
        }
    }

    fn and_after(self) -> impl Iterator<Item = Piece> {
        Piece::ORDER.into_iter().skip_while(move |&p| p != self)
    }
}

impl StageTables {
    /// Dword `n` from piece `from`'s first on, reading on into the pieces after it; `None` past
    /// the last piece.
    pub(super) fn word_on(&self, from: Piece, n: usize) -> Option<u32> {
        let mut n = n;
        for piece in from.and_after() {
            let count = piece.count(self);
            if n < count {
                return Some(piece.word(self, n));
            }
            n -= count;
        }
        None
    }

    /// Byte `at` from piece `from`'s first on (`word_on`'s bytes, and the tail's last two).
    fn byte_on(&self, from: Piece, at: usize) -> Option<u8> {
        let mut at = at;
        for piece in from.and_after() {
            let len = piece.len(self);
            if at < len {
                return Some(piece.byte(self, at));
            }
            at -= len;
        }
        None
    }

    /// The tables from piece `from` on as a reader of bytes (for `rc2k_core`'s walks).
    pub fn bytes_on(&self, from: Piece) -> BytesOn<'_> {
        BytesOn { tables: self, from }
    }
}

/// The stage tables' bytes from a piece on, in the old layout's order (`StageTables::bytes_on`).
pub struct BytesOn<'a> {
    tables: &'a StageTables,
    from: Piece,
}

impl rc2k_core::strings::bytes::Bytes for BytesOn<'_> {
    fn byte(&self, at: usize) -> Option<u8> {
        self.tables.byte_on(self.from, at)
    }
}

impl StageTables {
    pub const fn new() -> Self {
        let mut tracks = [const { track([0; 16]) }; 36];
        let mut n = 0;
        while n < 36 {
            tracks[n] = track(TRACK_ROWS[n]);
            n += 1;
        }
        let mut championship = [const { Record::new([0; Record::WORDS]) }; 66];
        let mut n = 0;
        while n < 66 {
            championship[n] = Record::new(CHAMPIONSHIP_ROWS[n]);
            n += 1;
        }
        let mut events = [const { EventRecords::new([[0; Record::WORDS]; 7]) }; 12];
        let mut n = 0;
        while n < 12 {
            events[n] = EventRecords::new(EVENT_ROWS[n]);
            n += 1;
        }
        StageTables {
            trak_pak: TrakPak {
                link: PlainDword::new(0x0),
                pack_id: PlainDword::new(0x0),
                name: PlainBytes::new(PACK_NAME),
                count: PlainDword::new(0x24),
                tracks,
                end: PlainDword::new(0xffffffff),
            },
            championship,
            championship_end: PlainDword::new(0xffffffff),
            events,
            tail: PlainBytes::new(TAIL_BYTES),
        }
    }
}

impl EventRecords {
    /// An event's table of `rows` and the -1 that ends it.
    const fn new(rows: [[u32; Record::WORDS]; 7]) -> Self {
        let mut records = [const { Record::new([0; Record::WORDS]) }; 7];
        let mut n = 0;
        while n < 7 {
            records[n] = Record::new(rows[n]);
            n += 1;
        }
        EventRecords {
            records,
            end: PlainDword::new(0xffffffff),
        }
    }
}

/// The track pack's name, NUL-padded to its field.
const PACK_NAME: [u8; 0x28] = {
    let text = b"Track Pack 1";
    let mut name = [0; 0x28];
    let mut k = 0;
    while k < text.len() {
        name[k] = text[k];
        k += 1;
    }
    name
};

/// The 36 tracks' entries (`TrackEntry`'s dwords).
#[rustfmt::skip]
const TRACK_ROWS: [[u32; 16]; 36] = [
    [
        0x3e8, 0, 0, 0x4b0, 0x40c, 0x4b6, 1, 13, 12, 0, 95, 5, 0, 41, 108, 1,
    ],
    [
        0x3e9, 0, 1, 0x3b7, 0x40f, 0x4b7, 1, 10, 37, 0, 100, 0, 0, 42, 115, 1,
    ],
    [
        0x3ea, 0, 2, 0x5ea, 0x412, 0x4b8, 1, 7, 38, 0, 100, 0, 0, 43, 116, 1,
    ],
    [
        0x3eb, 0, 3, 0x5e8, 0x415, 0x4b9, 1, 2, 37, 0, 0, 0, 100, 44, 117, 1,
    ],
    [
        0x3ec, 0, 4, 0x665, 0x418, 0x4ba, 1, 0, 80, 0, 0, 0, 100, 45, 117, 1,
    ],
    [
        0x3ed, 0, 5, 0x4f3, 0x41b, 0x4bb, 1, 0xfffffffe, 133, 0, 0, 0, 100, 46, 117, 1,
    ],
    [
        0x3ee, 0, 6, 0x96a, 0x41e, 0x4bc, 1, 3, 40, 0, 100, 0, 0, 61, 79, 1,
    ],
    [
        0x3ef, 0, 7, 0x725, 0x421, 0x4bd, 1, 12, 29, 0, 100, 0, 0, 62, 80, 1,
    ],
    [
        0x3f0, 0, 8, 0x739, 0x424, 0x4be, 1, 10, 32, 0, 100, 0, 0, 63, 85, 1,
    ],
    [
        0x3f1, 0, 9, 0xa65, 0x427, 0x4bf, 1, 5, 121, 0, 100, 0, 0, 64, 86, 1,
    ],
    [
        0x3f2, 0, 10, 0x370, 0x42a, 0x4c0, 1, 4, 124, 0, 100, 0, 0, 65, 87, 1,
    ],
    [
        0x3f3, 0, 11, 0x71e, 0x42d, 0x4c1, 1, 3, 100, 0, 100, 0, 0, 66, 87, 1,
    ],
    [
        0x3f4, 0, 12, 0x234, 0x430, 0x4c2, 1, 7, 2, 7, 93, 0, 0, 31, 73, 1,
    ],
    [
        0x3f5, 0, 13, 0x227, 0x433, 0x4c3, 1, 12, 0, 0, 100, 0, 0, 32, 76, 1,
    ],
    [
        0x3f6, 0, 14, 0x27e, 0x436, 0x4c4, 1, 16, 1, 0, 100, 0, 0, 33, 77, 1,
    ],
    [
        0x3f7, 0, 15, 0x262, 0x439, 0x4c5, 1, 12, 5, 0, 100, 0, 0, 34, 78, 1,
    ],
    [
        0x3f8, 0, 16, 0x736, 0x43c, 0x4c6, 1, 6, 38, 0, 100, 0, 0, 35, 79, 1,
    ],
    [
        0x3f9, 0, 17, 0x762, 0x43f, 0x4c7, 1, 4, 117, 0, 100, 0, 0, 36, 83, 1,
    ],
    [
        0x3fa, 0, 18, 0x1c3, 0x442, 0x4c8, 1, 22, 8, 95, 4, 1, 0, 71, 67, 1,
    ],
    [
        0x3fb, 0, 19, 0x28d, 0x445, 0x4c9, 1, 21, 7, 90, 9, 1, 0, 72, 74, 1,
    ],
    [
        0x3fc, 0, 20, 0x2d8, 0x448, 0x4ca, 1, 19, 7, 91, 9, 0, 0, 73, 74, 1,
    ],
    [
        0x3fd, 0, 21, 0x26d, 0x44b, 0x4cb, 1, 14, 10, 85, 13, 2, 0, 74, 74, 1,
    ],
    [
        0x3fe, 0, 22, 202, 0x44e, 0x4cc, 1, 12, 12, 80, 14, 6, 0, 75, 74, 1,
    ],
    [
        0x3ff, 0, 23, 0x2d4, 0x451, 0x4cd, 1, 10, 11, 83, 17, 0, 0, 76, 74, 1,
    ],
    [
        0x400, 0, 24, 0x3cf, 0x454, 0x4ce, 1, 14, 0x11f, 97, 0, 3, 0, 51, 8, 0,
    ],
    [
        0x401, 0, 25, 0x48e, 0x457, 0x4cf, 1, 16, 159, 88, 5, 7, 0, 52, 13, 0,
    ],
    [
        0x402, 0, 26, 0x456, 0x45a, 0x4d0, 1, 19, 110, 96, 3, 1, 0, 53, 14, 0,
    ],
    [
        0x403, 0, 27, 0x464, 0x45d, 0x4d1, 1, 24, 104, 74, 25, 1, 0, 54, 15, 0,
    ],
    [
        0x404, 0, 28, 0x38f, 0x460, 0x4d2, 1, 26, 41, 74, 25, 1, 0, 55, 19, 0,
    ],
    [
        0x405, 0, 29, 0x3a4, 0x463, 0x4d3, 1, 23, 23, 88, 7, 5, 0, 56, 20, 0,
    ],
    [
        0x406, 0, 30, 0x278, 0x466, 0x4d4, 1, 31, 15, 100, 0, 0, 0, 21, 95, 1,
    ],
    [
        0x407, 0, 31, 0x367, 0x469, 0x4d5, 1, 29, 40, 99, 0, 1, 0, 22, 95, 1,
    ],
    [
        0x408, 0, 32, 0x455, 0x46c, 0x4d6, 1, 26, 2, 77, 23, 0, 0, 23, 95, 1,
    ],
    [
        0x409, 0, 33, 0x19a, 0x46f, 0x4d7, 1, 24, 0, 98, 2, 0, 0, 24, 95, 1,
    ],
    [
        0x40a, 0, 34, 0x46e, 0x472, 0x4d8, 1, 22, 0, 99, 1, 0, 0, 25, 95, 1,
    ],
    [
        0x40b, 0, 35, 0x56a, 0x475, 0x4d9, 1, 21, 1, 98, 1, 1, 0, 26, 95, 1,
    ],
];

/// The championship's 66 records (`Record`'s 41 dwords).
#[rustfmt::skip]
const CHAMPIONSHIP_ROWS: [[u32; Record::WORDS]; 66] = [
    [
        5, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x13880, 0x3dc, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    ],
    [
        1, 0, 0, 0, 0x4dd, 0x960, 0, 0, 0, 0, 0, 0, 0, 0x13880, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    ],
    [
        0, 0, 0, 1, 0x514, 5, 0xd73c, 0xee48, 0xf938, 0x101d0, 0x23bce, 0, 0x5f0, 0x13880, 0, 0, 1,
        0x14c, 193, 96, 244, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    ],
    [
        0, 1, 0, 1, 0x54b, 10, 0xc6d7, 0xcf7b, 0xd41c, 0xd9e7, 0x1edae, 0, 0x5f0, 0x13880, 0, 1, 2,
        0x14c, 255, 241, 184, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    ],
    [
        1, 0, 0, 0, 0x5a5, 0x384, 0, 0, 0, 0, 0, 0, 0, 0x13880, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    ],
    [
        0, 2, 0, 1, 0x5e1, 12, 0x10fad, 0x13028, 0x132a2, 0x13575, 0x2b8bb, 0, 0x5f0, 0x13880, 0, 2,
        3, 0x13b, 162, 75, 0x1a0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    ],
    [
        0, 3, 0, 1, 0x64f, 10, 0x13486, 0x13bb7, 0x1453c, 0x145cd, 0x2ba4b, 0, 0x611, 0x13880, 0, 3,
        4, 0x106, 0x141, 0x159, 210, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    ],
    [
        1, 0, 0, 0, 0x6d6, 0x5dc, 0, 0, 0, 0, 0, 0, 0, 0x13880, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    ],
    [
        0, 4, 0, 1, 0x735, 12, 0x131c2, 0x15b9c, 0x16e60, 0x1657f, 0x2451b, 0, 0x529, 0x13880, 0, 4,
        5, 53, 0x19c, 0x125, 202, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    ],
    [
        0, 5, 0, 1, 0x7f8, 12, 0x10abc, 0x11814, 0x125c0, 0x1335d, 0x2451b, 0, 0x529, 0x13880, 0, 5,
        6, 206, 0x18d, 0x136, 0x198, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    ],
    [
        4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x13880, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    ],
    [
        5, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x13880, 0x3dd, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    ],
    [
        1, 0, 0, 0, 0x267, 0x960, 0, 0, 0, 0, 0, 0, 0, 0x13880, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    ],
    [
        0, 6, 0, 1, 0x276, 10, 0x1ba6c, 0x1dbe6, 0x1e5b2, 0x1ea07, 0x3086b, 0, 0x549, 0x13880, 0, 6,
        1, 68, 0x18c, 0x11e, 0x182, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    ],
    [
        1, 0, 0, 0, 0x32f, 0x384, 0, 0, 0, 0, 0, 0, 0, 0x13880, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    ],
    [
        0, 7, 0, 1, 0x384, 12, 0x160a2, 0x17700, 0x188b4, 0x1a24f, 0x1cfeb, 0, 0x549, 0x13880, 0, 7,
        2, 0x144, 0x173, 87, 0x15f, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    ],
    [
        1, 0, 0, 0, 0x406, 0x30c, 0, 0, 0, 0, 0, 0, 0, 0x13880, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    ],
    [
        0, 8, 0, 1, 0x451, 8, 0x142a8, 0x153d8, 0x15b44, 0x165d0, 0x2d82d, 0, 0x4c5, 0x13880, 0, 8,
        3, 54, 0x144, 159, 0x1c2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    ],
    [
        1, 0, 0, 0, 0x4d3, 0x3c0, 0, 0, 0, 0, 0, 0, 0, 0x13880, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    ],
    [
        0, 9, 0, 1, 0x523, 25, 0x19258, 0x1af40, 0x1c32c, 0x1d4c0, 0x3b86f, 0, 0x611, 0x13880, 0, 9,
        4, 88, 0x164, 0x147, 0x16c, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    ],
    [
        1, 0, 0, 0, 0x5a5, 0x5a0, 0, 0, 0, 0, 0, 0, 0, 0x13880, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    ],
    [
        0, 10, 0, 1, 0x5f5, 18, 0xafe4, 0xc289, 0xc4e0, 0xc739, 0x25ee3, 0, 0x549, 0x13880, 0, 10,
        5, 0x15b, 207, 157, 0x1c0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    ],
    [
        0, 11, 0, 1, 0x654, 18, 0x151a1, 0x165af, 0x17bd9, 0x1825b, 0x25ee3, 0, 0x549, 0x13880, 0,
        11, 6, 127, 177, 120, 0x177, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    ],
    [
        4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x13880, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    ],
    [
        5, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x13880, 0x3de, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    ],
    [
        1, 0, 0, 0, 0x1f4, 0x960, 0, 0, 0, 0, 0, 0, 0, 0x13880, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    ],
    [
        0, 12, 0, 1, 0x208, 10, 0xb570, 0xc59d, 0xcfe8, 0xd26c, 0x2ba4b, 0, 0x4c7, 0x13880, 0, 12,
        1, 160, 0x156, 0x12a, 236, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    ],
    [
        0, 13, 0, 1, 0x262, 5, 0x6bce, 0x6f54, 0x73a5, 0x7c61, 0x3391f, 0, 0x50c, 0x13880, 0, 13, 2,
        0x12f, 0x178, 170, 202, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    ],
    [
        0, 14, 0, 1, 0x2bc, 4, 0x7a0c, 0x7c9c, 0x7f58, 0x82ac, 0x1f6fb, 0, 0x5ac, 0x13880, 0, 14, 3,
        0x145, 0x122, 115, 220, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    ],
    [
        1, 0, 0, 0, 0x2ee, 0x294, 0, 0, 0, 0, 0, 0, 0, 0x13880, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    ],
    [
        0, 15, 0, 1, 0x343, 4, 0x9222, 0x954e, 0x9978, 0x9e7c, 0x1f6fb, 0, 0x5ac, 0x13880, 0, 15, 4,
        156, 0x1c0, 0x142, 139, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    ],
    [
        0, 16, 0, 1, 0x39d, 4, 0x17308, 0x18cc2, 0x197a1, 0x19b41, 0x1f6fb, 0, 0x5ac, 0x13880, 0,
        16, 5, 145, 0x168, 0x109, 0x1d0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    ],
    [
        1, 0, 0, 0, 0x41f, 0x384, 0, 0, 0, 0, 0, 0, 0, 0x13880, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    ],
    [
        0, 17, 0, 1, 0x460, 4, 0x19025, 0x1a9c8, 0x1c2d1, 0x1caa3, 0x1f6fb, 0, 0x5ac, 0x13880, 0,
        17, 6, 81, 0x108, 231, 0x1cb, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    ],
    [
        4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x13880, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    ],
    [
        5, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x13880, 0x3df, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    ],
    [
        1, 0, 0, 0, 0x72b, 0x960, 0, 0, 0, 0, 0, 0, 0, 0x13880, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    ],
    [
        0, 18, 0, 1, 0x73f, 15, 0x4601, 0x4fe6, 0x53d0, 0x539a, 0x23bce, 0, 0x5f0, 0x13880, 0, 18,
        1, 0x114, 0x1b6, 0x149, 147, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    ],
    [
        0, 19, 0, 1, 0x799, 18, 0x81ef, 0x849c, 0x8852, 0x8a9d, 0x1edae, 0, 0x5f0, 0x13880, 0, 19,
        2, 189, 0x1a5, 0x113, 0x1c3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    ],
    [
        0, 20, 0, 1, 0x807, 12, 0x6d25, 0x6d2c, 0x72ff, 0x771a, 0x2b8bb, 0, 0x5f0, 0x13880, 0, 20,
        3, 69, 0x18d, 0x11d, 168, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    ],
    [
        1, 0, 0, 0, 0x861, 0x294, 0, 0, 0, 0, 0, 0, 0, 0x13880, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    ],
    [
        0, 21, 0, 1, 0x89d, 6, 0x7bbd, 0x8253, 0x83c1, 0x87c2, 0x2ba4b, 0, 0x611, 0x13880, 0, 21, 4,
        150, 190, 0x159, 0x18c, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    ],
    [
        0, 22, 0, 1, 0x8fc, 15, 0x2447, 0x2714, 0x2931, 0x2a47, 0x2451b, 0, 0x529, 0x13880, 0, 22,
        5, 0x110, 0x155, 203, 0x18b, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    ],
    [
        0, 23, 0, 1, 0x91f, 15, 0x82f3, 0x89ec, 0x8d0c, 0x9212, 0x2451b, 0, 0x529, 0x13880, 0, 23,
        6, 0x12e, 236, 0x15e, 0x14d, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    ],
    [
        4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x13880, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    ],
    [
        5, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x13880, 0x3e0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    ],
    [
        1, 0, 0, 0, 0x3f7, 0x960, 0, 0, 0, 0, 0, 0, 0, 0x13880, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    ],
    [
        0, 24, 0, 1, 0x406, 10, 0xbf7c, 0xc942, 0xd12c, 0xd629, 0x3086b, 0, 0x549, 0x13880, 0, 24,
        1, 0x145, 156, 221, 0x1d1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    ],
    [
        0, 25, 0, 1, 0x46f, 5, 0xe994, 0xfe8c, 0x105e2, 0x106af, 0x3b86f, 0, 0x611, 0x13880, 0, 25,
        2, 0x110, 0x1d2, 237, 154, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    ],
    [
        1, 0, 0, 0, 0x4d8, 0x348, 0, 0, 0, 0, 0, 0, 0, 0x13880, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    ],
    [
        0, 26, 0, 1, 0x519, 18, 0xdb60, 0xe42f, 0xe906, 0xf145, 0x25ee3, 0, 0x549, 0x13880, 0, 26,
        3, 255, 0x1c0, 172, 207, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    ],
    [
        0, 27, 0, 1, 0x578, 10, 0xf406, 0x10371, 0x10d9f, 0x107da, 0x2ba4b, 0, 0x4c7, 0x13880, 0,
        27, 4, 120, 0x192, 0x15a, 0x12c, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    ],
    [
        1, 0, 0, 0, 0x5e6, 0x384, 0, 0, 0, 0, 0, 0, 0, 0x13880, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    ],
    [
        0, 28, 0, 1, 0x5ff, 5, 0xb301, 0xc0a6, 0xc685, 0xcc1c, 0x3391f, 0, 0x50c, 0x13880, 0, 28, 5,
        0x119, 0x141, 0x154, 0x1b7, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    ],
    [
        0, 29, 0, 1, 0x654, 5, 0xabd3, 0xb33f, 0xbac3, 0xc56f, 0x3391f, 0, 0x50c, 0x13880, 0, 29, 6,
        0x155, 0x197, 0x100, 0x1b1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    ],
    [
        4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x13880, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    ],
    [
        5, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x13880, 0x3e1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    ],
    [
        1, 0, 0, 0, 0x6b3, 0x960, 0, 0, 0, 0, 0, 0, 0, 0x13880, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    ],
    [
        0, 30, 0, 1, 0x6c2, 9, 0x61c7, 0x6610, 0x6a35, 0x6ff5, 0x1f6fb, 0, 0x5ac, 0x13880, 0, 30, 1,
        0x140, 207, 62, 0x126, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    ],
    [
        0, 31, 0, 1, 0x71c, 7, 0x9df9, 0x9f3f, 0xa348, 0xa83e, 0x1f6fb, 0, 0x5ac, 0x13880, 0, 31, 2,
        32, 0x18a, 0x15d, 232, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    ],
    [
        0, 32, 0, 1, 0x77b, 4, 0xd530, 0xdfde, 0xeb6b, 0xf0f9, 0x1f6fb, 0, 0x5ac, 0x13880, 0, 32, 3,
        101, 0x1a9, 0x154, 0x137, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    ],
    [
        1, 0, 0, 0, 0x7e9, 0x3c0, 0, 0, 0, 0, 0, 0, 0, 0x13880, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    ],
    [
        0, 33, 0, 1, 0x834, 4, 0x53b4, 0x58e4, 0x5d04, 0x5de7, 0x1f6fb, 0, 0x5ac, 0x13880, 0, 33, 4,
        0x113, 159, 226, 0x1c0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    ],
    [
        0, 34, 0, 1, 0x866, 10, 0xcbfd, 0xdc36, 0xdea8, 0xe138, 0x1f6fb, 0, 0x5ac, 0x13880, 0, 34,
        5, 0x13f, 0x1cd, 96, 249, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    ],
    [
        0, 35, 0, 1, 0x8c5, 10, 0xde04, 0xe804, 0xebff, 0xf380, 0x1f6fb, 0, 0x5ac, 0x13880, 0, 35,
        6, 236, 0x1c5, 0x13a, 126, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    ],
    [
        3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x13880, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    ],
];

/// The twelve events' seven records each.
#[rustfmt::skip]
const EVENT_ROWS: [[[u32; Record::WORDS]; 7]; 12] = [
    [
        [
            5, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x13880, 0x3e7, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        ],
        [
            0, 0, 0, 1, 0x514, 5, 0xd73c, 0xee48, 0xf938, 0x101d0, 0x23bce, 0, 0x5f0, 0x11f5, 0, 0,
            1, 96, 244, 0x14c, 193, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        ],
        [
            0, 1, 0, 1, 0x54b, 10, 0xc6d7, 0xcf7b, 0xd41c, 0xd9e7, 0x1edae, 0, 0x5f0, 0x257d, 0, 1,
            2, 241, 184, 0x14c, 255, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        ],
        [
            0, 2, 0, 1, 0x5e1, 12, 0x10fad, 0x13028, 0x132a2, 0x13575, 0x2b8bb, 0, 0x5f0, 0x1967, 0,
            2, 3, 75, 0x1a0, 0x13b, 162, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        ],
        [
            0, 3, 0, 1, 0x64f, 10, 0x13486, 0x13bb7, 0x1453c, 0x145cd, 0x2ba4b, 0, 0x611, 0x11f5, 0,
            3, 4, 0x159, 210, 0x106, 0x141, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0,
        ],
        [
            0, 4, 0, 1, 0x735, 12, 0x131c2, 0x15b9c, 0x16e60, 0x1657f, 0x2451b, 0, 0x529, 0x505, 0,
            4, 5, 0x125, 202, 53, 0x19c, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        ],
        [
            0, 5, 0, 1, 0x7f8, 12, 0x10abc, 0x11814, 0x125c0, 0x1335d, 0x2451b, 0, 0x529, 0x5f3, 0,
            5, 6, 0x136, 0x198, 206, 0x18d, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0,
        ],
    ],
    [
        [
            5, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x13880, 0x3e6, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        ],
        [
            0, 6, 0, 1, 0x276, 10, 0x1ba6c, 0x1dbe6, 0x1e5b2, 0x1ea07, 0x3086b, 0, 0x549, 0x32f5, 0,
            6, 1, 0x11e, 0x182, 68, 0x18c, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0,
        ],
        [
            0, 7, 0, 1, 0x384, 12, 0x160a2, 0x17700, 0x188b4, 0x1a24f, 0x1cfeb, 0, 0x549, 0x2bc3, 0,
            7, 2, 87, 0x15f, 0x144, 0x173, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0,
        ],
        [
            0, 8, 0, 1, 0x451, 8, 0x142a8, 0x153d8, 0x15b44, 0x165d0, 0x2d82d, 0, 0x4c5, 0x2c21, 0,
            8, 3, 159, 0x1c2, 54, 0x144, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        ],
        [
            0, 9, 0, 1, 0x523, 25, 0x19258, 0x1af40, 0x1c32c, 0x1d4c0, 0x3b86f, 0, 0x611, 0x34e7, 0,
            9, 4, 0x147, 0x16c, 88, 0x164, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0,
        ],
        [
            0, 10, 0, 1, 0x5f5, 18, 0xafe4, 0xc289, 0xc4e0, 0xc739, 0x25ee3, 0, 0x549, 0x40a1, 0,
            10, 5, 157, 0x1c0, 0x15b, 207, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0,
        ],
        [
            0, 11, 0, 1, 0x654, 18, 0x151a1, 0x165af, 0x17bd9, 0x1825b, 0x25ee3, 0, 0x549, 0x47f7,
            0, 11, 6, 120, 0x177, 127, 177, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0,
        ],
    ],
    [
        [
            5, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x13880, 0x3e5, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        ],
        [
            0, 12, 0, 1, 0x208, 10, 0xb570, 0xc59d, 0xcfe8, 0xd26c, 0x2ba4b, 0, 0x4c7, 0x256f, 0,
            12, 1, 0x12a, 236, 160, 0x156, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0,
        ],
        [
            0, 13, 0, 1, 0x262, 5, 0x6bce, 0x6f54, 0x73a5, 0x7c61, 0x3391f, 0, 0x50c, 0x689, 0, 13,
            2, 170, 202, 0x12f, 0x178, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        ],
        [
            0, 14, 0, 1, 0x2bc, 4, 0x7a0c, 0x7c9c, 0x7f58, 0x82ac, 0x1f6fb, 0, 0x5ac, 0x727, 0, 14,
            3, 115, 220, 0x145, 0x122, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        ],
        [
            0, 15, 0, 1, 0x343, 4, 0x9222, 0x954e, 0x9978, 0x9e7c, 0x1f6fb, 0, 0x5ac, 0x822, 0, 15,
            4, 0x142, 139, 156, 0x1c0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        ],
        [
            0, 16, 0, 1, 0x39d, 4, 0x17308, 0x18cc2, 0x197a1, 0x19b41, 0x1f6fb, 0, 0x5ac, 0x7a1, 0,
            16, 5, 0x109, 0x1d0, 145, 0x168, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0,
        ],
        [
            0, 17, 0, 1, 0x460, 4, 0x19025, 0x1a9c8, 0x1c2d1, 0x1caa3, 0x1f6fb, 0, 0x5ac, 0x78d, 0,
            17, 6, 231, 0x1cb, 81, 0x108, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0,
        ],
    ],
    [
        [
            5, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x13880, 0x3e4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        ],
        [
            0, 18, 0, 1, 0x73f, 15, 0x4601, 0x4fe6, 0x53d0, 0x539a, 0x23bce, 0, 0x5f0, 0x30a4, 0,
            18, 1, 0x149, 147, 0x114, 0x1b6, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0,
        ],
        [
            0, 19, 0, 1, 0x799, 18, 0x81ef, 0x849c, 0x8852, 0x8a9d, 0x1edae, 0, 0x5f0, 0x3ff4, 0,
            19, 2, 0x113, 0x1c3, 189, 0x1a5, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0,
        ],
        [
            0, 20, 0, 1, 0x807, 12, 0x6d25, 0x6d2c, 0x72ff, 0x771a, 0x2b8bb, 0, 0x5f0, 0x19c1, 0,
            20, 3, 0x11d, 168, 69, 0x18d, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0,
        ],
        [
            0, 21, 0, 1, 0x89d, 6, 0x7bbd, 0x8253, 0x83c1, 0x87c2, 0x2ba4b, 0, 0x611, 0xcdb, 0, 21,
            4, 0x159, 0x18c, 150, 190, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        ],
        [
            0, 22, 0, 1, 0x8fc, 15, 0x2447, 0x2714, 0x2931, 0x2a47, 0x2451b, 0, 0x529, 0x31c0, 0,
            22, 5, 203, 0x18b, 0x110, 0x155, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0,
        ],
        [
            0, 23, 0, 1, 0x91f, 15, 0x82f3, 0x89ec, 0x8d0c, 0x9212, 0x2451b, 0, 0x529, 0x2d70, 0,
            23, 6, 0x15e, 0x14d, 0x12e, 236, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0,
        ],
    ],
    [
        [
            5, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x13880, 0x3e3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        ],
        [
            0, 24, 0, 1, 0x406, 10, 0xbf7c, 0xc942, 0xd12c, 0xd629, 0x3086b, 0, 0x549, 0x31bc, 0,
            24, 1, 221, 0x1d1, 0x145, 156, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0,
        ],
        [
            0, 25, 0, 1, 0x46f, 5, 0xe994, 0xfe8c, 0x105e2, 0x106af, 0x3b86f, 0, 0x611, 0xccc, 0,
            25, 2, 237, 154, 0x110, 0x1d2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0,
        ],
        [
            0, 26, 0, 1, 0x519, 18, 0xdb60, 0xe42f, 0xe906, 0xf145, 0x25ee3, 0, 0x549, 0x6bb, 0, 26,
            3, 172, 207, 255, 0x1c0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        ],
        [
            0, 27, 0, 1, 0x578, 10, 0xf406, 0x10371, 0x10d9f, 0x107da, 0x2ba4b, 0, 0x4c7, 0x71e, 0,
            27, 4, 0x15a, 0x12c, 120, 0x192, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0,
        ],
        [
            0, 28, 0, 1, 0x5ff, 5, 0xb301, 0xc0a6, 0xc685, 0xcc1c, 0x3391f, 0, 0x50c, 0x723, 0, 28,
            5, 0x154, 0x1b7, 0x119, 0x141, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0,
        ],
        [
            0, 29, 0, 1, 0x654, 5, 0xabd3, 0xb33f, 0xbac3, 0xc56f, 0x3391f, 0, 0x50c, 0x282b, 0, 29,
            6, 0x100, 0x1b1, 0x155, 0x197, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0,
        ],
    ],
    [
        [
            5, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x13880, 0x3e2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        ],
        [
            0, 30, 0, 1, 0x6c2, 9, 0x61c7, 0x6610, 0x6a35, 0x6ff5, 0x1f6fb, 0, 0x5ac, 0x1538, 0, 30,
            1, 62, 0x126, 0x140, 207, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        ],
        [
            0, 31, 0, 1, 0x71c, 7, 0x9df9, 0x9f3f, 0xa348, 0xa83e, 0x1f6fb, 0, 0x5ac, 0x18a8, 0, 31,
            2, 0x15d, 232, 32, 0x18a, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        ],
        [
            0, 32, 0, 1, 0x77b, 4, 0xd530, 0xdfde, 0xeb6b, 0xf0f9, 0x1f6fb, 0, 0x5ac, 0xa48, 0, 32,
            3, 0x154, 0x137, 101, 0x1a9, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        ],
        [
            0, 33, 0, 1, 0x834, 4, 0x53b4, 0x58e4, 0x5d04, 0x5de7, 0x1f6fb, 0, 0x5ac, 0xdce, 0, 33,
            4, 226, 0x1c0, 0x113, 159, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        ],
        [
            0, 34, 0, 1, 0x866, 10, 0xcbfd, 0xdc36, 0xdea8, 0xe138, 0x1f6fb, 0, 0x5ac, 0x22e4, 0,
            34, 5, 96, 249, 0x13f, 0x1cd, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0,
        ],
        [
            0, 35, 0, 1, 0x8c5, 10, 0xde04, 0xe804, 0xebff, 0xf380, 0x1f6fb, 0, 0x5ac, 0x266f, 0,
            35, 6, 0x13a, 126, 236, 0x1c5, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0,
        ],
    ],
    [
        [
            5, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x13880, 0x3dc, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        ],
        [
            0, 0, 0, 1, 0x514, 5, 0xd73c, 0xee48, 0xf938, 0x101d0, 0x23bce, 0, 0x5f0, 0x13880, 0, 0,
            1, 0x14c, 193, 96, 244, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        ],
        [
            0, 1, 0, 1, 0x54b, 10, 0xc6d7, 0xcf7b, 0xd41c, 0xd9e7, 0x1edae, 0, 0x5f0, 0x13880, 0, 1,
            2, 0x14c, 255, 241, 184, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        ],
        [
            0, 2, 0, 1, 0x5e1, 12, 0x10fad, 0x13028, 0x132a2, 0x13575, 0x2b8bb, 0, 0x5f0, 0x13880,
            0, 2, 3, 0x13b, 162, 75, 0x1a0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0,
        ],
        [
            0, 3, 0, 1, 0x64f, 10, 0x13486, 0x13bb7, 0x1453c, 0x145cd, 0x2ba4b, 0, 0x611, 0x13880,
            0, 3, 4, 0x106, 0x141, 0x159, 210, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0,
        ],
        [
            0, 4, 0, 1, 0x735, 12, 0x131c2, 0x15b9c, 0x16e60, 0x1657f, 0x2451b, 0, 0x529, 0x13880,
            0, 4, 5, 53, 0x19c, 0x125, 202, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0,
        ],
        [
            0, 5, 0, 1, 0x7f8, 12, 0x10abc, 0x11814, 0x125c0, 0x1335d, 0x2451b, 0, 0x529, 0x13880,
            0, 5, 6, 206, 0x18d, 0x136, 0x198, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0,
        ],
    ],
    [
        [
            5, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x13880, 0x3dd, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        ],
        [
            0, 6, 0, 1, 0x276, 10, 0x1ba6c, 0x1dbe6, 0x1e5b2, 0x1ea07, 0x3086b, 0, 0x549, 0x13880,
            0, 6, 1, 68, 0x18c, 0x11e, 0x182, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0,
        ],
        [
            0, 7, 0, 1, 0x384, 12, 0x160a2, 0x17700, 0x188b4, 0x1a24f, 0x1cfeb, 0, 0x549, 0x13880,
            0, 7, 2, 0x144, 0x173, 87, 0x15f, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0,
        ],
        [
            0, 8, 0, 1, 0x451, 8, 0x142a8, 0x153d8, 0x15b44, 0x165d0, 0x2d82d, 0, 0x4c5, 0x13880, 0,
            8, 3, 54, 0x144, 159, 0x1c2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        ],
        [
            0, 9, 0, 1, 0x523, 25, 0x19258, 0x1af40, 0x1c32c, 0x1d4c0, 0x3b86f, 0, 0x611, 0x13880,
            0, 9, 4, 88, 0x164, 0x147, 0x16c, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0,
        ],
        [
            0, 10, 0, 1, 0x5f5, 18, 0xafe4, 0xc289, 0xc4e0, 0xc739, 0x25ee3, 0, 0x549, 0x13880, 0,
            10, 5, 0x15b, 207, 157, 0x1c0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0,
        ],
        [
            0, 11, 0, 1, 0x654, 18, 0x151a1, 0x165af, 0x17bd9, 0x1825b, 0x25ee3, 0, 0x549, 0x13880,
            0, 11, 6, 127, 177, 120, 0x177, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0,
        ],
    ],
    [
        [
            5, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x13880, 0x3de, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        ],
        [
            0, 12, 0, 1, 0x208, 10, 0xb570, 0xc59d, 0xcfe8, 0xd26c, 0x2ba4b, 0, 0x4c7, 0x13880, 0,
            12, 1, 160, 0x156, 0x12a, 236, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0,
        ],
        [
            0, 13, 0, 1, 0x262, 5, 0x6bce, 0x6f54, 0x73a5, 0x7c61, 0x3391f, 0, 0x50c, 0x13880, 0,
            13, 2, 0x12f, 0x178, 170, 202, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0,
        ],
        [
            0, 14, 0, 1, 0x2bc, 4, 0x7a0c, 0x7c9c, 0x7f58, 0x82ac, 0x1f6fb, 0, 0x5ac, 0x13880, 0,
            14, 3, 0x145, 0x122, 115, 220, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0,
        ],
        [
            0, 15, 0, 1, 0x343, 4, 0x9222, 0x954e, 0x9978, 0x9e7c, 0x1f6fb, 0, 0x5ac, 0x13880, 0,
            15, 4, 156, 0x1c0, 0x142, 139, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0,
        ],
        [
            0, 16, 0, 1, 0x39d, 4, 0x17308, 0x18cc2, 0x197a1, 0x19b41, 0x1f6fb, 0, 0x5ac, 0x13880,
            0, 16, 5, 145, 0x168, 0x109, 0x1d0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0,
        ],
        [
            0, 17, 0, 1, 0x460, 4, 0x19025, 0x1a9c8, 0x1c2d1, 0x1caa3, 0x1f6fb, 0, 0x5ac, 0x13880,
            0, 17, 6, 81, 0x108, 231, 0x1cb, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0,
        ],
    ],
    [
        [
            5, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x13880, 0x3df, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        ],
        [
            0, 18, 0, 1, 0x73f, 15, 0x4601, 0x4fe6, 0x53d0, 0x539a, 0x23bce, 0, 0x5f0, 0x13880, 0,
            18, 1, 0x114, 0x1b6, 0x149, 147, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0,
        ],
        [
            0, 19, 0, 1, 0x799, 18, 0x81ef, 0x849c, 0x8852, 0x8a9d, 0x1edae, 0, 0x5f0, 0x13880, 0,
            19, 2, 189, 0x1a5, 0x113, 0x1c3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0,
        ],
        [
            0, 20, 0, 1, 0x807, 12, 0x6d25, 0x6d2c, 0x72ff, 0x771a, 0x2b8bb, 0, 0x5f0, 0x13880, 0,
            20, 3, 69, 0x18d, 0x11d, 168, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0,
        ],
        [
            0, 21, 0, 1, 0x89d, 6, 0x7bbd, 0x8253, 0x83c1, 0x87c2, 0x2ba4b, 0, 0x611, 0x13880, 0,
            21, 4, 150, 190, 0x159, 0x18c, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0,
        ],
        [
            0, 22, 0, 1, 0x8fc, 15, 0x2447, 0x2714, 0x2931, 0x2a47, 0x2451b, 0, 0x529, 0x13880, 0,
            22, 5, 0x110, 0x155, 203, 0x18b, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0,
        ],
        [
            0, 23, 0, 1, 0x91f, 15, 0x82f3, 0x89ec, 0x8d0c, 0x9212, 0x2451b, 0, 0x529, 0x13880, 0,
            23, 6, 0x12e, 236, 0x15e, 0x14d, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0,
        ],
    ],
    [
        [
            5, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x13880, 0x3e0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        ],
        [
            0, 24, 0, 1, 0x406, 10, 0xbf7c, 0xc942, 0xd12c, 0xd629, 0x3086b, 0, 0x549, 0x13880, 0,
            24, 1, 0x145, 156, 221, 0x1d1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0,
        ],
        [
            0, 25, 0, 1, 0x46f, 5, 0xe994, 0xfe8c, 0x105e2, 0x106af, 0x3b86f, 0, 0x611, 0x13880, 0,
            25, 2, 0x110, 0x1d2, 237, 154, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0,
        ],
        [
            0, 26, 0, 1, 0x519, 18, 0xdb60, 0xe42f, 0xe906, 0xf145, 0x25ee3, 0, 0x549, 0x13880, 0,
            26, 3, 255, 0x1c0, 172, 207, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        ],
        [
            0, 27, 0, 1, 0x578, 10, 0xf406, 0x10371, 0x10d9f, 0x107da, 0x2ba4b, 0, 0x4c7, 0x13880,
            0, 27, 4, 120, 0x192, 0x15a, 0x12c, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0,
        ],
        [
            0, 28, 0, 1, 0x5ff, 5, 0xb301, 0xc0a6, 0xc685, 0xcc1c, 0x3391f, 0, 0x50c, 0x13880, 0,
            28, 5, 0x119, 0x141, 0x154, 0x1b7, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0,
        ],
        [
            0, 29, 0, 1, 0x654, 5, 0xabd3, 0xb33f, 0xbac3, 0xc56f, 0x5283, 0, 0x50c, 0x13880, 0, 29,
            6, 0x155, 0x197, 0x100, 0x1b1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0,
        ],
    ],
    [
        [
            5, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x13880, 0x3e1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        ],
        [
            0, 30, 0, 1, 0x6c2, 9, 0x61c7, 0x6610, 0x6a35, 0x6ff5, 0x1f6fb, 0, 0x5ac, 0x13880, 0,
            30, 1, 0x140, 207, 62, 0x126, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0,
        ],
        [
            0, 31, 0, 1, 0x71c, 7, 0x9df9, 0x9f3f, 0xa348, 0xa83e, 0x1f6fb, 0, 0x5ac, 0x13880, 0,
            31, 2, 32, 0x18a, 0x15d, 232, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0,
        ],
        [
            0, 32, 0, 1, 0x77b, 4, 0xd530, 0xdfde, 0xeb6b, 0xf0f9, 0x1f6fb, 0, 0x5ac, 0x13880, 0,
            32, 3, 101, 0x1a9, 0x154, 0x137, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0,
        ],
        [
            0, 33, 0, 1, 0x834, 4, 0x53b4, 0x58e4, 0x5d04, 0x5de7, 0x1f6fb, 0, 0x5ac, 0x13880, 0,
            33, 4, 0x113, 159, 226, 0x1c0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0,
        ],
        [
            0, 34, 0, 1, 0x866, 10, 0xcbfd, 0xdc36, 0xdea8, 0xe138, 0x1f6fb, 0, 0x5ac, 0x13880, 0,
            34, 5, 0x13f, 0x1cd, 96, 249, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0,
        ],
        [
            0, 35, 0, 1, 0x8c5, 10, 0xde04, 0xe804, 0xebff, 0xf380, 0x1f6fb, 0, 0x5ac, 0x13880, 0,
            35, 6, 236, 0x1c5, 0x13a, 126, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0,
        ],
    ],
];

/// The bytes nothing names.
#[rustfmt::skip]
const TAIL_BYTES: [u8; TAIL_LEN] = [
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 100, 0, 0, 0, 100, 0, 0, 0, 100, 0, 0, 0, 100, 0, 0,
    0, 100, 0, 0, 0, 100, 0, 0, 0, 100, 0, 0, 0, 100, 0, 0, 0, 100, 0, 0, 0, 100, 0, 0, 0, 100, 0,
    0, 0, 100, 0, 0, 0, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
    255, 255, 255, 255, 255, 255, 255, 255, 255, 0, 0, 0, 0, 255, 255, 255, 255, 255, 255, 255, 255,
    255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 0, 0, 0, 0, 255,
    255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
    255, 255, 255, 255, 255, 255, 255, 255,
];

/// The event table a list's entry names; refused for the -1 that ends the list.
fn listed_table(entry: Listed<RecordTable>) -> Result<RecordTable, BadInput> {
    match entry {
        Listed::Entry(table) => Ok(table),
        Listed::End => Err(BadInput("a stage list's end mark read as a table")),
    }
}

impl StageTables {
    /// The event table (a stage group's rally) `sub` of stage group `group`. A `sub` past the
    /// group's list reads on into the lists after it; refused for a group past the four, past
    /// the lists, or on a list's -1.
    pub fn group_event_table(&self, group: u32, sub: u32) -> Result<RecordTable, BadInput> {
        let list = GROUP_LISTS
            .get(group as usize)
            .ok_or(BadInput("a stage group past the four"))?;
        let entry = list
            .entry_on(sub as usize)
            .ok_or(BadInput("a stage list read past the lists"))?;
        listed_table(entry)
    }

    /// Arcade level `level`'s event table; refused for a level past the six (the -1 ends the
    /// list).
    pub fn level_table(&self, level: u32) -> Result<RecordTable, BadInput> {
        let entry = LEVEL_TABLES
            .get(level as usize)
            .ok_or(BadInput("an arcade level past the list"))?;
        listed_table(*entry)
    }

    /// The variable that holds best-times table `n` (the list's four, in
    /// `menu::champ_store::ChampView`); refused for a place past the four.
    pub fn result_table(&self, n: u32) -> Result<TimesVariable, BadInput> {
        match RESULT_TABLES.get(n as usize) {
            Some(Listed::Entry(variable)) => Ok(*variable),
            _ => Err(BadInput("a results table past the four")),
        }
    }

    /// The championship's records.
    pub fn championship(&self) -> RecordTable {
        RecordTable::Championship
    }

    /// Event table `k`.
    pub fn event_table(&self, k: usize) -> RecordTable {
        RecordTable::Event(k)
    }
}

/// A table of records: the championship's or one of the twelve events'.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RecordTable {
    Championship,
    Event(usize),
}

/// A record of a table: row `index`, or (at `index` == the rows' count) the -1 that ends
/// the table, whose tag a walk may read.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RecordRef {
    pub table: RecordTable,
    pub index: usize,
}

impl RecordTable {
    pub fn rows(self, stages: &StageTables) -> &[Record] {
        match self {
            RecordTable::Championship => &stages.championship,
            RecordTable::Event(k) => &stages.events[k].records,
        }
    }

    fn end(self, stages: &StageTables) -> &PlainDword {
        match self {
            RecordTable::Championship => &stages.championship_end,
            RecordTable::Event(k) => &stages.events[k].end,
        }
    }

    /// Its title string: its first record's name.
    pub fn title(self, stages: &StageTables) -> Result<u32, BadInput> {
        Ok(self.row(stages, 0)?.record(stages)?.name())
    }

    /// Its rows and end mark as bytes, for the walks.
    pub fn bytes(self, stages: &StageTables) -> Vec<u8> {
        self.rows(stages)
            .iter()
            .flat_map(Record::image_bytes)
            .chain(self.end(stages).get().to_le_bytes())
            .collect()
    }

    /// Row `n` (or the end mark). Refused past the end mark.
    pub fn row(self, stages: &StageTables, n: u32) -> Result<RecordRef, BadInput> {
        let index = n as usize;
        if index > self.rows(stages).len() {
            return Err(BadInput("a record index past its table's end"));
        }
        Ok(RecordRef { table: self, index })
    }
}

impl RecordRef {
    /// The record; refused for the end mark.
    pub fn record(self, stages: &StageTables) -> Result<&Record, BadInput> {
        self.table
            .rows(stages)
            .get(self.index)
            .ok_or(BadInput("a table's end mark read as a record"))
    }

    /// Holder `h`'s name up to its NUL. A name that fills its bytes runs on into the words
    /// after it, then into the next rows; refused without a NUL before the table's end mark
    /// (or in more than a `Held` has room for).
    pub fn holder_text(self, stages: &StageTables, h: Holder) -> Result<Held, BadInput> {
        let bytes = self.table.bytes(stages);
        let from = self.index * RECORD_LEN as usize + h.name_at() as usize;
        let text = bytes
            .get(from..)
            .and_then(|rest| {
                rest.split(|&b| b == 0)
                    .next()
                    .filter(|t| t.len() < rest.len())
            })
            .ok_or(BadInput("a record holder's name without its end"))?;
        Held::new(text).ok_or(BadInput("a record holder's name past its room"))
    }

    /// Its tag (the end mark's: -1).
    pub fn tag(self, stages: &StageTables) -> u32 {
        self.record(stages)
            .map_or(self.table.end(stages).get(), Record::tag)
    }

    /// Its dword at byte `at` (the end mark's tag only).
    pub fn word_at(self, stages: &StageTables, at: u32) -> Result<u32, BadInput> {
        match at {
            0 => Ok(self.tag(stages)),
            _ => self.record(stages)?.word_at(at),
        }
    }

    /// The record `n` rows on (wrapping); refused past the table's end mark or before its
    /// first row.
    pub fn offset(self, stages: &StageTables, n: u32) -> Result<Self, BadInput> {
        self.table.row(stages, (self.index as u32).wrapping_add(n))
    }
}

/// Row `n` of `table` (or its end mark); refused past it.
pub fn row(stages: &StageTables, table: RecordTable, n: u32) -> Result<RecordRef, BadInput> {
    table.row(stages, n)
}

#[cfg(test)]
/// The old layout's lists, as its dwords.
const OLD_LISTS: [u32; 30] = [
    0x7556d8, 0x7556e0, 0x7556e8, 0x7556e4, 0xffffffff, // the results' variables
    0x73553e, 0x735522, 0x73553e, 0x735522, // per stage group
    0x739642, 0x7391c2, 0x738d42, 0x7388c2, 0x738442, 0x737fc2,
    0xffffffff, // first group's tables
    0x739ac2, 0x739f42, 0x73a3c2, 0x73a842, 0x73acc2, 0x73b142,
    0xffffffff, // second group's tables
    0x739642, 0x7391c2, 0x738d42, 0x7388c2, 0x738442, 0x737fc2,
    0xffffffff, // the levels' tables
];

#[cfg(test)]
mod tests {
    use super::*;

    /// The event tables the old lists named, by the old dwords (the addresses of the tables'
    /// first records).
    fn table_at(address: u32) -> Option<RecordTable> {
        let k = address.checked_sub(0x737FC2)?;
        (k.is_multiple_of(0x480) && k / 0x480 < 12)
            .then_some(RecordTable::Event(k as usize / 0x480))
    }

    /// The lists resolve to what the old dwords named: each group's entries through its list, the
    /// levels' tables, the results' variables.
    #[test]
    fn the_lists_resolve_to_what_they_name() {
        let t = StageTables::new();
        let lists = &OLD_LISTS[9..];
        for group in 0..4u32 {
            let first = (OLD_LISTS[5 + group as usize] - 0x735522) as usize / 4;
            for sub in 0..(lists.len() - first) as u32 + 2 {
                let old = lists
                    .get(first + sub as usize)
                    .and_then(|&name| table_at(name));
                assert_eq!(t.group_event_table(group, sub).ok(), old, "{group} {sub}");
            }
        }
        assert!(t.group_event_table(4, 0).is_err());
        for level in 0..9u32 {
            let old = OLD_LISTS[23..]
                .get(level as usize)
                .and_then(|&n| table_at(n));
            assert_eq!(t.level_table(level).ok(), old, "level {level}");
        }
        for n in 0..8u32 {
            let old = OLD_LISTS[..4]
                .get(n as usize)
                .and_then(|&name| TimesVariable::named(name));
            assert_eq!(t.result_table(n).ok(), old, "result {n}");
        }
    }

    #[test]
    fn holders_are_written_by_name() {
        let record = Record::new([0x5555_5555; 41]);
        let row = HolderRow {
            nation: 1,
            car: 2,
            speed: 8,
        };
        record.set_holder(Holder::TimeTrial, &row, b"Jill");
        let held = record.holder(Holder::TimeTrial);
        assert_eq!((held.nation, held.car, held.speed), (1, 2, 8));
        assert_eq!(record.word_at(0x2C), Ok(8));
        assert_eq!(record.word_at(0xA0), Ok(2));
        // No NUL written: the bytes after the name stay.
        let name = record.holder_name(Holder::TimeTrial);
        assert_eq!((&name[..4], name[4]), (&b"Jill"[..], 0x55));
        assert_eq!(record.word_at(0x7C), Ok(u32::from_le_bytes(*b"Jill")));
        // The other holder untouched.
        assert_eq!(record.holder(Holder::Fastest).speed, 0x5555_5555);
        assert_eq!(record.word_at(0x54), Ok(0x5555_5555));
    }

    /// The named fields sit where the old dword array had them (`words[k]` at +4 k): each
    /// accessor reads the dword of its old offset, and the record's old layout is the dwords in.
    #[test]
    fn record_fields_keep_the_old_offsets() {
        let v: [u32; 41] = std::array::from_fn(|k| 0x0101_0000 * (k as u32 + 1) + k as u32);
        let r = Record::new(v);
        assert_eq!(
            (r.tag(), r.track(), r.pack_id(), r.hhmm()),
            (v[0], v[1], v[2], v[4])
        );
        assert_eq!(r.class_times(), [v[6], v[7], v[8], v[9]]);
        assert_eq!(
            (r.race_stage(), r.name(), r.event_slot(), r.number()),
            (v[13], v[14], v[15], v[16])
        );
        assert_eq!((r.service_figure.get(), r.service_time.get()), (v[3], v[5]));
        assert_eq!(
            r.markers(),
            [(v[17] as i32, v[18] as i32), (v[19] as i32, v[20] as i32)]
        );
        for (h, [nation, car, speed], name) in [
            (Holder::Fastest, [29, 30, 12], 21),
            (Holder::TimeTrial, [39, 40, 11], 31),
        ] {
            let row = r.holder(h);
            assert_eq!(
                (row.nation, row.car, row.speed),
                (v[nation], v[car], v[speed])
            );
            let old: Vec<u8> = v[name..name + 8]
                .iter()
                .flat_map(|w| w.to_le_bytes())
                .collect();
            assert_eq!(r.holder_name(h)[..], old[..]);
            assert_eq!(h.name_at() as usize, 4 * name);
        }
        let image: Vec<u8> = v.iter().flat_map(|w| w.to_le_bytes()).collect();
        assert_eq!(r.image_bytes().collect::<Vec<_>>(), image);
        for (k, &word) in v.iter().enumerate() {
            assert_eq!(r.word_at(4 * k as u32), Ok(word));
            assert_eq!(r.bits(k), word);
            r.set_bits(k, !word);
            assert_eq!(r.word_at(4 * k as u32), Ok(!word));
        }
        assert!(r.word_at(0xA4).is_err() && r.word_at(2).is_err());
    }

    #[test]
    fn records_stay_in_their_table() {
        let _g = crate::test_support::test_lock();
        let t = &crate::test_support::own::system().records().stage_tables;
        let champ = RecordTable::Championship;
        assert_eq!(champ.row(t, 66).map(|r| r.index), Ok(66), "the end mark");
        assert!(champ.row(t, 67).is_err());
        assert!(RecordTable::Event(0).row(t, 8).is_err());
        let third = champ.row(t, 3).unwrap();
        assert_eq!(third.offset(t, u32::MAX).map(|r| r.index), Ok(2));
        assert_eq!(row(t, RecordTable::Event(1), 7).map(|r| r.index), Ok(7));
        assert!(row(t, RecordTable::Event(1), 8).is_err());
        assert_eq!(
            RecordTable::Event(1).row(t, 7).map(|r| r.tag(t)),
            Ok(u32::MAX)
        );
        assert!(RecordTable::Event(1).row(t, 7).unwrap().record(t).is_err());
    }
}

/// The chain against the old layout's addresses: each piece starts where it did there, the
/// lists read as the old dwords, and a read on goes from piece to piece in that order up to
/// the end.
#[cfg(test)]
mod chain_tests {
    use super::*;

    /// The old address of the first byte.
    const START: usize = 0x734BC6;
    /// The old address of the end.
    const END: usize = 0x73B674;

    /// Where each piece of `Piece::ORDER` began in the old layout.
    const IDA_STARTS: [usize; 12] = [
        0x734BC6, // the pack head
        0x734BFA, // its entries
        0x7354FA, // the pack's -1
        0x7354FE, // the results' variables
        0x735512, // per stage group
        0x735522, // first group's tables
        0x73553E, // second group's tables
        0x73555A, // the levels' tables
        0x735576, // the championship's records
        0x737FBE, // its -1
        0x737FC2, // the events' tables
        0x73B5C2, // the tail
    ];

    #[test]
    fn pieces_start_at_their_ida_addresses() {
        let t = StageTables::new();
        let mut at = START;
        for (piece, ida) in Piece::ORDER.into_iter().zip(IDA_STARTS) {
            assert_eq!(at, ida, "{piece:?}");
            at += piece.len(&t);
        }
        assert_eq!(at, END);
        // The handles' addresses are the pieces' own.
        for (list, piece) in [
            (StageList::GroupA, Piece::GroupListA),
            (StageList::GroupB, Piece::GroupListB),
            (StageList::Levels, Piece::LevelTables),
        ] {
            let k = Piece::ORDER.iter().position(|&p| p == piece).unwrap();
            assert_eq!(list.image_name() as usize, IDA_STARTS[k]);
        }
        let champ = Piece::ORDER
            .iter()
            .position(|&p| p == Piece::Championship)
            .unwrap();
        assert_eq!(
            RecordTable::Championship.image_name() as usize,
            IDA_STARTS[champ]
        );
        assert_eq!(
            RecordTable::Event(0).image_name() as usize,
            IDA_STARTS[champ + 2]
        );
    }

    #[test]
    fn the_lists_read_as_the_images_dwords() {
        let t = StageTables::new();
        let lists: Vec<u32> = (0..30)
            .map(|n| t.word_on(Piece::ResultTables, n).unwrap())
            .collect();
        assert_eq!(lists[..], OLD_LISTS[..]);
    }

    /// Every dword from the pack's entries on reads the cell at its old byte offset: the
    /// chain's dword `n` is the old dword at the entries' start + 4 n, which the typed fields hold
    /// at their old places (numbered cells, read back by name).
    #[test]
    fn a_read_on_reaches_the_fields_in_the_images_order() {
        let t = Box::new(StageTables::new());
        let mut n = 0u32;
        let mut next = || {
            n += 1;
            0x5000_0000 + n
        };
        for entry in &t.trak_pak.tracks {
            for k in 0..TrackEntry::WORDS {
                entry.set_bits(k, next());
            }
        }
        t.trak_pak.end.set(next());
        for record in t
            .championship
            .iter()
            .chain(t.events.iter().flat_map(|e| &e.records))
        {
            for k in 0..Record::WORDS {
                record.set_bits(k, next());
            }
        }
        // By the old address: what each named place holds.
        let at = |address: usize| -> Option<u32> {
            let tracks = 0x734BFA;
            let lists = 0x7354FE;
            let champ = 0x735576;
            let events = 0x737FC2;
            let tail = 0x73B5C2;
            Some(match address {
                a if a < tracks => return None,
                a if a < 0x7354FA => {
                    let d = (a - tracks) / 4;
                    t.trak_pak.tracks[d / 16].get(match d % 16 {
                        0 => TrackField::Name,
                        2 => TrackField::Duration,
                        3 => TrackField::Length,
                        13 => TrackField::Stage,
                        _ => return Some(t.trak_pak.tracks[d / 16].bits(d % 16)),
                    })
                }
                a if a < lists => t.trak_pak.end.get(),
                a if a < champ => OLD_LISTS[(a - lists) / 4],
                a if a < 0x737FBE => {
                    let d = (a - champ) / 4;
                    t.championship[d / 41].word_at(4 * (d % 41) as u32).unwrap()
                }
                a if a < events => t.championship_end.get(),
                a if a < tail => {
                    let d = (a - events) / 4;
                    let event = &t.events[d / 288];
                    match event.records.get((d % 288) / 41) {
                        Some(r) => r.word_at(4 * (d % 288 % 41) as u32).unwrap(),
                        None => event.end.get(),
                    }
                }
                a if a + 4 <= END => t.tail.get_u32(a - tail),
                _ => return None,
            })
        };
        let words = (END - 0x734BFA) / 4 + 2;
        for k in 0..words {
            assert_eq!(t.word_on(Piece::Tracks, k), at(0x734BFA + 4 * k), "{k}");
        }
        // The bytes: the dwords' (little-endian), then the tail's last two, then none.
        let len = END - 0x735576;
        for k in 0..len - 2 {
            let word = t.word_on(Piece::Championship, k / 4).unwrap();
            assert_eq!(
                t.byte_on(Piece::Championship, k),
                Some(word.to_le_bytes()[k % 4])
            );
        }
        for k in [len - 2, len - 1] {
            let tail = k - (0x73B5C2 - 0x735576);
            assert_eq!(t.byte_on(Piece::Championship, k), Some(t.tail.get(tail)));
        }
        assert_eq!(t.byte_on(Piece::Championship, len), None);
    }
}

#[cfg(test)]
mod image_bytes_tests {
    use super::*;

    fn dump(t: &StageTables) -> Vec<u8> {
        let mut out = Vec::new();
        let mut run = |r: &dyn Run| {
            for i in 0..r.count() {
                out.extend(r.bits(i).to_le_bytes());
            }
        };
        run(&t.trak_pak.link);
        run(&t.trak_pak.pack_id);
        run(&t.trak_pak.name);
        run(&t.trak_pak.count);
        for e in &t.trak_pak.tracks {
            run(e);
        }
        run(&t.trak_pak.end);
        for r in &t.championship {
            run(r);
        }
        run(&t.championship_end);
        for e in &t.events {
            run(e);
        }
        for i in 0..TAIL_LEN {
            out.push(t.tail.get(i));
        }
        out
    }

    /// The record's whole bytes, as a checksum: a wrong dword in a table shows here.
    #[test]
    fn the_stage_tables_keep_the_bytes_of_the_image() {
        let bytes = dump(&StageTables::new());
        let mut hash = 0xcbf2_9ce4_8422_2325_u64;
        for byte in &bytes {
            hash = (hash ^ u64::from(*byte)).wrapping_mul(0x100_0000_01b3);
        }
        assert_eq!(bytes.len(), 27190);
        assert_eq!(hash, 0xaee702843be35d25);
    }
}
