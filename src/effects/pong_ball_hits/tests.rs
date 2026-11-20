use crate::effects::asm_layout::with_laid;
use crate::effects::asm_test::Kind::*;
use crate::effects::asm_test::{Op, Spec, recorded};
use crate::effects::pong::{PongRules, PongState};

#[test]
fn centres_x87_matches() {
    const SPEC: Spec = Spec {
        operands: &[Op::Base {
            lo: 0x3718,
            hi: 0x42F2,
            reads: &[
                (0x3748, F32),
                (0x374C, F32),
                (0x3760, F32),
                (0x3764, F32),
                (0x3778, F32),
                (0x377C, F32),
                (0x3790, F32),
                (0x3794, F32),
                (0x379C, F32),
                (0x37A0, F32),
                (0x37A8, F32),
                (0x37AC, F32),
                (0x4286, F32),
                (0x428A, F32),
                (0x428E, F32),
                (0x4292, F32),
            ],
            writes: &[
                (0x379C, F32),
                (0x37A0, F32),
                (0x37A8, F32),
                (0x37AC, F32),
                (0x37B4, F32),
                (0x37B8, F32),
            ],
        }],
        sincos: false,
    };
    recorded("effects.pong_ball_hits.centres_x87", &SPEC, |b| {
        with_laid::<PongState, _>(b[0], |state| {
            with_laid::<PongRules, _>(b[0], |rules| super::centres_x87(state, rules));
        });
    });
}

#[test]
fn new_way_x87_matches() {
    const SPEC: Spec = Spec {
        operands: &[Op::Base {
            lo: 0x22C,
            hi: 0x42F2,
            reads: &[
                (0x22C, F32),
                (0x37B4, F32),
                (0x37B8, F32),
                (0x429A, F32),
                (0x429E, F32),
            ],
            writes: &[(0x37B4, F32), (0x37B8, F32)],
        }],
        sincos: false,
    };
    recorded("effects.pong_ball_hits.new_way_x87", &SPEC, |b| {
        let length =
            // SAFETY: `at` checks that the four bytes lie in the operand's test buffer, which
            // the recorded run keeps alive and nothing else reaches meanwhile; unaligned, any
            // bits a `u32`.
            f32::from_bits(unsafe { ((b[0] + 0x22C).at(0, 4) as *const u32).read_unaligned() });
        with_laid::<PongState, _>(b[0], |state| {
            with_laid::<PongRules, _>(b[0], |rules| super::new_way_x87(state, rules, length));
        });
    });
}
