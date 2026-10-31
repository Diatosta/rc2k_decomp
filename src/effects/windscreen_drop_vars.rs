//! The windscreen drops' variables out of the marks' block (phase 7): the item table, the view's moves kept and the drift, the shape constants.

use crate::session::{Bytes, Dword, Float};

/// The windscreen drops' variables (IDA 0x816192 .. 0x82E342, the image's layout; phase 7, typed state).
#[repr(C)]
pub struct WindscreenDropVars {
    /// 0x816192: `flt_816192` .. `flt_81619E`: four floats `set_up_windscreen_drops` scales once
    pub scale: [Float; 4],
    /// 0x8161A2: `flt_8161A2` ..: the item's place as floats (`draw_plane`, `windscreen_drops_top`, `windscreen_flakes_frame`)
    pub place: [Float; 3],
    /// 0x8161AE: `dword_8161AE`: the drift has run once (`windscreen_drops_drift`)
    pub started: Dword,
    /// 0x8161B2: `dword_8161B2`: a move has been kept (`windscreen_drops_drift`)
    pub moved: Dword,
    /// 0x8161B6: `dword_8161B6`, `dword_8161BA`: cleared with the two above by `set_up_windscreen_drops`
    pub spare: [Dword; 2],
    /// 0x8161BE: Not named: 24 bytes (a gap, or unreferenced data).
    pub _gap_8161be: Bytes<24>,
    /// 0x8161D6: `flt_8161D6` ..: the view's place kept
    pub kept_place: [Float; 3],
    /// 0x8161E2: `flt_8161E2` ..: the view's move kept
    pub kept_move: [Float; 3],
    /// 0x8161EE: `flt_8161EE` ..: the view's move (its place less the kept place, times `flt_764FE0`)
    pub view_move: [Float; 3],
    /// 0x8161FA: `flt_8161FA` ..: the move less the last, plus the last times `flt_82E326` ..
    pub move_change: [Float; 3],
    /// 0x816206: Not named: 24 bytes (a gap, or unreferenced data).
    pub _gap_816206: Bytes<24>,
    /// 0x81621E: `flt_81621E` ..: the drift turned back by the view's turn, negated
    pub drift: [Float; 3],
    /// 0x81622A: `flt_81622A`: the ease by the ticks (`windscreen_drops_top`)
    pub ease: Float,
    /// 0x81622E: `dword_81622E`: the steps since the last look (`windscreen_drops_step`)
    pub steps: Dword,
    /// 0x816232: `dword_816232`: where the next sprite goes (`windscreen_drops_frame`)
    pub buffer: Dword,
    /// 0x816236: Not named: 4 bytes (a gap, or unreferenced data).
    pub _gap_816236: Bytes<4>,
    /// 0x81623A: `dword_81623A`: 0 at each `windscreen_flakes_step`
    pub windscreen_flakes_cleared: Dword,
    /// 0x81623E: `g_marksWindscreenDropItems`: 0x800 items of 0x30 bytes (+0 1 live)
    pub items: Bytes<98304>,
    /// 0x82E23E: `g_marksWindscreenDropsDone`: the once-only mark of `set_up_windscreen_drops`
    pub done: Dword,
    /// 0x82E242: `flt_82E242` .. `flt_82E256`: the factors the four scale floats are scaled by (1, 1, 2, 2)
    pub factors: [Float; 6],
    /// 0x82E25A: `flt_82E25A`: the quarter-circle curve's first constant
    pub curve_a: Float,
    /// 0x82E25E: `flt_82E25E`: its second
    pub curve_b: Float,
    /// 0x82E262: `dword_82E262`: the circles' step count
    pub step_c: Dword,
    /// 0x82E266: `flt_82E266`: the circles' step
    pub step_d: Float,
    /// 0x82E26A: `flt_82E26A` .. `flt_82E276`: constants of `windscreen_drops_top` (the last is the length it divides by)
    pub top_k: [Float; 4],
    /// 0x82E27A: `dword_82E27A`: a count per vertex (`windscreen_drops_top`)
    pub per_vertex: Dword,
    /// 0x82E27E: Not named: 8 bytes (a gap, or unreferenced data).
    pub _gap_82e27e: Bytes<8>,
    /// 0x82E286: `flt_82E286` ..: what `windscreen_drops_setup` scales the two axes by
    pub axes_k: [Float; 6],
    /// 0x82E29E: Not named: 4 bytes (a gap, or unreferenced data).
    pub _gap_82e29e: Bytes<4>,
    /// 0x82E2A2: `dword_82E2A2`: the texture square's side
    pub side: Dword,
    /// 0x82E2A6: `flt_82E2A6`: a constant of `windscreen_drops_frame`
    pub frame_k: Float,
    /// 0x82E2AA: `flt_82E2AA`, `flt_82E2AE`: constants of `windscreen_drops_spawn`
    pub spawn_k: [Float; 2],
    /// 0x82E2B2: `flt_82E2B2`: the share constant of `windscreen_drops_spawn`
    pub share_k: Float,
    /// 0x82E2B6: `dword_82E2B6`: the windscreen drops' random (times 0xC5 plus 0x5D)
    pub random: Dword,
    /// 0x82E2BA: Not named: 12 bytes (a gap, or unreferenced data).
    pub _gap_82e2ba: Bytes<12>,
    /// 0x82E2C6: `dword_82E2C6` ..: the view's turn copied by columns (3 x 3)
    pub turn: [Float; 9],
    /// 0x82E2EA: Not named: 12 bytes (a gap, or unreferenced data).
    pub _gap_82e2ea: Bytes<12>,
    /// 0x82E2F6: `unk_82E2F6`: its adjugate (3 x 3)
    pub adjugate: [Float; 9],
    /// 0x82E31A: Not named: 12 bytes (a gap, or unreferenced data).
    pub _gap_82e31a: Bytes<12>,
    /// 0x82E326: `flt_82E326` ..: what the last move is multiplied by
    pub decay: [Float; 3],
    /// 0x82E332: `flt_82E332`: the turn about x
    pub angle: Float,
    /// 0x82E336: `flt_82E336` ..: what the drift is multiplied by into `flt_811BDF` ..
    pub result_k: [Float; 3],
}

