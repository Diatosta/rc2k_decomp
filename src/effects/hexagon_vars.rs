//! The hexagons' variables out of the marks' block (phase 7): the item table, the view's moves kept and the drift, the shape constants.

use crate::session::{Bytes, Dword, Float};

/// The hexagons' variables (IDA 0x816192 .. 0x82E342, the image's layout; phase 7, typed state).
#[repr(C)]
pub struct HexagonVars {
    /// 0x816192: `flt_816192` .. `flt_81619E`: four floats `set_up_hexagons` scales once
    pub scale: [Float; 4],
    /// 0x8161A2: `flt_8161A2` ..: the item's place as floats (`draw_plane`, `hexagons_top`, `squares_frame`)
    pub place: [Float; 3],
    /// 0x8161AE: `dword_8161AE`: the drift has run once (`hexagons_drift`)
    pub started: Dword,
    /// 0x8161B2: `dword_8161B2`: a move has been kept (`hexagons_drift`)
    pub moved: Dword,
    /// 0x8161B6: `dword_8161B6`, `dword_8161BA`: cleared with the two above by `set_up_hexagons`
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
    /// 0x81622A: `flt_81622A`: the ease by the ticks (`hexagons_top`)
    pub ease: Float,
    /// 0x81622E: `dword_81622E`: the steps since the last look (`hexagons_step`)
    pub steps: Dword,
    /// 0x816232: `dword_816232`: where the next sprite goes (`hexagons_frame`)
    pub buffer: Dword,
    /// 0x816236: Not named: 4 bytes (a gap, or unreferenced data).
    pub _gap_816236: Bytes<4>,
    /// 0x81623A: `dword_81623A`: 0 at each `squares_step`
    pub squares_cleared: Dword,
    /// 0x81623E: `g_marksHexagonItems`: 0x800 items of 0x30 bytes (+0 1 live)
    pub items: Bytes<98304>,
    /// 0x82E23E: `g_marksHexagonsDone`: the once-only mark of `set_up_hexagons`
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
    /// 0x82E26A: `flt_82E26A` .. `flt_82E276`: constants of `hexagons_top` (the last is the length it divides by)
    pub top_k: [Float; 4],
    /// 0x82E27A: `dword_82E27A`: a count per vertex (`hexagons_top`)
    pub per_vertex: Dword,
    /// 0x82E27E: Not named: 8 bytes (a gap, or unreferenced data).
    pub _gap_82e27e: Bytes<8>,
    /// 0x82E286: `flt_82E286` ..: what `hexagons_setup` scales the two axes by
    pub axes_k: [Float; 6],
    /// 0x82E29E: Not named: 4 bytes (a gap, or unreferenced data).
    pub _gap_82e29e: Bytes<4>,
    /// 0x82E2A2: `dword_82E2A2`: the texture square's side
    pub side: Dword,
    /// 0x82E2A6: `flt_82E2A6`: a constant of `hexagons_frame`
    pub frame_k: Float,
    /// 0x82E2AA: `flt_82E2AA`, `flt_82E2AE`: constants of `hexagons_spawn`
    pub spawn_k: [Float; 2],
    /// 0x82E2B2: `flt_82E2B2`: the share constant of `hexagons_spawn`
    pub share_k: Float,
    /// 0x82E2B6: `dword_82E2B6`: the hexagons' random (times 0xC5 plus 0x5D)
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

pub static HEX: HexagonVars = HexagonVars {
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
    squares_cleared: Dword::new(0x0),
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

impl HexagonVars {
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
        assert_eq!(size_of::<HexagonVars>(), HexagonVars::LEN);
        assert_eq!(
            offset_of!(HexagonVars, scale),
            0x816192 - HexagonVars::START
        );
        assert_eq!(
            offset_of!(HexagonVars, place),
            0x8161A2 - HexagonVars::START
        );
        assert_eq!(
            offset_of!(HexagonVars, started),
            0x8161AE - HexagonVars::START
        );
        assert_eq!(
            offset_of!(HexagonVars, moved),
            0x8161B2 - HexagonVars::START
        );
        assert_eq!(
            offset_of!(HexagonVars, spare),
            0x8161B6 - HexagonVars::START
        );
        assert_eq!(
            offset_of!(HexagonVars, kept_place),
            0x8161D6 - HexagonVars::START
        );
        assert_eq!(
            offset_of!(HexagonVars, kept_move),
            0x8161E2 - HexagonVars::START
        );
        assert_eq!(
            offset_of!(HexagonVars, view_move),
            0x8161EE - HexagonVars::START
        );
        assert_eq!(
            offset_of!(HexagonVars, move_change),
            0x8161FA - HexagonVars::START
        );
        assert_eq!(
            offset_of!(HexagonVars, drift),
            0x81621E - HexagonVars::START
        );
        assert_eq!(offset_of!(HexagonVars, ease), 0x81622A - HexagonVars::START);
        assert_eq!(
            offset_of!(HexagonVars, steps),
            0x81622E - HexagonVars::START
        );
        assert_eq!(
            offset_of!(HexagonVars, buffer),
            0x816232 - HexagonVars::START
        );
        assert_eq!(
            offset_of!(HexagonVars, squares_cleared),
            0x81623A - HexagonVars::START
        );
        assert_eq!(
            offset_of!(HexagonVars, items),
            0x81623E - HexagonVars::START
        );
        assert_eq!(offset_of!(HexagonVars, done), 0x82E23E - HexagonVars::START);
        assert_eq!(
            offset_of!(HexagonVars, factors),
            0x82E242 - HexagonVars::START
        );
        assert_eq!(
            offset_of!(HexagonVars, curve_a),
            0x82E25A - HexagonVars::START
        );
        assert_eq!(
            offset_of!(HexagonVars, curve_b),
            0x82E25E - HexagonVars::START
        );
        assert_eq!(
            offset_of!(HexagonVars, step_c),
            0x82E262 - HexagonVars::START
        );
        assert_eq!(
            offset_of!(HexagonVars, step_d),
            0x82E266 - HexagonVars::START
        );
        assert_eq!(
            offset_of!(HexagonVars, top_k),
            0x82E26A - HexagonVars::START
        );
        assert_eq!(
            offset_of!(HexagonVars, per_vertex),
            0x82E27A - HexagonVars::START
        );
        assert_eq!(
            offset_of!(HexagonVars, axes_k),
            0x82E286 - HexagonVars::START
        );
        assert_eq!(offset_of!(HexagonVars, side), 0x82E2A2 - HexagonVars::START);
        assert_eq!(
            offset_of!(HexagonVars, frame_k),
            0x82E2A6 - HexagonVars::START
        );
        assert_eq!(
            offset_of!(HexagonVars, spawn_k),
            0x82E2AA - HexagonVars::START
        );
        assert_eq!(
            offset_of!(HexagonVars, share_k),
            0x82E2B2 - HexagonVars::START
        );
        assert_eq!(
            offset_of!(HexagonVars, random),
            0x82E2B6 - HexagonVars::START
        );
        assert_eq!(offset_of!(HexagonVars, turn), 0x82E2C6 - HexagonVars::START);
        assert_eq!(
            offset_of!(HexagonVars, adjugate),
            0x82E2F6 - HexagonVars::START
        );
        assert_eq!(
            offset_of!(HexagonVars, decay),
            0x82E326 - HexagonVars::START
        );
        assert_eq!(
            offset_of!(HexagonVars, angle),
            0x82E332 - HexagonVars::START
        );
        assert_eq!(
            offset_of!(HexagonVars, result_k),
            0x82E336 - HexagonVars::START
        );
        let p = core::ptr::addr_of!(HEX) as *const u8;
        let now: Vec<u8> = (0..HexagonVars::LEN)
            .map(|i| unsafe { p.add(i).read() })
            .collect();
        assert!(
            now == crate::moved::image_bytes(HexagonVars::START, HexagonVars::LEN),
            "HEX starts differently from the image"
        );
    }
}
