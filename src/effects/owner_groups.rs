//! The effects' remaining stores in groups (owners, stage 1): what was a static of its
//! own each is a field of one group of `effects::owner::Effects`.

use crate::effects::blend_corner::BlendWork;
use crate::effects::blocks::{CornerSlopes, MarkCorners, MarkLinks, PlacesBlock};
use crate::effects::clip::{ClipScreen, GlowState};
use crate::effects::emitters::EmitterVars;
use crate::effects::frame_state::{DrawFrame, DrawWork, FlareParts, FlareScales, ViewVars};
use crate::effects::kind_store::{KindExtra2, KindExtras1, QuadWork};
use crate::effects::kind_values::MarksKind;
use crate::effects::mark_slots::{MarkSlots, SlotTuning};
use crate::effects::mask_constants::MaskConstants;
use crate::effects::run_particles::{BurstColours, ParticleTuning};
use crate::effects::setup::{ClockK, MarksClock, SettingsPointer};
use crate::effects::tuning::{MarksFloats, MarksFrame, MarksKindFloats};
use crate::effects::view_vectors::{KeptView, ViewSources};
use crate::effects::work_store::MarksWork;

/// The clip screen, the glow and the corner blend's work.
pub struct Clipping {
    pub clip: ClipScreen,
    pub glow: GlowState,
    pub blend_work: BlendWork,
    /// Set while a part of a split screen of three or four players draws its effects: the quads
    /// are cut to the part's rectangle on all four sides.
    pub cut_to_part: std::sync::atomic::AtomicBool,
}

impl Clipping {
    pub const fn new() -> Self {
        Clipping {
            clip: ClipScreen::new(),
            glow: GlowState::new(),
            blend_work: BlendWork::new(),
            cut_to_part: std::sync::atomic::AtomicBool::new(false),
        }
    }
}

/// The marks' places, links, corners and slopes blocks, and the mark slots with their tuning.
pub struct Blocks {
    pub places: PlacesBlock,
    pub links: MarkLinks,
    pub corner_slopes: CornerSlopes,
    pub corners: MarkCorners,
    pub slots: MarkSlots,
    pub slot_tuning: SlotTuning,
}

impl Blocks {
    pub const fn new() -> Self {
        Blocks {
            places: PlacesBlock::new(),
            links: MarkLinks::new(),
            corner_slopes: CornerSlopes::new(),
            corners: MarkCorners::new(),
            slots: MarkSlots::new(),
            slot_tuning: SlotTuning::new(),
        }
    }
}

/// The draw frame's work, the views, the flare and the kept view vectors.
pub struct FrameState {
    pub draw_work: DrawWork,
    pub view_vars: ViewVars,
    pub draw_frame: DrawFrame,
    pub flare_parts: FlareParts,
    pub flare_scales: FlareScales,
    pub view_sources: ViewSources,
    pub kept_view: KeptView,
}

impl FrameState {
    pub const fn new() -> Self {
        FrameState {
            draw_work: DrawWork::new(),
            view_vars: ViewVars::new(),
            draw_frame: DrawFrame::new(),
            flare_parts: FlareParts::new(),
            flare_scales: FlareScales::new(),
            view_sources: ViewSources::new(),
            kept_view: KeptView::new(),
        }
    }
}

/// The kinds' values and quads, the emitters and the particles' colours and tuning. The three quads
/// are the three kind sets: the overlays' (`effects::draw_overlays`, `kind_values::first`), the
/// car's ground shadow's (`effects::draw_shadow`) and the plane's (`effects::draw_plane`);
/// `overlay_extras` / `shadow_extra` hold the lift of the first two along the up vector.
pub struct Kinds {
    pub marks_kind: MarksKind,
    pub overlay_quad: QuadWork,
    pub shadow_quad: QuadWork,
    pub plane_quad: QuadWork,
    pub overlay_extras: KindExtras1,
    pub shadow_extra: KindExtra2,
    pub emitter: EmitterVars,
    pub burst_colours: BurstColours,
    pub particle_tuning: ParticleTuning,
}

impl Kinds {
    pub const fn new() -> Self {
        Kinds {
            marks_kind: MarksKind::new(),
            overlay_quad: QuadWork::new_1(),
            shadow_quad: QuadWork::new_2(),
            plane_quad: QuadWork::new_3(),
            overlay_extras: KindExtras1::new(),
            shadow_extra: KindExtra2::new(),
            emitter: EmitterVars::new(),
            burst_colours: BurstColours::new(),
            particle_tuning: ParticleTuning::new(),
        }
    }
}

/// The marks' clock, settings pointer, tuning floats, mask constants and work piece.
pub struct Setup {
    pub clock: MarksClock,
    pub clock_k: ClockK,
    pub settings: SettingsPointer,
    /// The race's settings record `init_marks` was handed, what `Effects::settings` gives.
    pub marks:
        core::cell::Cell<Option<&'static crate::race::effects_settings_store::EffectsSettings>>,
    /// The race's car blocks, handed with `marks`: the cars' records name their blocks by
    /// handle (`CarEffects::whole`).
    pub car_blocks: core::cell::Cell<Option<&'static crate::race::blocks_store::CarBlocks>>,
    /// The race's stage tables, handed with `marks`: the objects' faces and points, the course
    /// info (`Effects::stage`).
    pub stage: core::cell::Cell<Option<&'static crate::race::stage_data::StageData>>,
    pub frame: MarksFrame,
    pub floats: MarksFloats,
    pub kind_floats: MarksKindFloats,
    pub mask_constants: MaskConstants,
    pub work: MarksWork,
}

impl Setup {
    pub const fn new() -> Self {
        Setup {
            clock: MarksClock::new(),
            clock_k: ClockK::new(),
            settings: SettingsPointer::new(),
            marks: core::cell::Cell::new(None),
            car_blocks: core::cell::Cell::new(None),
            stage: core::cell::Cell::new(None),
            frame: MarksFrame::new(),
            floats: MarksFloats::new(),
            kind_floats: MarksKindFloats::new(),
            mask_constants: MaskConstants::new(),
            work: MarksWork::new(),
        }
    }
}
