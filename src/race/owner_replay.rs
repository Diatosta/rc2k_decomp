//! The race owner's replay group (`race::owner::Race::replay`): the replay and the viewer: finish, replay frames and work, viewer buffers and state, snapshot, seal, last race, cars and replay.

use crate::drive::finish_store::Finish;
use crate::drive::replay_frames_store::ReplayFrames;
use crate::race::groups_store::{CarsAndReplay, ViewerAndKinds};
use crate::race::replay_path::LAST_RACE_LEN;
use crate::race::seal_store::SealWork;
use crate::race::viewer_state_store::ReplayViewBlock;
use crate::session::{Bytes, Dword, with_runs};

/// The race's replay state (step 11, owned state: the stores move in as fields).
pub struct Replay {
    /// 0x64BB18 .. 0x64BB88 (`drive::finish_store`).
    pub finish: Finish,
    /// 0x6EF7CC .. 0x70F3EC (`drive::replay_frames_store`).
    pub replay_frames: ReplayFrames,
    /// The replay viewer's state (`race::viewer_state_store`).
    pub replay_view: ReplayViewBlock,
    /// `aSavedataSnapsh`: the viewer's snapshot picture's name (`race::find_snapshot_number`).
    pub snapshot: Bytes<26>,
    /// `dword_4F66F9`: the race record's seal key (`race::replay`).
    pub seal_key: Dword,
    /// The seal's work (`race::seal_store`).
    pub seal_work: SealWork,
    /// `aVarTmprpl`: the last race's replay file's name (`race::replay_path`).
    pub last_race: [u8; LAST_RACE_LEN],
    /// The cars' blocks and the replay's work (`race::groups_store`).
    pub cars_and_replay: CarsAndReplay,
    /// The viewer's panel .. the kinds' tables (`race::groups_store`).
    pub viewer_and_kinds: ViewerAndKinds,
}

impl Replay {
    pub const fn new() -> Self {
        Replay {
            finish: Finish::new(),
            replay_frames: ReplayFrames::new(),
            replay_view: ReplayViewBlock::new(),
            snapshot: Bytes::new(*b"savedata\\snapshot\\000.pcx\0"),
            seal_key: Dword::new(0),
            seal_work: SealWork::new(),
            last_race: with_runs(&[(0, b"var\\tmprpl\0")]),
            cars_and_replay: CarsAndReplay::new(),
            viewer_and_kinds: ViewerAndKinds::new(),
        }
    }
}
