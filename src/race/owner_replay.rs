//! The race owner's replay group (`race::owner::Race::replay`): the replay and the viewer: finish,
//! replay frames and work, viewer buffers and state, snapshot, seal, last race, cars and replay.

use crate::drive::finish_store::Finish;
use crate::drive::replay_frames_store::ReplayFrames;
use crate::memory::plain_buffer::PlainBuffer;
use crate::memory::session::{GameThread, with_runs};
use crate::race::groups_store::ViewerAndKinds;
use crate::race::replay_path::LAST_RACE_LEN;
use crate::race::replay_staging_store::ReplayStaging;
use crate::race::replay_v2_store::ReplayV2;
use crate::race::replay_work_store::ReplayWork;
use crate::race::seal_store::SealWork;
use crate::race::viewer_state_store::ReplayViewBlock;
use std::cell::{Cell, Ref, RefCell, RefMut};

/// The race's replay state.
pub struct Replay {
    /// The finish state (`drive::finish_store`). Borrowed for one statement at a time; the
    /// drawing parts are handed `no_place` by value (`scene::workers`).
    finish: GameThread<RefCell<Finish>>,
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
    replay_frames: RefCell<ReplayFrames>,
    replay_work: RefCell<ReplayWork>,
    staging: RefCell<ReplayStaging>,
    replay_view: ReplayViewBlock,
    snapshot: PlainBuffer<26>,
    /// The replay header's checksum, kept as the key the stage time is sealed with.
    seal_key: Cell<u32>,
    seal_work: RefCell<SealWork>,
    v2: ReplayV2,
}

impl Replay {
    pub const fn new() -> Self {
        Replay {
            finish: GameThread::new(RefCell::new(Finish::new())),
            last_race: with_runs(&[(0, b"var\\tmprpl\0")]),
            given_file: std::sync::OnceLock::new(),
            viewer_and_kinds: ViewerAndKinds::new(),
            render_row: crate::race::render_row::RenderRow::new(),
            viewer_tip: crate::race::viewer_tip::ViewerTip::new(),
            main: GameThread::new(ReplayMain {
                replay_frames: RefCell::new(ReplayFrames::new()),
                replay_work: RefCell::new(ReplayWork::new()),
                staging: RefCell::new(ReplayStaging::new()),
                replay_view: ReplayViewBlock::new(),
                snapshot: PlainBuffer::new(*b"savedata\\snapshot\\000.png\0"),
                seal_key: Cell::new(0),
                seal_work: RefCell::new(SealWork::new()),
                v2: ReplayV2::new(),
            }),
        }
    }

    /// The finish state, read.
    pub fn finish(&self) -> Ref<'_, Finish> {
        self.finish.get().borrow()
    }

    /// The finish state, to write: no other borrow may be held.
    pub fn finish_mut(&self) -> RefMut<'_, Finish> {
        self.finish.get().borrow_mut()
    }

    /// The finish state's "no place" switch: set while the race has no place or panel to draw.
    pub fn no_place(&self) -> u32 {
        self.finish().no_place
    }

    /// The replay's frame variables, read.
    pub fn replay_frames(&self) -> Ref<'_, ReplayFrames> {
        self.main.get().replay_frames.borrow()
    }

    /// The replay's frame variables, to write: no other borrow may be held.
    pub fn replay_frames_mut(&self) -> RefMut<'_, ReplayFrames> {
        self.main.get().replay_frames.borrow_mut()
    }

    /// The replay's work (`race::replay_work_store`).
    pub fn replay_work(&self) -> Ref<'_, ReplayWork> {
        self.main.get().replay_work.borrow()
    }

    /// The replay's work, to write: no other borrow may be held.
    pub fn replay_work_mut(&self) -> RefMut<'_, ReplayWork> {
        self.main.get().replay_work.borrow_mut()
    }

    /// The replay's frame as the race keeps it (`race::replay_staging_store`).
    pub fn staging(&self) -> Ref<'_, ReplayStaging> {
        self.main.get().staging.borrow()
    }

    /// The replay's frame, to write: no other borrow may be held.
    pub fn staging_mut(&self) -> RefMut<'_, ReplayStaging> {
        self.main.get().staging.borrow_mut()
    }

    pub fn replay_view(&self) -> &ReplayViewBlock {
        &self.main.get().replay_view
    }

    pub fn snapshot(&self) -> &PlainBuffer<26> {
        &self.main.get().snapshot
    }

    /// The key the stage time is sealed with.
    pub fn seal_key(&self) -> u32 {
        self.main.get().seal_key.get()
    }

    pub fn set_seal_key(&self, key: u32) {
        self.main.get().seal_key.set(key);
    }

    /// The seal cipher's work, read.
    pub fn seal_work(&self) -> Ref<'_, SealWork> {
        self.main.get().seal_work.borrow()
    }

    /// The seal cipher's work, to write: no other borrow may be held.
    pub fn seal_work_mut(&self) -> RefMut<'_, SealWork> {
        self.main.get().seal_work.borrow_mut()
    }

    /// The own-format replay's cars and tail (`race::replay_v2_store`).
    pub fn v2(&self) -> &ReplayV2 {
        &self.main.get().v2
    }
}
