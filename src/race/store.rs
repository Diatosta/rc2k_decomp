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

/// Tables only the original's dead code read (IDA 2026-10-30, every xref from the debug pages
/// or the developers' front end; the debug UI has its own): the watch list, the surface names,
/// the front end's name tables, the pace notes' `off_61FE18` (its one live read, 0x441DE9, is
/// overwritten at once by `pop ebx`), the debug pages' titles (`g_debugPageTitles`, read only
/// by `debug::draw_page_title`) and the developers' front end's digit texts (`off_621C44`,
/// read only by `dev_front_end__draw`). Their addresses are not pointed anywhere and nothing
/// may read them.
pub const UNREAD: [(usize, usize); 8] = [
/// overwritten at once by `pop ebx`), the texture kinds' names `off_64CD0C` (its one read,
/// 0x459D6F in `sub_459CC2`, a stage-by-stage dump of the texture kinds to the log reached only
/// from the `running sub program` chunk at 0x442BE3, no function of the Rust side). Their addresses are not pointed anywhere and nothing may
/// read them.
pub const UNREAD: [(usize, usize); 7] = [
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
/// pieces that left it (`LEFT`).
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

/// The typed pieces that left the block for statics of their own (2026-10-29, phase 13):
/// the twelve the census saw reached by address only from their own stores, and three
/// groups the original reaches across (`race::groups_store`); then every other piece and a
/// fourth group (batch Z); then the stores typed over the block's own bytes (batch AA). Their old
/// places in the block are dead (`is_dead`).
#[cfg(test)]
pub const LEFT: [(usize, usize); 56] = [
    crate::race::groups_store::VIEWER_AND_KINDS_RANGE,
    crate::race::groups_store::LIGHTS_AND_GEAR_RANGE,
    crate::race::groups_store::CARS_AND_REPLAY_RANGE,
    (
        crate::race::text_state_store::START,
        crate::race::text_state_store::END,
    ),
    (
        crate::race::clock_ticks_store::START,
        crate::race::clock_ticks_store::END,
    ),
    (
        crate::race::kind_records_store::START,
        crate::race::kind_records_store::END,
    ),
    (
        crate::scene::kind_texture_store::START,
        crate::scene::kind_texture_store::END,
    ),
    (
        crate::race::frame_counter_store::START,
        crate::race::frame_counter_store::END,
    ),
    (
        crate::race::switches_store::START,
        crate::race::switches_store::END,
    ),
    (
        crate::scene::draw_switches_store::START,
        crate::scene::draw_switches_store::END,
    ),
    (
        crate::scene::light_work_store::START,
        crate::scene::light_work_store::END,
    ),
    (
        crate::scene::object_store::START,
        crate::scene::object_store::END,
    ),
    (
        crate::physics::ground_store::START,
        crate::physics::ground_store::END,
    ),
    (
        crate::race::draw_batches_store::START,
        crate::race::draw_batches_store::END,
    ),
    (
        crate::effects::work_store::START,
        crate::effects::work_store::END,
    ),
    crate::race::groups_store::PANEL_AND_OVERLAY_RANGE,
    (
        crate::race::replay_file_store::START,
        crate::race::replay_file_store::END,
    ),
    (
        crate::race::viewer_state_store::START,
        crate::race::viewer_state_store::END,
    ),
    (
        crate::race::numbers_store::START,
        crate::race::numbers_store::END,
    ),
    (
        crate::race::scratch_store::START,
        crate::race::scratch_store::END,
    ),
    (
        crate::race::course_lists_store::START,
        crate::race::course_lists_store::END,
    ),
    (
        crate::race::car_specs_store::START,
        crate::race::car_specs_store::END,
    ),
    (
        crate::race::course_store::START,
        crate::race::course_store::END,
    ),
    (
        crate::scene::object_tables_store::START,
        crate::scene::object_tables_store::END,
    ),
    (
        crate::scene::texture_places_store::START,
        crate::scene::texture_places_store::END,
    ),
    (
        crate::race::matrices_store::START,
        crate::race::matrices_store::END,
    ),
    (
        crate::race::tables_store::START,
        crate::race::tables_store::END,
    ),
    crate::race::groups_store::VIEW_AND_RECORDS_RANGE,
    (
        crate::physics::camera_store::START,
        crate::physics::camera_store::END,
    ),
    (
        crate::race::prefs_store::START,
        crate::race::prefs_store::END,
    ),
    (
        crate::scene::file_numbers_store::START,
        crate::scene::file_numbers_store::END,
    ),
    (
        crate::physics::tables_store::START,
        crate::physics::tables_store::END,
    ),
    (crate::race::vars_store::START, crate::race::vars_store::END),
    (
        crate::scene::draw_store::START,
        crate::scene::draw_store::END,
    ),
    (
        crate::scene::light_store::START,
        crate::scene::light_store::END,
    ),
    (crate::race::view_store::START, crate::race::view_store::END),
    (
        crate::race::level_grid_store::START,
        crate::race::level_grid_store::END,
    ),
    (
        crate::race::frame_store::START,
        crate::race::frame_store::END,
    ),
    (
        crate::race::sound_spots_store::START,
        crate::race::sound_spots_store::END,
    ),
    (
        crate::physics::contact_store::START,
        crate::physics::contact_store::END,
    ),
    (
        crate::drive::replay_frames_store::START,
        crate::drive::replay_frames_store::END,
    ),
    (
        crate::physics::car_store::START,
        crate::physics::car_store::END,
    ),
    (crate::race::work_store::START, crate::race::work_store::END),
    (
        crate::physics::split_tables_store::START,
        crate::physics::split_tables_store::END,
    ),
    (
        crate::race::league_store::START,
        crate::race::league_store::END,
    ),
    (
        crate::race::lights_store::START,
        crate::race::lights_store::END,
    ),
    (
        crate::race::level_paths_store::START,
        crate::race::level_paths_store::END,
    ),
    (
        crate::physics::pace_store::START,
        crate::physics::pace_store::END,
    ),
    (
        crate::physics::ground_probe_store::START,
        crate::physics::ground_probe_store::END,
    ),
    (
        crate::race::panel_clock_store::START,
        crate::race::panel_clock_store::END,
    ),
    (
        crate::scene::object_slots_store::START,
        crate::scene::object_slots_store::END,
    ),
    (
        crate::scene::light_tables_store::START,
        crate::scene::light_tables_store::END,
    ),
    (
        crate::race::error_list_store::START,
        crate::race::error_list_store::END,
    ),
    (
        crate::race::flares_store::START,
        crate::race::flares_store::END,
    ),
    (
        crate::race::shadow_skids_store::START,
        crate::race::shadow_skids_store::END,
    ),
    (
        crate::drive::finish_store::START,
        crate::drive::finish_store::END,
    ),
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
        let mut k = 0;
        while k < run.len() {
            let at = offset + k;
            if at >= from && at < from + N {
                bytes[at - from] = run[k];
            }
            k += 1;
        }
        r += 1;
    }
    bytes
}

/// The typed pieces over the block (IDA start, end, the store's name), in order: for the
/// census of accesses by address (2026-10-29, `notes/history.md`) and the moves out.
#[cfg(test)]
pub const PIECES: [(usize, usize, &str); 47] = [
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

/// Where IDA's address `ida` of the block is.
pub fn home(ida: usize) -> usize {
    vars::base() + (ida - vars::START)
}

/// What an RVA of the block is added to in inline asm (`[{m} + RVA]`): the block's address
/// less its first byte's RVA. Only the tests' kept assembly uses it now (2026-10-28: the
/// game's code reaches the block through the typed stores).
#[cfg(test)]
pub fn asm_base() -> u32 {
    (vars::base() as u32).wrapping_sub(crate::framework::rva(vars::START) as u32)
}

/// Tests: the left pieces holding any of IDA's addresses `idas`, laid into their old places
/// in the block for kept assembly that reads the block by RVA from `asm_base()`; dropped,
/// the old places are taken back into the statics (the assembly's writes) and their bytes
/// put back as they were.
#[cfg(all(test, target_arch = "x86"))]
pub struct OldPlaces(Vec<(*mut u8, *mut u8, Vec<u8>)>);

#[cfg(all(test, target_arch = "x86"))]
impl OldPlaces {
    pub fn lay(idas: &[usize]) -> OldPlaces {
        let mut kept = Vec::new();
        for &(start, end) in &LEFT {
            if !idas.iter().any(|&ida| (start..end).contains(&ida)) {
                continue;
            }
            let old = crate::pointers::host(home(start) as u32);
            let new =
                crate::pointers::host(crate::framework::game_addr(crate::framework::rva(start)));
            let len = end - start;
            // SAFETY: both places hold the piece's `len` bytes (the static's are cells,
            // written through as the game's code did); the tests hold the statics' lock.
            let bytes = unsafe { core::slice::from_raw_parts(old, len) }.to_vec();
            unsafe { core::ptr::copy_nonoverlapping(new, old, len) };
            kept.push((old, new, bytes));
        }
        OldPlaces(kept)
    }
}

#[cfg(all(test, target_arch = "x86"))]
impl Drop for OldPlaces {
    fn drop(&mut self) {
        for (old, new, bytes) in &self.0 {
            // SAFETY: as in `lay`.
            unsafe {
                core::ptr::copy_nonoverlapping(*old, *new, bytes.len());
                core::ptr::copy_nonoverlapping(bytes.as_ptr(), *old, bytes.len());
            }
        }
    }
}

/// Tests: runs `f` with the left pieces holding any of `idas` laid into their old places
/// (`OldPlaces`).
#[cfg(all(test, target_arch = "x86"))]
pub fn in_old_places<R>(idas: &[usize], f: impl FnOnce() -> R) -> R {
    let _old = OldPlaces::lay(idas);
    f()
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
        assert_eq!(piece_of(0x5C2F84), "VIEWER_PANEL");
    }

    #[test]
    fn the_holes_are_the_moved_ranges() {
        // Each gap between two own ranges is a chain of moved ranges (a hole, a left piece).
        for w in OWN.windows(2) {
            let (mut at, next) = (w[0].1, w[1].0);
            while at < next {
                let hole = crate::moved::MOVED.iter().find(|m| m.ida == at);
                let Some(m) = hole else { panic!("{at:#x}") };
                at = m.ida + m.len;
            }
            assert_eq!(at, next);
        }
        for &(start, end) in &LEFT {
            assert!(
                crate::moved::MOVED
                    .iter()
                    .any(|m| m.ida == start && m.len == end - start)
            );
        }
    }

    #[test]
    fn the_left_places_are_dead() {
        for &(start, end) in &LEFT {
            assert!(is_dead(home(start) as u32));
            assert!(is_dead((home(end) - 1) as u32));
            assert!(!is_dead(crate::framework::game_addr(
                crate::framework::rva(start)
            )));
        }
        assert!(!is_dead(home(0x5BEC48) as u32));
    }

    /// The texture kinds' names table (`off_64CD0C`, 39 dwords) is unread and holds the image's
    /// addresses of its names, which lie just after it.
    #[test]
    fn the_texture_kinds_names_are_unread() {
        let (start, end) = (0x64CD0C, 0x64CDA8);
        assert!(UNREAD.contains(&(start, end)));
        let image = crate::moved::image_bytes(start, end - start);
        for (n, word) in image.chunks(4).enumerate() {
            let name = u32::from_le_bytes(word.try_into().unwrap()) as usize;
            assert!((end..end + 0x100).contains(&name), "entry {n}");
            assert!(crate::relocations::PLACES.contains(&((start + 4 * n) as u32)));
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
