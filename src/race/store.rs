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
//! address here); the inline asm reads them as `[{m} + RVA]` with `m = asm_base()`.
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

/// The typed pieces over the block (IDA start, end, the store's name), in order: for the
/// census of accesses by address (2026-10-29, `notes/history.md`) and the moves out.
#[cfg(test)]
pub const PIECES: [(usize, usize, &str); 48] = [
    (
        crate::race::replay_file_store::START,
        crate::race::replay_file_store::END,
        "REPLAY_FILE",
    ),
    (
        crate::race::viewer_state_store::START,
        crate::race::viewer_state_store::END,
        "REPLAY_VIEW",
    ),
    (
        crate::race::viewer_screen_store::START,
        crate::race::viewer_screen_store::END,
        "VIEWER_PANEL",
    ),
    (
        crate::race::effects_settings_store::START,
        crate::race::effects_settings_store::END,
        "EFFECTS_SETTINGS",
    ),
    (
        crate::race::kind_store::START,
        crate::race::kind_store::END,
        "KIND_TABLES",
    ),
    (
        crate::race::numbers_store::START,
        crate::race::numbers_store::END,
        "NUMBERS",
    ),
    (
        crate::race::scratch_store::START,
        crate::race::scratch_store::END,
        "FRAME_SCRATCH",
    ),
    (
        crate::race::text_state_store::START,
        crate::race::text_state_store::END,
        "TEXT_STATE",
    ),
    (
        crate::race::course_lists_store::START,
        crate::race::course_lists_store::END,
        "COURSE_LISTS",
    ),
    (
        crate::race::car_specs_store::START,
        crate::race::car_specs_store::END,
        "CAR_SPECS",
    ),
    (
        crate::race::light_records_store::START,
        crate::race::light_records_store::END,
        "LIGHT_RECORDS",
    ),
    (
        crate::physics::gear_store::START,
        crate::physics::gear_store::END,
        "GEAR",
    ),
    (
        crate::race::clock_ticks_store::START,
        crate::race::clock_ticks_store::END,
        "CLOCK_TICKS",
    ),
    (
        crate::race::kind_records_store::START,
        crate::race::kind_records_store::END,
        "KIND_RECORDS",
    ),
    (
        crate::race::course_store::START,
        crate::race::course_store::END,
        "COURSE",
    ),
    (
        crate::scene::object_tables_store::START,
        crate::scene::object_tables_store::END,
        "OBJECT_TABLES",
    ),
    (
        crate::scene::kind_texture_store::START,
        crate::scene::kind_texture_store::END,
        "KIND_TEXTURES",
    ),
    (
        crate::scene::texture_places_store::START,
        crate::scene::texture_places_store::END,
        "TEXTURE_PLACES",
    ),
    (
        crate::race::matrices_store::START,
        crate::race::matrices_store::END,
        "MATRICES",
    ),
    (
        crate::race::frame_counter_store::START,
        crate::race::frame_counter_store::END,
        "FRAME_COUNTER",
    ),
    (
        crate::race::tables_store::START,
        crate::race::tables_store::END,
        "RACE_TABLES",
    ),
    (
        crate::race::view_vars_store::START,
        crate::race::view_vars_store::END,
        "VIEW_VARS",
    ),
    (
        crate::physics::camera_store::START,
        crate::physics::camera_store::END,
        "CAMERA_WORK",
    ),
    (
        crate::race::prefs_store::START,
        crate::race::prefs_store::END,
        "STAGE_PREFS",
    ),
    (
        crate::scene::file_numbers_store::START,
        crate::scene::file_numbers_store::END,
        "FILE_NUMBERS",
    ),
    (
        crate::physics::tables_store::START,
        crate::physics::tables_store::END,
        "PHYSICS",
    ),
    (
        crate::race::switches_store::START,
        crate::race::switches_store::END,
        "FRAME_SWITCHES",
    ),
    (
        crate::scene::draw_switches_store::START,
        crate::scene::draw_switches_store::END,
        "DRAW_SWITCHES",
    ),
    (
        crate::race::vars_store::START,
        crate::race::vars_store::END,
        "RACE_VARS",
    ),
    (
        crate::scene::draw_store::START,
        crate::scene::draw_store::END,
        "DRAW",
    ),
    (
        crate::scene::light_work_store::START,
        crate::scene::light_work_store::END,
        "LIGHT_WORK",
    ),
    (
        crate::scene::object_store::START,
        crate::scene::object_store::END,
        "OBJECT_WORK",
    ),
    (
        crate::scene::light_store::START,
        crate::scene::light_store::END,
        "LIGHT_STATE",
    ),
    (
        crate::physics::ground_store::START,
        crate::physics::ground_store::END,
        "GROUND_WORK",
    ),
    (
        crate::race::view_store::START,
        crate::race::view_store::END,
        "RACE_VIEW",
    ),
    (
        crate::race::level_grid_store::START,
        crate::race::level_grid_store::END,
        "LEVEL_GRID",
    ),
    (
        crate::race::frame_store::START,
        crate::race::frame_store::END,
        "FRAME_WORK",
    ),
    (
        crate::race::panel_store::START,
        crate::race::panel_store::END,
        "PANEL",
    ),
    (
        crate::race::overlay_store::START,
        crate::race::overlay_store::END,
        "RACE_OVERLAY",
    ),
    (
        crate::race::sound_spots_store::START,
        crate::race::sound_spots_store::END,
        "RACE_SOUND_SPOTS",
    ),
    (
        crate::physics::contact_store::START,
        crate::physics::contact_store::END,
        "CONTACT_WORK",
    ),
    (
        crate::race::draw_batches_store::START,
        crate::race::draw_batches_store::END,
        "DRAW_BATCHES",
    ),
    (
        crate::drive::replay_frames_store::START,
        crate::drive::replay_frames_store::END,
        "REPLAY_FRAMES",
    ),
    (
        crate::race::blocks_store::START,
        crate::race::blocks_store::END,
        "CAR_BLOCKS",
    ),
    (
        crate::race::replay_work_store::START,
        crate::race::replay_work_store::END,
        "REPLAY_WORK",
    ),
    (
        crate::physics::car_store::START,
        crate::physics::car_store::END,
        "CAR",
    ),
    (
        crate::race::work_store::START,
        crate::race::work_store::END,
        "RACE_WORK",
    ),
    (
        crate::effects::work_store::START,
        crate::effects::work_store::END,
        "MARKS_WORK",
    ),
];

