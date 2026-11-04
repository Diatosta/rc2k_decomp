//! The rain's variables, out of the marks' block (phase 7, typed state): `RainState` (IDA
//! 0x8BC405 .. 0x8BDE55: the drift, the view, the drops' noise, the streak being drawn) and
//! `RainRules` (0x8C5F91 .. 0x8C5FFD: its constants). `RainState` starts on an address that
//! is 1 more than a multiple of 4 in the image; here its dwords are aligned from the
//! first.

use crate::session::{Bytes, Dword, Float};

/// The rain's variables (IDA 0x8BC405 .. 0x8BDE55, the image's layout; phase 7, typed state).
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct RainState {
    /// 0x8BC405: `dword_8BC405`: 1 once a rain frame has been drawn, 0 with no drops (and after `effects::seed_noise`, which also clears it); `effects::draw_rain_streak` draws nothing while it is 0.
    pub drawn: Dword,
    /// 0x8BC409: `dword_8BC409`: the scene clock at the last drift targets (`effects::drift_offsets`).
    pub drift_since: Dword,
    /// 0x8BC40D: `dword_8BC40D`: the ticks to wait for the next targets.
    pub drift_wait: Dword,
    /// 0x8BC411: `flt_8BC411`, `flt_8BC419`: the wandering offsets' targets (x, [y not used], z).
    pub drift_target: [Float; 3],
    /// 0x8BC41D: `flt_8BC41D`, `flt_8BC425`: their pace, eased toward the targets (x, [y not used], z).
    pub drift_pace: [Float; 3],
    /// 0x8BC429: `dword_8BC429`, `dword_8BC431`: a random's halves times 0x100 (x, [y not used], z); `run_rain` adds the first and third to the drops' places.
    pub drift_halves: [Dword; 3],
    /// 0x8BC435: `dword_8BC435` ..: the settings' +0x1C .. +0x24, the view's place (whole numbers).
    pub view_place: [Dword; 3],
    /// 0x8BC441: `dword_8BC441` ..: the settings' +0x28 .. +0x48, the view's 3 x 3 matrix (identity at first).
    pub view_matrix: [Float; 9],
    /// 0x8BC465: Not named: 24 bytes (a gap, or unreferenced data).
    pub _gap_8bc465: Bytes<24>,
    /// 0x8BC47D: `dword_8BC47D` .. `dword_8BC485`: the view's move since the last frame (floats), held to a length.
    pub motion: [Float; 3],
    /// 0x8BC489: `flt_8BC489`: the view's move since the last frame, scaled, by view entry (the settings' +8; 5 of 12 bytes).
    pub moves: [[Float; 3]; 5],
    /// 0x8BC4C5: `dword_8BC4C5`: the view's place at the last frame, by view entry (5 of 12 bytes; whole numbers, the y with the drift).
    pub last_views: [[Dword; 3]; 5],
    /// 0x8BC501: Not named: 4 bytes (a gap, or unreferenced data).
    pub _gap_8bc501: Bytes<4>,
    /// 0x8BC505: `dword_8BC505`: the drift added to the drops' y (the clock's low 20 bits times `flt_8C5FB5` when drifting).
    pub drift: Dword,
    /// 0x8BC509: Not named: 4 bytes (a gap, or unreferenced data).
    pub _gap_8bc509: Bytes<4>,
    /// 0x8BC50D: `dword_8BC50D`: the number of drops (the settings' +0x2764, held to 0x100; initially 0x100). Also read by `effects::ring_frame`.
    pub drops: Dword,
    /// 0x8BC511: `g_marksNoiseA` (0x8BC511), `g_marksNoiseB` (0x8BC911), `g_marksNoiseC` (0x8BCD11): the drops' random places (18 bits; `effects::seed_noise`).
    pub noise: [[Dword; 256]; 3],
    /// 0x8BD111: `dword_8BD111`, 0x8BD511, 0x8BD911: each drop's turned place kept for the next frame (when the settings' +0x10 is set).
    pub kept: [[Dword; 256]; 3],
    /// 0x8BDD11: Not named: 24 bytes (a gap, or unreferenced data).
    pub _gap_8bdd11: Bytes<24>,
    /// 0x8BDD29: `flt_8BDD29`, `flt_8BDD2D`: the streak's first end's side offset (y, x), the line across the screen scaled by the half width and the end's scale (a reading).
    pub edge_a: [Float; 2],
    /// 0x8BDD31: Not named: 4 bytes (a gap, or unreferenced data).
    pub _gap_8bdd31: Bytes<4>,
    /// 0x8BDD35: `flt_8BDD35`, `flt_8BDD39`: the same for the second end.
    pub edge_b: [Float; 2],
    /// 0x8BDD3D: Not named: 4 bytes (a gap, or unreferenced data).
    pub _gap_8bdd3d: Bytes<4>,
    /// 0x8BDD41: `flt_8BDD41` .. `flt_8BDD55`: the streak's two ends projected: (x, y, scale) each.
    pub ends: [[Float; 3]; 2],
    /// 0x8BDD59: Not named: 4 bytes (a gap, or unreferenced data).
    pub _gap_8bdd59: Bytes<4>,
    /// 0x8BDD5D: `flt_8BDD5D`: the first end's depth (and the length of the move).
    pub depth: Float,
    /// 0x8BDD61: `flt_8BDD61`: one over `flt_6241C8`, what the vertices' depth is multiplied by.
    pub depth_scale: Float,
    /// 0x8BDD65: `flt_8BDD65` ..: what the drop's place is scaled by before it is turned (1.0 each).
    pub point_scale: [Float; 3],
    /// 0x8BDD71: `dword_8BDD71` ..: the turned point's whole numbers added (`effects::place_view`).
    pub point: [Dword; 3],
    /// 0x8BDD7D: `dword_8BDD7D` ..: the view's place less the point (`effects::place_view`), drifting by `effects::drift_offsets`.
    pub view: [Dword; 3],
    /// 0x8BDD89: `dword_8BDD89` ..: the view matrix copied (`effects::place_view`).
    pub matrix_copy: [Float; 9],
    /// 0x8BDDAD: Not named: 144 bytes (a gap, or unreferenced data).
    pub _gap_8bddad: Bytes<144>,
    /// 0x8BDE3D: `flt_8BDE3D` ..: the drop's place turned.
    pub turned: [Float; 3],
    /// 0x8BDE49: `flt_8BDE49` ..: the place scaled, then the second end (less the motion) turned; `flt_8BDE51` its depth.
    pub second: [Float; 3],
}

