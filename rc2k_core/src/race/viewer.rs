//! The replay viewer's rules: its files, frame-by-frame play, edits (the marked clip, a camera
//! spread over the next blocks), menu item marks, picture numbers, marks, frame pictures, the
//! snapshot number and the viewer's actions.

use crate::Invalid;

/// The blocks a replay's table holds.
pub const MOST_BLOCKS: u32 = crate::race::replay_v2::MOST_BLOCKS;
/// The frames of a block.
pub(crate) const BLOCK_FRAMES: u32 = 100;
/// The slot of the last race's replay.
pub const LAST_RACE: u32 = 0xDE;
/// The slot of a replay file named when the program was started.
pub const GIVEN_FILE: u32 = 0xDD;
/// The slots from which the game's own replays are numbered.
pub(crate) const OWN_FROM: u32 = 100;

/// A replay's file by its slot.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReplayFile {
    /// The last race's replay in the var folder.
    LastRace,
    /// `savedata\replay\replayNN`.
    Saved(u32),
    /// `files\replay\replayNN`, numbered from slot 100.
    Own(u32),
    /// A file named when the program was started.
    Given,
}

impl ReplayFile {
    pub fn of(slot: u32) -> Self {
        if slot == LAST_RACE {
            Self::LastRace
        } else if slot == GIVEN_FILE {
            Self::Given
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
    /// A new block reaching the play's end: stopped (not at the end); `end` when exactly at it.
    PlayEnd { end: bool },
    /// On to `frame` of `block`; `read` when the block is new (it is read first).
    On { block: u32, frame: u32, read: bool },
}

/// One frame on (`frame` + 1; past 99 the next block from frame 0).
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

/// The marked clip (first block, blocks) from the marks; None when the out mark is not after
/// the in mark (signed). Refused with an in mark below 0 or an out mark past the table.
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

/// From the block changed, the next blocks (`byte(n)`, n from 1, asked in order) that held
/// `old`, until `count` - 1 are gone over or one holds another value. Refused when the walk
/// would pass the table's blocks (the count is not checked).
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

/// The last camera's number.
pub(crate) const LAST_CAMERA: u8 = 0x2E;
/// The names of the cameras a step passes over.
pub(crate) const SKIPPED_NAMES: [u16; 2] = [0x1C1, 0x1C2];

/// Whether a camera of this name is passed over (unless every camera is offered).
pub fn camera_skipped(name: u16) -> bool {
    SKIPPED_NAMES.contains(&name)
}

/// Whether every camera is offered: one player whose first car (the header's) is not kind
/// 0x17 ..= 0x20.
pub fn every_camera(one_player: bool, first_car: u32) -> bool {
    one_player && !(0x17..0x21).contains(&first_car)
}

/// The camera after (`forward`) or before `old`, wrapping, passing over the skipped ones
/// (`skipped(camera)`) unless `every`. Refused when every camera is skipped (it would loop
/// for ever).
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

/// What the menu items' marking reads.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ViewerState {
    /// The last race's replay is viewed.
    pub last_race: bool,
    /// The block shown.
    pub block: u32,
    /// The replay's blocks.
    pub blocks: u32,
    /// The first mark.
    pub mark_in: u32,
    /// The second mark.
    pub mark_out: u32,
    /// The clip's blocks (0: nothing copied).
    pub clip_blocks: u32,
    /// The undo's blocks (0: nothing to undo).
    pub undo_blocks: u32,
    /// The block's camera (its byte 5).
    pub camera: u8,
    /// The replay has one player and a single car (its view cannot be changed).
    pub one_player: bool,
}

/// The item's bit for "greyed".
pub(crate) const GREY: u8 = 0x80;
/// The item's bit for "ticked".
pub(crate) const TICK: u8 = 0x40;

/// The items marked (item, bit), in menu order.
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
    // The first car alone, or (a replay with the extra tape) the third or fourth alone: the
    // second camera's items are not for them.
    if matches!(s.camera, 0 | 4 | 5) {
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

/// A picture's number from its name's first three characters (asked in order until one is not
/// a digit); None unless all are.
pub fn snapshot_number(name: impl Fn(usize) -> u8) -> Option<u32> {
    (0..3).try_fold(0u32, |n, k| {
        let d = name(k).wrapping_sub(b'0');
        (d <= 9).then(|| n.wrapping_mul(10).wrapping_add(d as u32))
    })
}

/// The marks after an edit: the block shown held to the replay's blocks (signed), the marks
/// round the whole replay. Returns the block, first mark and last.
pub fn reset_marks(block: u32, blocks: u32) -> (u32, u32, u32) {
    let block = if (block as i32) > (blocks as i32) {
        blocks
    } else {
        block
    };
    (block, 0, blocks)
}

/// The frames' pictures' rate (a second), held to 1 ..= 100 (signed).
pub fn picture_rate(rate: u32) -> u32 {
    (rate as i32).clamp(1, 100) as u32
}

/// The time of a picture (ms): the one asked for, or for 0 1000 / the rate.
pub fn picture_time(asked: u32, rate: u32) -> u32 {
    if asked == 0 { 1000 / rate } else { asked }
}

/// The camera byte a camera-switch item shows: 2 for item kind 0xE, else 3.
pub fn switch_byte(item_kind: u8) -> u32 {
    if item_kind == 0xE { 2 } else { 3 }
}

/// An item's picture x in its box: x + (box width less picture width) / 2 (arithmetic).
pub fn centred(x: u32, box_width: u32, width: u32) -> u32 {
    (((box_width.wrapping_sub(width) as i32) >> 1) as u32).wrapping_add(x)
}

/// An item's colour: grey when marked (`GREY`), white unless highlighted; the highlighted one
/// red while a camera is edited, else alternating yellow and red (bit 3 of the clocks'
/// difference, asked only then).
pub fn item_colour(
    flags: u8,
    highlighted: bool,
    editing: impl FnOnce() -> bool,
    clocks: impl FnOnce() -> u32,
) -> u32 {
    if flags & GREY != 0 {
        0x0066_6666
    } else if !highlighted {
        0x00FF_FFFF
    } else if editing() || clocks() & 8 == 0 {
        0x00FF_0000
    } else {
        0x00FF_FF00
    }
}

/// Where a go-to action reads the replay from (frame 0 of the block).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Goto {
    /// The first block.
    Start,
    /// The first mark.
    MarkIn,
    /// The second mark.
    MarkOut,
    /// The last block.
    End,
    /// Blocks back, not below 0 (signed).
    Back(u32),
    /// Blocks on, not past the count (signed).
    On(u32),
}

/// One of the viewer's actions (numbers are action codes).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    /// A go-to.
    Goto(Goto),
    FrameBack,
    FrameOn,
    /// The first mark at the block shown.
    MarkIn,
    /// The second mark at the block shown.
    MarkOut,
    /// Play from the start, or from here.
    PlayHere {
        /// From the block and frame shown, not from the start.
        from_here: bool,
    },
    /// Play the marked clip.
    PlayMarked,
    /// 0x23: from the start again and again until a key ends a play.
    PlayAgain,
    /// Cut the marked clip.
    Cut,
    /// Copy the marked clip.
    CopyMarked,
    Paste,
    /// 0x13, 0x20 (`end_edit`: the camera edit ended after).
    Undo {
        end_edit: bool,
    },
    /// Open another replay.
    AnotherReplay,
    /// 0x15 (`back`: into its own slot), 0x16.
    Save {
        back: bool,
    },
    /// Take a picture of the frame.
    Snapshot,
    /// 0xF, 0x10: the block's camera byte 5 / 4 one on (`next_byte`).
    Cycle {
        byte: u32,
    },
    /// 0xE, 0x18: the block's camera byte 2 / 3 flipped.
    Flip {
        byte: u32,
    },
    /// 0xD, 0x19: camera 1 / 2 edited.
    Edit {
        camera: u32,
    },
    /// 0x1C, 0x24, 0x1D: camera 1 / 1 / 2 edited, stepped on / back / on, the edit ended.
    EditAndStep {
        camera: u32,
        back: bool,
    },
    /// 0x1E, 0x1F: the edited camera back / on.
    Step {
        back: bool,
    },
    /// End the camera edit.
    EndEdit,
    /// 0x25 .. 0x28: the marked clip played saving pictures every so many ms (0: the
    /// rate's).
    Pictures {
        time: u32,
    },
}

