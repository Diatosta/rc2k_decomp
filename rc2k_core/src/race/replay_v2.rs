//! The replay format of this project (version 2), beside the first format's: up to eight cars, every
//! one of them recorded. A file is
//!
//! - the first format's `HEADER_LEN` (0x430) header with its own magic ("RC2R") and version
//!   (`VERSION`), the same settings, sealed part and physics switches; its dword at `EXT_LEN_AT`
//!   (a place the first format leaves zero) says how long the extension is;
//! - the extension at 0x430: sections of (tag, length, bytes), each padded to a dword, so that a
//!   later version can add sections and an older reader skips what it does not know: `CARS`
//!   (the cars: model, kind, how recorded, name), `BLKS` (the layout of a block), `NEXT` (what
//!   playback of sound for more than two cars and of a split screen will need);
//! - the blocks, one second each, as in the first format's file: the first format's block of
//!   `EXTENDED_FRAME_BYTES` (cars 0 .. 3 as their state at the second's start, the loose parts, the
//!   computer driver's state, and the inputs of cars 0 .. 3 for the second's hundred steps), then
//!   the **tail** (`TAIL_BYTES`): the state of cars 4 .. 7 at the second's start, their inputs,
//!   and the **states**: for the cars not driven on the machine that recorded (a network game's
//!   other players and bots, which that machine saw glided to the host's places) their
//!   motion at each of the second's steps, as that machine showed it;
//! - the index at the file's end, as the first format's (`RCIB`, up to 1800 blocks).
//!
//! The blocks are packed by runs and indexed as the first format's are, so the viewer's cutting,
//! pasting and camera editing work on them unchanged.

use crate::race::layouts;
use crate::race::replay::HEADER_MAGIC;

/// The header's first dword of a replay of this format, "RC2R".
pub const MAGIC: u32 = 0x5232_4352;
/// The header's version (the first format's are 0x50400 and 0x50401).
pub const VERSION: u32 = 0x0002_0000;
/// Where the header keeps the extension's length (the first format's header has zero there).
pub const EXT_LEN_AT: usize = 0x1B0;
/// The header's bytes before the extension.
pub const HEADER_LEN: usize = 0x430;
/// The most cars a replay holds.
pub const MAX_CARS: usize = 8;
/// The cars the first format's block holds the state of.
pub const HEAD_CARS: usize = 4;
/// The cars whose state the tail holds (4 .. 7).
pub const TAIL_CARS: usize = MAX_CARS - HEAD_CARS;
/// Steps in a block.
pub const STEPS: usize = 100;
/// The blocks the own format's index holds: 90 minutes (the first format's holds 1800, 30 minutes).
pub const MOST_BLOCKS: u32 = 5400;
/// The own format's index at a file's end: the first format's four head dwords and a (place, size)
/// pair for each of `MOST_BLOCKS` blocks.
pub const OWN_INDEX_LEN: usize = 16 + 8 * MOST_BLOCKS as usize;

/// The words of a car's state in a block (the first 0x684 bytes of its record).
pub use crate::drive::replay_frames::CAR_DWORDS;

/// The tail's first part: the state of cars 4 .. 7.
pub(crate) const TAIL_KEY_BYTES: usize = TAIL_CARS * CAR_DWORDS * 4;
/// The tail's second part: the inputs of cars 4 .. 7, 8 bytes a car and step.
pub(crate) const TAIL_TAPE_BYTES: usize = STEPS * TAIL_CARS * 8;
/// Where the states begin in the tail: their length (a dword), then their bytes.
pub(crate) const STATES_AT: usize = TAIL_KEY_BYTES + TAIL_TAPE_BYTES;
/// The most bytes the states of one block take; a second that needs more is held at its last
/// step that fits (the file's size stays bounded).
pub(crate) const STATES_CAP: usize = 42_000;
/// The tail's bytes.
pub(crate) const TAIL_BYTES: usize = STATES_AT + 4 + STATES_CAP;
/// The dwords of the tail.
pub const TAIL_DWORDS: usize = TAIL_BYTES / 4;
/// A block's bytes once unpacked.
pub const BLOCK_BYTES: usize = crate::drive::replay_frames::EXTENDED_FRAME_BYTES + TAIL_BYTES;
const _: () = assert!(TAIL_BYTES.is_multiple_of(4));

#[cfg(test)]
/// Whether a header's first dword is this format's.
pub(crate) fn is_magic(word: u32) -> bool {
    word == MAGIC
}

/// Whether a header's first dword is one of the two formats' (the first format's or this one).
pub(crate) fn is_replay_magic(word: u32) -> bool {
    word == HEADER_MAGIC || word == MAGIC
}

// ---- the state a recorded car keeps at each step ----

/// The dwords of a car's state (as the 0x684 bytes of a block keep them: file offset / 4) that
/// are recorded at each step for a car this machine did not drive: its place, orientation and
/// motion, the wheels' and the steering's turn, and the engine's and the sound's speeds.
pub const STATE_AT: [usize; STATE_WORDS] = {
    let mut at = [0usize; STATE_WORDS];
    let runs: [(usize, usize); 12] = [
        (0x134 / 4, 3), // position
        (0xB4 / 4, 16), // orientation (4 x 4)
        (0x198 / 4, 3), // velocity
        (0x1A4 / 4, 3), // velocity along the car
        (0x1C4 / 4, 1), // speed
        (0x1D4 / 4, 1), // gear
        (0x1D8 / 4, 1), // engine revs
        (0x1E4 / 4, 1), // revs the sound follows
        (0x1F8 / 4, 2), // the driven wheels' turn
        (0x288 / 4, 4), // slip and the sound's levels
        (0x64C / 4, 2), // steering angle, and as drawn
        (0x184 / 4, 3), // spin
    ];
    let (mut n, mut r) = (0, 0);
    while r < runs.len() {
        let mut k = 0;
        while k < runs[r].1 {
            at[n] = runs[r].0 + k;
            n += 1;
            k += 1;
        }
        r += 1;
    }
    at
};
/// The dwords recorded of a car at each step.
pub(crate) const STATE_WORDS: usize = 3 + 16 + 3 + 3 + 1 + 1 + 1 + 1 + 2 + 4 + 2 + 3;

/// What is recorded of a car at one step.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CarState {
    pub words: [u32; STATE_WORDS],
}

impl CarState {
    /// The recorded words of a car's state in a block's layout.
    pub fn of(car: &[u32; CAR_DWORDS]) -> Self {
        CarState {
            words: STATE_AT.map(|at| car[at]),
        }
    }

    /// The words put into a car's state in a block's layout.
    pub fn put(&self, car: &mut [u32; CAR_DWORDS]) {
        for (&at, &w) in STATE_AT.iter().zip(&self.words) {
            car[at] = w;
        }
    }
}

