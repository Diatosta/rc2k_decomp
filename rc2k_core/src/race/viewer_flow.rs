//! The replay viewer's flows: opening a replay and saving one, the undo, the camera
//! edit's switch, the physics patches a replay was recorded with, the plays, the frame's
//! bar and the tidying (converted 2026-10-21 from `race::open_viewed_replay` 0x41E9FF,
//! `save_viewed_replay` 0x41ED1C, `close_viewed_replay` 0x41EB7C, `keep_undo` 0x41E438,
//! `undo_edit` 0x41E46B, `edit_camera_1` 0x41E599 / `edit_camera_2` 0x41E5BA,
//! `apply_replay_patches` 0x41F2C9, `restore_replay_patches` 0x41F31D, `play_marked`
//! 0x41E224, `play_from_here` 0x41E1E2, `viewer_frame` 0x41D5FB, `viewer_tidy_stage`
//! 0x41CEB4, `replay_mode_1` 0x41CD75 / `replay_mode_3` 0x41CD89, `step_replay_cars`
//! 0x460835, `draw_viewer_panel` 0x41DA25 and `viewer_palettes` 0x41D8BF).

use crate::Invalid;
use crate::race::replay::{HEADER_MAGIC, HEADER_VERSION, INDEX_MAGIC};
use crate::race::viewer::{MOST_BLOCKS, OWN_FROM};

// ---- opening ----

/// The file's header read (`dword_528504`).
pub const HEADER_LEN: u32 = 0x430;
/// The index at the file's end ("RCIB", the count, the blocks' places).
pub const INDEX_LEN: u32 = 0x3850;
/// The cars' header kept from the first block (`unk_528534` into `dword_719B14`).
pub const CAR_HEADER_LEN: usize = 0x400;

/// The last race's replay asked for when none was recorded (`dword_5BEC84` not 1, asked
/// only then): the opening fails at once.
pub fn last_race_missing(is_last_race: bool, recorded: impl FnOnce() -> u32) -> bool {
    is_last_race && recorded() != 1
}

/// A saved replay (a slot below 100) is copied over the last race's file first.
pub fn copied_first(slot: u32) -> bool {
    slot < OWN_FROM
}

/// A read of the header or the index went well: no carry, all asked read, the magic
/// word right ("REPL" for the header, "RCIB" for the index).
pub fn read_good(carry: bool, read: u32, asked: u32, first_word: u32, header: bool) -> bool {
    !carry && read == asked && first_word == if header { HEADER_MAGIC } else { INDEX_MAGIC }
}

/// The header's version (+8) is the game's.
pub fn version_good(version: u32) -> bool {
    HEADER_VERSION ^ version == 0
}

/// The index's block count, as the opening takes it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Blocks {
    /// 0 or negative (signed): the opening fails.
    Bad,
    /// Kept as it is.
    Kept,
    /// Past the table (signed): held to its 0x708.
    Held(u32),
}

/// `open_viewed_replay`'s test of the index's count.
pub fn index_blocks(count: u32) -> Blocks {
    if (count as i32) <= 0 {
        Blocks::Bad
    } else if (count as i32) > MOST_BLOCKS as i32 {
        Blocks::Held(MOST_BLOCKS)
    } else {
        Blocks::Kept
    }
}

/// The opened replay's marks: round the whole (in, out).
pub fn opened_marks(blocks: u32) -> (u32, u32) {
    (0, blocks)
}

/// The block's cameras: the first two dwords of each block read.
pub const CAMERA_DWORDS: usize = 2;

/// The opening's answer: carry clear, ZF when the patches' call left eax 0.
pub fn opened_zero(eax: u32) -> bool {
    eax == 0
}

// ---- saving ----

/// The new header's first three dwords ("REPL", 0, the version); the date follows.
pub const SAVED_HEADER: [u32; 3] = [HEADER_MAGIC, 0, HEADER_VERSION];
/// A slot's name (`byte_528934`): 32 bytes a slot, copied after the date.
pub const NAME_LEN: usize = 0x20;
/// A block's data read and written packed (`dword_9613A4`).
pub const BLOCK_LEN: u32 = 0x2810;
/// The index's head written after the blocks ("RCIB" and the count).
pub const INDEX_HEAD_LEN: u32 = 0x10;

/// Nothing to save with no blocks.
pub fn has_blocks(blocks: u32) -> bool {
    blocks != 0
}