impl Action {
    /// The action of a code; None for an unknown one.
    pub fn of(code: u32) -> Option<Self> {
        use Action::*;
        Some(match code {
            0 => Goto(self::Goto::Start),
            1 => Goto(self::Goto::MarkIn),
            2 => MarkIn,
            3 => Goto(self::Goto::Back(10)),
            4 => Goto(self::Goto::Back(1)),
            5 => FrameBack,
            6 => PlayHere { from_here: false },
            7 => FrameOn,
            8 => Goto(self::Goto::On(1)),
            9 => Goto(self::Goto::On(10)),
            0xA => MarkOut,
            0xB => Goto(self::Goto::MarkOut),
            0xC => Goto(self::Goto::End),
            0xD => Edit { camera: 1 },
            0xE => Flip { byte: 2 },
            0xF => Cycle { byte: 5 },
            0x10 => Cycle { byte: 4 },
            0x11 => Cut,
            0x12 => Paste,
            0x13 => Undo { end_edit: false },
            0x14 => AnotherReplay,
            0x15 => Save { back: true },
            0x16 => Save { back: false },
            0x17 => Snapshot,
            0x18 => Flip { byte: 3 },
            0x19 => Edit { camera: 2 },
            0x1A => PlayMarked,
            0x1B => CopyMarked,
            0x1C => EditAndStep {
                camera: 1,
                back: false,
            },
            0x1D => EditAndStep {
                camera: 2,
                back: false,
            },
            0x1E => Step { back: true },
            0x1F => Step { back: false },
            0x20 => Undo { end_edit: true },
            0x21 => EndEdit,
            0x22 => PlayHere { from_here: true },
            0x23 => PlayAgain,
            0x24 => EditAndStep {
                camera: 1,
                back: true,
            },
            0x25..=0x28 => Pictures {
                time: [0, 0xA0, 0x50, 0x28][code as usize - 0x25],
            },
            _ => return None,
        })
    }
}

