//! The race core's variables, out of RAL.EXE's image (phase 5, `notes/conversion.md`): IDA
//! 0x5BEC48 .. 0x747AD8, from the PNG reader's memory list to the menus' tables (its tail
//! from 0x7475D0, the best times' memory and the screens' items, typed since: phase 7). The
//! replay's file state and index, the course, the cars' state, physics and sounds, the
//! camera and views, the scene's work, the stage's and the records' tables, the timing.
//!
//! One block with the image's layout (`crate::store`), first values the image's
//! (`store_data.rs`, generated). Six ranges inside it had moved before into blocks of
//! their own (the mouse, `g_raceSoundsOn`, the windscreen drops' batch, ...): their bytes here are
//! not used, and `crate::moved::MOVED` lists this block as the ranges around them.
//! The code names the variables by IDA's address (`framework::game_addr` takes such an
//! address here).
//!
//! The image's zeroed tail after it (IDA 0x960000 .. 0xA8C180) is `race::buffers_store`.

use crate::block;

block!(pub mod vars {
    start: 0x5BEC48,
    end: 0x747AD8,
    runs: crate::race::store_data::RUNS,
});

/// Tables only the original's dead code read (every reference is from the debug pages
/// or the developers' front end; the debug UI has its own): the watch list, the surface names,
/// the front end's name tables, the pace notes' `off_61FE18` (its one live read is
/// overwritten at once), the debug pages' titles (`g_debugPageTitles`, read only
/// by `debug::draw_page_title`), the developers' front end's digit texts (`off_621C44`, read
/// only by `dev_front_end__draw`) and the texture kinds' names `off_64CD0C` (its one read,
/// 0x459D6F in 0x459CC2, a stage-by-stage dump of the texture kinds to the log reached only
/// from the `running sub program` chunk at 0x442BE3, no function of the Rust side). Their
/// addresses are not pointed anywhere and nothing may read them.
///
/// Also `g_objectMatrix` (0x5EA40C; `scene::place_object_points` takes the matrix it always
/// names) and two sample records' levels before the sample table (no reference in IDA).
pub const UNREAD: [(usize, usize); 12] = [
    (0x50A744, 0x50A748),
    (0x50A784, 0x50A788),
    (0x5EA40C, 0x5EA410),
    (0x5EBB5A, 0x5EE258),
    (0x5FFF12, 0x5FFF32),
    (0x5FFFCC, 0x60000C),
    (0x61FE18, 0x620218),
    (0x621C44, 0x621C5C),
    (0x621C68, 0x621E68),
    (0x622084, 0x622114),
    (0x622238, 0x6223C8),
    (0x64CD0C, 0x64CDA8),
];

/// Whether IDA's `ida` lies in a table of `UNREAD`.
pub fn is_unread(ida: usize) -> bool {
    UNREAD.iter().any(|&(s, e)| (s..e).contains(&ida))
}

/// The ranges of the block that are its own (IDA start, end): all but the holes and the
/// pieces that left it for statics of their own.
pub const OWN: [(usize, usize); 23] = [
    (0x5BEC48, 0x5BEC78),
    (0x5C2714, 0x5C291C),
    (0x5C5E50, 0x5C5F48),
    (0x5C5FB8, 0x5C6604),
    (0x5C6610, 0x5C6924),
    (0x5C6948, 0x5C694C),
    (0x5C7908, 0x5C790C),
    (0x5EA8C8, 0x5EB100),
    (0x5EB104, 0x60101C),
    (0x60230C, 0x60410C),
    (0x60526C, 0x60626C),
    (0x60B0BD, 0x60B2CC),
    (0x60B2D0, 0x613828),
    (0x61FE18, 0x620458),
    (0x620474, 0x621480),
    (0x6218B0, 0x6218F0),
    (0x6218F4, 0x621900),
    (0x621A84, 0x62290C),
    (0x622910, 0x6235E8),
    (0x628A44, 0x62DA44),
    (0x64098C, 0x64158C),
    (0x64BB88, 0x6D13FE),
    (0x71CA60, 0x731B82),
];

/// Whether the game address `addr` lies in the block's bytes but in none of its own ranges:
/// a hole's or a left piece's old place, which nothing may reach any more (an access by
/// address that ran past a neighbour's end and would read stale bytes).
pub fn is_dead(addr: u32) -> bool {
    let start = vars::base() as u32;
    if !(start..start + vars::LEN as u32).contains(&addr) {
        return false;
    }
    let ida = vars::START + (addr - start) as usize;
    is_unread(ida) || !OWN.iter().any(|&(s, e)| (s..e).contains(&ida))
}

