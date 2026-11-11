//! The windscreen drops' variables: the item table, the view's moves kept and the drift, the shape constants.

use crate::session::{PlainBytes, PlainDword, PlainFloat};

/// The windscreen drops' variables (IDA 0x816192 .. 0x82E342).
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct WindscreenDropVars {
    /// 0x816192: `flt_816192` .. `flt_81619E`: four floats `set_up_windscreen_drops` scales once
    pub scale: [PlainFloat; 4],
    /// 0x8161A2: `flt_8161A2` ..: the item's place as floats (`draw_plane`, `windscreen_drops_top`, `windscreen_flakes_frame`)
    pub place: [PlainFloat; 3],
    /// 0x8161AE: `dword_8161AE`: the drift has run once (`windscreen_drops_drift`)
    pub started: PlainDword,
    /// 0x8161B2: `dword_8161B2`: a move has been kept (`windscreen_drops_drift`)
    pub moved: PlainDword,
    /// 0x8161B6: `dword_8161B6`, `dword_8161BA`: cleared with the two above by `set_up_windscreen_drops`
    pub spare: [PlainDword; 2],
    /// 0x8161BE: Not named: 24 bytes (a gap, or unreferenced data).
    pub _gap_8161be: PlainBytes<24>,
    /// 0x8161D6: `flt_8161D6` ..: the view's place kept
    pub kept_place: [PlainFloat; 3],
    /// 0x8161E2: `flt_8161E2` ..: the view's move kept
    pub kept_move: [PlainFloat; 3],
    /// 0x8161EE: `flt_8161EE` ..: the view's move (its place less the kept place, times `flt_764FE0`)
    pub view_move: [PlainFloat; 3],
    /// 0x8161FA: `flt_8161FA` ..: the move less the last, plus the last times `flt_82E326` ..
    pub move_change: [PlainFloat; 3],
    /// 0x816206: Not named: 24 bytes (a gap, or unreferenced data).
    pub _gap_816206: PlainBytes<24>,
    /// 0x81621E: `flt_81621E` ..: the drift turned back by the view's turn, negated
    pub drift: [PlainFloat; 3],
    /// 0x81622A: `flt_81622A`: the ease by the ticks (`windscreen_drops_top`)
    pub ease: PlainFloat,
    /// 0x81622E: `dword_81622E`: the steps since the last look (`windscreen_drops_step`)
    pub steps: PlainDword,
    /// 0x816232: `dword_816232`: where the next sprite goes: its byte offset in the batch's records (the original kept the address; `windscreen_drops_frame`)
    pub buffer: PlainDword,
    /// 0x816236: Not named: 4 bytes (a gap, or unreferenced data).
    pub _gap_816236: PlainBytes<4>,
    /// 0x81623A: `dword_81623A`: 0 at each `windscreen_flakes_step`
    pub windscreen_flakes_cleared: PlainDword,
    /// 0x81623E: `g_marksWindscreenDropItems`: 0x800 items of 0x30 bytes (+0 1 live)
    pub items: PlainBytes<98304>,
    /// 0x82E23E: `g_marksWindscreenDropsDone`: the once-only mark of `set_up_windscreen_drops`
    pub done: PlainDword,
    /// 0x82E242: `flt_82E242` .. `flt_82E256`: the factors the four scale floats are scaled by (1, 1, 2, 2)
    pub factors: [PlainFloat; 6],
    /// 0x82E25A: `flt_82E25A`: the quarter-circle curve's first constant
    pub curve_a: PlainFloat,
    /// 0x82E25E: `flt_82E25E`: its second
    pub curve_b: PlainFloat,
    /// 0x82E262: `dword_82E262`: the circles' step count
    pub step_c: PlainDword,
    /// 0x82E266: `flt_82E266`: the circles' step
    pub step_d: PlainFloat,
    /// 0x82E26A: `flt_82E26A` .. `flt_82E276`: constants of `windscreen_drops_top` (the last is the length it divides by)
    pub top_k: [PlainFloat; 4],
    /// 0x82E27A: `dword_82E27A`: a count per vertex (`windscreen_drops_top`)
    pub per_vertex: PlainDword,
    /// 0x82E27E: Not named: 8 bytes (a gap, or unreferenced data).
    pub _gap_82e27e: PlainBytes<8>,
    /// 0x82E286: `flt_82E286` ..: what `windscreen_drops_setup` scales the two axes by
    pub axes_k: [PlainFloat; 6],
    /// 0x82E29E: Not named: 4 bytes (a gap, or unreferenced data).
    pub _gap_82e29e: PlainBytes<4>,
    /// 0x82E2A2: `dword_82E2A2`: the texture square's side
    pub side: PlainDword,
    /// 0x82E2A6: `flt_82E2A6`: a constant of `windscreen_drops_frame`
    pub frame_k: PlainFloat,
    /// 0x82E2AA: `flt_82E2AA`, `flt_82E2AE`: constants of `windscreen_drops_spawn`
    pub spawn_k: [PlainFloat; 2],
    /// 0x82E2B2: `flt_82E2B2`: the share constant of `windscreen_drops_spawn`
    pub share_k: PlainFloat,
    /// 0x82E2B6: `dword_82E2B6`: the windscreen drops' random (times 0xC5 plus 0x5D)
    pub random: PlainDword,
    /// 0x82E2BA: Not named: 12 bytes (a gap, or unreferenced data).
    pub _gap_82e2ba: PlainBytes<12>,
    /// 0x82E2C6: `dword_82E2C6` ..: the view's turn copied by columns (3 x 3)
    pub turn: [PlainFloat; 9],
    /// 0x82E2EA: Not named: 12 bytes (a gap, or unreferenced data).
    pub _gap_82e2ea: PlainBytes<12>,
    /// 0x82E2F6: `unk_82E2F6`: its adjugate (3 x 3)
    pub adjugate: [PlainFloat; 9],
    /// 0x82E31A: Not named: 12 bytes (a gap, or unreferenced data).
    pub _gap_82e31a: PlainBytes<12>,
    /// 0x82E326: `flt_82E326` ..: what the last move is multiplied by
    pub decay: [PlainFloat; 3],
    /// 0x82E332: `flt_82E332`: the turn about x
    pub angle: PlainFloat,
    /// 0x82E336: `flt_82E336` ..: what the drift is multiplied by into `flt_811BDF` ..
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
            _gap_8161be: PlainBytes::new([0; 24]),
            kept_place: [const { PlainFloat::new(0.0) }; 3],
            kept_move: [const { PlainFloat::new(0.0) }; 3],
            view_move: [const { PlainFloat::new(0.0) }; 3],
            move_change: [const { PlainFloat::new(0.0) }; 3],
            _gap_816206: PlainBytes::new([0; 24]),
            drift: [const { PlainFloat::new(0.0) }; 3],
            ease: PlainFloat::new(0.0),
            steps: PlainDword::new(0x0),
            buffer: PlainDword::new(0x0),
            _gap_816236: PlainBytes::new([0; 4]),
            windscreen_flakes_cleared: PlainDword::new(0x0),
            items: PlainBytes::new([0; 98304]),
            done: PlainDword::new(0x0),
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
            step_c: PlainDword::new(0x6),
            step_d: PlainFloat::new(2.0),
            top_k: [
                PlainFloat::new(0.003),
                PlainFloat::new(0.7),
                PlainFloat::new(2048.0),
                PlainFloat::new(1024.0),
            ],
            per_vertex: PlainDword::new(0xC0),
            _gap_82e27e: PlainBytes::new([0x00, 0x00, 0x80, 0x46, 0x00, 0x00, 0x80, 0x46]),
            axes_k: [const { PlainFloat::new(0.5) }; 6],
            _gap_82e29e: PlainBytes::new([0x00, 0x00, 0x80, 0x43]),
            side: PlainDword::new(0x3F000000),
            frame_k: PlainFloat::new(1024.0),
            spawn_k: [PlainFloat::new(0.1), PlainFloat::new(0.0)],
            share_k: PlainFloat::new(65536.0),
            random: PlainDword::new(0x0),
            _gap_82e2ba: PlainBytes::new([0; 12]),
            turn: [const { PlainFloat::new(0.0) }; 9],
            _gap_82e2ea: PlainBytes::new([0; 12]),
            adjugate: [const { PlainFloat::new(0.0) }; 9],
            _gap_82e31a: PlainBytes::new([0; 12]),
            decay: [const { PlainFloat::new(0.02) }; 3],
            angle: PlainFloat::new(-45.0),
            result_k: [const { PlainFloat::new(0.02) }; 3],
        }
    }
}