/// The viewer's variables a go-to reads.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Var {
    /// The block shown.
    Block,
    /// The replay's blocks.
    Blocks,
    /// The first mark.
    MarkIn,
    /// The second mark.
    MarkOut,
}

/// The block a go-to reads (each variable asked only where needed).
pub fn goto_block(goto: Goto, var: impl Fn(Var) -> u32) -> u32 {
    let held = |v: u32| {
        let blocks = var(Var::Blocks);
        if (v as i32) > (blocks as i32) {
            blocks
        } else {
            v
        }
    };
    let at_least_0 = |v: u32| if (v as i32) < 0 { 0 } else { v };
    match goto {
        Goto::Start => 0,
        Goto::MarkIn => var(Var::MarkIn),
        Goto::MarkOut => var(Var::MarkOut),
        Goto::End => var(Var::Blocks),
        Goto::Back(n) => at_least_0(var(Var::Block).wrapping_sub(n)),
        Goto::On(n) => held(var(Var::Block).wrapping_add(n)),
    }
}

/// A frame back: from frame 0 the last frame (99) of the block before (not below 0, signed);
/// at block 0 frame 0 it stays. Returns (block, frame).
pub fn frame_back(block: u32, frame: u32) -> (u32, u32) {
    if block == 0 && frame == 0 {
        return (0, 0);
    }
    let frame = frame.wrapping_sub(1);
    if (frame as i32) < 0 {
        let block = block.wrapping_sub(1);
        (if (block as i32) < 0 { 0 } else { block }, BLOCK_FRAMES - 1)
    } else {
        (block, frame)
    }
}

/// A frame on: past 99 frame 0 of the next block (not past the count, signed); at the last
/// block it stays. Returns (block, frame).
pub fn frame_on(block: u32, frame: u32, blocks: u32) -> (u32, u32) {
    if block == blocks {
        return (block, frame);
    }
    let frame = frame.wrapping_add(1);
    if frame >= BLOCK_FRAMES {
        let next = block.wrapping_add(1);
        (
            if (next as i32) > (blocks as i32) {
                blocks
            } else {
                next
            },
            0,
        )
    } else {
        (block, frame)
    }
}

