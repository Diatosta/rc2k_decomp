//! The marks' variables, out of RAL.EXE's image (phase 5, `notes/conversion.md`): IDA
//! 0x764F40 .. 0x93CCC8, the whole of it the marks' (the weather and surface effects: their
//! settings, floats, work areas, the spline, shape, square, hexagon, ring and particle
//! tables). Nothing outside `marks` reads or writes it but the one setting `race::frame_views`
//! asks (`marks::setup::SETTINGS`, typed).
//!
//! Phase 7 (typed state) takes the block apart: each piece that has its typed struct and
//! static (`CARVED`: pong, clock, rain, waves, clip, glow, mark slots, view vectors, snow, the
//! settings pointer) leaves it, with its own `moved::MOVED` entry; what is left is `SEGS`.
//! The block's bytes under a carved range are dead (a `Slot` there does not compile).
//!
//! One block with the image's layout (gaps and unreferenced bytes kept, so a slip that
//! reads past a table reads what the original read, and 16-byte alignment holds for the
//! `xmmword_8D2000` constants): an address `X` of IDA is at `STORE + (X - START)`. A `Slot`
//! is such an address (`Slot::at(0x814724)`), checked against the block when it is written.
//! `game_read` / `game_write` / `game_addr` here take a `Slot` or, as the framework's, an
//! RVA of the image: a name the marks keep for a variable of the block cannot reach the
//! image's old, unchanging copy, because it is not a `usize`. The x87 blocks (`marks::plain`)
//! reach the block through `base()`: `mk + offset` with `offset` IDA's address less `START`.
//!
//! Threads: the marks run on the game's thread alone, as the image's bytes did.

use crate::framework;
use crate::marks::store_data::RUNS;
use crate::moved::Place;
use core::ops::Add;

/// IDA's address of the block's first byte.
pub const START: usize = 0x764F40;
/// IDA's address past the block's last byte (the next is the zlib texts, 0x93CCC8).
pub const END: usize = 0x93CCC8;
pub const LEN: usize = END - START;

#[repr(C, align(16))]
pub struct Store([u8; LEN]);

const fn initial() -> Store {
    let mut bytes = [0u8; LEN];
    let mut r = 0;
    while r < RUNS.len() {
        let (offset, run) = RUNS[r];
        let mut k = 0;
        while k < run.len() {
            bytes[offset + k] = run[k];
            k += 1;
        }
        r += 1;
    }
    Store(bytes)
}

/// The block (its first values are the image's bytes, `RUNS`).
pub static STORE: Place<Store> = Place::new(initial());

/// The ranges that left the block, each its own typed static in the module that owns it
/// (phase 7; each with its own `moved::MOVED` entry). The block's bytes there are dead: a
/// `Slot` in one does not compile or stops the program, and `SEGS` (the rest of the block,
/// each with its own `MOVED` entry) leaves them out.
pub const CARVED: &[(usize, usize)] = &[
    (0x764F90, 0x764FC8), // `marks::view_vectors::ViewSources`
    (0x764FC8, 0x764FFC), // `marks::setup::MarksClock`
    (0x767A28, 0x767A2C), // `marks::setup::MarksSettings`
    (0x768658, 0x7687DC), // `marks::pong::PongState`
    (0x7687E4, 0x7688B4), // `marks::clip::ClipScreen`
    (0x769182, 0x769232), // `marks::pong::PongRules`
    (0x769236, 0x769312), // `marks::view_vectors::KeptView`
    (0x8BC405, 0x8BDE55), // `marks::rain::RainState`
    (0x8C5F91, 0x8C5FFD), // `marks::rain::RainRules`
    (0x8C6000, 0x8D2000), // `marks::snow::SnowTables`
    (0x8D2000, 0x8D2310), // `marks::snow::SnowState`
    (0x8D2310, 0x8DA310), // `marks::snow::SnowCurve`
    (0x8DA410, 0x8DB430), // `marks::snow::SnowCurveVars`
    (0x8DB430, 0x8DB558), // `marks::snow::SnowLight`
    (0x92001E, 0x92051E), // `marks::waves::MarksSines`
    (0x9205F6, 0x92063A), // `marks::waves::WaveLevels`
    (0x9206C2, 0x9206D2), // `marks::waves::WaveState`
    (0x934F7C, 0x935300), // `marks::mark_slots::MarkSlots`
    (0x9353A6, 0x935412), // `marks::mark_slots::SlotTuning`
    (0x93CC70, 0x93CCB0), // `marks::clip::GlowState`
];

/// What is left of the block after `CARVED`: (first address, address past the last), in
/// order. `MOVED` has an entry for each (`home_of`).
pub const SEGS: &[(usize, usize)] = &[
    (START, 0x764F90),
    (0x764FFC, 0x767A28),
    (0x767A2C, 0x768658),
    (0x7687DC, 0x7687E4),
    (0x7688B4, 0x769182),
    (0x769232, 0x769236),
    (0x769312, 0x8BC405),
    (0x8BDE55, 0x8C5F91),
    (0x8C5FFD, 0x8C6000),
    (0x8DA310, 0x8DA410),
    (0x8DB558, 0x92001E),
    (0x92051E, 0x9205F6),
    (0x92063A, 0x9206C2),
    (0x9206D2, 0x934F7C),
    (0x935300, 0x9353A6),
    (0x935412, 0x93CC70),
    (0x93CCB0, END),
];