/// The image's first values of the `N` bytes from IDA's `start` in the block (a left
/// piece's static).
pub const fn first_values<const N: usize>(start: usize) -> [u8; N] {
    let runs = crate::race::store_data::RUNS;
    let from = start - vars::START;
    let mut bytes = [0u8; N];
    let mut r = 0;
    while r < runs.len() {
        let (offset, run) = runs[r];
        // Only the overlap of the run with the wanted range (a long run was walked whole).
        let mut k = from.saturating_sub(offset);
        while k < run.len() && offset + k < from + N {
            bytes[offset + k - from] = run[k];
            k += 1;
        }
        r += 1;
    }
    bytes
}

/// A part of a record of cells, from byte `offset` to its end: a moved range that starts
/// inside a store (`SealWork`'s fields past its alignment lead, the panel past its two lead
/// bytes). Reads and writes go through the whole record's cells.
pub struct RecordTail<T: crate::pointers::Cells + Sync + 'static> {
    whole: &'static T,
    offset: usize,
}

impl<T: crate::pointers::Cells + Sync + 'static> RecordTail<T> {
    pub const fn new(whole: &'static T, offset: usize) -> Self {
        Self { whole, offset }
    }
}

// SAFETY: `first` is the record's byte at the window's offset, and `size` what is left of it.
#[allow(
    unsafe_code,
    reason = "Store's promise: the window's bytes are the record's"
)]
unsafe impl<T: crate::pointers::Cells + Sync + 'static> crate::pointers::Store for RecordTail<T> {
    fn size(&self) -> usize {
        size_of::<T>() - self.offset
    }

    fn first(&self) -> *const u8 {
        std::ptr::from_ref(self.whole)
            .cast::<u8>()
            .wrapping_add(self.offset)
    }

    fn cell_run_at(&self, offset: usize) -> Option<(usize, crate::pointers::CellRun<'_>)> {
        // A run that begins before the window is taken from the window's first byte, when a
        // cell begins there.
        let (start, run) = crate::pointers::Cells::cell_run(self.whole, self.offset + offset);
        match start.checked_sub(self.offset) {
            Some(start) => Some((start, run)),
            None => Some((0, run.skip(self.offset - start)?)),
        }
    }

    fn load(&self, offset: usize, to: &mut [u8]) -> Result<(), crate::errors::BadInput> {
        crate::pointers::load_cells(self.whole, self.offset + offset, to)
    }

    fn store(&self, offset: usize, from: &[u8]) -> Result<(), crate::errors::BadInput> {
        crate::pointers::store_cells(self.whole, self.offset + offset, from)
    }
}

/// `OWN`'s ranges as stores, in the same order.
pub static OWN_RANGES: [crate::store::BlockRange; OWN.len()] = {
    let mut ranges = [const { crate::store::BlockRange::new(first_byte, 0, 0) }; OWN.len()];
    let mut i = 0;
    while i < OWN.len() {
        ranges[i] =
            crate::store::BlockRange::new(first_byte, OWN[i].0 - vars::START, OWN[i].1 - OWN[i].0);
        i += 1;
    }
    ranges
};

/// The dword at IDA's `ida` in the block's own bytes (the debug pages' figures that no typed
/// field holds); none outside `OWN` or when it runs past the end of its range.
#[cfg(debug_ui)]
pub fn block_dword(ida: usize) -> Option<u32> {
    use crate::pointers::Store;
    let i = OWN
        .iter()
        .position(|&(start, end)| (start..end).contains(&ida))?;
    let mut bytes = [0u8; 4];
    OWN_RANGES[i].load(ida - OWN[i].0, &mut bytes).ok()?;
    Some(u32::from_le_bytes(bytes))
}

/// The block's first IDA byte in memory (past the alignment padding).
fn first_byte() -> *mut u8 {
    vars::STORE
        .ptr()
        .cast::<u8>()
        .wrapping_add(vars::START % 16)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The texture kinds' names table (`off_64CD0C`, 39 dwords) is unread and holds the image's
    /// addresses of its names, which lie just after it.
    #[test]
    fn the_texture_kinds_names_are_unread() {
        let image =
            crate::test_support::first_values::bytes("race.store.UNREAD.texture_kind_names");
        let &(start, end) = UNREAD
            .iter()
            .find(|r| r.1 - r.0 == image.len())
            .expect("the table is among the unread ranges");
        for (n, word) in image.chunks(4).enumerate() {
            let name = u32::from_le_bytes(word.try_into().unwrap()) as usize;
            assert!((end..end + 0x100).contains(&name), "entry {n}");
            assert!(is_unread(start + 4 * n));
        }
    }
}