/// Whether the first mark moves to the block shown: not past the second (unsigned).
pub fn mark_in_here(block: u32, mark_out: u32) -> bool {
    block <= mark_out
}

/// Whether the second mark moves to the block shown: not before the first (unsigned).
pub fn mark_out_here(block: u32, mark_in: u32) -> bool {
    block >= mark_in
}

/// Whether the replay's cars can be chosen to show (the view byte 5 cycled): every replay but a
/// one-player one with a single car. A network race's replay is a one-player one (the header's
/// players is 1) with a car for each machine.
pub fn views_choosable(one_player: bool, cars: u32) -> bool {
    !one_player || cars >= 2
}

/// The number of the car a layout shows alone, when its picture has no digit of its own: the
/// panel's pictures hold the digits 1 and 2 only, so the third to the eighth player (layouts 4
/// .. 9) are the first car's picture with its digit covered and the number drawn over it.
pub fn layout_digit(layout: u8) -> Option<u8> {
    (4..=9).contains(&layout).then(|| layout - 1)
}

/// Where the car picture's digit is, relative to the picture's top left (the picture is 48 by
/// 48, its box from x 1 to 46 and y 7 to 39, the digit "1" inside it from x 13 to 28 and y 14 to
/// 31; read off the panel's atlas): a patch of the box's plain face over the digit, and a 4 by
/// 4 piece of the face to fill it from.
pub const DIGIT_PATCH: [u32; 4] = [10, 11, 20, 24];
pub const FACE_SAMPLE: [u32; 4] = [32, 10, 4, 4];
/// The centre of the box's face (twice the x, which is on a half unit), relative to the picture:
/// where a digit is centred.
pub const BOX_CENTRE_X2: u32 = 47;
pub const BOX_CENTRE_Y: u32 = 23;
/// The ink of the game's heavy font (the race's notice font): from the glyph's top to its ink's
/// middle, in units.
pub const FONT_INK_MIDDLE: u32 = 12;

/// Where the heavy font's digit goes for a car item placed at (`x`, `y`) in the 800 by 600
/// layout: the layout x of its centre (twice) and the y its glyph's top is drawn at, so that
/// the digit's ink is centred in the box.
pub fn digit_spot(x: u32, y: u32) -> (u32, i32) {
    (
        x.wrapping_mul(2).wrapping_add(BOX_CENTRE_X2),
        y.wrapping_add(BOX_CENTRE_Y).wrapping_sub(FONT_INK_MIDDLE) as i32,
    )
}

/// The picture the car item shows for a layout: the pictures are the first car alone (0), the
/// second alone (1) and the two together (2 side by side, 3 one above the other). A further
/// player alone shows the single car's, and every player of a split screen the two together; a
/// byte that is no layout is left as it is (the picture table then refuses it).
pub fn layout_picture(layout: u8) -> u32 {
    match layout {
        4..=9 => 0,
        crate::race::layouts::FIRST_GROUP | crate::race::layouts::SECOND_GROUP => 2,
        other => u32::from(other),
    }
}

/// A camera byte one on: byte 5 (the view) past 3 back to 2, byte 4 past 2 back to 0 (signed
/// bytes). `whole_cycle` (a one-player replay's view: 0 the first car, 1 the second, 2 and 3 both
/// side by side or above each other) takes byte 5 back to 0 instead.
/// `extra_cars` (a replay with the extra tape: its players beyond two) adds the layouts that show
/// the third and the fourth car alone (4 and 5) to the cycle, which then turns back to 2.
pub fn next_byte(byte: u32, old: u8, whole_cycle: bool, extra_cars: u32) -> u8 {
    let next = old.wrapping_add(1);
    let (last, back) = match (byte, whole_cycle) {
        (5, false) => (3 + extra_cars as i8, 2),
        (5, true) => (3, 0),
        _ => (2, 0),
    };
    if (next as i8) > last { back } else { next }
}