/// A slot's name's offset in the names (the original's `shl`, high bits lost).
pub fn name_offset(slot: u32) -> u32 {
    slot.wrapping_mul(NAME_LEN as u32)
}

/// The new file's length after a block written: its size negative when packed.
pub fn next_offset(offset: u32, size: u32) -> u32 {
    offset.wrapping_add((size as i32).unsigned_abs())
}

// ---- the undo, the camera edit, closing ----

/// `keep_undo` and `undo_edit`'s copies: the places and cameras into their keeping.
pub fn copy_tables(to: (&mut [u32], &mut [u32]), from: (&[u32], &[u32])) {
    to.0.copy_from_slice(from.0);
    to.1.copy_from_slice(from.1);
}

/// `undo_edit`: the block count to put back, None when nothing is kept (0).
pub fn undo_count(kept: u32) -> Option<u32> {
    (kept != 0).then_some(kept)
}

/// `edit_camera_1` / `_2`: the camera is edited unless the block shown is the last.
pub fn edits_camera(block: u32, blocks: u32) -> bool {
    block != blocks
}

/// `close_viewed_replay`: only an open replay (`dword_5BEC88` 1) is closed.
pub fn closes(open: u32) -> bool {
    open == 1
}

/// `replay_mode_1` / `_3`: the viewer runs only with a replay there (`dword_5BEC84`).
pub fn views_replay(there: u32) -> bool {
    there != 0
}

// ---- the physics patches ----

/// The patch switches (`g_patchSwitch`): 13 dwords.
pub const PATCH_DWORDS: usize = 13;

/// Whether the patches change: applying when the header has them and they are not in
/// force, restoring when it has them and they are.
pub fn patches_change(in_header: u32, in_force: u32, restoring: bool) -> bool {
    in_header != 0 && (in_force != 0) == restoring
}

// ---- the plays ----

/// A play: from a block and frame up to (not into) the block to stop at.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Play {
    pub from: u32,
    pub frame: u32,
    pub end: u32,
}

/// `play_marked`: from the first mark's start to the second mark.
pub fn marked_play(mark_in: u32, mark_out: u32) -> Play {
    Play {
        from: mark_in,
        frame: 0,
        end: mark_out,
    }
}

/// `play_from_here`: from the block and frame shown to the end.
pub fn play_from_here(block: u32, frame: u32, blocks: u32) -> Play {
    Play {
        from: block,
        frame,
        end: blocks,
    }
}

// ---- the frame ----

/// The render states set for the stopped viewer's list, (value, state) in order.
pub const LIST_STATES: [(u32, u32); 8] = [
    (2, 0x15),
    (0, 0x1C),
    (5, 0x13),
    (6, 0x14),
    (1, 0x1B),
    (3, 3),
    (2, 0x11),
    (2, 0x12),
];

/// A place along the bar: `at * width / of` as `mul` / `div` (quotient, remainder);
/// None where the `div` faults (a divisor of 0, a quotient past 32 bits).
pub fn along(at: u32, width: u32, of: u32) -> Option<(u32, u32)> {
    let product = at as u64 * width as u64;
    let q = product.checked_div(of as u64)?;
    Some((u32::try_from(q).ok()?, (product % of as u64) as u32))
}

/// The bar's three places: the two marks, and the frame shown counted in frames.
pub fn bar_places(
    marks: (u32, u32),
    block: u32,
    frame: u32,
    width: u32,
    blocks: u32,
) -> Result<[(u32, u32); 3], Invalid> {
    let first = along(marks.0, width, blocks);
    let second = along(marks.1, width, blocks);
    let here = along(
        block.wrapping_mul(100).wrapping_add(frame),
        width,
        blocks.wrapping_mul(100),
    );
    match (first, second, here) {
        (Some(a), Some(b), Some(c)) => Ok([a, b, c]),
        _ => Err(Invalid(
            "a mark past the bar: the original's division faults",
        )),
    }
}

/// A mark's rectangle's x (2 left of its place), the place's (12 left).
pub fn mark_x(place: u32, bar_x: u32) -> u32 {
    place.wrapping_add(bar_x).wrapping_sub(2)
}
pub fn place_x(place: u32, bar_x: u32) -> u32 {
    place.wrapping_add(bar_x).wrapping_sub(12)
}

// ---- the panel ----

/// The time shown, in hundredths: the block's 100 and the frame.
pub fn shown_time(block: u32, frame: u32) -> u32 {
    block.wrapping_mul(100).wrapping_add(frame)
}

