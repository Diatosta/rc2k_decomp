//! The effects' remaining stores in groups (owners, stage 1): what was a static of its
//! own each is a field of one group of `effects::owner::Effects`.

use crate::effects::blocks_store::{CornerSlopes, MarkCorners, MarkLinks, PlacesBlock};
use crate::effects::clip_store::{ClipScreen, GlowState};
use crate::effects::emitters_store::EmitterVars;
use crate::effects::frame_state_store::{DrawFrame, DrawWork, FlareParts, FlareScales, ViewVars};
use crate::effects::kind_store::{KindExtra2, KindExtras1, QuadWork};
use crate::effects::mark_slots_store::{MarkSlots, SlotTuning};
use crate::effects::mask_constants_store::MaskConstants;
use crate::effects::particles_store::{BurstColours, ParticleTuning};
use crate::effects::setup_store::{MarksClock, SettingsPointer};
use crate::effects::tuning_store::{MarksFloats, MarksFrame, MarksKindFloats};
use crate::effects::view_vectors_store::{KeptView, ViewSources};
use crate::effects::work_store::MarksWork;

/// The clip screen, the glow and the corner blend's work.
pub struct Clipping {
    pub clip: core::cell::RefCell<ClipScreen>,
    pub glow: core::cell::RefCell<GlowState>,
    /// Set while a part of a split screen of three or four players draws its effects: the quads
    /// are cut to the part's rectangle on all four sides.
    pub cut_to_part: std::sync::atomic::AtomicBool,
}

impl Clipping {
    pub const fn new() -> Self {
        Clipping {
            clip: core::cell::RefCell::new(ClipScreen::new()),
            glow: core::cell::RefCell::new(GlowState::new()),
            cut_to_part: std::sync::atomic::AtomicBool::new(false),
        }
    }
}

/// The marks' places, links, corners and slopes blocks, and the mark slots with their tuning.
pub struct Blocks {
    pub places: PlacesBlock,
    pub links: MarkLinks,
    pub corner_slopes: std::cell::RefCell<CornerSlopes>,
    pub corners: MarkCorners,
    pub slots: core::cell::RefCell<MarkSlots>,
    pub slot_tuning: core::cell::RefCell<SlotTuning>,
}

impl Blocks {
    pub const fn new() -> Self {
        Blocks {
            places: PlacesBlock::new(),
            links: MarkLinks::new(),
            corner_slopes: std::cell::RefCell::new(CornerSlopes::new()),
            corners: MarkCorners::new(),
            slots: core::cell::RefCell::new(MarkSlots::new()),
            slot_tuning: core::cell::RefCell::new(SlotTuning::new()),
        }
    }
}

/// The draw frame's work, the views, the flare and the kept view vectors.
pub struct FrameState {
    pub draw_work: core::cell::RefCell<DrawWork>,
    pub view_vars: core::cell::RefCell<ViewVars>,
    pub draw_frame: core::cell::RefCell<DrawFrame>,
    pub flare_parts: core::cell::RefCell<FlareParts>,
    pub flare_scales: core::cell::RefCell<FlareScales>,
    pub view_sources: core::cell::RefCell<ViewSources>,
    pub kept_view: core::cell::RefCell<KeptView>,
}

impl FrameState {
    pub const fn new() -> Self {
        FrameState {
            draw_work: core::cell::RefCell::new(DrawWork::new()),
            view_vars: core::cell::RefCell::new(ViewVars::new()),
            draw_frame: core::cell::RefCell::new(DrawFrame::new()),
            flare_parts: core::cell::RefCell::new(FlareParts::new()),
            flare_scales: core::cell::RefCell::new(FlareScales::new()),
            view_sources: core::cell::RefCell::new(ViewSources::new()),
            kept_view: core::cell::RefCell::new(KeptView::new()),
        }
    }
}

/// The kinds' values and quads, the emitters and the particles' colours and tuning. The three quads
/// are the three kind sets: the overlays' (`effects::draw_overlays`, `kind_values::first`), the
/// car's ground shadow's (`effects::draw_shadow`) and the plane's (`effects::draw_plane`);
/// `overlay_extras` / `shadow_extra` hold the lift of the first two along the up vector.
pub struct Kinds {
    /// The kind of the marks (the surface / weather: 0 .. 0x21), set at `effects::init_marks`.
    pub marks_kind: core::cell::Cell<u32>,
    pub overlay_quad: core::cell::RefCell<QuadWork>,
    pub shadow_quad: core::cell::RefCell<QuadWork>,
    pub plane_quad: core::cell::RefCell<QuadWork>,
    pub overlay_extras: KindExtras1,
    pub shadow_extra: KindExtra2,
    pub emitter: core::cell::RefCell<EmitterVars>,
    pub burst_colours: BurstColours,
    pub particle_tuning: ParticleTuning,
}

impl Kinds {
    pub const fn new() -> Self {
        Kinds {
            marks_kind: core::cell::Cell::new(0),
            overlay_quad: core::cell::RefCell::new(QuadWork::new_1()),
            shadow_quad: core::cell::RefCell::new(QuadWork::new_2()),
            plane_quad: core::cell::RefCell::new(QuadWork::new_3()),
            overlay_extras: KindExtras1::new(),
            shadow_extra: KindExtra2::new(),
            emitter: core::cell::RefCell::new(EmitterVars::new()),
            burst_colours: BurstColours::new(),
            particle_tuning: ParticleTuning::new(),
        }
    }
}

/// The marks' clock, settings pointer, tuning floats, mask constants and work piece.
pub struct Setup {
    pub clock: core::cell::RefCell<MarksClock>,
    pub settings: SettingsPointer,
    /// The race's settings record `init_marks` was handed, what `Effects::settings` gives.
    pub marks:
        core::cell::Cell<Option<&'static crate::race::effects_settings_store::EffectsSettings>>,
    /// The race's car blocks, handed with `marks`: the cars' records name their blocks by
    /// handle (`CarEffects::whole`).
    pub car_blocks: core::cell::Cell<
        Option<&'static crate::memory::session::DrawArc<crate::race::blocks_store::CarBlocks>>,
    >,
    /// The cars' drawing state the race handed `init_marks` (the matrices the effects follow).
    pub car_parts: core::cell::Cell<
        Option<&'static crate::memory::session::DrawArc<crate::race::car_parts_store::CarParts>>,
    >,
    /// The race's stage tables, handed with `marks`: the objects' faces and points, the course
    /// info (`Effects::stage`).
    pub stage: core::cell::Cell<Option<&'static crate::race::stage_data_store::StageData>>,
    pub frame: MarksFrame,
    pub floats: MarksFloats,
    pub kind_floats: std::cell::RefCell<MarksKindFloats>,
    pub mask_constants: core::cell::RefCell<MaskConstants>,
    pub work: core::cell::RefCell<MarksWork>,
}

impl Setup {
    pub const fn new() -> Self {
        Setup {
            clock: core::cell::RefCell::new(MarksClock::new()),
            settings: SettingsPointer::new(),
            marks: core::cell::Cell::new(None),
            car_blocks: core::cell::Cell::new(None),
            car_parts: core::cell::Cell::new(None),
            stage: core::cell::Cell::new(None),
            frame: MarksFrame::new(),
            floats: MarksFloats::new(),
            kind_floats: std::cell::RefCell::new(MarksKindFloats::new()),
            mask_constants: core::cell::RefCell::new(MaskConstants::new()),
            work: core::cell::RefCell::new(MarksWork::new()),
        }
    }
}