impl WindscreenDropVars {
    pub const START: usize = 0x816192;
    pub const LEN: usize = 0x181B0;
}

/// The windscreen drops' state and the shape stores they and the other marks share, one
/// part of the effects' owner (`effects::owner::Effects`).
pub struct WindscreenDrops {
    pub hex: WindscreenDropVars,
    pub state: crate::effects::windscreen_drop_state::WindscreenDropState,
    pub work: crate::effects::windscreen_drop_state::WindscreenDropWork,
    pub shape_work: crate::effects::shape_work::ShapeWork,
    pub tilt: crate::effects::tilt_shape::ShapeTilt,
    pub inputs: crate::effects::shape_points::ShapeInputs,
    pub tuning: crate::effects::shape_points::ShapeSetTuning,
    pub picks: crate::effects::mask_picks::MaskPicks,
}

impl WindscreenDrops {
    pub const fn new() -> Self {
        WindscreenDrops {
            hex: WindscreenDropVars::new(),
            state: crate::effects::windscreen_drop_state::WindscreenDropState::new(),
            work: crate::effects::windscreen_drop_state::WindscreenDropWork::new(),
            shape_work: crate::effects::shape_work::ShapeWork::new(),
            tilt: crate::effects::tilt_shape::ShapeTilt::new(),
            inputs: crate::effects::shape_points::ShapeInputs::new(),
            tuning: crate::effects::shape_points::ShapeSetTuning::new(),
            picks: crate::effects::mask_picks::MaskPicks::new(),
        }
    }
}