/// The replay's length in hundredths.
pub fn replay_length(blocks: u32) -> u32 {
    blocks.wrapping_mul(100)
}

/// A block's camera byte as a name's index: held to the last camera (signed, as a byte
/// zero-extended it never is below 0).
pub fn camera_name(byte: u8) -> u32 {
    let camera = byte as u32;
    if (camera as i32) > crate::race::viewer::LAST_CAMERA as i32 {
        crate::race::viewer::LAST_CAMERA as u32
    } else {
        camera
    }
}

/// The two switch lights' colours (the first's, the second's) by the block's camera
/// byte 5: 1 dims the first, 0 the second.
pub const DIM: u32 = 0x0055_5555;
pub const LIT: u32 = 0x00FF_FFFF;

pub fn switch_lights(byte5: u8) -> (u32, u32) {
    (
        if byte5 == 1 { DIM } else { LIT },
        if byte5 == 0 { DIM } else { LIT },
    )
}

// ---- the fonts' palettes ----

/// `viewer_palettes`: a font colour made opaque between taking it apart in the screen's
/// format and putting it together in the texture's.
pub const OPAQUE: u32 = 0xFF00_0000;

pub fn font_colour(
    colour: u32,
    components: impl FnOnce(u32) -> u32,
    pixel: impl FnOnce(u32) -> u32,
) -> u32 {
    pixel(components(colour) | OPAQUE)
}

/// The fonts' colours there are.
pub const FONT_COLOURS: usize = 0x100;

// ---- the stage tidied ----

/// `viewer_tidy_stage`'s steps.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TidyStep {
    TidyEverything,
    MenuScreenMode,
    LoadingPicture,
    ClearRaceSounds,
    SoundSettings,
    FreeFonts,
    SetUpEngine,
    SetUpFrontEnd,
    TakeKey,
}

/// With the stage loaded (`dword_5BECC0` 1): the race tidied, then the fonts freed when
/// the viewer built them (`dword_5BECC4` 1, asked after these), then `TIDY_AFTER`; the
/// key always last.
pub fn tidies(loaded: u32) -> bool {
    loaded == 1
}

pub const TIDY_FIRST: [TidyStep; 5] = [
    TidyStep::TidyEverything,
    TidyStep::MenuScreenMode,
    TidyStep::LoadingPicture,
    TidyStep::ClearRaceSounds,
    TidyStep::SoundSettings,
];

pub const TIDY_AFTER: [TidyStep; 3] = [
    TidyStep::SetUpEngine,
    TidyStep::SetUpFrontEnd,
    TidyStep::SetUpEngine,
];

/// The fonts are freed only when the viewer built them.
pub fn frees_fonts(built: u32) -> bool {
    built == 1
}

// ---- the replay's cars stepped ----

/// The cars a step takes: 1 ..= 8 (0 would go round 2^32 times).
pub fn check_replay_cars(cars: u32) -> Result<(), Invalid> {
    if (1..=8).contains(&cars) {
        Ok(())
    } else {
        Err(Invalid("a cars' count of 0 or past 8"))
    }
}

/// A car's block: the current car & 7.
pub fn car_block(current: u32) -> u32 {
    current & 7
}

/// A running car's steps after it is loaded: controls, gear rules, steering, pedals,
/// the reset key, the physics' step; then its two clocks one on.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RunStep {
    Controls,
    GearRules,
    Steering,
    Pedals,
    ResetKey,
    Physics,
}

pub const RUN_STEPS: [RunStep; 6] = [
    RunStep::Controls,
    RunStep::GearRules,
    RunStep::Steering,
    RunStep::Pedals,
    RunStep::ResetKey,
    RunStep::Physics,
];

/// Whether the loaded car runs (`unk_71A234`, read after the loading).
pub fn car_runs(running: u32) -> bool {
    running != 0
}