/// What a block records of the cars driven elsewhere: for each step, the state of each car in
/// the block's mask.
pub type Tick = [Option<CarState>; MAX_CARS];

/// How a recorded word is coded.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Code {
    /// A float, kept to its sign, exponent and 13 bits of its mantissa; the difference to the step
    /// before.
    Float,
    /// A whole number; the difference to the step before.
    Step,
    /// A whole number that moves steadily (a place, a wheel's turn); the difference to what the
    /// two steps before predict (the step before plus the change that step made).
    Steady,
}

/// The code of each recorded word, in `STATE_AT`'s order.
const CODES: [Code; STATE_WORDS] = {
    use Code::{Float, Steady, Step};
    let runs: [(Code, usize); 12] = [
        (Steady, 3), // position
        (Float, 16), // orientation
        (Float, 3),  // velocity
        (Float, 3),  // velocity along the car
        (Float, 1),  // speed
        (Step, 1),   // gear
        (Float, 1),  // engine revs
        (Step, 1),   // the revs the sound follows
        (Steady, 2), // the driven wheels' turn
        (Float, 4),  // slip and the sound's levels
        (Step, 2),   // steering angle, and as drawn
        (Float, 3),  // spin
    ];
    let mut out = [Float; STATE_WORDS];
    let (mut n, mut r) = (0, 0);
    while r < runs.len() {
        let mut k = 0;
        while k < runs[r].1 {
            out[n] = runs[r].0;
            n += 1;
            k += 1;
        }
        r += 1;
    }
    out
};

/// The low bits of a float's mantissa that are not kept (a car's orientation to 1 in 8000).
const FLOAT_DROPPED: u32 = 10;

/// A word in the domain the differences are taken in.
fn coded(code: Code, w: u32) -> u32 {
    match code {
        Code::Float => w.wrapping_add(1 << (FLOAT_DROPPED - 1)) >> FLOAT_DROPPED,
        Code::Step | Code::Steady => w,
    }
}

/// The word a coded value is.
fn uncoded(code: Code, c: u32) -> u32 {
    match code {
        Code::Float => c << FLOAT_DROPPED,
        Code::Step | Code::Steady => c,
    }
}

impl CarState {
    #[cfg(test)]
    /// The state as a recorded one reads back: the floats kept to their coded bits.
    pub(crate) fn as_recorded(&self) -> CarState {
        CarState {
            words: std::array::from_fn(|n| uncoded(CODES[n], coded(CODES[n], self.words[n]))),
        }
    }
}

fn put_varint(out: &mut Vec<u8>, mut v: u32) {
    while v >= 0x80 {
        out.push((v & 0x7F) as u8 | 0x80);
        v >>= 7;
    }
    out.push(v as u8);
}

fn get_varint(bytes: &[u8], at: &mut usize) -> Option<u32> {
    let mut v = 0u32;
    for shift in (0..35).step_by(7) {
        let b = *bytes.get(*at)?;
        *at += 1;
        v |= u32::from(b & 0x7F) << shift;
        if b & 0x80 == 0 {
            return Some(v);
        }
    }
    None
}

fn zigzag(d: i32) -> u32 {
    ((d << 1) ^ (d >> 31)) as u32
}

fn unzigzag(z: u32) -> i32 {
    ((z >> 1) as i32) ^ -((z & 1) as i32)
}

/// What the difference of a word is taken from: the word at the step before and at the one
/// before that, in the coded domain.
#[derive(Clone, Copy)]
struct History {
    last: [u32; STATE_WORDS],
    before: [u32; STATE_WORDS],
}

impl History {
    fn of(state: &CarState) -> Self {
        let last = std::array::from_fn(|n| coded(CODES[n], state.words[n]));
        History { last, before: last }
    }

    fn predicted(&self, n: usize) -> u32 {
        match CODES[n] {
            Code::Steady => self.last[n]
                .wrapping_add(self.last[n])
                .wrapping_sub(self.before[n]),
            _ => self.last[n],
        }
    }

    fn push(&mut self, coded: [u32; STATE_WORDS]) {
        self.before = self.last;
        self.last = coded;
    }
}

/// The states of a block's steps for the cars in `mask` (bit n: car n), each word as the
/// difference to what the steps before give (`Code`; the first step's to the car's state at
/// the second's start, `bases`), the differences as varints. The steps from the first that does
/// not fit in `cap` bytes are left out. A car of the mask missing at a step keeps its last
/// state.
pub(crate) fn encode_states(
    bases: &[CarState; MAX_CARS],
    ticks: &[Tick],
    mask: u8,
    cap: usize,
) -> Vec<u8> {
    let mut out = vec![0u8];
    let mut history: [History; MAX_CARS] = std::array::from_fn(|c| History::of(&bases[c]));
    let mut done = 0u8;
    for tick in ticks.iter().take(STEPS) {
        let mark = out.len();
        let mut now = history;
        for car in (0..MAX_CARS).filter(|c| mask >> c & 1 == 1) {
            let h = &history[car];
            let state = tick[car];
            let target: [u32; STATE_WORDS] = match state {
                Some(s) => std::array::from_fn(|n| coded(CODES[n], s.words[n])),
                None => h.last,
            };
            for (n, &t) in target.iter().enumerate() {
                put_varint(&mut out, zigzag(t.wrapping_sub(h.predicted(n)) as i32));
            }
            now[car].push(target);
        }
        if out.len() > cap {
            out.truncate(mark);
            break;
        }
        history = now;
        done += 1;
    }
    out[0] = done;
    out
}

/// What `encode_states` made, back into the steps (the floats as recorded: `as_recorded`); none
/// for damaged bytes.
pub(crate) fn decode_states(
    bases: &[CarState; MAX_CARS],
    bytes: &[u8],
    mask: u8,
) -> Option<Vec<Tick>> {
    let steps = *bytes.first()? as usize;
    if steps > STEPS {
        return None;
    }
    let mut at = 1;
    let mut history: [History; MAX_CARS] = std::array::from_fn(|c| History::of(&bases[c]));
    let mut out = Vec::with_capacity(steps);
    for _ in 0..steps {
        let mut tick: Tick = [None; MAX_CARS];
        for car in (0..MAX_CARS).filter(|c| mask >> c & 1 == 1) {
            let h = &mut history[car];
            let mut coded_now = [0u32; STATE_WORDS];
            for (n, c) in coded_now.iter_mut().enumerate() {
                let d = unzigzag(get_varint(bytes, &mut at)?);
                *c = h.predicted(n).wrapping_add(d as u32);
            }
            h.push(coded_now);
            tick[car] = Some(CarState {
                words: std::array::from_fn(|n| uncoded(CODES[n], coded_now[n])),
            });
        }
        out.push(tick);
    }
    Some(out)
}