pub static RAIN: RainState = RainState {
    drawn: Dword::new(0x0),
    drift_since: Dword::new(0x0),
    drift_wait: Dword::new(0x0),
    drift_target: [
        Float::from_bits(0x0),
        Float::from_bits(0x0),
        Float::from_bits(0x0),
    ],
    drift_pace: [
        Float::from_bits(0x0),
        Float::from_bits(0x0),
        Float::from_bits(0x0),
    ],
    drift_halves: [Dword::new(0x0), Dword::new(0x0), Dword::new(0x0)],
    view_place: [Dword::new(0x0), Dword::new(0x0), Dword::new(0x0)],
    view_matrix: [
        Float::from_bits(0x3F800000),
        Float::from_bits(0x0),
        Float::from_bits(0x0),
        Float::from_bits(0x0),
        Float::from_bits(0x3F800000),
        Float::from_bits(0x0),
        Float::from_bits(0x0),
        Float::from_bits(0x0),
        Float::from_bits(0x3F800000),
    ],
    _gap_8bc465: Bytes::new([
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x80, 0x3F, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x80, 0x3F,
    ]),
    motion: [
        Float::from_bits(0x0),
        Float::from_bits(0x0),
        Float::from_bits(0x3F800000),
    ],
    moves: [const {
        [
            Float::from_bits(0x0),
            Float::from_bits(0x0),
            Float::from_bits(0x3F800000),
        ]
    }; 5],
    last_views: [const { [Dword::new(0x0), Dword::new(0x0), Dword::new(0x3F800000)] }; 5],
    _gap_8bc501: Bytes::new([0; 4]),
    drift: Dword::new(0x0),
    _gap_8bc509: Bytes::new([0; 4]),
    drops: Dword::new(0x100),
    noise: [const { [const { Dword::new(0x0) }; 256] }; 3],
    kept: [const { [const { Dword::new(0x0) }; 256] }; 3],
    _gap_8bdd11: Bytes::new([
        0x00, 0x00, 0x80, 0x3F, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x20, 0x41, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x80, 0x3F, 0x00, 0x00, 0x20, 0x41,
    ]),
    edge_a: [Float::from_bits(0x0), Float::from_bits(0x0)],
    _gap_8bdd31: Bytes::new([0; 4]),
    edge_b: [Float::from_bits(0x0), Float::from_bits(0x0)],
    _gap_8bdd3d: Bytes::new([0; 4]),
    ends: [const {
        [
            Float::from_bits(0x0),
            Float::from_bits(0x0),
            Float::from_bits(0x0),
        ]
    }; 2],
    _gap_8bdd59: Bytes::new([0; 4]),
    depth: Float::from_bits(0x0),
    depth_scale: Float::from_bits(0x0),
    point_scale: [
        Float::from_bits(0x3F800000),
        Float::from_bits(0x3F800000),
        Float::from_bits(0x3F800000),
    ],
    point: [Dword::new(0x0), Dword::new(0x0), Dword::new(0x0)],
    view: [Dword::new(0x0), Dword::new(0x0), Dword::new(0x0)],
    matrix_copy: [
        Float::from_bits(0x3F800000),
        Float::from_bits(0x0),
        Float::from_bits(0x0),
        Float::from_bits(0x0),
        Float::from_bits(0x3F800000),
        Float::from_bits(0x0),
        Float::from_bits(0x0),
        Float::from_bits(0x0),
        Float::from_bits(0x3F800000),
    ],
    _gap_8bddad: Bytes::new([
        0x00, 0x00, 0x80, 0x3F, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x80, 0x3F, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x80, 0x3F, 0x00, 0x00, 0x80, 0x3F, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x80, 0x3F, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x80, 0x3F, 0x00, 0x00, 0x80,
        0x3F, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x80, 0x3F, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x80, 0x3F, 0x00, 0x80, 0x31, 0x46, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x80, 0x31, 0x46, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    ]),
    turned: [
        Float::from_bits(0x0),
        Float::from_bits(0x0),
        Float::from_bits(0x0),
    ],
    second: [
        Float::from_bits(0x0),
        Float::from_bits(0x0),
        Float::from_bits(0x0),
    ],
};