/// A cut's marks (first, second); None when the second is not after the first (signed).
/// Refused with marks or a count outside the table.
pub fn cut_marks(mark_in: u32, mark_out: u32, blocks: u32) -> Result<Option<(u32, u32)>, Invalid> {
    if (mark_out as i32) <= (mark_in as i32) {
        return Ok(None);
    }
    if (mark_in as i32) < 0 || mark_out > MOST_BLOCKS || blocks > MOST_BLOCKS || mark_out > blocks {
        return Err(Invalid("a cut outside the replay's table"));
    }
    Ok(Some((mark_in, mark_out)))
}

/// Whether the clipboard (`clip` blocks) is pasted: not empty and fitting the table. Refused
/// at a block shown past the count.
pub fn pastes(clip: u32, blocks: u32, block: u32) -> Result<bool, Invalid> {
    if clip == 0 || blocks.wrapping_add(clip) > MOST_BLOCKS {
        return Ok(false);
    }
    if block > blocks {
        return Err(Invalid(
            "a paste past the replay's end: the original moves memory by a negative count",
        ));
    }
    Ok(true)
}

/// The snapshot's three digits one more (999 wraps to 000), and where the counting stopped:
/// the last digit touched and the loop counter.
pub fn next_snapshot(digits: [u8; 3]) -> ([u8; 3], usize, u32) {
    count_up(digits)
}