// ---- the tail ----

/// A block's tail: the state of cars 4 .. 7 at the second's start, their inputs at each step
/// (the two words a car's entry holds, as the first format's tape does), and the recorded states.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Tail {
    pub keyframes: [[u32; CAR_DWORDS]; TAIL_CARS],
    /// `[step][car - 4]`: the keys word and the packed analogue words.
    pub tape: [[[u32; 2]; TAIL_CARS]; STEPS],
    pub states: Vec<Tick>,
}

impl Tail {
    pub fn new() -> Self {
        Tail {
            keyframes: [[0; CAR_DWORDS]; TAIL_CARS],
            tape: [[[0; 2]; TAIL_CARS]; STEPS],
            states: Vec::new(),
        }
    }

    /// The cars' states at the second's start: the head's four, then the tail's four.
    pub fn bases(&self, head: &[[u32; CAR_DWORDS]; HEAD_CARS]) -> [CarState; MAX_CARS] {
        std::array::from_fn(|c| {
            if c < HEAD_CARS {
                CarState::of(&head[c])
            } else {
                CarState::of(&self.keyframes[c - HEAD_CARS])
            }
        })
    }

    /// The tail's bytes (`TAIL_BYTES`).
    pub fn to_bytes(&self, head: &[[u32; CAR_DWORDS]; HEAD_CARS], mask: u8) -> Vec<u8> {
        let mut out = Vec::with_capacity(TAIL_BYTES);
        out.extend(
            self.keyframes
                .iter()
                .flatten()
                .flat_map(|d| d.to_le_bytes()),
        );
        out.extend(
            self.tape
                .iter()
                .flatten()
                .flatten()
                .flat_map(|d| d.to_le_bytes()),
        );
        let states = encode_states(&self.bases(head), &self.states, mask, STATES_CAP);
        out.extend((states.len() as u32).to_le_bytes());
        out.extend(&states);
        out.resize(TAIL_BYTES, 0);
        out
    }

    /// The tail from its bytes; none unless they are `TAIL_BYTES` with sound states.
    pub fn from_bytes(
        bytes: &[u8],
        head: &[[u32; CAR_DWORDS]; HEAD_CARS],
        mask: u8,
    ) -> Option<Self> {
        if bytes.len() != TAIL_BYTES {
            return None;
        }
        let dword = |at: usize| u32::from_le_bytes(bytes[at..at + 4].try_into().expect("4"));
        let mut tail = Tail::new();
        for (n, d) in tail.keyframes.iter_mut().flatten().enumerate() {
            *d = dword(4 * n);
        }
        for (n, d) in tail.tape.iter_mut().flatten().flatten().enumerate() {
            *d = dword(TAIL_KEY_BYTES + 4 * n);
        }
        let len = dword(STATES_AT) as usize;
        let states = bytes.get(STATES_AT + 4..STATES_AT + 4 + len.min(STATES_CAP))?;
        if len > STATES_CAP {
            return None;
        }
        tail.states = if mask == 0 || len == 0 {
            Vec::new()
        } else {
            decode_states(&tail.bases(head), states, mask)?
        };
        Some(tail)
    }
}

impl Default for Tail {
    fn default() -> Self {
        Self::new()
    }
}

// ---- the extension ----

/// The car's entry flags.
pub mod flag {
    /// A human player's car.
    pub const PLAYER: u32 = 1;
    /// A computer-driven car.
    pub const COMPUTER: u32 = 2;
    /// Driven on the machine that recorded (its inputs or its driver's state are in the file).
    pub const LOCAL: u32 = 4;
    /// Not driven there: the car's motion is recorded at each step (`Tail::states`).
    pub const STATES: u32 = 8;
    /// The car of the machine that recorded.
    pub const OWN: u32 = 16;
}

/// One car of the replay.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CarEntry {
    /// The car model (0 .. 33).
    pub kind: u32,
    pub flags: u32,
    /// The driver's name, at most `NAME_LEN` bytes.
    pub name: Vec<u8>,
}

/// A car's name in the file.
pub const NAME_LEN: usize = 0x20;
/// A car's entry in the file.
pub const ENTRY_LEN: usize = 0x30;

impl CarEntry {
    pub fn is_player(&self) -> bool {
        self.flags & flag::PLAYER != 0
    }

    pub(crate) fn is_recorded_by_states(&self) -> bool {
        self.flags & flag::STATES != 0
    }
}

/// The extension: the cars and what the layout of a block is.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Extension {
    pub cars: Vec<CarEntry>,
    /// The rivals a split's place was ranked against in the race (`RIVL`); none in a file
    /// written before it was kept.
    pub rivals: Option<Vec<Rival>>,
    /// The league table a split's time was compared against in the race (`LEAG`); none in a
    /// file written before it was kept.
    pub league: Option<LeagueTable>,
}

/// The ten drivers a car's time at a split is compared against (a time trial's or a
/// championship's): their names (as the best-times files number them) and, for each of the
/// sixteen splits, their ten times.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LeagueTable {
    pub names: [u32; LEAGUE_DRIVERS],
    pub rows: [u32; LEAGUE_DRIVERS * LEAGUE_SPLITS],
}

/// The league table's drivers.
pub const LEAGUE_DRIVERS: usize = 10;
/// The league table's splits.
pub const LEAGUE_SPLITS: usize = 16;

/// One of the rivals a car's time at a split is ranked against: its name (as the best-times
/// files number it), car and time over the stage in hundredths.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rival {
    pub name: u32,
    pub car: u32,
    pub time: u32,
}

/// The most rivals a file's table holds.
pub const MOST_RIVALS: usize = 0x5A;

const TAG_CARS: [u8; 4] = *b"CARS";
const TAG_BLKS: [u8; 4] = *b"BLKS";
const TAG_NEXT: [u8; 4] = *b"NEXT";
const TAG_RIVL: [u8; 4] = *b"RIVL";
const TAG_LEAG: [u8; 4] = *b"LEAG";

/// What the sound for more than two cars and the split-screen replay use, in words: kept in the
/// file as the `NEXT` section so that a reader sees it. Every car's engine, gear and speed
/// words are in its state (`STATE_AT`) at each step for a car recorded by states, and in the
/// state at each second's start with its inputs for the others: sound for each is derived the
/// same way. The split screen is the layout `SPLIT_LAYOUT` of the head byte of a block.
const NEXT_TEXT: &[u8] = b"sound: every car's gear, engine revs and speeds are in STATE_AT; split screen: layout 10 shows every local player";

