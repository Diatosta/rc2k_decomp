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
use crate::race::game_map::MapVars;
use crate::race::groups_store::{LightsAndGear, PanelAndOverlay, ViewAndRecords};
use crate::race::matrices_store::SceneMatrices;
use crate::race::panel_clock_store::PanelClock;
use crate::race::scratch_store::FrameScratch;
use crate::race::shadow_skids_store::ShadowSkids;
use crate::race::sound_spots_store::SoundSpotBlock;
use crate::race::text_state_store::TextState;

/// The race's frame state. The view
/// variables, the panel and the overlay are fields of `view_and_records` and
/// `panel_and_overlay` (`view_vars_store`, `panel_store`, `overlay_store` read them there).
pub struct Frame {
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
    scratch: FrameScratch,
    counter: FrameCounter,
    draw_batches: DrawBatches,
    panel_clock: PanelClock,
    text_state: TextState,
    clock_ticks: ClockTicks,
    sound_spots: SoundSpotBlock,
    error_list: ErrorList,
    credits: Credits,
    map: MapVars,
}

impl Frame {
    pub const fn new() -> Self {
        Frame {
            frame_work: DrawShared::new(FrameWork::new()),
            matrices: DrawShared::new(SceneMatrices::new()),
            shadow_skids: DrawShared::new(ShadowSkids::new()),
            panel_and_overlay: DrawShared::new(PanelAndOverlay::new()),
            view_and_records: DrawShared::new(ViewAndRecords::new()),
            lights_and_gear: DrawShared::new(LightsAndGear::new()),
            cars: crate::race::frame_view::FrameCars::new(),
            main: GameThread::new(FrameMain {
                scratch: FrameScratch::new(),
                counter: FrameCounter::new(),
                draw_batches: DrawBatches::new(),
                panel_clock: PanelClock::new(),
                text_state: TextState::new(),
                clock_ticks: ClockTicks::new(),
                sound_spots: SoundSpotBlock::new(),
                error_list: ErrorList::new(),
                credits: Credits::new(),
                map: MapVars::new(),
            }),
        }
    }

    pub fn scratch(&self) -> &FrameScratch {
        &self.main.get().scratch
    }

    pub fn counter(&self) -> &FrameCounter {
        &self.main.get().counter
    }

    pub fn draw_batches(&self) -> &DrawBatches {
        &self.main.get().draw_batches
    }

    pub fn panel_clock(&self) -> &PanelClock {
        &self.main.get().panel_clock
    }

    pub fn text_state(&self) -> &TextState {
        &self.main.get().text_state
    }

    pub fn clock_ticks(&self) -> &ClockTicks {
        &self.main.get().clock_ticks
    }

    pub fn sound_spots(&self) -> &SoundSpotBlock {
        &self.main.get().sound_spots
    }

    pub fn error_list(&self) -> &ErrorList {
        &self.main.get().error_list
    }

    pub fn credits(&self) -> &Credits {
        &self.main.get().credits
    }

    pub fn map(&self) -> &MapVars {
        &self.main.get().map
    }
}