pub static HEX: WindscreenDropVars = WindscreenDropVars {
    scale: [
        Float::from_bits(0x3A23D70A),
        Float::from_bits(0x3A5A740E),
        Float::from_bits(0x3A23D70A),
        Float::from_bits(0x3A5A740E),
    ],
    place: [const { Float::from_bits(0x0) }; 3],
    started: Dword::new(0x0),
    moved: Dword::new(0x0),
    spare: [const { Dword::new(0x0) }; 2],
    _gap_8161be: Bytes::new([0; 24]),
    kept_place: [const { Float::from_bits(0x0) }; 3],
    kept_move: [const { Float::from_bits(0x0) }; 3],
    view_move: [const { Float::from_bits(0x0) }; 3],
    move_change: [const { Float::from_bits(0x0) }; 3],
    _gap_816206: Bytes::new([0; 24]),
    drift: [const { Float::from_bits(0x0) }; 3],
    ease: Float::from_bits(0x0),
    steps: Dword::new(0x0),
    buffer: Dword::new(0x0),
    _gap_816236: Bytes::new([0; 4]),
    windscreen_flakes_cleared: Dword::new(0x0),
    items: Bytes::new([0; 98304]),
    done: Dword::new(0x0),
    factors: [
        Float::from_bits(0x40000000),
        Float::from_bits(0x40000000),
        Float::from_bits(0xC0000000),
        Float::from_bits(0x41800000),
        Float::from_bits(0xC0000000),
        Float::from_bits(0x41800000),
    ],
    curve_a: Float::from_bits(0x44800000),
    curve_b: Float::from_bits(0x3A0EA4A9),
    step_c: Dword::new(0x6),
    step_d: Float::from_bits(0x40000000),
    top_k: [
        Float::from_bits(0x3B449BA6),
        Float::from_bits(0x3F333333),
        Float::from_bits(0x45000000),
        Float::from_bits(0x44800000),
    ],
    per_vertex: Dword::new(0xC0),
    _gap_82e27e: Bytes::new([0x00, 0x00, 0x80, 0x46, 0x00, 0x00, 0x80, 0x46]),
    axes_k: [const { Float::from_bits(0x3F000000) }; 6],
    _gap_82e29e: Bytes::new([0x00, 0x00, 0x80, 0x43]),
    side: Dword::new(0x3F000000),
    frame_k: Float::from_bits(0x44800000),
    spawn_k: [Float::from_bits(0x3DCCCCCD), Float::from_bits(0x0)],
    share_k: Float::from_bits(0x47800000),
    random: Dword::new(0x0),
    _gap_82e2ba: Bytes::new([0; 12]),
    turn: [const { Float::from_bits(0x0) }; 9],
    _gap_82e2ea: Bytes::new([0; 12]),
    adjugate: [const { Float::from_bits(0x0) }; 9],
    _gap_82e31a: Bytes::new([0; 12]),
    decay: [const { Float::from_bits(0x3CA3D70A) }; 3],
    angle: Float::from_bits(0xC2340000),
    result_k: [const { Float::from_bits(0x3CA3D70A) }; 3],
};

impl WindscreenDropVars {
    pub const START: usize = 0x816192;
    pub const LEN: usize = 0x181B0;
    /// Where the byte IDA calls `START` is now (`moved::MOVED`'s home).
    pub fn home() -> usize {
        (crate::pointers::game_address(core::ptr::addr_of!(HEX))) as usize
    }

    /// The base the x87 blocks used for the marks' block (`store::base()`), moved so that
    /// `base + (IDA address - 0x764F40)` lands on this static's variables.
    pub fn base() -> usize {
        Self::home().wrapping_sub(Self::START - crate::effects::store::START)
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
        let p = core::ptr::addr_of!(HEX).cast::<u8>();
        let now: Vec<u8> = (0..WindscreenDropVars::LEN)
            .map(|i| unsafe { p.add(i).read() })
            .collect();
        assert!(
            now == crate::moved::image_bytes(WindscreenDropVars::START, WindscreenDropVars::LEN),
            "HEX starts differently from the image"
        );
    }
}