fn put_section(out: &mut Vec<u8>, tag: [u8; 4], body: &[u8]) {
    out.extend(tag);
    out.extend((body.len() as u32).to_le_bytes());
    out.extend(body);
    out.resize(out.len().next_multiple_of(4), 0);
}

impl Extension {
    /// The extension's bytes.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut out = Vec::new();
        let mut cars = vec![];
        cars.extend((self.cars.len() as u32).to_le_bytes());
        for car in &self.cars {
            cars.extend(car.kind.to_le_bytes());
            cars.extend(car.flags.to_le_bytes());
            let mut name = car.name.clone();
            name.resize(NAME_LEN, 0);
            cars.extend(&name);
            cars.resize(cars.len() + (ENTRY_LEN - 8 - NAME_LEN), 0);
        }
        put_section(&mut out, TAG_CARS, &cars);
        let mut blks = vec![];
        for v in [
            BLOCK_BYTES as u32,
            HEAD_CARS as u32,
            STEPS as u32,
            u32::from(self.state_mask()),
            STATES_CAP as u32,
            MOST_BLOCKS,
        ] {
            blks.extend(v.to_le_bytes());
        }
        put_section(&mut out, TAG_BLKS, &blks);
        if let Some(rivals) = &self.rivals {
            let mut body = (rivals.len() as u32).to_le_bytes().to_vec();
            for r in rivals {
                for v in [r.name, r.car, r.time] {
                    body.extend(v.to_le_bytes());
                }
            }
            put_section(&mut out, TAG_RIVL, &body);
        }
        if let Some(league) = &self.league {
            let body: Vec<u8> = league
                .names
                .iter()
                .chain(&league.rows)
                .flat_map(|v| v.to_le_bytes())
                .collect();
            put_section(&mut out, TAG_LEAG, &body);
        }
        put_section(&mut out, TAG_NEXT, NEXT_TEXT);
        out
    }

    /// The extension from its bytes: the sections it knows, the others skipped; none when the
    /// cars are missing or damaged, or the block's layout is not this program's.
    pub fn parse(bytes: &[u8]) -> Option<Self> {
        let mut at = 0;
        let (mut cars, mut layout_ok, mut rivals, mut league) = (None, false, None, None);
        while at + 8 <= bytes.len() {
            let tag: [u8; 4] = bytes[at..at + 4].try_into().ok()?;
            let len = u32::from_le_bytes(bytes[at + 4..at + 8].try_into().ok()?) as usize;
            let body = bytes.get(at + 8..at + 8 + len)?;
            at += 8 + len.next_multiple_of(4);
            match tag {
                TAG_CARS => cars = Some(parse_cars(body)?),
                TAG_BLKS => {
                    let w = |k: usize| {
                        u32::from_le_bytes(body[4 * k..4 * k + 4].try_into().expect("4"))
                    };
                    layout_ok = body.len() >= 24
                        && w(5) == MOST_BLOCKS
                        && w(0) as usize == BLOCK_BYTES
                        && w(1) as usize == HEAD_CARS
                        && w(2) as usize == STEPS
                        && w(4) as usize == STATES_CAP;
                }
                // A damaged table is left out: the race's own rivals are used, as for an older file.
                TAG_RIVL => rivals = parse_rivals(body),
                TAG_LEAG => league = parse_league(body),
                _ => {}
            }
        }
        let ext = Extension {
            cars: cars?,
            rivals,
            league,
        };
        layout_ok.then_some(ext)
    }

    /// The cars recorded by states, bit n for car n.
    pub fn state_mask(&self) -> u8 {
        self.cars
            .iter()
            .enumerate()
            .filter(|(_, c)| c.is_recorded_by_states())
            .fold(0, |m, (n, _)| m | 1 << n)
    }

    /// The car numbers of the human players, in order.
    pub fn player_cars(&self) -> Vec<usize> {
        self.cars
            .iter()
            .enumerate()
            .filter(|(_, c)| c.is_player())
            .map(|(n, _)| n)
            .collect()
    }

    /// The cars' models when every car of the replay is heard (each with a sound object of its
    /// own): a replay of more than two cars that the first format's format could not hold (three or
    /// four players, or cars of other machines). A replay the first format's format holds sounds as
    /// the first format's does.
    pub fn heard_kinds(&self) -> Option<Vec<u32>> {
        (self.cars.len() > 2 && !self.old_format_holds())
            .then(|| self.cars.iter().map(|c| c.kind).collect())
    }

    /// How many players the first group of a split-screen replay shows at once (each in a part
    /// of the screen as in the race): the players of a replay of three to eight, up to four, when
    /// the players come first among the cars (the views show the cars of the same numbers, as in
    /// the race); else none. Players on other machines (cars recorded by their motion) are shown
    /// together too.
    pub fn split_players(&self) -> u32 {
        let players = self.player_cars();
        let first = players.iter().enumerate().all(|(k, &n)| k == n);
        if (layouts::GROUP_LEAST..=MAX_CARS).contains(&players.len()) && first {
            players.len().min(layouts::GROUP_MOST) as u32
        } else {
            0
        }
    }

    /// Whether the first format's format can hold the race: at most two players, none of them
    /// elsewhere, and no car recorded by its states.
    pub fn old_format_holds(&self) -> bool {
        self.player_cars().len() <= 2 && self.state_mask() == 0
    }

    /// Whether the viewer has more than one layout to choose from: two players or more.
    pub fn layouts_choosable(&self) -> bool {
        self.player_cars().len() >= 2
    }

    /// What `layout` shows of this replay's players (`layouts::shown`); the groups only where
    /// the players can be split (`split_players`).
    pub fn shows(&self, layout: u8) -> Option<layouts::Shown> {
        let shown = layouts::shown(layout, self.player_cars().len())?;
        let group = matches!(shown, layouts::Shown::Group { .. });
        (!group || self.split_players() != 0).then_some(shown)
    }

    /// The layout after `old` in the cycle the viewer's car item goes through: each player alone,
    /// for a replay of two players the two together (2 side by side, 3 one above the other), and
    /// after the single players the groups of players split as the race showed them (10, then
    /// 11 and 12 for the players after the fourth: a pair side by side and one above the other
    /// when they are two, a group when they are three or four); never a computer car. Wraps to
    /// the first.
    pub fn next_layout(&self, old: u8) -> u8 {
        (old.saturating_add(1)..=layouts::LAST_LAYOUT)
            .chain(0..=old)
            .find(|&l| self.shows(l).is_some())
            .unwrap_or(0)
    }
}