#[cfg(test)]
mod tests_hex {
    use super::*;
    use core::mem::{offset_of, size_of};

    #[test]
    fn hex_starts_as_the_games() {
        assert_eq!(size_of::<WindscreenDropVars>(), WindscreenDropVars::LEN);
        assert_eq!(
            offset_of!(WindscreenDropVars, scale),
            0x816192 - WindscreenDropVars::START
        );
        assert_eq!(
            offset_of!(WindscreenDropVars, place),
            0x8161A2 - WindscreenDropVars::START
        );
        assert_eq!(
            offset_of!(WindscreenDropVars, started),
            0x8161AE - WindscreenDropVars::START
        );
        assert_eq!(
            offset_of!(WindscreenDropVars, moved),
            0x8161B2 - WindscreenDropVars::START
        );
        assert_eq!(
            offset_of!(WindscreenDropVars, spare),
            0x8161B6 - WindscreenDropVars::START
        );
        assert_eq!(
            offset_of!(WindscreenDropVars, kept_place),
            0x8161D6 - WindscreenDropVars::START
        );
        assert_eq!(
            offset_of!(WindscreenDropVars, kept_move),
            0x8161E2 - WindscreenDropVars::START
        );
        assert_eq!(
            offset_of!(WindscreenDropVars, view_move),
            0x8161EE - WindscreenDropVars::START
        );
        assert_eq!(
            offset_of!(WindscreenDropVars, move_change),
            0x8161FA - WindscreenDropVars::START
        );
        assert_eq!(
            offset_of!(WindscreenDropVars, drift),
            0x81621E - WindscreenDropVars::START
        );
        assert_eq!(
            offset_of!(WindscreenDropVars, ease),
            0x81622A - WindscreenDropVars::START
        );
        assert_eq!(
            offset_of!(WindscreenDropVars, steps),
            0x81622E - WindscreenDropVars::START
        );
        assert_eq!(
            offset_of!(WindscreenDropVars, buffer),
            0x816232 - WindscreenDropVars::START
        );
        assert_eq!(
            offset_of!(WindscreenDropVars, windscreen_flakes_cleared),
            0x81623A - WindscreenDropVars::START
        );
        assert_eq!(
            offset_of!(WindscreenDropVars, items),
            0x81623E - WindscreenDropVars::START
        );
        assert_eq!(
            offset_of!(WindscreenDropVars, done),
            0x82E23E - WindscreenDropVars::START
        );
        assert_eq!(
            offset_of!(WindscreenDropVars, factors),
            0x82E242 - WindscreenDropVars::START
        );
        assert_eq!(
            offset_of!(WindscreenDropVars, curve_a),
            0x82E25A - WindscreenDropVars::START
        );
        assert_eq!(
            offset_of!(WindscreenDropVars, curve_b),
            0x82E25E - WindscreenDropVars::START
        );
        assert_eq!(
            offset_of!(WindscreenDropVars, step_c),
            0x82E262 - WindscreenDropVars::START
        );
        assert_eq!(
            offset_of!(WindscreenDropVars, step_d),
            0x82E266 - WindscreenDropVars::START
        );
        assert_eq!(
            offset_of!(WindscreenDropVars, top_k),
            0x82E26A - WindscreenDropVars::START
        );
        assert_eq!(
            offset_of!(WindscreenDropVars, per_vertex),
            0x82E27A - WindscreenDropVars::START
        );
        assert_eq!(
            offset_of!(WindscreenDropVars, axes_k),
            0x82E286 - WindscreenDropVars::START
        );
        assert_eq!(
            offset_of!(WindscreenDropVars, side),
            0x82E2A2 - WindscreenDropVars::START
        );
        assert_eq!(
            offset_of!(WindscreenDropVars, frame_k),
            0x82E2A6 - WindscreenDropVars::START
        );
        assert_eq!(
            offset_of!(WindscreenDropVars, spawn_k),
            0x82E2AA - WindscreenDropVars::START
        );
        assert_eq!(
            offset_of!(WindscreenDropVars, share_k),
            0x82E2B2 - WindscreenDropVars::START
        );
        assert_eq!(
            offset_of!(WindscreenDropVars, random),
            0x82E2B6 - WindscreenDropVars::START
        );
        assert_eq!(
            offset_of!(WindscreenDropVars, turn),
            0x82E2C6 - WindscreenDropVars::START
        );
        assert_eq!(
            offset_of!(WindscreenDropVars, adjugate),
            0x82E2F6 - WindscreenDropVars::START
        );
        assert_eq!(
            offset_of!(WindscreenDropVars, decay),
            0x82E326 - WindscreenDropVars::START
        );
        assert_eq!(
            offset_of!(WindscreenDropVars, angle),
            0x82E332 - WindscreenDropVars::START
        );
        assert_eq!(
            offset_of!(WindscreenDropVars, result_k),
            0x82E336 - WindscreenDropVars::START
        );
        // The x87 tests put their buffer's bytes in HEX for a while (`typed_sync::with_overlays`).
        let _lock = crate::store::test_lock();
        let p =
            core::ptr::addr_of!(crate::effects::owner::effects().windscreen_drops.hex).cast::<u8>();
        let now: Vec<u8> = (0..WindscreenDropVars::LEN)
            .map(|i| unsafe { p.add(i).read() })
            .collect();
        assert!(
            now == crate::moved::image_bytes(WindscreenDropVars::START, WindscreenDropVars::LEN),
            "HEX starts differently from the image"
        );
    }
}
