//! The windscreen drops' variables: the item table, the view's moves kept and the drift, the shape
//! constants.

/// The windscreen drops' variables.
pub struct WindscreenDropVars {
    /// Four floats `set_up_windscreen_drops` scales once
    pub scale: [f32; 4],
    /// The item's place as floats (`draw_plane`, `windscreen_drops_top`, `windscreen_flakes_frame`)
    pub place: [f32; 3],
    /// The drift has run once (`windscreen_drops_drift`)
    pub started: u32,
    /// A move has been kept (`windscreen_drops_drift`)
    pub moved: u32,
    /// Cleared with the two above by `set_up_windscreen_drops`
    pub spare: [u32; 2],
    /// The view's place kept
    pub kept_place: [f32; 3],
    /// The view's move kept
    pub kept_move: [f32; 3],
    /// The view's move (its place less the kept place, times a constant)
    pub view_move: [f32; 3],
    /// The move less the last, plus the last times a constant
    pub move_change: [f32; 3],
    /// The drift turned back by the view's turn, negated
    pub drift: [f32; 3],
    /// The ease by the ticks (`windscreen_drops_top`)
    pub ease: f32,
    /// The steps since the last look (`windscreen_drops_step`)
    pub steps: u32,
    /// Where the next sprite goes: its byte offset in the batch's records
    /// (`windscreen_drops_frame`)
    pub buffer: u32,
    /// 0 at each `windscreen_flakes_step`
    pub windscreen_flakes_cleared: u32,
    /// 0x800 items of 0x30 bytes (+0 1 live)
    pub items: [u8; 98304],
    /// Set once `set_up_windscreen_drops` has scaled its floats.
    pub done: u32,
    /// The factors the four scale floats are scaled by (1, 1, 2, 2)
    pub factors: [f32; 6],
    /// The quarter-circle curve's first constant
    pub curve_a: f32,
    /// Its second
    pub curve_b: f32,
    /// The circles' step count
    pub step_c: i32,
    /// The circles' step
    pub step_d: f32,
    /// Constants of `windscreen_drops_top` (the last is the length it divides by)
    pub top_k: [f32; 4],
    /// A count per vertex (`windscreen_drops_top`)
    pub per_vertex: i32,
    /// What `windscreen_drops_setup` scales the two axes by
    pub axes_k: [f32; 6],
    /// The texture square's side
    pub side: f32,
    /// A constant of `windscreen_drops_frame`
    pub frame_k: f32,
    /// Constants of `windscreen_drops_spawn`
    pub spawn_k: [f32; 2],
    /// The share constant of `windscreen_drops_spawn`
    pub share_k: f32,
    /// The windscreen drops' random (times 0xC5 plus 0x5D)
    pub random: u32,
    /// The view's turn copied by columns (3 x 3)
    pub turn: [f32; 9],
    /// Its adjugate (3 x 3)
    pub adjugate: [f32; 9],
    /// What the last move is multiplied by
    pub decay: [f32; 3],
    /// The turn about x
    pub angle: f32,
    /// What the drift is multiplied by into the result
    pub result_k: [f32; 3],
}

impl WindscreenDropVars {
    pub const fn new() -> Self {
        WindscreenDropVars {
            scale: [0.000625, 0.00083333335, 0.000625, 0.00083333335],
            place: [0.0; 3],
            started: 0x0,
            moved: 0x0,
            spare: [0x0; 2],
            kept_place: [0.0; 3],
            kept_move: [0.0; 3],
            view_move: [0.0; 3],
            move_change: [0.0; 3],
            drift: [0.0; 3],
            ease: 0.0,
            steps: 0x0,
            buffer: 0x0,
            windscreen_flakes_cleared: 0x0,
            items: [0; 98304],
            done: 0,
            factors: [2.0, 2.0, -2.0, 16.0, -2.0, 16.0],
            curve_a: 1024.0,
            curve_b: 0.00054414064,
            step_c: 0x6,
            step_d: 2.0,
            top_k: [0.003, 0.7, 2048.0, 1024.0],
            per_vertex: 0xC0,
            axes_k: [0.5; 6],
            side: 0.5,
            frame_k: 1024.0,
            spawn_k: [0.1, 0.0],
            share_k: 65536.0,
            random: 0x0,
            turn: [0.0; 9],
            adjugate: [0.0; 9],
            decay: [0.02; 3],
            angle: -45.0,
            result_k: [0.02; 3],
        }
    }
}

impl WindscreenDropVars {
    /// The dword at byte `at` of the items.
    pub fn item(&self, at: usize) -> u32 {
        u32::from_le_bytes([
            self.items[at],
            self.items[at + 1],
            self.items[at + 2],
            self.items[at + 3],
        ])
    }

    /// `value` into the dword at byte `at` of the items.
    pub fn set_item(&mut self, at: usize, value: u32) {
        self.items[at..at + 4].copy_from_slice(&value.to_le_bytes());
    }

    /// The item dword at `at`, loaded as the x87 loads a float.
    pub(crate) fn ld_item(&self, at: usize) -> crate::math::x87_float::X {
        crate::math::x87_float::ld(self.item(at))
    }

    /// Stores into the item dword at `at` as the x87 stores a float.
    pub(crate) fn st_item(&mut self, at: usize, x: crate::math::x87_float::X) {
        self.set_item(at, crate::math::x87_float::st(x));
    }
}

/// The windscreen drops' state and the shape stores they and the other marks share, one
/// part of the effects' owner (`effects::owner::Effects`).
pub struct WindscreenDrops {
    pub hex: std::cell::RefCell<WindscreenDropVars>,
    pub state: std::cell::RefCell<crate::effects::windscreen_drop_state_store::WindscreenDropState>,
    pub work: std::cell::RefCell<crate::effects::windscreen_drop_state_store::WindscreenDropWork>,
    pub shape_work: crate::effects::shape_work_store::ShapeWork,
    pub tilt: std::cell::RefCell<crate::effects::tilt_shape_store::ShapeTilt>,
    pub inputs: crate::effects::shape_points_store::ShapeInputs,
    pub tuning: crate::effects::shape_points_store::ShapeSetTuning,
    pub picks: crate::effects::mask_picks_store::MaskPicks,
}

impl WindscreenDrops {
    pub const fn new() -> Self {
        WindscreenDrops {
            hex: std::cell::RefCell::new(WindscreenDropVars::new()),
            state: std::cell::RefCell::new(
                crate::effects::windscreen_drop_state_store::WindscreenDropState::new(),
            ),
            work: std::cell::RefCell::new(
                crate::effects::windscreen_drop_state_store::WindscreenDropWork::new(),
            ),
            shape_work: crate::effects::shape_work_store::ShapeWork::new(),
            tilt: std::cell::RefCell::new(crate::effects::tilt_shape_store::ShapeTilt::new()),
            inputs: crate::effects::shape_points_store::ShapeInputs::new(),
            tuning: crate::effects::shape_points_store::ShapeSetTuning::new(),
            picks: crate::effects::mask_picks_store::MaskPicks::new(),
        }
    }
}
