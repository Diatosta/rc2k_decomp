//! The rain's variables: `RainState` (IDA
//! 0x8BC405 .. 0x8BDE55: the drift, the view, the drops' noise, the streak being drawn) and
//! `RainRules` (0x8C5F91 .. 0x8C5FFD: its constants). `RainState` starts on an address that
//! is 1 more than a multiple of 4 in the image; here its dwords are aligned from the
//! first.

#[cfg(test)]
use crate::effects::owner::effects;
use crate::session::{PlainBytes, PlainDword, PlainFloat};

/// The rain's variables (IDA 0x8BC405 .. 0x8BDE55).
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct RainState {
    /// 0x8BC405: `dword_8BC405`: 1 once a rain frame has been drawn, 0 with no drops (and after `effects::seed_noise`, which also clears it); `effects::draw_rain_streak` draws nothing while it is 0.
    pub drawn: PlainDword,
    /// 0x8BC409: `dword_8BC409`: the scene clock at the last drift targets (`effects::drift_offsets`).
    pub drift_since: PlainDword,
    /// 0x8BC40D: `dword_8BC40D`: the ticks to wait for the next targets.
    pub drift_wait: PlainDword,
    /// 0x8BC411: `flt_8BC411`, `flt_8BC419`: the wandering offsets' targets (x, [y not used], z).
    pub drift_target: [PlainFloat; 3],
    /// 0x8BC41D: `flt_8BC41D`, `flt_8BC425`: their pace, eased toward the targets (x, [y not used], z).
    pub drift_pace: [PlainFloat; 3],
    /// 0x8BC429: `dword_8BC429`, `dword_8BC431`: a random's halves times 0x100 (x, [y not used], z); `run_rain` adds the first and third to the drops' places.
    pub drift_halves: [PlainDword; 3],
    /// 0x8BC435: `dword_8BC435` ..: the settings' +0x1C .. +0x24, the view's place (whole numbers).
    pub view_place: [PlainDword; 3],
    /// 0x8BC441: `dword_8BC441` ..: the settings' +0x28 .. +0x48, the view's 3 x 3 matrix (identity at first).
    pub view_matrix: [PlainFloat; 9],
    /// 0x8BC465: Not named: 24 bytes (a gap, or unreferenced data).
    pub _gap_8bc465: PlainBytes<24>,
    /// 0x8BC47D: `dword_8BC47D` .. `dword_8BC485`: the view's move since the last frame (floats), held to a length.
    pub motion: [PlainFloat; 3],
    /// 0x8BC489: `flt_8BC489`: the view's move since the last frame, scaled, by view entry (the settings' +8; 5 of 12 bytes).
    pub moves: [[PlainFloat; 3]; 5],
    /// 0x8BC4C5: `dword_8BC4C5`: the view's place at the last frame, by view entry (5 of 12 bytes; whole numbers, the y with the drift).
    pub last_views: [[PlainDword; 3]; 5],
    /// 0x8BC501: Not named: 4 bytes (a gap, or unreferenced data).
    pub _gap_8bc501: PlainBytes<4>,
    /// 0x8BC505: `dword_8BC505`: the drift added to the drops' y (the clock's low 20 bits times `flt_8C5FB5` when drifting).
    pub drift: PlainDword,
    /// 0x8BC509: Not named: 4 bytes (a gap, or unreferenced data).
    pub _gap_8bc509: PlainBytes<4>,
    /// 0x8BC50D: `dword_8BC50D`: the number of drops (the settings' +0x2764, held to 0x100; initially 0x100). Also read by `effects::ring_frame`.
    pub drops: PlainDword,
    /// 0x8BC511: `g_marksNoiseA` (0x8BC511), `g_marksNoiseB` (0x8BC911), `g_marksNoiseC` (0x8BCD11): the drops' random places (18 bits; `effects::seed_noise`).
    pub noise: [[PlainDword; 256]; 3],
    /// 0x8BD111: `dword_8BD111`, 0x8BD511, 0x8BD911: each drop's turned place kept for the next frame (when the settings' +0x10 is set).
    pub kept: [[PlainDword; 256]; 3],
    /// 0x8BDD11: Not named: 24 bytes (a gap, or unreferenced data).
    pub _gap_8bdd11: PlainBytes<24>,
    /// 0x8BDD29: `flt_8BDD29`, `flt_8BDD2D`: the streak's half-width offset (x, y) at its first end: the line across the streak's screen direction (the direction turned a quarter: -dy, dx) normalised, times the kind's half width and the first end's perspective scale (`effects::draw_rain_streak`: the quad's corners are the ends' points less / plus it).
    pub edge_a: [PlainFloat; 2],
    /// 0x8BDD31: Not named: 4 bytes (a gap, or unreferenced data).
    pub _gap_8bdd31: PlainBytes<4>,
    /// 0x8BDD35: `flt_8BDD35`, `flt_8BDD39`: the same for the second end.
    pub edge_b: [PlainFloat; 2],
    /// 0x8BDD3D: Not named: 4 bytes (a gap, or unreferenced data).
    pub _gap_8bdd3d: PlainBytes<4>,
    /// 0x8BDD41: `flt_8BDD41` .. `flt_8BDD55`: the streak's two ends projected: (x, y, scale) each.
    pub ends: [[PlainFloat; 3]; 2],
    /// 0x8BDD59: Not named: 4 bytes (a gap, or unreferenced data).
    pub _gap_8bdd59: PlainBytes<4>,
    /// 0x8BDD5D: `flt_8BDD5D`: the first end's depth (and the length of the move).
    pub depth: PlainFloat,
    /// 0x8BDD61: `flt_8BDD61`: one over `flt_6241C8`, what the vertices' depth is multiplied by.
    pub depth_scale: PlainFloat,
    /// 0x8BDD65: `flt_8BDD65` ..: what the drop's place is scaled by before it is turned (1.0 each).
    pub point_scale: [PlainFloat; 3],
    /// 0x8BDD71: `dword_8BDD71` ..: the turned point's whole numbers added (`effects::place_rain_view`).
    pub point: [PlainDword; 3],
    /// 0x8BDD7D: `dword_8BDD7D` ..: the view's place less the point (`effects::place_rain_view`), drifting by `effects::drift_offsets`.
    pub view: [PlainDword; 3],
    /// 0x8BDD89: `dword_8BDD89` ..: the view matrix copied (`effects::place_rain_view`).
    pub matrix_copy: [PlainFloat; 9],
    /// 0x8BDDAD: Not named: 144 bytes (a gap, or unreferenced data).
    pub _gap_8bddad: PlainBytes<144>,
    /// 0x8BDE3D: `flt_8BDE3D` ..: the drop's place turned.
    pub turned: [PlainFloat; 3],
    /// 0x8BDE49: `flt_8BDE49` ..: the place scaled, then the second end (less the motion) turned; `flt_8BDE51` its depth.
    pub second: [PlainFloat; 3],
}

