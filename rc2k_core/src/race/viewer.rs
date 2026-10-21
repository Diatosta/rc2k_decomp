//! The replay viewer's rules: its files, the play stepped frame by frame, the edits (the
//! marked clip, a camera spread over the next blocks), the menu's items marked, the
//! pictures' numbers (converted 2026-10-21 from `race::replay_path` 0x41EF27,
//! `step_viewed_replay` 0x41EC47, `copy_marked` 0x41E2F6, `spread_camera` 0x41E6BD,
//! `mark_viewer_items` 0x41D456, `find_snapshot_number` 0x41EF51, and the camera step of
//! `next_camera` 0x41E5EB / `previous_camera`).

use crate::Invalid;

/// The blocks a replay's table holds.
pub const MOST_BLOCKS: u32 = 0x708;
/// The frames of a block.
pub const BLOCK_FRAMES: u32 = 100;
/// The slot of the last race's replay (`var\tmprpl`).
pub const LAST_RACE: u32 = 0xDE;
/// The slots from which the game's own replays are numbered.
pub const OWN_FROM: u32 = 100;

/// A replay's file by its slot (`race::replay_path`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReplayFile {
    /// `var\tmprpl`.
    LastRace,
    /// `savedata\replay\replayNN`.
    Saved(u32),
    /// `files\replay\replayNN`, numbered from slot 100.
    Own(u32),
}

impl ReplayFile {
    pub fn of(slot: u32) -> Self {
        if slot == LAST_RACE {
            Self::LastRace
        } else if slot < OWN_FROM {
            Self::Saved(slot)
        } else {
            Self::Own(slot - OWN_FROM)
        }
    }
}

/// What `step` does with the viewed replay.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Step {
    /// Not open: stopped at the end.
    Closed,
    /// At or past the last block: stopped at the end; `last` when exactly at it.
    AtLast { last: bool },
    /// A new block that reaches the play's end: stopped (not at the end); `end` when
    /// exactly at it.
    PlayEnd { end: bool },
    /// On to `frame` of `block`; `read` when the block is new (it is read first).
    On { block: u32, frame: u32, read: bool },
}

/// `race::step_viewed_replay`: one frame on (`frame` + 1; past 99 the next block from
/// frame 0).
pub fn step(open: bool, block: u32, frame: u32, blocks: u32, play_end: u32) -> Step {
    if !open {
        return Step::Closed;
    }
    if block >= blocks {
        return Step::AtLast {
            last: block == blocks,
        };
    }
    let frame = frame.wrapping_add(1);
    if frame < BLOCK_FRAMES {
        return Step::On {
            block,
            frame,
            read: false,
        };
    }
    let block = block.wrapping_add(1);
    if block >= play_end {
        return Step::PlayEnd {
            end: block == play_end,
        };
    }
    Step::On {
        block,
        frame: 0,
        read: true,
    }
}

/// `race::copy_marked`: the marked clip (first block, blocks) from the marks, None when
/// the out mark is not after the in mark (signed); refused with an in mark below 0 or an
/// out mark past the table.
pub fn clip(mark_in: u32, mark_out: u32) -> Result<Option<(u32, u32)>, Invalid> {
    if (mark_out as i32) <= (mark_in as i32) {
        return Ok(None);
    }
    if (mark_in as i32) < 0 || mark_out > MOST_BLOCKS {
        return Err(Invalid("marks outside the replay's table"));
    }
    Ok(Some((mark_in, mark_out - mark_in)))
}

/// How far `spread` went.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Spread {
    /// The blocks gone over (from 1).
    pub gone: u32,
    /// What was left of the count.
    pub left: u32,
    /// The blocks after the one changed that take the new value: 1 ..= `taken`.
    pub taken: u32,
}

/// `race::spread_camera`: from the block changed, the next blocks (`byte(n)`, n from 1,
/// asked in order) that held `old`, until `count` - 1 are gone over or one held another
/// value. The count is not checked: refused when the walk would pass the table's blocks.
pub fn spread(count: u32, old: u8, byte: impl Fn(u32) -> u8) -> Result<Spread, Invalid> {
    let (mut gone, mut left, mut taken) = (0u32, count, 0u32);
    loop {
        gone = gone.wrapping_add(1);
        left = left.wrapping_sub(1);
        if left == 0 {
            break;
        }
        if gone > MOST_BLOCKS {
            return Err(Invalid("the camera's run goes past the replay's table"));
        }
        if byte(gone) != old {
            break;
        }
        taken = gone;
    }
    Ok(Spread { gone, left, taken })
}

