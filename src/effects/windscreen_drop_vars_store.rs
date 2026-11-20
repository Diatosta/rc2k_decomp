//! The windscreen drops' variables: the item table, the view's moves kept and the drift, the shape
//! constants.

use crate::memory::session::{PlainBytes, PlainDword, PlainFlag, PlainFloat, PlainInt};

/// The windscreen drops' variables.
#[derive(rc2k_derive::Cells)]
pub struct WindscreenDropVars {
    /// Four floats `set_up_windscreen_drops` scales once
    pub scale: [PlainFloat; 4],
    /// The item's place as floats (`draw_plane`, `windscreen_drops_top`, `windscreen_flakes_frame`)
    pub place: [PlainFloat; 3],
    /// The drift has run once (`windscreen_drops_drift`)
    pub started: PlainDword,
    /// A move has been kept (`windscreen_drops_drift`)
    pub moved: PlainDword,
    /// Cleared with the two above by `set_up_windscreen_drops`
    pub spare: [PlainDword; 2],
    /// The view's place kept
    pub kept_place: [PlainFloat; 3],
    /// The view's move kept
    pub kept_move: [PlainFloat; 3],
    /// The view's move (its place less the kept place, times a constant)
    pub view_move: [PlainFloat; 3],
    /// The move less the last, plus the last times a constant
    pub move_change: [PlainFloat; 3],
    /// The drift turned back by the view's turn, negated
    pub drift: [PlainFloat; 3],
    /// The ease by the ticks (`windscreen_drops_top`)
    pub ease: PlainFloat,
    /// The steps since the last look (`windscreen_drops_step`)
    pub steps: PlainDword,
    /// Where the next sprite goes: its byte offset in the batch's records
    /// (`windscreen_drops_frame`)
    pub buffer: PlainDword,
    /// 0 at each `windscreen_flakes_step`
    pub windscreen_flakes_cleared: PlainDword,
    /// 0x800 items of 0x30 bytes (+0 1 live)
    pub items: PlainBytes<98304>,
    /// Set once `set_up_windscreen_drops` has scaled its floats.
    pub done: PlainFlag,
    /// The factors the four scale floats are scaled by (1, 1, 2, 2)
    pub factors: [PlainFloat; 6],
    /// The quarter-circle curve's first constant
    pub curve_a: PlainFloat,
    /// Its second
    pub curve_b: PlainFloat,
    /// The circles' step count
    pub step_c: PlainInt,
    /// The circles' step
    pub step_d: PlainFloat,
    /// Constants of `windscreen_drops_top` (the last is the length it divides by)
    pub top_k: [PlainFloat; 4],
    /// A count per vertex (`windscreen_drops_top`)
    pub per_vertex: PlainInt,
    /// What `windscreen_drops_setup` scales the two axes by
    pub axes_k: [PlainFloat; 6],
    /// The texture square's side
    pub side: PlainFloat,
    /// A constant of `windscreen_drops_frame`
    pub frame_k: PlainFloat,
    /// Constants of `windscreen_drops_spawn`
    pub spawn_k: [PlainFloat; 2],
    /// The share constant of `windscreen_drops_spawn`
    pub share_k: PlainFloat,
    /// The windscreen drops' random (times 0xC5 plus 0x5D)
    pub random: PlainDword,
    /// The view's turn copied by columns (3 x 3)
    pub turn: [PlainFloat; 9],
    /// Its adjugate (3 x 3)
    pub adjugate: [PlainFloat; 9],
    /// What the last move is multiplied by
    pub decay: [PlainFloat; 3],
    /// The turn about x
    pub angle: PlainFloat,
    /// What the drift is multiplied by into the result
    pub result_k: [PlainFloat; 3],
}

