//! The race owner's replay group (`race::owner::Race::replay`): the replay and the viewer: finish,
//! replay frames and work, viewer buffers and state, snapshot, seal, last race, cars and replay.

use crate::drive::finish_store::Finish;
use crate::drive::replay_frames_store::ReplayFrames;
use crate::memory::session::{DrawShared, GameThread, PlainBytes, PlainDword, with_runs};
use crate::race::groups_store::ViewerAndKinds;
use crate::race::replay_path::LAST_RACE_LEN;
use crate::race::replay_staging_store::ReplayStaging;
use crate::race::replay_v2_store::ReplayV2;
use crate::race::replay_work_store::ReplayWork;
use crate::race::seal_store::SealWork;
use crate::race::viewer_state_store::ReplayViewBlock;

/// The race's replay state.
pub struct Replay {
    /// The finish state (`drive::finish_store`); lent to the drawing parts
    /// (`scene::workers::DrawReads`): no part writes it.
    pub finish: DrawShared<Finish>,
    /// The last race's replay file's name (`race::replay_path`).
    pub last_race: [u8; LAST_RACE_LEN],
    /// A replay file named when the program was started (`menu::launch`): its path, once known.
    pub given_file: std::sync::OnceLock<Vec<u8>>,
    /// The viewer's panel through the kinds' tables (`race::groups_store`).
    pub viewer_and_kinds: ViewerAndKinds,
    /// The render row under the viewer's buttons.
    pub render_row: crate::race::render_row::RenderRow,
    /// The name shown under the render button.
    pub viewer_tip: crate::race::viewer_tip::ViewerTip,
    main: GameThread<ReplayMain>,
}

/// What only the game's thread reaches (no drawing part and no other thread touches it).
struct ReplayMain {
    replay_frames: ReplayFrames,
    replay_work: ReplayWork,
    staging: ReplayStaging,
    replay_view: ReplayViewBlock,
    snapshot: PlainBytes<26>,
    seal_key: PlainDword,
    seal_work: SealWork,
    v2: ReplayV2,
}

impl Replay {
    pub const fn new() -> Self {
        Replay {
            finish: DrawShared::new(Finish::new()),
            last_race: with_runs(&[(0, b"var\\tmprpl\0")]),
            given_file: std::sync::OnceLock::new(),
            viewer_and_kinds: ViewerAndKinds::new(),
            render_row: crate::race::render_row::RenderRow::new(),
            viewer_tip: crate::race::viewer_tip::ViewerTip::new(),
            main: GameThread::new(ReplayMain {
                replay_frames: ReplayFrames::new(),
                replay_work: ReplayWork::new(),
                staging: ReplayStaging::new(),
                replay_view: ReplayViewBlock::new(),
                snapshot: PlainBytes::new(*b"savedata\\snapshot\\000.png\0"),
                seal_key: PlainDword::new(0),
                seal_work: SealWork::new(),
                v2: ReplayV2::new(),
            }),
        }
    }

    pub fn replay_frames(&self) -> &ReplayFrames {
        &self.main.get().replay_frames
    }

    /// The replay's work (`race::replay_work_store`).
    pub fn replay_work(&self) -> &ReplayWork {
        &self.main.get().replay_work
    }

    /// The replay's frame as the race keeps it (`race::replay_staging_store`).
    pub fn staging(&self) -> &ReplayStaging {
        &self.main.get().staging
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

    /// The own-format replay's cars and tail (`race::replay_v2_store`).
    pub fn v2(&self) -> &ReplayV2 {
        &self.main.get().v2
    }
}