impl RainState {
    pub const fn new() -> Self {
        RainState {
            drawn: PlainDword::new(0x0),
            drift_since: PlainDword::new(0x0),
            drift_wait: PlainDword::new(0x0),
            drift_target: [
                PlainFloat::new(0.0),
                PlainFloat::new(0.0),
                PlainFloat::new(0.0),
            ],
            drift_pace: [
                PlainFloat::new(0.0),
                PlainFloat::new(0.0),
                PlainFloat::new(0.0),
            ],
            drift_halves: [
                PlainDword::new(0x0),
                PlainDword::new(0x0),
                PlainDword::new(0x0),
            ],
            view_place: [
                PlainDword::new(0x0),
                PlainDword::new(0x0),
                PlainDword::new(0x0),
            ],
            view_matrix: [
                PlainFloat::new(1.0),
                PlainFloat::new(0.0),
                PlainFloat::new(0.0),
                PlainFloat::new(0.0),
                PlainFloat::new(1.0),
                PlainFloat::new(0.0),
                PlainFloat::new(0.0),
                PlainFloat::new(0.0),
                PlainFloat::new(1.0),
            ],
            _gap_8bc465: PlainBytes::new([
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x80, 0x3F, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x80, 0x3F,
            ]),
            motion: [
                PlainFloat::new(0.0),
                PlainFloat::new(0.0),
                PlainFloat::new(1.0),
            ],
            moves: [const {
                [
                    PlainFloat::new(0.0),
                    PlainFloat::new(0.0),
                    PlainFloat::new(1.0),
                ]
            }; 5],
            last_views: [const {
                [
                    PlainDword::new(0x0),
                    PlainDword::new(0x0),
                    PlainDword::new(0x3F800000),
                ]
            }; 5],
            _gap_8bc501: PlainBytes::new([0; 4]),
            drift: PlainDword::new(0x0),
            _gap_8bc509: PlainBytes::new([0; 4]),
            drops: PlainDword::new(0x100),
            noise: [const { [const { PlainDword::new(0x0) }; 256] }; 3],
            kept: [const { [const { PlainDword::new(0x0) }; 256] }; 3],
            _gap_8bdd11: PlainBytes::new([
                0x00, 0x00, 0x80, 0x3F, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x20, 0x41, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00, 0x80, 0x3F, 0x00, 0x00, 0x20, 0x41,
            ]),
            edge_a: [PlainFloat::new(0.0), PlainFloat::new(0.0)],
            _gap_8bdd31: PlainBytes::new([0; 4]),
            edge_b: [PlainFloat::new(0.0), PlainFloat::new(0.0)],
            _gap_8bdd3d: PlainBytes::new([0; 4]),
            ends: [const {
                [
                    PlainFloat::new(0.0),
                    PlainFloat::new(0.0),
                    PlainFloat::new(0.0),
                ]
            }; 2],
            _gap_8bdd59: PlainBytes::new([0; 4]),
            depth: PlainFloat::new(0.0),
            depth_scale: PlainFloat::new(0.0),
            point_scale: [
                PlainFloat::new(1.0),
                PlainFloat::new(1.0),
                PlainFloat::new(1.0),
            ],
            point: [
                PlainDword::new(0x0),
                PlainDword::new(0x0),
                PlainDword::new(0x0),
            ],
            view: [
                PlainDword::new(0x0),
                PlainDword::new(0x0),
                PlainDword::new(0x0),
            ],
            matrix_copy: [
                PlainFloat::new(1.0),
                PlainFloat::new(0.0),
                PlainFloat::new(0.0),
                PlainFloat::new(0.0),
                PlainFloat::new(1.0),
                PlainFloat::new(0.0),
                PlainFloat::new(0.0),
                PlainFloat::new(0.0),
                PlainFloat::new(1.0),
            ],
            _gap_8bddad: PlainBytes::new([
                0x00, 0x00, 0x80, 0x3F, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00, 0x80, 0x3F, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x80, 0x3F, 0x00, 0x00, 0x80, 0x3F, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x80, 0x3F,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x80, 0x3F, 0x00, 0x00, 0x80, 0x3F, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x80, 0x3F, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x80, 0x3F, 0x00, 0x80, 0x31, 0x46,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x80,
                0x31, 0x46, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00,
            ]),
            turned: [
                PlainFloat::new(0.0),
                PlainFloat::new(0.0),
                PlainFloat::new(0.0),
            ],
            second: [
                PlainFloat::new(0.0),
                PlainFloat::new(0.0),
                PlainFloat::new(0.0),
            ],
        }
    }
}