/// The next car, and whether the step is over (the count reached; the loop's test at
/// its end).
pub fn next_car(current: u32, cars: u32) -> (u32, bool) {
    let next = current.wrapping_add(1);
    (next, next == cars)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn opening() {
        assert!(last_race_missing(true, || 0) && !last_race_missing(true, || 1));
        assert!(!last_race_missing(false, || unreachable!()));
        assert!(copied_first(99) && !copied_first(100));
        assert!(read_good(false, 0x430, 0x430, HEADER_MAGIC, true));
        assert!(!read_good(false, 0x430, 0x430, INDEX_MAGIC, true));
        assert!(read_good(false, 5, 5, INDEX_MAGIC, false));
        assert!(!read_good(true, 5, 5, INDEX_MAGIC, false));
        assert!(!read_good(false, 4, 5, INDEX_MAGIC, false));
        assert!(version_good(0x50400) && !version_good(0x50401));
        assert_eq!(index_blocks(0), Blocks::Bad);
        assert_eq!(index_blocks(0x8000_0000), Blocks::Bad);
        assert_eq!(index_blocks(0x708), Blocks::Kept);
        assert_eq!(index_blocks(0x709), Blocks::Held(0x708));
        assert_eq!(opened_marks(7), (0, 7));
    }

    #[test]
    fn saving_and_edits() {
        assert!(!has_blocks(0) && has_blocks(1));
        assert_eq!(name_offset(3), 0x60);
        assert_eq!(name_offset(0x0800_0001), 0x20);
        assert_eq!(next_offset(0x430, 0x100), 0x530);
        assert_eq!(next_offset(0x430, (-0x80i32) as u32), 0x4B0);
        assert_eq!(next_offset(0, 0x8000_0000), 0x8000_0000);
        assert_eq!(undo_count(0), None);
        assert_eq!(undo_count(9), Some(9));
        assert!(edits_camera(3, 9) && !edits_camera(9, 9));
        assert!(closes(1) && !closes(2) && !closes(0));
        assert!(views_replay(5) && !views_replay(0));
        assert!(patches_change(5, 0, false) && !patches_change(5, 1, false));
        assert!(patches_change(5, 1, true) && !patches_change(5, 0, true));
        assert!(!patches_change(0, 0, false) && !patches_change(0, 1, true));
        let (mut p, mut c) = ([0u32; 3], [0u32; 3]);
        copy_tables((&mut p, &mut c), (&[1, 2, 3], &[4, 5, 6]));
        assert_eq!((p, c), ([1, 2, 3], [4, 5, 6]));
    }

    #[test]
    fn plays_and_bar() {
        assert_eq!(
            marked_play(2, 5),
            Play {
                from: 2,
                frame: 0,
                end: 5
            }
        );
        assert_eq!(
            play_from_here(3, 40, 9),
            Play {
                from: 3,
                frame: 40,
                end: 9
            }
        );
        assert_eq!(along(3, 100, 9), Some((33, 3)));
        assert_eq!(along(3, 100, 0), None);
        assert_eq!(along(u32::MAX, u32::MAX, 1), None);
        assert_eq!(
            bar_places((0, 9), 4, 50, 90, 9),
            Ok([(0, 0), (90, 0), (45, 0)])
        );
        assert!(bar_places((0, 9), 0, 0, 90, 0).is_err());
        assert_eq!(mark_x(10, 100), 108);
        assert_eq!(place_x(10, 100), 98);
        assert_eq!(place_x(0, 0), (-12i32) as u32);
    }

    #[test]
    fn panel_and_palettes() {
        assert_eq!(shown_time(3, 40), 340);
        assert_eq!(replay_length(9), 900);
        assert_eq!(camera_name(0x2E), 0x2E);
        assert_eq!(camera_name(0xFF), 0x2E);
        assert_eq!(switch_lights(0), (LIT, DIM));
        assert_eq!(switch_lights(1), (DIM, LIT));
        assert_eq!(switch_lights(2), (LIT, LIT));
        assert_eq!(font_colour(2, |c| c << 4, |p| p ^ 1), 0xFF00_0021);
    }

    #[test]
    fn tidy_and_cars() {
        use TidyStep::*;
        assert!(tidies(1) && !tidies(2) && !tidies(0));
        assert!(frees_fonts(1) && !frees_fonts(0));
        assert_eq!(TIDY_FIRST[0], TidyEverything);
        assert_eq!(TIDY_AFTER, [SetUpEngine, SetUpFrontEnd, SetUpEngine]);
        assert!(check_replay_cars(0).is_err() && check_replay_cars(9).is_err());
        assert!(check_replay_cars(1).is_ok() && check_replay_cars(8).is_ok());
        assert_eq!(car_block(9), 1);
        assert!(car_runs(2) && !car_runs(0));
        assert_eq!(next_car(0, 1), (1, true));
        assert_eq!(next_car(0, 3), (1, false));
    }
}
