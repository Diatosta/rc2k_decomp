//! The race owner's replay group (`race::owner::Race::replay`): the replay and the viewer: finish, replay frames and work, viewer buffers and state, snapshot, seal, last race, cars and replay.

use crate::drive::finish_store::Finish;
use crate::drive::replay_frames_store::ReplayFrames;
use crate::race::groups_store::{CarsAndReplay, ViewerAndKinds};
use crate::race::replay_path::LAST_RACE_LEN;
use crate::race::seal_store::SealWork;
use crate::race::viewer_state_store::ReplayViewBlock;
use crate::session::{GameThread, PlainBytes, PlainDword, with_runs};

/// The race's replay state (step 11, owned state: the stores move in as fields).
pub struct Replay {
    /// 0x64BB18 .. 0x64BB88 (`drive::finish_store`).
    pub finish: Finish,
    /// `aVarTmprpl`: the last race's replay file's name (`race::replay_path`).
    pub last_race: [u8; LAST_RACE_LEN],
    /// The cars' blocks and the replay's work (`race::groups_store`).
    pub cars_and_replay: CarsAndReplay,
    /// The viewer's panel .. the kinds' tables (`race::groups_store`).
    pub viewer_and_kinds: ViewerAndKinds,
    main: GameThread<ReplayMain>,
}

/// What only the game's thread reaches (no drawing part and no other thread touches it).
struct ReplayMain {
    replay_frames: ReplayFrames,
    replay_view: ReplayViewBlock,
    snapshot: PlainBytes<26>,
    seal_key: PlainDword,
    seal_work: SealWork,
}

impl Replay {
    pub const fn new() -> Self {
        Replay {
            finish: Finish::new(),
            last_race: with_runs(&[(0, b"var\\tmprpl\0")]),
            cars_and_replay: CarsAndReplay::new(),
            viewer_and_kinds: ViewerAndKinds::new(),
            main: GameThread::new(ReplayMain {
                replay_frames: ReplayFrames::new(),
                replay_view: ReplayViewBlock::new(),
                snapshot: PlainBytes::new(*b"savedata\\snapshot\\000.pcx\0"),
                seal_key: PlainDword::new(0),
                seal_work: SealWork::new(),
            }),
        }
    }

    pub fn replay_frames(&self) -> &ReplayFrames {
        &self.main.get().replay_frames
    }

    pub fn replay_view(&self) -> &ReplayViewBlock {
        &self.main.get().replay_view
    }

    pub fn snapshot(&self) -> &PlainBytes<26> {
        &self.main.get().snapshot
    }

    pub fn seal_key(&self) -> &PlainDword {
        &self.main.get().seal_key
    }

    pub fn seal_work(&self) -> &SealWork {
        &self.main.get().seal_work
    }
}