impl RainState {
    pub const START: usize = 0x8BC405;
    pub const LEN: usize = 0x1A50;
}

/// The rain's constants (IDA 0x8C5F91 .. 0x8C5FFD).
#[repr(C)]
pub struct RainRules {
    /// 0x8C5F91: `dword_8C5F91`: the quad vertices' rhw, 256.0 (the original loaded its bits as a
    /// whole number: `run_rain`).
    pub rhw: u32,
    /// 0x8C5F95: `flt_8C5F95` ..: the view's move since the last frame is scaled by them (30.0 each).
    pub move_scale: [f32; 3],
    /// 0x8C5FA1: `flt_8C5FA1`: a move longer than this is held (100000.0).
    pub move_limit: f32,
    /// 0x8C5FA5: `flt_8C5FA5`: the length a long move is held to (1.0).
    pub move_held: f32,
    /// 0x8C5FA9: `flt_8C5FA9` ..: a held move's factors (100000.0 each).
    pub move_factors: [f32; 3],
    /// 0x8C5FB5: `flt_8C5FB5`: the clock's low 20 bits times it is the drift (-500.0).
    pub drift_k: f32,
    /// 0x8C5FB9: Not named: 24 bytes (a gap, or unreferenced data).
    pub _gap_8c5fb9: [u8; 24],
    /// 0x8C5FD1: `flt_8C5FD1`, `flt_8C5FD5`: the drift targets' scales (10.0).
    pub target_scale: [f32; 2],
    /// 0x8C5FD9: `flt_8C5FD9`, `flt_8C5FDD`: how fast the pace follows its target (0.05).
    pub ease: [f32; 2],
    /// 0x8C5FE1: `flt_8C5FE1`: 131072.0, the view's height (`effects::place_rain_view`).
    pub height: f32,
    /// 0x8C5FE5: `flt_8C5FE5`, `flt_8C5FE9`: the streak's ends' screen scale (500.0 each).
    pub view_scale: [f32; 2],
    /// 0x8C5FED: `flt_8C5FED`: the streak's half width (1.0).
    pub half_width: f32,
    /// 0x8C5FF1: Not named: 12 bytes (a gap, or unreferenced data).
    pub _gap_8c5ff1: [u8; 12],
}