/// Decimal digits one more from the last (nines wrap to zeros, carrying to the digit before):
/// the digits, the last touched, and the loop counter. Also used for the screenshot's picture
/// name.
pub fn count_up<const N: usize>(mut digits: [u8; N]) -> ([u8; N], usize, u32) {
    let mut counter = 0;
    let mut last = N - 1;
    for n in (0..N).rev() {
        last = n;
        digits[n] = digits[n].wrapping_add(1);
        counter = n as u32;
        if digits[n] <= b'9' {
            break;
        }
        digits[n] = b'0';
        counter = (n as u32).wrapping_sub(1);
    }
    (digits, last, counter)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_far_cars_have_a_digit_over_the_first_picture() {
        assert_eq!(
            (3..=11).map(layout_digit).collect::<Vec<_>>(),
            [
                None,
                Some(3),
                Some(4),
                Some(5),
                Some(6),
                Some(7),
                Some(8),
                None,
                None
            ]
        );
        let [px, py, pw, ph] = DIGIT_PATCH;
        let [sx, sy, sw, sh] = FACE_SAMPLE;
        // The patch lies inside the box (x 2 .. 45, y 8 .. 38) and the sample is outside it.
        assert!(px >= 2 && py >= 8 && px + pw <= 46 && py + ph <= 39);
        assert!(sx >= px + pw && sx + sw <= 46 && sy >= 8 && sy + sh <= 39);
        assert_eq!(digit_spot(100, 200), (247, 211));
    }

    #[test]
    fn every_layout_has_a_picture() {
        assert_eq!(
            (0..=11).map(layout_picture).collect::<Vec<_>>(),
            [0, 1, 2, 3, 0, 0, 0, 0, 0, 0, 2, 2]
        );
    }

    #[test]
    fn files_and_steps() {
        assert_eq!(ReplayFile::of(0xDE), ReplayFile::LastRace);
        assert_eq!(ReplayFile::of(99), ReplayFile::Saved(99));
        assert_eq!(ReplayFile::of(105), ReplayFile::Own(5));
        assert_eq!(ReplayFile::of(GIVEN_FILE), ReplayFile::Given);
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
    fn marks_and_pictures() {
        assert_eq!(reset_marks(5, 3), (3, 0, 3));
        assert_eq!(reset_marks(2, 3), (2, 0, 3));
        assert_eq!(reset_marks(0xFFFF_FFFF, 3), (0xFFFF_FFFF, 0, 3));
        assert_eq!(picture_rate(0), 1);
        assert_eq!(picture_rate(0x8000_0000), 1);
        assert_eq!(picture_rate(500), 100);
        assert_eq!(picture_time(0, 25), 40);
        assert_eq!(picture_time(0x50, 25), 0x50);
        assert_eq!(next_snapshot(*b"041"), (*b"042", 2, 2));
        assert_eq!(next_snapshot(*b"099"), (*b"100", 0, 0));
        assert_eq!(next_snapshot(*b"999"), (*b"000", 0, u32::MAX));
        assert_eq!(count_up(*b"000199"), (*b"000200", 3, 3));
        assert_eq!(count_up(*b"00000:"), (*b"000010", 4, 4));
    }

    #[test]
    fn items_drawn() {
        assert_eq!((switch_byte(0xE), switch_byte(3)), (2, 3));
        assert_eq!(centred(10, 30, 10), 20);
        assert_eq!(centred(10, 5, 10), 10u32.wrapping_sub(3));
        assert_eq!(
            item_colour(GREY, true, || unreachable!(), || unreachable!()),
            0x666666
        );
        assert_eq!(item_colour(0, false, || unreachable!(), || 0), 0xFFFFFF);
        assert_eq!(item_colour(0, true, || true, || unreachable!()), 0xFF0000);
        assert_eq!(item_colour(0, true, || false, || 8), 0xFFFF00);
        assert_eq!(item_colour(0, true, || false, || 7), 0xFF0000);
    }

    #[test]
    fn actions() {
        assert_eq!(Action::of(0x29), None);
        assert_eq!(Action::of(0x26), Some(Action::Pictures { time: 0xA0 }));
        assert_eq!(Action::of(9), Some(Action::Goto(Goto::On(10))));
        let var = |v: Var| match v {
            Var::Block => 4,
            Var::Blocks => 10,
            Var::MarkIn => 2,
            Var::MarkOut => 7,
        };
        assert_eq!(goto_block(Goto::Back(10), var), 0);
        assert_eq!(goto_block(Goto::On(10), var), 10);
        assert_eq!(goto_block(Goto::On(1), var), 5);
        assert_eq!(goto_block(Goto::MarkOut, var), 7);
        assert_eq!(goto_block(Goto::Start, |_| unreachable!()), 0);
        assert_eq!(frame_back(0, 0), (0, 0));
        assert_eq!(frame_back(3, 0), (2, 99));
        assert_eq!(frame_back(0, 0xFFFF_FFFF), (0, 99));
        assert_eq!(frame_on(10, 5, 10), (10, 5));
        assert_eq!(frame_on(9, 99, 10), (10, 0));
        assert_eq!(frame_on(3, 4, 10), (3, 5));
        assert!(mark_in_here(3, 3) && !mark_in_here(4, 3));
        assert!(mark_out_here(3, 3) && !mark_out_here(2, 3));
        assert_eq!(
            [3u8, 4, 0x7F].map(|b| next_byte(5, b, false, 0)),
            [2, 2, 0x80]
        );
        assert_eq!(
            [1u8, 2, 0xFE].map(|b| next_byte(4, b, false, 0)),
            [2, 0, 0xFF]
        );
        assert_eq!(
            [0u8, 1, 2, 3].map(|b| next_byte(5, b, true, 0)),
            [1, 2, 3, 0]
        );
        assert_eq!([1u8, 2].map(|b| next_byte(4, b, true, 0)), [2, 0]);
        // Three players: 2, 3, then the third car alone, then round; four: the fourth car too.
        assert_eq!([2u8, 3, 4].map(|b| next_byte(5, b, false, 1)), [3, 4, 2]);
        assert_eq!(
            [2u8, 3, 4, 5].map(|b| next_byte(5, b, false, 2)),
            [3, 4, 5, 2]
        );
        assert!(views_choosable(false, 1) && views_choosable(true, 2));
        assert!(!views_choosable(true, 1));
        assert_eq!(cut_marks(3, 3, 10), Ok(None));
        assert_eq!(cut_marks(2, 7, 10), Ok(Some((2, 7))));
        assert!(cut_marks(2, 11, 10).is_err());
        assert_eq!(pastes(0, 5, 9), Ok(false));
        assert_eq!(pastes(3, MOST_BLOCKS - 2, 9), Ok(false));
        assert_eq!(pastes(3, 10, 10), Ok(true));
        assert!(pastes(3, 10, 11).is_err());
    }

    #[test]
    fn edits() {
        assert_eq!(clip(5, 5), Ok(None));
        assert_eq!(clip(0, u32::MAX), Ok(None));
        assert_eq!(clip(2, 7), Ok(Some((2, 5))));
        assert!(clip(u32::MAX, 3).is_err());
        assert!(clip(0, MOST_BLOCKS + 1).is_err());
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