/// The head byte (layout) that shows the first group of players (every player of a local
/// split-screen race of three or four) at once, each in the part of the screen the race gave it.
pub const SPLIT_LAYOUT: u32 = layouts::FIRST_GROUP as u32;
/// `SPLIT_LAYOUT` as the head byte.
pub const SPLIT_LAYOUT_BYTE: u8 = layouts::FIRST_GROUP;

/// The head byte (layout) that shows player `k` alone: 0 and 1 the first two players, then 4 on
/// (the same numbers as the extra tape's replays use for the third and fourth car).
pub fn layout_of_player(k: u32) -> u32 {
    u32::from(layouts::layout_of_player(k as usize))
}

fn parse_rivals(body: &[u8]) -> Option<Vec<Rival>> {
    let n = u32::from_le_bytes(body.get(..4)?.try_into().ok()?) as usize;
    if n > MOST_RIVALS || body.len() < 4 + 12 * n {
        return None;
    }
    let dword = |at: usize| u32::from_le_bytes(body[at..at + 4].try_into().expect("4"));
    Some(
        (0..n)
            .map(|k| Rival {
                name: dword(4 + 12 * k),
                car: dword(8 + 12 * k),
                time: dword(12 + 12 * k),
            })
            .collect(),
    )
}

fn parse_league(body: &[u8]) -> Option<LeagueTable> {
    let (words, rest) = body.as_chunks::<4>();
    let words: Vec<u32> = words.iter().map(|w| u32::from_le_bytes(*w)).collect();
    if !rest.is_empty() || words.len() != LEAGUE_DRIVERS * (1 + LEAGUE_SPLITS) {
        return None;
    }
    Some(LeagueTable {
        names: words[..LEAGUE_DRIVERS].try_into().ok()?,
        rows: words[LEAGUE_DRIVERS..].try_into().ok()?,
    })
}

fn parse_cars(body: &[u8]) -> Option<Vec<CarEntry>> {
    let n = u32::from_le_bytes(body.get(..4)?.try_into().ok()?) as usize;
    if n > MAX_CARS {
        return None;
    }
    let mut cars = Vec::with_capacity(n);
    for k in 0..n {
        let e = body.get(4 + k * ENTRY_LEN..4 + (k + 1) * ENTRY_LEN)?;
        let dword = |at: usize| u32::from_le_bytes(e[at..at + 4].try_into().expect("4"));
        let name = &e[8..8 + NAME_LEN];
        let name = name[..name.iter().position(|&b| b == 0).unwrap_or(NAME_LEN)].to_vec();
        cars.push(CarEntry {
            kind: dword(0),
            flags: dword(4),
            name,
        });
    }
    Some(cars)
}

// ---- whole files ----

/// The first format's index at a file's end: its bytes.
pub const INDEX_LEN: usize = 0x3850;

/// Which format a file is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FileFormat {
    /// The first format's, with or without the extra tape.
    Original { extra_tape: bool },
    /// This project's own.
    Own,
}

/// A replay file read: its header, its extension (the own format's) and its blocks unpacked.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReplayFile {
    pub format: FileFormat,
    pub header: Vec<u8>,
    pub extension: Option<Extension>,
    pub blocks: Vec<Vec<u8>>,
}

fn dword_at(data: &[u8], at: usize) -> Option<u32> {
    Some(u32::from_le_bytes(data.get(at..at + 4)?.try_into().ok()?))
}

impl ReplayFile {
    /// A file read as the viewer reads it: the header, the extension the header names, the
    /// index at the end and each block it lists, unpacked to the size its format has. None for
    /// anything that is not a whole replay of either format.
    pub fn parse(data: &[u8]) -> Option<Self> {
        let header = data.get(..HEADER_LEN)?.to_vec();
        let (format, block_len) = match (dword_at(data, 0)?, dword_at(data, 8)?) {
            (HEADER_MAGIC, 0x50400) => (
                FileFormat::Original { extra_tape: false },
                crate::drive::replay_frames::FRAME_BYTES,
            ),
            (HEADER_MAGIC, 0x50401) => (
                FileFormat::Original { extra_tape: true },
                crate::drive::replay_frames::EXTENDED_FRAME_BYTES,
            ),
            (MAGIC, VERSION) => (FileFormat::Own, BLOCK_BYTES),
            _ => return None,
        };
        let extension = if format == FileFormat::Own {
            let len = dword_at(data, EXT_LEN_AT)? as usize;
            Some(Extension::parse(data.get(HEADER_LEN..HEADER_LEN + len)?)?)
        } else {
            None
        };
        let (index_len, most) = match format {
            FileFormat::Own => (OWN_INDEX_LEN, MOST_BLOCKS as usize),
            FileFormat::Original { .. } => (INDEX_LEN, crate::race::replay::MOST_BLOCKS as usize),
        };
        let index = data.len().checked_sub(index_len)?;
        if data.get(index..index + 4)? != b"RCIB" {
            return None;
        }
        let count = dword_at(data, index + 8)? as usize;
        if count == 0 || count > most {
            return None;
        }
        let mut blocks = Vec::with_capacity(count);
        for k in 0..count {
            let place = dword_at(data, index + 16 + 8 * k)? as usize;
            let size = dword_at(data, index + 20 + 8 * k)? as i32;
            let block = if size < 0 {
                let packed = data.get(place..place + size.unsigned_abs() as usize)?;
                let mut out = vec![0u8; block_len];
                let n = crate::drive::rle::unpack_rle(&packed, |at, b| {
                    if let Some(slot) = out.get_mut(at as usize) {
                        *slot = b;
                    }
                })
                .ok()?;
                (n as usize == block_len).then_some(out)?
            } else {
                data.get(place..place + size as usize)
                    .filter(|b| b.len() == block_len)?
                    .to_vec()
            };
            blocks.push(block);
        }
        Some(ReplayFile {
            format,
            header,
            extension,
            blocks,
        })
    }