impl RainState {
    pub const START: usize = 0x8BC405;
    pub const LEN: usize = 0x1A50;
}

/// The rain's constants (IDA 0x8C5F91 .. 0x8C5FFD, the image's layout; phase 7, typed state).
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct RainRules {
    /// 0x8C5F91: `dword_8C5F91`: the quad vertices' rhw (a whole number's bits).
    pub rhw: Dword,
    /// 0x8C5F95: `flt_8C5F95` ..: the view's move since the last frame is scaled by them (30.0 each).
    pub move_scale: [Float; 3],
    /// 0x8C5FA1: `flt_8C5FA1`: a move longer than this is held (100000.0).
    pub move_limit: Float,
    /// 0x8C5FA5: `flt_8C5FA5`: the length a long move is held to (1.0).
    pub move_held: Float,
    /// 0x8C5FA9: `flt_8C5FA9` ..: a held move's factors (100000.0 each).
    pub move_factors: [Float; 3],
    /// 0x8C5FB5: `flt_8C5FB5`: the clock's low 20 bits times it is the drift (-500.0).
    pub drift_k: Float,
    /// 0x8C5FB9: Not named: 24 bytes (a gap, or unreferenced data).
    pub _gap_8c5fb9: Bytes<24>,
    /// 0x8C5FD1: `flt_8C5FD1`, `flt_8C5FD5`: the drift targets' scales (10.0).
    pub target_scale: [Float; 2],
    /// 0x8C5FD9: `flt_8C5FD9`, `flt_8C5FDD`: how fast the pace follows its target (0.05).
    pub ease: [Float; 2],
    /// 0x8C5FE1: `flt_8C5FE1`: 131072.0, the view's height (`effects::place_view`).
    pub height: Float,
    /// 0x8C5FE5: `flt_8C5FE5`, `flt_8C5FE9`: the streak's ends' screen scale (500.0 each).
    pub view_scale: [Float; 2],
    /// 0x8C5FED: `flt_8C5FED`: the streak's half width (1.0).
    pub half_width: Float,
    /// 0x8C5FF1: Not named: 12 bytes (a gap, or unreferenced data).
    pub _gap_8c5ff1: Bytes<12>,
}

pub static RAIN_RULES: RainRules = RainRules {
    rhw: Dword::new(0x43800000),
    move_scale: [
        Float::from_bits(0x41F00000),
        Float::from_bits(0x41F00000),
        Float::from_bits(0x41F00000),
    ],
    move_limit: Float::from_bits(0x47C35000),
    move_held: Float::from_bits(0x3F800000),
    move_factors: [
        Float::from_bits(0x47C35000),
        Float::from_bits(0x47C35000),
        Float::from_bits(0x47C35000),
    ],
    drift_k: Float::from_bits(0xC3FA0000),
    _gap_8c5fb9: Bytes::new([
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x6F, 0x12, 0x83,
        0x3A, 0x6F, 0x12, 0x03, 0x3B, 0xA6, 0x9B, 0x44, 0x3B,
    ]),
    target_scale: [Float::from_bits(0x41200000), Float::from_bits(0x41200000)],
    ease: [Float::from_bits(0x3D4CCCCD), Float::from_bits(0x3D4CCCCD)],
    height: Float::from_bits(0x48000000),
    view_scale: [Float::from_bits(0x43FA0000), Float::from_bits(0x43FA0000)],
    half_width: Float::from_bits(0x3F800000),
    _gap_8c5ff1: Bytes::new([
        0x00, 0x00, 0xC8, 0x4A, 0x33, 0x33, 0x33, 0x3F, 0x00, 0x00, 0x00, 0x00,
    ]),
};