/// The last camera (`word_5C291C` names 0x2F).
pub const LAST_CAMERA: u8 = 0x2E;
/// The names of the cameras a step passes over.
pub const SKIPPED_NAMES: [u16; 2] = [0x1C1, 0x1C2];

/// Whether a camera of this name is passed over (unless every camera is offered).
pub fn camera_skipped(name: u16) -> bool {
    SKIPPED_NAMES.contains(&name)
}

/// Whether every camera is offered: one player (`dword_719B34` 1) whose first car
/// (`dword_719B24`, the header's) is not a kind 0x17 .. 0x20.
pub fn every_camera(one_player: bool, first_car: u32) -> bool {
    one_player && !(0x17..0x21).contains(&first_car)
}

/// `race::next_camera` (`forward`) / `race::previous_camera`: the camera after (before)
/// `old`, round from the last to 0 (0 to the last, by sign), passing over the skipped
/// ones (`skipped(camera)`, asked for each tried) unless `every`. Refused when every
/// camera is skipped (the original goes round for ever).
pub fn step_camera(
    old: u8,
    forward: bool,
    every: bool,
    skipped: impl Fn(u8) -> bool,
) -> Result<u8, Invalid> {
    if !every && (0..=LAST_CAMERA).all(&skipped) {
        return Err(Invalid(
            "every camera is skipped: the original goes round for ever",
        ));
    }
    let mut camera = old;
    loop {
        camera = if forward {
            let n = camera.wrapping_add(1);
            if (n as i8) > LAST_CAMERA as i8 { 0 } else { n }
        } else {
            let n = camera.wrapping_sub(1);
            if (n as i8) >= 0 { n } else { LAST_CAMERA }
        };
        if every || !skipped(camera) {
            return Ok(camera);
        }
    }
}

/// What `race::mark_viewer_items` reads.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ViewerState {
    /// `dword_5C2544`: the last race's replay is viewed.
    pub last_race: bool,
    /// The block shown and the blocks.
    pub block: u32,
    pub blocks: u32,
    pub mark_in: u32,
    pub mark_out: u32,
    /// The clip's blocks (0: nothing copied).
    pub clip_blocks: u32,
    /// The undo's blocks (0: nothing to undo).
    pub undo_blocks: u32,
    /// The block's camera (its +5 byte).
    pub camera: u8,
    /// `dword_719B34` (the replay header's) 1: one player.
    pub one_player: bool,
}

/// The item's bit for "greyed".
pub const GREY: u8 = 0x80;
/// The item's bit for "ticked".
pub const TICK: u8 = 0x40;

/// `race::mark_viewer_items`: the items marked (item, bit), in the original's order.
pub fn item_marks(s: &ViewerState) -> Vec<(u32, u8)> {
    let mut marks = Vec::new();
    if s.last_race {
        marks.push((0x14, TICK));
    }
    if s.block == s.blocks {
        marks.extend([0xF, 0x10, 0xD, 0xE, 0x19, 0x18, 0x17].map(|i| (i, GREY)));
    }
    if (s.mark_out as i32) <= (s.mark_in as i32) {
        marks.push((0x11, GREY));
    }
    if s.clip_blocks == 0 || s.blocks.wrapping_add(s.clip_blocks) > MOST_BLOCKS {
        marks.push((0x12, GREY));
    }
    if s.undo_blocks == 0 {
        marks.push((0x13, GREY));
    }
    if s.block <= s.mark_in {
        marks.push((0xA, GREY));
    }
    if s.block >= s.mark_out {
        marks.push((2, GREY));
    }
    if s.blocks == 0 {
        marks.extend((0..=0x11).chain(0x15..=0x19).map(|i| (i, GREY)));
    }
    if s.camera == 0 {
        marks.extend([(0x18, GREY), (0x19, GREY)]);
    }
    if s.camera == 1 {
        marks.extend([(0xE, GREY), (0xD, GREY)]);
    }
    if s.one_player {
        marks.push((0xF, GREY));
    }
    marks
}

