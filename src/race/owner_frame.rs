//! The race owner's frame group (`race::owner::Race::frame`): the frame drawn: frame work,
//! matrices, view vars, scratch, batches, panel, overlay, text, shadows and skids, sound spots,
//! map.

use crate::memory::session::{DrawShared, GameThread};
use crate::race::clock_ticks_store::ClockTicks;
use crate::race::draw_batches_store::DrawBatches;
use crate::race::error_list_store::ErrorList;
use crate::race::frame_counter_store::FrameCounter;
use crate::race::frame_store::FrameWork;
use crate::race::frame_text::Credits;
use crate::race::game_map_store::MapVars;
use crate::race::groups_store::{LightsAndGear, PanelAndOverlay, ViewAndRecords};
use crate::race::matrices_store::SceneMatrices;
use crate::race::scratch_store::FrameScratch;
use crate::race::shadow_skids_store::ShadowSkids;
use crate::race::sound_spots_store::SoundSpotBlock;
use std::cell::{Ref, RefCell, RefMut};

/// The race's frame state. The view
/// variables, the panel and the overlay are fields of `view_and_records` and
/// `panel_and_overlay` (`view_vars_store`, `panel_store`, `overlay_store` read them there).
pub struct Frame {
    /// Whether the race is a split screen of three or four players: then the view slots are the
    /// players' views (slot `n` player `n`) and there are no mirrors, where with one or two
    /// players slots 2 and 3 are the first two players' mirrors. Set by the frame's drawing,
    /// before its parts run.
    pub many_views: std::sync::atomic::AtomicBool,
    /// The frames drawn between ticks: each car's last two ticks and the next frame's place.
    pub blend: crate::race::blend::Blend,
    /// The overview map's picture of the stage: its colours (0xRRGGBB, `None` where the map has no
    /// tile); `None` until it is made.
    pub overview_picture: std::sync::Mutex<Option<Vec<Option<u32>>>>,
    /// The whole nanosecond count at the last frame-rate measure.
    pub last_frame_nanos: std::sync::atomic::AtomicU64,
    /// One copy, lent to the drawing parts (`scene::workers::DrawReads`): no part writes it (the two
    /// counters nobody reads stay atomic).
    pub frame_work: DrawShared<FrameWork>,
    /// One copy, lent to the drawing parts (`scene::workers::DrawReads`): no part writes it (the held
    /// columns are each part's own, `scene::view_matrices::Held`, merged in the parts' order).
    pub matrices: DrawShared<SceneMatrices>,
    /// One copy, lent to the drawing parts (`scene::workers::DrawReads`): the drawing only reads it.
    pub shadow_skids: DrawShared<ShadowSkids>,
    /// One copy, lent to the drawing parts (`scene::workers::DrawReads`): the mirror's window and the
    /// polygon drawing read it; the hud and the race write it on the game's thread.
    pub panel_and_overlay: DrawShared<PanelAndOverlay>,
    /// One copy, lent to the drawing parts (`scene::workers::DrawReads`): no part writes it. Each
    /// view draws on its own view variables (`ViewScratch::vars`).
    pub view_and_records: DrawShared<ViewAndRecords>,
    /// One copy, lent to the drawing parts (`scene::workers::DrawReads`): no part writes it.
    pub lights_and_gear: DrawShared<LightsAndGear>,
    /// The cars as the frame's drawing sees them (`race::frame_view`); behind atomics, read by
    /// the drawing parts.
    pub cars: crate::race::frame_view::FrameCars,
    main: GameThread<FrameMain>,
}

/// What only the game's thread reaches (no drawing part and no other thread touches it).
struct FrameMain {
    scratch: RefCell<FrameScratch>,
    counter: RefCell<FrameCounter>,
    draw_batches: DrawBatches,
    clock_ticks: RefCell<ClockTicks>,
    sound_spots: SoundSpotBlock,
    error_list: RefCell<ErrorList>,
    credits: RefCell<Credits>,
    map: MapVars,
}

impl Frame {
    pub const fn new() -> Self {
        Frame {
            many_views: std::sync::atomic::AtomicBool::new(false),
            blend: crate::race::blend::Blend::new(),
            overview_picture: std::sync::Mutex::new(None),
            last_frame_nanos: std::sync::atomic::AtomicU64::new(0),
            frame_work: DrawShared::new(FrameWork::new()),
            matrices: DrawShared::new(SceneMatrices::new()),
            shadow_skids: DrawShared::new(ShadowSkids::new()),
            panel_and_overlay: DrawShared::new(PanelAndOverlay::new()),
            view_and_records: DrawShared::new(ViewAndRecords::new()),
            lights_and_gear: DrawShared::new(LightsAndGear::new()),
            cars: crate::race::frame_view::FrameCars::new(),
            main: GameThread::new(FrameMain {
                scratch: RefCell::new(FrameScratch::new()),
                counter: RefCell::new(FrameCounter::new()),
                draw_batches: DrawBatches::new(),
                clock_ticks: RefCell::new(ClockTicks::new()),
                sound_spots: SoundSpotBlock::new(),
                error_list: RefCell::new(ErrorList::new()),
                credits: RefCell::new(Credits::new()),
                map: MapVars::new(),
            }),
        }
    }

    /// The work vector, read.
    pub fn scratch(&self) -> Ref<'_, FrameScratch> {
        self.main.get().scratch.borrow()
    }

    /// The work vector, to write: no other borrow may be held.
    pub fn scratch_mut(&self) -> RefMut<'_, FrameScratch> {
        self.main.get().scratch.borrow_mut()
    }

    /// The frame counter setting, read: no write borrow may be held.
    pub fn counter(&self) -> Ref<'_, FrameCounter> {
        self.main.get().counter.borrow()
    }

    /// The frame counter setting, to write: no other borrow may be held.
    pub fn counter_mut(&self) -> RefMut<'_, FrameCounter> {
        self.main.get().counter.borrow_mut()
    }

    pub fn draw_batches(&self) -> &DrawBatches {
        &self.main.get().draw_batches
    }

    /// The sound tick clock, read: no write borrow may be held.
    pub fn clock_ticks(&self) -> Ref<'_, ClockTicks> {
        self.main.get().clock_ticks.borrow()
    }

    /// The sound tick clock, to write: no other borrow may be held.
    pub fn clock_ticks_mut(&self) -> RefMut<'_, ClockTicks> {
        self.main.get().clock_ticks.borrow_mut()
    }

    pub fn sound_spots(&self) -> &SoundSpotBlock {
        &self.main.get().sound_spots
    }

    /// The error list, read: no write borrow may be held.
    pub fn error_list(&self) -> Ref<'_, ErrorList> {
        self.main.get().error_list.borrow()
    }

    /// The error list, to write: no other borrow may be held.
    pub fn error_list_mut(&self) -> RefMut<'_, ErrorList> {
        self.main.get().error_list.borrow_mut()
    }

    /// The credits' state, read: no write borrow may be held.
    pub fn credits(&self) -> Ref<'_, Credits> {
        self.main.get().credits.borrow()
    }

    /// The credits' state, to write: no other borrow may be held.
    pub fn credits_mut(&self) -> RefMut<'_, Credits> {
        self.main.get().credits.borrow_mut()
    }

    pub fn map(&self) -> &MapVars {
        &self.main.get().map
    }
}