impl RainRules {
    pub const START: usize = 0x8C5F91;
    pub const LEN: usize = 0x6C;
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use core::mem::{offset_of, size_of};

    fn bytes<T>(v: &T) -> Vec<u8> {
        let p = std::ptr::from_ref::<T>(v).cast::<u8>();
        (0..size_of::<T>())
            .map(|i| unsafe { p.add(i).read() })
            .collect()
    }

    /// Runs `f` on the rain's variables in a test buffer standing for the marks' block at
    /// `mk` (IDA 0x764F40); the buffer's bytes are copied out for `f` (the image's address
    /// of the state is not aligned) and put back.
    #[cfg(target_arch = "x86")]
    pub fn with_state_at<R>(mk: usize, f: impl FnOnce(&RainState, &RainRules) -> R) -> R {
        let state = crate::pointers::host_at(
            mk.wrapping_add(RainState::START)
                .wrapping_sub(crate::effects::store::START),
        )
        .cast::<RainState>();
        let rules = crate::pointers::host_at(
            mk.wrapping_add(RainRules::START)
                .wrapping_sub(crate::effects::store::START),
        )
        .cast::<RainRules>();
        // On the heap: the two are near 2 MB together, a test thread's whole stack (copied
        // by value there they overflowed it, and Wine hung on the fault).
        for start in [RainState::START, RainRules::START] {
            crate::effects::typed_sync::fits_buffer(
                mk.wrapping_add(start)
                    .wrapping_sub(crate::effects::store::START),
                if start == RainState::START {
                    size_of::<RainState>()
                } else {
                    size_of::<RainRules>()
                },
            );
        }
        unsafe {
            let (s, r) = (boxed_copy(state), boxed_copy(rules));
            let out = f(&s, &r);
            core::ptr::copy_nonoverlapping(
                (&raw const *s).cast::<u8>(),
                state.cast::<u8>(),
                size_of_val(&*s),
            );
            core::ptr::copy_nonoverlapping(
                (&raw const *r).cast::<u8>(),
                rules.cast::<u8>(),
                size_of_val(&*r),
            );
            out
        }
    }

    /// A heap copy of the (possibly unaligned) value at `from`, never on the stack.
    #[cfg(target_arch = "x86")]
    unsafe fn boxed_copy<T>(from: *const T) -> Box<T> {
        let mut b = Box::<T>::new_uninit();
        unsafe {
            core::ptr::copy_nonoverlapping(
                from.cast::<u8>(),
                b.as_mut_ptr().cast::<u8>(),
                size_of::<T>(),
            );
            b.assume_init()
        }
    }

    #[test]
    fn the_rain_starts_as_the_games() {
        assert_eq!(size_of::<RainState>(), RainState::LEN);
        assert_eq!(size_of::<RainRules>(), RainRules::LEN);
        assert!(
            bytes(&RAIN) == crate::moved::image_bytes(RainState::START, RainState::LEN),
            "RAIN starts differently from the image"
        );
        assert!(
            bytes(&RAIN_RULES) == crate::moved::image_bytes(RainRules::START, RainRules::LEN),
            "RAIN_RULES starts differently from the image"
        );
        let s = RainState::START;
        assert_eq!(offset_of!(RainState, motion), 0x8BC47D - s);
        assert_eq!(offset_of!(RainState, moves), 0x8BC489 - s);
        assert_eq!(offset_of!(RainState, last_views), 0x8BC4C5 - s);
        assert_eq!(offset_of!(RainState, drops), 0x8BC50D - s);
        assert_eq!(offset_of!(RainState, noise), 0x8BC511 - s);
        assert_eq!(offset_of!(RainState, kept), 0x8BD111 - s);
        assert_eq!(offset_of!(RainState, depth), 0x8BDD5D - s);
        assert_eq!(offset_of!(RainState, matrix_copy), 0x8BDD89 - s);
        assert_eq!(offset_of!(RainState, second), 0x8BDE49 - s);
        let r = RainRules::START;
        assert_eq!(offset_of!(RainRules, move_limit), 0x8C5FA1 - r);
        assert_eq!(offset_of!(RainRules, drift_k), 0x8C5FB5 - r);
        assert_eq!(offset_of!(RainRules, target_scale), 0x8C5FD1 - r);
        assert_eq!(offset_of!(RainRules, half_width), 0x8C5FED - r);
    }
}