/// The piece IDA's address `ida` of the block lies in ("block": none, the block's own bytes).
#[cfg(test)]
pub fn piece_of(ida: usize) -> &'static str {
    let i = PIECES.partition_point(|&(s, _, _)| s <= ida);
    match i.checked_sub(1).map(|i| PIECES[i]) {
        Some((_, e, name)) if ida < e => name,
        _ => "block",
    }
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

/// What an RVA of the block is added to in inline asm (`[{m} + RVA]`): the block's address
/// less its first byte's RVA. Only the tests' kept assembly uses it now (2026-10-28: the
/// game's code reaches the block through the typed stores).
#[cfg(test)]
pub fn asm_base() -> u32 {
    (vars::base() as u32).wrapping_sub(crate::framework::rva(vars::START) as u32)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_pieces_are_in_order_apart() {
        for w in PIECES.windows(2) {
            assert!(w[0].1 <= w[1].0, "{}", w[0].2);
        }
        assert_eq!(piece_of(0x5CB958), "GEAR");
        assert_eq!(piece_of(0x5CC197), "GEAR");
        assert_eq!(piece_of(0x5CB954), "block");
        assert_eq!(piece_of(0x5C2F80), "VIEWER_PANEL");
        assert_eq!(piece_of(0x5C2F84), "EFFECTS_SETTINGS");
        assert_eq!(piece_of(0x5C56EC), "KIND_TABLES");
    }

    /// The texture kinds' names table (`off_64CD0C`, 39 dwords) is unread and holds the image's
    /// addresses of its names, which lie just after it.
    #[test]
    fn the_texture_kinds_names_are_unread() {
        let (start, end) = (0x64CD0C, 0x64CDA8);
        assert!(UNREAD.contains(&(start, end)));
        let image =
            crate::test_support::first_values::bytes("race.store.UNREAD.texture_kind_names");
        assert_eq!(image.len(), end - start);
        for (n, word) in image.chunks(4).enumerate() {
            let name = u32::from_le_bytes(word.try_into().unwrap()) as usize;
            assert!((end..end + 0x100).contains(&name), "entry {n}");
            assert!(is_unread(start + 4 * n));
        }
    }

    #[test]
    fn asm_base_reaches_the_block() {
        let rva = crate::framework::rva(0x5BEC50) as u32;
        assert_eq!(
            asm_base().wrapping_add(rva),
            crate::framework::game_addr(rva as usize)
        );
    }
}