/// `race::find_snapshot_number`: a picture's number from its name's first three
/// characters (each asked in order until one is not a digit), None unless all are.
pub fn snapshot_number(name: impl Fn(usize) -> u8) -> Option<u32> {
    (0..3).try_fold(0u32, |n, k| {
        let d = name(k).wrapping_sub(b'0');
        (d <= 9).then(|| n.wrapping_mul(10).wrapping_add(d as u32))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn files_and_steps() {
        assert_eq!(ReplayFile::of(0xDE), ReplayFile::LastRace);
        assert_eq!(ReplayFile::of(99), ReplayFile::Saved(99));
        assert_eq!(ReplayFile::of(105), ReplayFile::Own(5));
        assert_eq!(step(false, 0, 0, 9, 9), Step::Closed);
        assert_eq!(step(true, 9, 0, 9, 9), Step::AtLast { last: true });
        assert_eq!(step(true, 10, 0, 9, 9), Step::AtLast { last: false });
        assert_eq!(
            step(true, 3, 5, 9, 9),
            Step::On {
                block: 3,
                frame: 6,
                read: false
            }
        );
        assert_eq!(
            step(true, 3, 99, 9, 9),
            Step::On {
                block: 4,
                frame: 0,
                read: true
            }
        );
        assert_eq!(step(true, 3, 99, 9, 4), Step::PlayEnd { end: true });
        assert_eq!(step(true, 3, 99, 9, 2), Step::PlayEnd { end: false });
    }

    #[test]
    fn edits() {
        assert_eq!(clip(5, 5), Ok(None));
        assert_eq!(clip(0, u32::MAX), Ok(None));
        assert_eq!(clip(2, 7), Ok(Some((2, 5))));
        assert!(clip(u32::MAX, 3).is_err());
        assert!(clip(0, 0x709).is_err());
        let table = |n: u32| if n < 4 { 7 } else { 1 };
        assert_eq!(
            spread(10, 7, table),
            Ok(Spread {
                gone: 4,
                left: 6,
                taken: 3
            })
        );
        assert_eq!(
            spread(3, 7, table),
            Ok(Spread {
                gone: 3,
                left: 0,
                taken: 2
            })
        );
        assert_eq!(
            spread(1, 7, |_| unreachable!()),
            Ok(Spread {
                gone: 1,
                left: 0,
                taken: 0
            })
        );
        assert!(spread(0, 7, |_| 7).is_err());
    }

    #[test]
    fn items_and_numbers() {
        let s = ViewerState {
            last_race: true,
            block: 3,
            blocks: 9,
            mark_in: 1,
            mark_out: 5,
            clip_blocks: 2,
            undo_blocks: 1,
            camera: 2,
            one_player: false,
        };
        assert_eq!(item_marks(&s), [(0x14, TICK)]);
        let m = item_marks(&ViewerState {
            blocks: 0,
            block: 0,
            camera: 1,
            ..s
        });
        assert!(m.contains(&(0x16, GREY)) && m.contains(&(0xD, GREY)));
        assert!(!m.contains(&(0x14, GREY)));
        assert_eq!(snapshot_number(|k| b"042"[k]), Some(42));
        assert_eq!(step_camera(0x2E, true, true, |_| unreachable!()), Ok(0));
        // By sign: 0x80 is not past the last.
        assert_eq!(step_camera(0x7F, true, true, |_| unreachable!()), Ok(0x80));
        assert_eq!(step_camera(0, false, true, |_| unreachable!()), Ok(0x2E));
        assert_eq!(step_camera(0x90, false, true, |_| unreachable!()), Ok(0x2E));
        assert_eq!(step_camera(3, true, false, |c| c < 7), Ok(7));
        assert_eq!(step_camera(3, false, false, |c| c < 7), Ok(0x2E));
        assert!(step_camera(3, true, false, |_| true).is_err());
        assert!(every_camera(true, 0x16) && !every_camera(true, 0x20) && !every_camera(false, 0));
        assert!(camera_skipped(0x1C2) && !camera_skipped(0x1C3));
        assert_eq!(snapshot_number(|k| b"4a"[k]), None);
        assert_eq!(snapshot_number(|k| b"/00"[k]), None);
    }
}