impl WindscreenDropVars {
    pub const fn new() -> Self {
        WindscreenDropVars {
            scale: [
                PlainFloat::new(0.000625),
                PlainFloat::new(0.00083333335),
                PlainFloat::new(0.000625),
                PlainFloat::new(0.00083333335),
            ],
            place: [const { PlainFloat::new(0.0) }; 3],
            started: PlainDword::new(0x0),
            moved: PlainDword::new(0x0),
            spare: [const { PlainDword::new(0x0) }; 2],
            kept_place: [const { PlainFloat::new(0.0) }; 3],
            kept_move: [const { PlainFloat::new(0.0) }; 3],
            view_move: [const { PlainFloat::new(0.0) }; 3],
            move_change: [const { PlainFloat::new(0.0) }; 3],
            drift: [const { PlainFloat::new(0.0) }; 3],
            ease: PlainFloat::new(0.0),
            steps: PlainDword::new(0x0),
            buffer: PlainDword::new(0x0),
            windscreen_flakes_cleared: PlainDword::new(0x0),
            items: PlainBytes::new([0; 98304]),
            done: PlainFlag::new(false),
            factors: [
                PlainFloat::new(2.0),
                PlainFloat::new(2.0),
                PlainFloat::new(-2.0),
                PlainFloat::new(16.0),
                PlainFloat::new(-2.0),
                PlainFloat::new(16.0),
            ],
            curve_a: PlainFloat::new(1024.0),
            curve_b: PlainFloat::new(0.00054414064),
            step_c: PlainInt::new(0x6),
            step_d: PlainFloat::new(2.0),
            top_k: [
                PlainFloat::new(0.003),
                PlainFloat::new(0.7),
                PlainFloat::new(2048.0),
                PlainFloat::new(1024.0),
            ],
            per_vertex: PlainInt::new(0xC0),
            axes_k: [const { PlainFloat::new(0.5) }; 6],
            side: PlainFloat::new(0.5),
            frame_k: PlainFloat::new(1024.0),
            spawn_k: [PlainFloat::new(0.1), PlainFloat::new(0.0)],
            share_k: PlainFloat::new(65536.0),
            random: PlainDword::new(0x0),
            turn: [const { PlainFloat::new(0.0) }; 9],
            adjugate: [const { PlainFloat::new(0.0) }; 9],
            decay: [const { PlainFloat::new(0.02) }; 3],
            angle: PlainFloat::new(-45.0),
            result_k: [const { PlainFloat::new(0.02) }; 3],
        }
    }
}

/// The windscreen drops' state and the shape stores they and the other marks share, one
/// part of the effects' owner (`effects::owner::Effects`).
pub struct WindscreenDrops {
    pub hex: WindscreenDropVars,
    pub state: crate::effects::windscreen_drop_state_store::WindscreenDropState,
    pub work: crate::effects::windscreen_drop_state_store::WindscreenDropWork,
    pub shape_work: crate::effects::shape_work_store::ShapeWork,
    pub tilt: std::cell::RefCell<crate::effects::tilt_shape_store::ShapeTilt>,
    pub inputs: crate::effects::shape_points_store::ShapeInputs,
    pub tuning: crate::effects::shape_points_store::ShapeSetTuning,
    pub picks: crate::effects::mask_picks_store::MaskPicks,
}

impl WindscreenDrops {
    pub const fn new() -> Self {
        WindscreenDrops {
            hex: WindscreenDropVars::new(),
            state: crate::effects::windscreen_drop_state_store::WindscreenDropState::new(),
            work: crate::effects::windscreen_drop_state_store::WindscreenDropWork::new(),
            shape_work: crate::effects::shape_work_store::ShapeWork::new(),
            tilt: std::cell::RefCell::new(crate::effects::tilt_shape_store::ShapeTilt::new()),
            inputs: crate::effects::shape_points_store::ShapeInputs::new(),
            tuning: crate::effects::shape_points_store::ShapeSetTuning::new(),
            picks: crate::effects::mask_picks_store::MaskPicks::new(),
        }
    }
}