    /// The file's bytes as the game writes them: the header (its extension length set for the own
    /// format, zero for the first format's), the extension, each block packed when that is smaller,
    /// and the index.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut header = self.header.clone();
        let extension = self
            .extension
            .as_ref()
            .map(Extension::to_bytes)
            .unwrap_or_default();
        header[EXT_LEN_AT..EXT_LEN_AT + 4].copy_from_slice(&(extension.len() as u32).to_le_bytes());
        let mut out = header;
        out.extend(&extension);
        let mut places = Vec::new();
        for block in &self.blocks {
            places.push(out.len() as u32);
            let packed = crate::race::seal::pack_runs(block);
            match crate::race::replay::packed_size(packed.len() as u32, block.len() as u32) {
                Some(size) => {
                    let mut p = packed;
                    p.resize(size as usize, 0);
                    places.push((size as i32).wrapping_neg() as u32);
                    out.extend(p);
                }
                None => {
                    places.push(block.len() as u32);
                    out.extend(block);
                }
            }
        }
        let index_len = if self.format == FileFormat::Own {
            OWN_INDEX_LEN
        } else {
            INDEX_LEN
        };
        let mut index = Vec::with_capacity(index_len);
        index.extend(crate::race::replay::INDEX_MAGIC.to_le_bytes());
        index.extend(0u32.to_le_bytes());
        index.extend((self.blocks.len() as u32).to_le_bytes());
        index.extend((100 * self.blocks.len() as u32).to_le_bytes());
        index.extend(places.iter().flat_map(|p| p.to_le_bytes()));
        index.resize(index_len, 0);
        out.extend(index);
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn state(seed: u32) -> CarState {
        CarState {
            words: std::array::from_fn(|n| {
                seed.wrapping_mul(2_654_435_761)
                    .wrapping_add((n as u32).wrapping_mul(40503))
                    >> (n % 7)
            }),
        }
    }

    fn entry(kind: u32, flags: u32, name: &str) -> CarEntry {
        CarEntry {
            kind,
            flags,
            name: name.as_bytes().to_vec(),
        }
    }

    #[test]
    fn the_state_words_are_distinct_and_inside_a_car() {
        let mut seen = STATE_AT.to_vec();
        seen.sort_unstable();
        seen.dedup();
        assert_eq!(seen.len(), STATE_WORDS);
        assert!(STATE_AT.iter().all(|&a| a < CAR_DWORDS));
    }

    #[test]
    fn a_state_goes_in_and_out_of_a_car() {
        let mut car = [0u32; CAR_DWORDS];
        let s = state(5);
        s.put(&mut car);
        assert_eq!(CarState::of(&car), s);
        assert!(car.iter().filter(|&&d| d != 0).count() <= STATE_WORDS);
    }

    #[test]
    fn states_round_trip_for_the_cars_of_the_mask() {
        let bases: [CarState; MAX_CARS] = std::array::from_fn(|c| state(c as u32));
        let mask = 0b1011_0100u8;
        let ticks: Vec<Tick> = (0..STEPS as u32)
            .map(|t| {
                std::array::from_fn(|c| {
                    (mask >> c & 1 == 1).then(|| {
                        let mut s = bases[c];
                        for (n, w) in s.words.iter_mut().enumerate() {
                            *w = w.wrapping_add(t * (n as u32 + 1) * 3);
                        }
                        s
                    })
                })
            })
            .collect();
        let bytes = encode_states(&bases, &ticks, mask, STATES_CAP);
        let recorded: Vec<Tick> = ticks
            .iter()
            .map(|t| t.map(|s| s.map(|s| s.as_recorded())))
            .collect();
        assert_eq!(decode_states(&bases, &bytes, mask), Some(recorded));
        // A mask of nothing makes an empty second.
        assert_eq!(
            decode_states(&bases, &encode_states(&bases, &ticks, 0, 99), 0)
                .unwrap()
                .len(),
            STEPS
        );
    }

    /// A car driving smoothly (a steady place, a slowly turning body) takes a few bytes a step.
    #[test]
    fn a_smooth_drive_is_cheap() {
        let mut base = CarState {
            words: [0; STATE_WORDS],
        };
        for (n, w) in base.words.iter_mut().enumerate() {
            *w = (1.0f32 + n as f32).to_bits();
        }
        for w in &mut base.words[..3] {
            *w = 1_000_000;
        }
        let bases = [base; MAX_CARS];
        let ticks: Vec<Tick> = (0..STEPS)
            .map(|t| {
                let mut s = base;
                for w in &mut s.words[..3] {
                    *w = w.wrapping_add(19_000 * (t as u32 + 1));
                }
                for (n, w) in s.words[3..19].iter_mut().enumerate() {
                    *w = (0.5 + 0.001 * t as f32 + 0.01 * n as f32).to_bits();
                }
                let mut tick: Tick = [None; MAX_CARS];
                tick[3] = Some(s);
                tick
            })
            .collect();
        let bytes = encode_states(&bases, &ticks, 0b1000, STATES_CAP);
        assert!(bytes.len() < STEPS * 60, "{} bytes", bytes.len());
        assert_eq!(decode_states(&bases, &bytes, 0b1000).unwrap().len(), STEPS);
    }

    #[test]
    fn steps_that_do_not_fit_are_left_out_whole() {
        let bases: [CarState; MAX_CARS] = std::array::from_fn(|c| state(c as u32));
        let ticks: Vec<Tick> = (0..STEPS as u32)
            .map(|t| std::array::from_fn(|c| (c == 2).then(|| state(1000 + t))))
            .collect();
        let bytes = encode_states(&bases, &ticks, 0b100, 300);
        assert!(bytes.len() <= 300);
        let back = decode_states(&bases, &bytes, 0b100).unwrap();
        assert!(back.len() < STEPS && !back.is_empty());
        let recorded: Vec<Tick> = ticks[..back.len()]
            .iter()
            .map(|t| t.map(|s| s.map(|s| s.as_recorded())))
            .collect();
        assert_eq!(back, recorded);
    }

    #[test]
    fn a_tail_round_trips_through_its_bytes() {
        let head: [[u32; CAR_DWORDS]; HEAD_CARS] =
            std::array::from_fn(|c| std::array::from_fn(|n| (c * 31 + n) as u32));
        let mut tail = Tail::new();
        tail.keyframes[1][7] = 0xABCD;
        tail.keyframes[3][CAR_DWORDS - 1] = 9;
        tail.tape[99][3] = [5, 6];
        tail.tape[0][0] = [1, 2];
        let mask = 0b0000_0110;
        let bases = tail.bases(&head);
        tail.states = (0..STEPS as u32)
            .map(|t| {
                std::array::from_fn(|c| {
                    (mask >> c & 1 == 1).then(|| {
                        let mut s = bases[c];
                        s.words[0] = s.words[0].wrapping_add(t * 7);
                        s.words[20] = f32::to_bits(t as f32 * 0.25);
                        s
                    })
                })
            })
            .collect();
        let bytes = tail.to_bytes(&head, mask);
        assert_eq!(bytes.len(), TAIL_BYTES);
        let mut recorded = tail.clone();
        recorded.states = tail
            .states
            .iter()
            .map(|t| t.map(|s| s.map(|s| s.as_recorded())))
            .collect();
        assert_eq!(Tail::from_bytes(&bytes, &head, mask), Some(recorded));
        assert_eq!(Tail::from_bytes(&bytes[1..], &head, mask), None);
    }

