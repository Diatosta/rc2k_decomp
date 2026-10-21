//! The replay file's rules (converted 2026-10-21 from `race::open_replay` 0x41E6EC,
//! `add_replay_frame` 0x41E8DE, `write_replay_block` 0x41E9A3, `close_replay` 0x41E85A,
//! `write_replay_chunk` 0x41EEE4 and `seal_replay_header` 0x41F0A3): when a replay is
//! recorded, the frames gathered into blocks of 100, the blocks' places in the index,
//! the chunks packed by runs when that is smaller, and the header's sealed part. The
//! race's settings in the replay's car header converted 2026-10-21 from
//! `race::record_replay_header` 0x460246 and `race::read_replay_header` 0x460428.

/// The frames a block holds.
pub const BLOCK_FRAMES: u32 = 100;
/// A frame's bytes (smaller pieces are put in without counting).
pub const FRAME_BYTES: u32 = 0x10;
/// The blocks the index holds.
pub const MOST_BLOCKS: u32 = 0x708;
/// The room (KB) a replay needs on the disc.
pub const NEEDED_KB: u32 = 0x7800;
/// The header's first dword, "REPL".
pub const HEADER_MAGIC: u32 = 0x4C50_4552;
/// The header's version.
pub const HEADER_VERSION: u32 = 0x50400;
/// The header's name, "tmpreplay" with its NUL, as three dwords.
pub const HEADER_NAME: [u32; 3] = [0x7270_6D74, 0x616C_7065, 0x79];
/// The index's first dword, "RCIB".
pub const INDEX_MAGIC: u32 = 0x4249_4352;
/// The sealed part's version byte.
pub const SEAL_VERSION: u8 = 6;
/// The sealed part's key, "halfmoon".
pub const SEAL_KEY: [u32; 2] = [0x666C_6168, 0x6E6F_6F6D];

/// Whether replays are recorded: on (1), not a demonstration (0), and neither of the two
/// other switches 1 (`dword_5BEC78`, `dword_5BEC7C`, `dword_52A910`, `dword_75ACDC`).
pub fn replays_on(on: u32, demo: u32, a: u32, b: u32) -> bool {
    on == 1 && demo == 0 && a != 1 && b != 1
}

/// Whether the disc has room for a replay (KB free, unsigned).
pub fn room_for_replay(free_kb: u32) -> bool {
    free_kb >= NEEDED_KB
}

/// What a piece put into the block does: whether it counts as a frame, and whether the
/// block is full after it (the frames in it, counted).
pub fn frame_added(bytes: u32, in_block: u32) -> (u32, bool) {
    let n = if bytes == FRAME_BYTES {
        in_block.wrapping_add(1)
    } else {
        in_block
    };
    (n, n >= BLOCK_FRAMES)
}

/// Whether `dwords` more fit in the block at `cursor` (bytes from the block's start;
/// `room` its size).
pub fn fits(cursor: u32, dwords: u32, room: u32) -> bool {
    cursor <= room && dwords <= (room - cursor) / 4
}

/// A block written: the size taken in the file (a packed one's is given negated) and
/// whether the index is then full.
pub fn block_written(written: u32, block: u32) -> (u32, bool) {
    let size = if (written as i32) < 0 {
        written.wrapping_neg()
    } else {
        written
    };
    (size, block + 1 == MOST_BLOCKS)
}

/// The last block's frame count at the close: 100, or for the replay's last block (the
/// one after `last_block` is the count) the frames modulo 100 plus one. Gives the count
/// and the original's eax and edx at that point (edx only for the last block).
pub fn closing_frames(last_block: u32, blocks: u32, frames: u32) -> (u32, u32, Option<u32>) {
    let next = last_block.wrapping_add(1);
    if next == blocks {
        let rest = frames % BLOCK_FRAMES;
        (rest + 1, frames / BLOCK_FRAMES, Some(rest))
    } else {
        (BLOCK_FRAMES, next, None)
    }
}

/// How a chunk is written (`write_replay_chunk`): its packed size rounded up to a dword,
/// used when below the plain size.
pub fn packed_size(packed: u32, plain: u32) -> Option<u32> {
    let rounded = packed.wrapping_add(3) & !3;
    (rounded < plain).then_some(rounded)
}

/// The sealed part's checksum: its dwords from the second on, each added then the sum
/// turned left by 5.
pub fn seal_checksum(dwords: &[u32]) -> u32 {
    dwords
        .iter()
        .fold(0u32, |sum, &d| sum.wrapping_add(d).rotate_left(5))
}