impl RainRules {
    pub const fn new() -> Self {
        RainRules {
            rhw: 0x43800000,
            move_scale: [30.0, 30.0, 30.0],
            move_limit: 100000.0_f32,
            move_held: 1.0_f32,
            move_factors: [100000.0, 100000.0, 100000.0],
            drift_k: -500.0_f32,
            _gap_8c5fb9: [
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x6F, 0x12,
                0x83, 0x3A, 0x6F, 0x12, 0x03, 0x3B, 0xA6, 0x9B, 0x44, 0x3B,
            ],
            target_scale: [10.0_f32, 10.0_f32],
            ease: [0.05_f32, 0.05_f32],
            height: 131072.0_f32,
            view_scale: [500.0_f32, 500.0_f32],
            half_width: 1.0_f32,
            _gap_8c5ff1: [
                0x00, 0x00, 0xC8, 0x4A, 0x33, 0x33, 0x33, 0x3F, 0x00, 0x00, 0x00, 0x00,
            ],
        }
    }
}

/// The rain's and the snow's state, one part of the effects' owner (`effects::owner::Effects`).
pub struct Weather {
    pub rain: RainState,
    pub rain_rules: RainRules,
    pub snow: crate::effects::snow::SnowState,
    pub snow_light: crate::effects::snow::SnowLight,
    pub snow_curve: crate::effects::snow::SnowCurve,
    pub snow_tables: crate::effects::snow::SnowTables,
    pub snow_vars: crate::effects::snow::SnowCurveVars,
}

impl Weather {
    pub const fn new() -> Self {
        Weather {
            rain: RainState::new(),
            rain_rules: RainRules::new(),
            snow: crate::effects::snow::SnowState::new(),
            snow_light: crate::effects::snow::SnowLight::new(),
            snow_curve: crate::effects::snow::SnowCurve::new(),
            snow_tables: crate::effects::snow::SnowTables::new(),
            snow_vars: crate::effects::snow::SnowCurveVars::new(),
        }
    }
}

#[cfg(test)]
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

    /// Runs `f` on the rain's variables in the test buffer `mk` standing for the marks'
    /// block; the buffer's bytes are copied out for `f` (their place in it is not aligned)
    /// and put back.
    pub fn with_state_at<R>(
        mk: crate::effects::asm_test::Operand,
        f: impl FnOnce(&RainState, &RainRules) -> R,
    ) -> R {
        let from_start =
            |start: usize, len: usize| mk.at(start.wrapping_sub(crate::effects::store::START), len);
        let state = from_start(RainState::START, size_of::<RainState>()).cast::<RainState>();
        let rules = from_start(RainRules::START, size_of::<RainRules>()).cast::<RainRules>();
        // On the heap: the two are near 2 MB together, a test thread's whole stack (copied
        // by value there they overflowed it, and Wine hung on the fault).
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
        let weather = &effects().weather;
        assert_eq!(size_of::<RainState>(), RainState::LEN);
        assert_eq!(size_of::<RainRules>(), RainRules::LEN);
        assert!(
            bytes(&weather.rain)
                == crate::test_support::first_values::bytes(
                    "effects.owner.EFFECTS.weather.rain/6736"
                ),
            "RAIN starts differently from the image"
        );
        assert!(
            bytes(&weather.rain_rules)
                == crate::test_support::first_values::bytes("effects.rain.RAIN_RULES"),
            "RAIN_RULES starts differently from the image"
        );
        assert_eq!(offset_of!(RainState, motion), 0x78);
        assert_eq!(offset_of!(RainState, moves), 0x84);
        assert_eq!(offset_of!(RainState, last_views), 0xc0);
        assert_eq!(offset_of!(RainState, drops), 0x108);
        assert_eq!(offset_of!(RainState, noise), 0x10c);
        assert_eq!(offset_of!(RainState, kept), 0xd0c);
        assert_eq!(offset_of!(RainState, depth), 0x1958);
        assert_eq!(offset_of!(RainState, matrix_copy), 0x1984);
        assert_eq!(offset_of!(RainState, second), 0x1a44);
        assert_eq!(offset_of!(RainRules, move_limit), 0x10);
        assert_eq!(offset_of!(RainRules, drift_k), 0x24);
        assert_eq!(offset_of!(RainRules, target_scale), 0x40);
        assert_eq!(offset_of!(RainRules, half_width), 0x5c);
    }
}