    #[test]
    fn the_extension_round_trips_and_skips_what_it_does_not_know() {
        let ext = Extension {
            rivals: None,
            league: None,
            cars: vec![
                entry(5, flag::PLAYER | flag::LOCAL | flag::OWN, "Ann"),
                entry(7, flag::PLAYER | flag::STATES, "Bob"),
                entry(9, flag::COMPUTER | flag::STATES, "Computer 1"),
            ],
        };
        let bytes = ext.to_bytes();
        assert_eq!(bytes.len() % 4, 0);
        assert_eq!(Extension::parse(&bytes), Some(ext.clone()));
        // A section added later is skipped.
        let mut more = bytes.clone();
        more.extend(*b"ZZZZ");
        more.extend(5u32.to_le_bytes());
        more.extend([1, 2, 3, 4, 5, 0, 0, 0]);
        assert_eq!(Extension::parse(&more), Some(ext.clone()));
        assert_eq!(ext.state_mask(), 0b110);
        assert_eq!(ext.player_cars(), vec![0, 1]);
        assert!(!ext.old_format_holds());
        assert_eq!(Extension::parse(&bytes[..bytes.len() - 40]), None);
    }

    #[test]
    fn the_rival_tables_round_trip_and_a_damaged_one_is_left_out() {
        let rivals: Vec<Rival> = (0..MOST_RIVALS as u32)
            .map(|c| Rival {
                name: 0x5A5 + c,
                car: c % 23,
                time: 5000 + 7 * c,
            })
            .collect();
        let ext = Extension {
            cars: vec![entry(5, flag::PLAYER | flag::LOCAL | flag::OWN, "Ann")],
            rivals: Some(rivals),
            league: Some(LeagueTable {
                names: std::array::from_fn(|n| 0x5A5 + n as u32),
                rows: std::array::from_fn(|n| 1000 + n as u32),
            }),
        };
        let bytes = ext.to_bytes();
        assert_eq!(Extension::parse(&bytes), Some(ext.clone()));
        // A file without the table (written before it was kept) has none.
        let without = Extension {
            rivals: None,
            league: None,
            ..ext.clone()
        };
        assert_eq!(Extension::parse(&without.to_bytes()), Some(without.clone()));
        // A count past the table's room: the table is left out, the rest is read.
        let mut damaged = bytes.clone();
        let at = damaged.windows(4).position(|w| w == b"RIVL").expect("RIVL") + 8;
        damaged[at..at + 4].copy_from_slice(&(MOST_RIVALS as u32 + 1).to_le_bytes());
        assert_eq!(Extension::parse(&damaged).map(|e| e.rivals), Some(None));
        // A league table of the wrong size: left out too.
        let mut short = without.to_bytes();
        put_section(&mut short, TAG_LEAG, &[0; 8]);
        assert_eq!(Extension::parse(&short), Some(without));
    }

    #[test]
    fn a_layout_names_the_player_never_a_computer_car() {
        let ext = Extension {
            rivals: None,
            league: None,
            cars: vec![
                entry(1, flag::COMPUTER | flag::LOCAL, "C"),
                entry(2, flag::PLAYER | flag::LOCAL, "A"),
                entry(3, flag::COMPUTER, "D"),
                entry(4, flag::PLAYER, "B"),
                entry(4, flag::PLAYER, "E"),
            ],
        };
        // The players are cars 1, 3 and 4 (a layout names the player, not the car); three
        // players have the layouts of the first three alone, not a fourth's.
        assert_eq!(ext.player_cars(), [1, 3, 4]);
        assert_eq!(ext.shows(0), Some(layouts::Shown::Player(0)));
        assert_eq!(ext.shows(1), Some(layouts::Shown::Player(1)));
        assert_eq!(ext.shows(4), Some(layouts::Shown::Player(2)));
        assert_eq!(ext.shows(5), None);
        // Three players who are not the first cars: each alone, round and round (no group, as
        // the views show cars by number, and from three players no pair either).
        let three = Extension {
            rivals: None,
            league: None,
            cars: vec![
                entry(1, flag::PLAYER, "a"),
                entry(1, flag::COMPUTER, "x"),
                entry(1, flag::PLAYER, "b"),
                entry(1, flag::PLAYER, "c"),
            ],
        };
        let mut l = 0u8;
        let mut seen = vec![l];
        for _ in 0..5 {
            l = three.next_layout(l);
            seen.push(l);
        }
        assert_eq!(seen, [0, 1, 4, 0, 1, 4]);
        assert_eq!(ext.next_layout(0), 1);
        assert!(three.layouts_choosable());
        assert_eq!(
            (0..8).map(layout_of_player).collect::<Vec<_>>(),
            [0, 1, 4, 5, 6, 7, 8, 9]
        );
        assert!(!ext.old_format_holds() || ext.player_cars().len() <= 2);
    }

    #[test]
    fn a_local_split_screen_race_has_a_layout_that_shows_every_player() {
        let local = |players: usize, computers: usize| Extension {
            rivals: None,
            league: None,
            cars: (0..players)
                .map(|n| entry(n as u32, flag::PLAYER | flag::LOCAL, "p"))
                .chain((0..computers).map(|n| entry(n as u32, flag::COMPUTER | flag::LOCAL, "c")))
                .collect(),
        };
        let four = local(4, 2);
        assert_eq!(four.split_players(), 4);
        assert_eq!(local(3, 0).split_players(), 3);
        // Two players have no such layout; players on other machines have it too.
        assert_eq!(local(2, 3).split_players(), 0);
        let mut net = local(3, 1);
        net.cars[1].flags = flag::PLAYER | flag::STATES;
        assert_eq!(net.split_players(), 3);
        // Players that are not the first cars cannot be split (the views show cars by number).
        net.cars.swap(0, 3);
        assert_eq!(net.split_players(), 0);
        net.cars.swap(0, 3);
        let mut l = 0u8;
        let mut seen = vec![l];
        for _ in 0..8 {
            l = four.next_layout(l);
            seen.push(l);
        }
        // The first two together only in the group, with the third and fourth.
        assert_eq!(seen, [0, 1, 4, 5, 10, 0, 1, 4, 5]);
        assert_eq!(four.next_layout(5), SPLIT_LAYOUT as u8);
        // Every car of a replay of more than two that the first format's format cannot hold is
        // heard; one it holds sounds as before.
        assert_eq!(four.heard_kinds().map(|k| k.len()), Some(6));
        assert_eq!(local(2, 3).heard_kinds(), None);
        assert!(net.heard_kinds().is_some());
        assert_eq!(local(2, 0).heard_kinds(), None);
    }

