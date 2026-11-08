//! The race owner's frame group (`race::owner::Race::frame`): the frame drawn: frame work, matrices, view vars, scratch, batches, panel, overlay, text, shadows and skids, sound spots, map.

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

/// The race's frame state (step 11, owned state: the stores move in as fields). The view
/// variables, the panel and the overlay are fields of `view_and_records` and
/// `panel_and_overlay` (`view_vars_store`, `panel_store`, `overlay_store` read them there).
pub struct Frame {
    pub frame_work: FrameWork,
    pub matrices: SceneMatrices,
    pub scratch: FrameScratch,
    pub counter: FrameCounter,
    pub draw_batches: DrawBatches,
    pub panel_clock: PanelClock,
    pub text_state: TextState,
    pub clock_ticks: ClockTicks,
    pub shadow_skids: ShadowSkids,
    pub sound_spots: SoundSpotBlock,
    pub error_list: ErrorList,
    pub credits: Credits,
    pub map: MapVars,
    pub panel_and_overlay: PanelAndOverlay,
    pub view_and_records: ViewAndRecords,
    pub lights_and_gear: LightsAndGear,
}

impl Frame {
    pub const fn new() -> Self {
        Frame {
            frame_work: FrameWork::new(),
            matrices: SceneMatrices::new(),
            scratch: FrameScratch::new(),
            counter: FrameCounter::new(),
            draw_batches: DrawBatches::new(),
            panel_clock: PanelClock::new(),
            text_state: TextState::new(),
            clock_ticks: ClockTicks::new(),
            shadow_skids: ShadowSkids::new(),
            sound_spots: SoundSpotBlock::new(),
            error_list: ErrorList::new(),
            credits: Credits::new(),
            map: MapVars::new(),
            panel_and_overlay: PanelAndOverlay::new(),
            view_and_records: ViewAndRecords::new(),
            lights_and_gear: LightsAndGear::new(),
        }
    }
}