/// A block's input to the cipher: its two dwords each mixed (xor) with the block sealed
/// before (0, 0 for the first).
pub fn chained(block: [u32; 2], before: [u32; 2]) -> [u32; 2] {
    [block[0] ^ before[0], block[1] ^ before[1]]
}

/// One of the race's settings the replay's car header (`dword_719B14` ..) keeps.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Setting {
    /// The arcade's noise seed (`dword_60519C`).
    Seed,
    /// The arcade's start place (`dword_6051BC`).
    StartPlace,
    /// The arcade's cars (`dword_6051B4`).
    ArcadeCars,
    /// The stage (`g_stageNumber`).
    Stage,
    /// The game mode (`dword_62389C`).
    Mode,
    /// The players (`dword_63DB94`).
    Players,
    /// The race's cars (`g_raceCars`).
    Cars,
    /// Car n's kind (its block's +0x24; read back also into `g_raceCarKinds`[n]).
    Kind(u32),
    /// Player n's game-over flag (its block's +0xB0).
    GameOver(u32),
}

impl Setting {
    /// Its place in the car header.
    pub fn offset(self) -> u32 {
        match self {
            Setting::Stage => 0,
            Setting::Mode => 4,
            Setting::Players => 8,
            Setting::Cars => 0xC,
            Setting::Kind(n) => 0x10 + 4 * n,
            Setting::StartPlace => 0x2C,
            Setting::ArcadeCars => 0x30,
            Setting::Seed => 0x34,
            Setting::GameOver(n) => 0x38 + 4 * n,
        }
    }
}

/// The settings `race::record_replay_header` copies and logs, in its order (and
/// `race::read_replay_header` copies back).
pub const HEADER_SETTINGS: [Setting; 13] = [
    Setting::Seed,
    Setting::StartPlace,
    Setting::ArcadeCars,
    Setting::Stage,
    Setting::Mode,
    Setting::Players,
    Setting::Cars,
    Setting::Kind(0),
    Setting::Kind(1),
    Setting::Kind(2),
    Setting::Kind(3),
    Setting::GameOver(0),
    Setting::GameOver(1),
];

/// The settings `race::read_replay_header` logs (all but the seed), in its order.
pub fn read_logged() -> &'static [Setting] {
    &HEADER_SETTINGS[1..]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recording() {
        assert!(replays_on(1, 0, 0, 0));
        assert!(!replays_on(2, 0, 0, 0) && !replays_on(1, 1, 0, 0));
        assert!(!replays_on(1, 0, 1, 0) && !replays_on(1, 0, 0, 1));
        assert!(room_for_replay(0x7800) && !room_for_replay(0x77FF));
        assert_eq!(frame_added(0x10, 98), (99, false));
        assert_eq!(frame_added(0x10, 99), (100, true));
        assert_eq!(frame_added(8, 99), (99, false));
        assert!(fits(0x2800, 4, 0x2810) && !fits(0x2800, 5, 0x2810) && !fits(0x2814, 0, 0x2810));
        assert_eq!(block_written((-0x100i32) as u32, 3), (0x100, false));
        assert_eq!(block_written(0x2810, MOST_BLOCKS - 1), (0x2810, true));
        assert_eq!(closing_frames(4, 5, 437), (38, 4, Some(37)));
        assert_eq!(closing_frames(2, 5, 437), (100, 3, None));
        assert_eq!(packed_size(0x101, 0x2810), Some(0x104));
        assert_eq!(packed_size(0x280E, 0x2810), None);
    }

    #[test]
    fn sealing() {
        assert_eq!(seal_checksum(&[1]), 1 << 5);
        assert_eq!(seal_checksum(&[1, 1]), ((1u32 << 5) + 1) << 5);
        assert_eq!(seal_checksum(&[0x8000_0000]), 0x10);
        assert_eq!(chained([1, 2], [3, 3]), [2, 1]);
        assert_eq!(&SEAL_KEY.map(u32::to_le_bytes).concat(), b"halfmoon");
        assert_eq!(
            &HEADER_NAME.map(u32::to_le_bytes).concat()[..10],
            b"tmpreplay\0"
        );
    }

    #[test]
    fn car_header() {
        let offsets: Vec<u32> = HEADER_SETTINGS.iter().map(|s| s.offset()).collect();
        assert_eq!(
            offsets,
            [
                0x34, 0x2C, 0x30, 0, 4, 8, 0xC, 0x10, 0x14, 0x18, 0x1C, 0x38, 0x3C
            ]
        );
        assert_eq!(read_logged().len(), 12);
        assert_eq!(read_logged()[0], Setting::StartPlace);
    }
}