    #[test]
    fn five_to_eight_players_have_a_second_split() {
        let players = |n: usize| Extension {
            rivals: None,
            league: None,
            cars: (0..n)
                .map(|k| entry(k as u32, flag::PLAYER | flag::STATES, "p"))
                .collect(),
        };
        let cycle = |ext: &Extension| {
            let mut l = 0u8;
            let mut seen = vec![l];
            loop {
                l = ext.next_layout(l);
                if l == 0 {
                    return seen;
                }
                seen.push(l);
            }
        };
        // Five: the singles and the first group of four (the fifth has nobody to split with; the
        // first two are together only in the group); six: also the fifth and sixth as a pair,
        // side by side (11) and stacked (12); seven: a group of three (11); eight: of four.
        assert_eq!(cycle(&players(5)), [0, 1, 4, 5, 6, 10]);
        assert_eq!(cycle(&players(6)), [0, 1, 4, 5, 6, 7, 10, 11, 12]);
        assert_eq!(cycle(&players(7)), [0, 1, 4, 5, 6, 7, 8, 10, 11]);
        assert_eq!(cycle(&players(8)), [0, 1, 4, 5, 6, 7, 8, 9, 10, 11]);
        assert_eq!(
            players(8).shows(11),
            Some(layouts::Shown::Group { first: 4, count: 4 })
        );
        assert_eq!(
            players(6).shows(12),
            Some(layouts::Shown::Pair {
                first: 4,
                stacked: true
            })
        );
        assert_eq!(players(5).shows(11), None);
        assert_eq!(players(7).shows(12), None);
        // Two players: just the singles and the pair.
        assert_eq!(cycle(&players(2)), [0, 1, 2, 3]);
    }

    /// A block of `len` bytes with runs and noise in it, as a block's are.
    fn block(len: usize, seed: u32) -> Vec<u8> {
        (0..len)
            .map(|n| {
                if n % 997 < 600 {
                    0
                } else {
                    (n as u32).wrapping_mul(2_654_435_761).wrapping_add(seed) as u8
                }
            })
            .collect()
    }

    fn header(magic: u32, version: u32) -> Vec<u8> {
        let mut h = vec![0u8; HEADER_LEN];
        h[..4].copy_from_slice(&magic.to_le_bytes());
        h[8..12].copy_from_slice(&version.to_le_bytes());
        h[0x10..0x16].copy_from_slice(b"a name");
        h
    }

    #[test]
    fn a_file_of_the_own_format_round_trips_with_every_section() {
        let extension = Extension {
            rivals: None,
            league: None,
            cars: vec![
                entry(5, flag::PLAYER | flag::LOCAL | flag::OWN, "Ann"),
                entry(7, flag::PLAYER | flag::STATES, "Bob"),
                entry(9, flag::COMPUTER | flag::STATES, "Computer 1"),
            ],
        };
        let file = ReplayFile {
            format: FileFormat::Own,
            header: header(MAGIC, VERSION),
            extension: Some(extension),
            blocks: (0..5).map(|k| block(BLOCK_BYTES, k)).collect(),
        };
        let bytes = file.to_bytes();
        let back = ReplayFile::parse(&bytes).expect("the file reads back");
        assert_eq!(back.format, FileFormat::Own);
        assert_eq!(back.extension, file.extension);
        assert_eq!(back.blocks, file.blocks);
        // The header is as written but for the extension's length.
        assert_eq!(back.header[..0x1B0], file.header[..0x1B0]);
        assert!(dword_at(&back.header, EXT_LEN_AT).unwrap() > 0);
        // A file cut short or with a wrong magic does not read.
        assert_eq!(ReplayFile::parse(&bytes[..bytes.len() - 1]), None);
        assert_eq!(ReplayFile::parse(&bytes[..0x500]), None);
    }

    /// A stage of 50 minutes (3000 blocks, past the first format's 1800) reads in the own format and
    /// is refused in the first format's.
    #[test]
    fn a_long_stage_fits_the_own_index_only() {
        let one = ReplayFile {
            format: FileFormat::Own,
            header: header(MAGIC, VERSION),
            extension: Some(Extension {
                rivals: None,
                league: None,
                cars: vec![entry(5, flag::PLAYER | flag::LOCAL, "Ann")],
            }),
            blocks: vec![vec![0u8; BLOCK_BYTES]],
        };
        let mut bytes = one.to_bytes();
        let index = bytes.len() - OWN_INDEX_LEN;
        let (place, size) = (
            u32::from_le_bytes(bytes[index + 16..index + 20].try_into().unwrap()),
            u32::from_le_bytes(bytes[index + 20..index + 24].try_into().unwrap()),
        );
        for k in 0..3000usize {
            let at = index + 16 + 8 * k;
            bytes[at..at + 4].copy_from_slice(&place.to_le_bytes());
            bytes[at + 4..at + 8].copy_from_slice(&size.to_le_bytes());
        }
        bytes[index + 8..index + 12].copy_from_slice(&3000u32.to_le_bytes());
        let back = ReplayFile::parse(&bytes).expect("3000 blocks read");
        assert_eq!(back.blocks.len(), 3000);
        // More than the index holds is refused.
        bytes[index + 8..index + 12].copy_from_slice(&(MOST_BLOCKS + 1).to_le_bytes());
        assert_eq!(ReplayFile::parse(&bytes), None);
        assert_eq!(OWN_INDEX_LEN, 43_216);
        const { assert!(MOST_BLOCKS > crate::race::replay::MOST_BLOCKS * 2) };
    }

    #[test]
    fn the_originals_files_are_still_read_both_versions() {
        use crate::drive::replay_frames::{EXTENDED_FRAME_BYTES, FRAME_BYTES};
        for (version, len, extra_tape) in [
            (0x50400, FRAME_BYTES, false),
            (0x50401, EXTENDED_FRAME_BYTES, true),
        ] {
            let file = ReplayFile {
                format: FileFormat::Original { extra_tape },
                header: header(HEADER_MAGIC, version),
                extension: None,
                blocks: (0..3).map(|k| block(len, k)).collect(),
            };
            let bytes = file.to_bytes();
            // The first format's header is untouched: nothing at the extension's place.
            assert_eq!(dword_at(&bytes, EXT_LEN_AT), Some(0));
            assert_eq!(bytes[..HEADER_LEN], file.header[..]);
            assert_eq!(ReplayFile::parse(&bytes), Some(file));
        }
    }

    #[test]
    fn the_magics_differ() {
        assert!(is_magic(MAGIC) && !is_magic(HEADER_MAGIC));
        assert!(is_replay_magic(HEADER_MAGIC) && is_replay_magic(MAGIC));
        assert_eq!(&MAGIC.to_le_bytes(), b"RC2R");
        assert_eq!(BLOCK_BYTES, 0x2E50 + TAIL_BYTES);
    }
}