/// `moved::MOVED`'s home for the segment of `SEGS` that starts at IDA address `FIRST`.
pub fn home_of<const FIRST: usize>() -> usize {
    STORE.addr() + (FIRST - START)
}

/// The start of the segment that holds IDA address `ida` (the profile ramps' lists, whose
/// pointers `moved::RAW_KEPT` keeps as IDA addresses).
pub const fn segment_start(ida: usize) -> usize {
    let mut n = 0;
    while n < SEGS.len() {
        if SEGS[n].0 <= ida && ida < SEGS[n].1 {
            return SEGS[n].0;
        }
        n += 1;
    }
    panic!("not in a segment of the marks' block")
}

const fn carved(ida: usize) -> bool {
    let mut n = 0;
    while n < CARVED.len() {
        if CARVED[n].0 <= ida && ida < CARVED[n].1 {
            return true;
        }
        n += 1;
    }
    false
}

/// A place in the block: the offset from its start.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Slot(usize);

impl Slot {
    /// The place of IDA's address `ida`; one outside the block does not compile (a
    /// constant) or stops the program.
    pub const fn at(ida: usize) -> Slot {
        assert!(
            START <= ida && ida < END,
            "not a variable of the marks' block"
        );
        assert!(!carved(ida), "a variable that left the marks' block");
        Slot(ida - START)
    }

    /// The place's address.
    pub fn address(self) -> usize {
        STORE.addr() + self.0
    }
}

impl Add<usize> for Slot {
    type Output = Slot;
    fn add(self, n: usize) -> Slot {
        Slot(self.0.wrapping_add(n))
    }
}

impl Add<u32> for Slot {
    type Output = Slot;
    fn add(self, n: u32) -> Slot {
        Slot(self.0.wrapping_add(n as usize))
    }
}

impl Add<i32> for Slot {
    type Output = Slot;
    fn add(self, n: i32) -> Slot {
        Slot(self.0.wrapping_add(n as usize))
    }
}

/// What `game_read` and the others take: a place in the block, or an RVA of the image.
pub trait Where: Copy {
    fn address(self) -> usize;
}

impl Where for usize {
    fn address(self) -> usize {
        framework::game_addr(self)
    }
}

impl Where for Slot {
    fn address(self) -> usize {
        Slot::address(self)
    }
}

/// The address of `place`.
pub fn game_addr(place: impl Where) -> usize {
    place.address()
}

/// Pointer to the game's memory at `place`.
pub fn game_ptr<T>(place: impl Where) -> *mut T {
    place.address() as *mut T
}

/// Reads a variable (`framework::game_read`, over the block or the image).
pub unsafe fn game_read<T: Copy>(place: impl Where) -> T {
    let p = game_ptr::<T>(place);
    unsafe {
        if p.is_aligned() {
            p.read_volatile()
        } else {
            p.read_unaligned()
        }
    }
}

/// Writes a variable (`framework::game_write`, over the block or the image).
pub unsafe fn game_write<T: Copy>(place: impl Where, value: T) {
    let p = game_ptr::<T>(place);
    unsafe {
        if p.is_aligned() {
            p.write_volatile(value)
        } else {
            p.write_unaligned(value)
        }
    }
}

/// The block's address for the x87 blocks (`mk`): `mk + offset` with `offset` IDA's
/// address less `START` (0x764F40) is the variable.
pub fn base() -> u32 {
    STORE.addr() as u32
}

/// The address of the block's variable a pointer of the block's data names by IDA's address
/// (the profile tables' lists at 0x9152F6, whose entries the image holds relocated): stops
/// the program for one outside the block.
pub fn from_ida(ida: usize) -> usize {
    assert!(
        (START..END).contains(&ida),
        "a pointer of the marks' data names {ida:#x}, outside the block"
    );
    STORE.addr() + (ida - START)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::framework::IMAGE_BASE;

    #[test]
    fn first_values_are_the_games() {
        // Read in place: a copy of the block would not fit the test thread's stack.
        let first = unsafe { &(&(*STORE.ptr()).0)[..] };
        let image = crate::moved::image_bytes(START, LEN);
        assert!(
            first == &image[..],
            "the block's bytes differ from the image's"
        );
    }

    #[test]
    fn segments_and_carved_ranges_partition_the_block() {
        let mut all: Vec<(usize, usize)> = SEGS.iter().chain(CARVED).copied().collect();
        all.sort();
        assert_eq!(all[0].0, START);
        assert_eq!(all.last().unwrap().1, END);
        for w in all.windows(2) {
            assert_eq!(
                w[0].1, w[1].0,
                "{:#x} .. {:#x} and the next",
                w[0].0, w[0].1
            );
        }
    }

    #[test]
    fn moved_knows_the_block() {
        assert_eq!(crate::moved::place(0x814724), Slot::at(0x814724).address());
        assert_eq!(crate::moved::place(START), STORE.addr());
        assert_eq!(
            crate::moved::place(END),
            framework::game_addr(END - IMAGE_BASE)
        );
        assert_eq!(
            crate::moved::place(START - 4),
            framework::game_addr(START - 4 - IMAGE_BASE)
        );
    }

    #[test]
    fn base_and_offset_land_on_the_block() {
        let offset = 0x814724 - START;
        assert_eq!(base() as usize + offset, Slot::at(0x814724).address());
    }
}
