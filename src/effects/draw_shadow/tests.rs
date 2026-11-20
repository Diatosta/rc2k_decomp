use super::Shadow;
use crate::effects::asm_layout::with_laid_at;
use crate::effects::asm_test::START;
use crate::effects::asm_test::{Operand, floats_at, with_copy};
use crate::effects::kind_store::{KindConsts, QuadWork};
use crate::memory::session::PlainFloat;

/// A shadow's variables: where the quad, the constants and the up start in the marks' block.
#[derive(Clone, Copy)]
struct Layout(usize, usize, usize, u32);
const SET_1: Layout = Layout(0x9331B4, 0x933E7C, 0x933E70, 0x26A0);
const SET_2: Layout = Layout(0x933ED0, 0x934B7C, 0x934B78, 0x26A4);

/// `f` on copies of a shadow's variables in the test buffer standing for the marks'
/// block at `mk`, put back afterwards.
fn with_shadow<R>(
    mk: crate::effects::asm_test::Operand,
    l: Layout,
    f: impl FnOnce(&Shadow) -> R,
) -> R {
    with_laid_at::<std::cell::RefCell<QuadWork>, R>(mk, l.0, |quad| {
        with_consts(mk, l.1, |consts| {
            with_copy::<PlainFloat, R>(mk, l.2, |up| {
                f(&Shadow {
                    quad,
                    consts,
                    up,
                    field: if l.3 == 0x26A0 {
                        |marks| &marks.engine_speed_copy
                    } else {
                        |marks| &marks.engine_speed_scaled_copy
                    },
                })
            })
        })
    })
}
/// `f` on a set's constants made from the floats the test buffer `mk` holds at `start`.
fn with_consts<R>(mk: Operand, start: usize, f: impl FnOnce(&KindConsts) -> R) -> R {
    let v = floats_at::<21>(mk, start);
    f(&KindConsts {
        size_k: v[0],
        start_k: v[1],
        step_k: v[2],
        middle_k: v[3],
        slope_k: v[4],
        spread_k: v[5],
        extra_k: v[6],
        place_k: [v[7], v[8], v[9], v[10], v[11], v[12]],
        place_less: v[13],
        light: [v[14], v[15], v[16]],
        lifts: [v[17], v[18], v[19], v[20]],
    })
}
/// `f` on the matrix and the two temporaries in the test buffers `m` and `t`.
fn with_matrix(
    m: crate::effects::asm_test::Operand,
    t: crate::effects::asm_test::Operand,
    f: impl FnOnce(&[f32; 9], &mut [f32; 2]),
) {
    with_copy::<[PlainFloat; 9], ()>(m, START, |m| {
        with_copy::<[PlainFloat; 2], ()>(t, START, |t| {
            let matrix: [f32; 9] = std::array::from_fn(|i| m[i].get());
            let mut temps: [f32; 2] = std::array::from_fn(|i| t[i].get());
            f(&matrix, &mut temps);
            for (cell, value) in t.iter().zip(temps) {
                cell.set(value);
            }
        });
    });
}
use crate::effects::asm_test::Kind::*;
use crate::effects::asm_test::{Op, Spec, recorded_writes};

#[test]
fn light_turned_x87_matches() {
    const SPEC: Spec = Spec {
        operands: &[
            Op::Base {
                lo: 0x1CEF90,
                hi: 0x1CFC90,
                reads: &[(0x1CFC74, F32), (0x1CFC78, F32), (0x1CFC7C, F32)],
                writes: &[(0x1CEFA0, F32), (0x1CEFA4, F32), (0x1CEFA8, F32)],
            },
            Op::Base {
                lo: 0x0,
                hi: 0x24,
                reads: &[
                    (0x0, F32),
                    (0x4, F32),
                    (0x8, F32),
                    (0xC, F32),
                    (0x10, F32),
                    (0x14, F32),
                    (0x18, F32),
                    (0x1C, F32),
                    (0x20, F32),
                ],
                writes: &[],
            },
            Op::Base {
                lo: 0x0,
                hi: 0x8,
                reads: &[(0x0, F32), (0x4, F32)],
                writes: &[(0x0, F32), (0x4, F32)],
            },
        ],
        sincos: false,
    };
    recorded_writes("effects.draw_shadow.light_turned_x87", &SPEC, |b| {
        with_shadow(b[0], SET_2, |s| {
            with_matrix(b[1], b[2], |m, t| super::light_turned_x87(s, m, t));
        });
    });
}

#[test]
fn second_turned_x87_matches() {
    const SPEC: Spec = Spec {
        operands: &[
            Op::Base {
                lo: 0x1CEF90,
                hi: 0x1CFC90,
                reads: &[(0x1CEFA0, F32), (0x1CEFA4, F32), (0x1CEFA8, F32)],
                writes: &[(0x1CF00C, F32), (0x1CF010, F32), (0x1CF014, F32)],
            },
            Op::Base {
                lo: 0x0,
                hi: 0x24,
                reads: &[
                    (0x0, F32),
                    (0x4, F32),
                    (0x8, F32),
                    (0xC, F32),
                    (0x10, F32),
                    (0x14, F32),
                    (0x18, F32),
                    (0x1C, F32),
                    (0x20, F32),
                ],
                writes: &[],
            },
            Op::Base {
                lo: 0x0,
                hi: 0x8,
                reads: &[(0x0, F32), (0x4, F32)],
                writes: &[(0x0, F32), (0x4, F32)],
            },
        ],
        sincos: false,
    };
    recorded_writes("effects.draw_shadow.second_turned_x87", &SPEC, |b| {
        with_shadow(b[0], SET_2, |s| {
            with_matrix(b[1], b[2], |m, t| super::second_turned_x87(s, m, t));
        });
    });
}

#[test]
fn corners_x87_matches() {
    const SPEC: Spec = Spec {
        operands: &[Op::Base {
            lo: 0x1CEF90,
            hi: 0x1CFC90,
            reads: &[
                (0x1CEF90, F32),
                (0x1CEF94, F32),
                (0x1CEF98, F32),
                (0x1CEFB0, F32),
                (0x1CEFBC, F32),
                (0x1CEFC8, F32),
                (0x1CF000, F32),
                (0x1CF004, F32),
                (0x1CF008, F32),
                (0x1CF00C, F32),
                (0x1CF010, F32),
                (0x1CF014, F32),
                (0x1CF028, F32),
                (0x1CF02C, F32),
                (0x1CF058, F32),
                (0x1CF05C, F32),
                (0x1CF060, F32),
                (0x1CF064, F32),
                (0x1CF068, F32),
                (0x1CF06C, F32),
                (0x1CF070, F32),
                (0x1CF074, F32),
                (0x1CF078, F32),
                (0x1CF07C, F32),
                (0x1CF080, F32),
                (0x1CF084, F32),
                (0x1CFC38, F32),
            ],
            writes: &[
                (0x1CF000, F32),
                (0x1CF004, F32),
                (0x1CF008, F32),
                (0x1CF00C, F32),
                (0x1CF010, F32),
                (0x1CF014, F32),
                (0x1CF04C, F32),
                (0x1CF050, F32),
                (0x1CF054, F32),
                (0x1CF058, F32),
                (0x1CF05C, F32),
                (0x1CF060, F32),
                (0x1CF064, F32),
                (0x1CF068, F32),
                (0x1CF06C, F32),
                (0x1CF070, F32),
                (0x1CF074, F32),
                (0x1CF078, F32),
                (0x1CF07C, F32),
                (0x1CF080, F32),
                (0x1CF084, F32),
            ],
        }],
        sincos: false,
    };
    recorded_writes("effects.draw_shadow.corners_x87", &SPEC, |b| {
        with_shadow(b[0], SET_2, super::corners_x87);
    });
}

#[test]
fn corners_in_x87_matches() {
    const SPEC: Spec = Spec {
        operands: &[Op::Base {
            lo: 0x1CEF90,
            hi: 0x1CFC90,
            reads: &[
                (0x1CF048, F32),
                (0x1CF04C, F32),
                (0x1CF050, F32),
                (0x1CF054, F32),
                (0x1CF058, F32),
                (0x1CF05C, F32),
                (0x1CF060, F32),
                (0x1CF064, F32),
                (0x1CF068, F32),
                (0x1CF06C, F32),
                (0x1CF070, F32),
                (0x1CF074, F32),
                (0x1CF078, F32),
                (0x1CF07C, F32),
                (0x1CF080, F32),
                (0x1CF084, F32),
            ],
            writes: &[
                (0x1CF04C, F32),
                (0x1CF050, F32),
                (0x1CF054, F32),
                (0x1CF058, F32),
                (0x1CF05C, F32),
                (0x1CF060, F32),
                (0x1CF064, F32),
                (0x1CF068, F32),
                (0x1CF06C, F32),
                (0x1CF070, F32),
                (0x1CF074, F32),
                (0x1CF078, F32),
                (0x1CF07C, F32),
                (0x1CF080, F32),
                (0x1CF084, F32),
            ],
        }],
        sincos: false,
    };
    recorded_writes("effects.draw_shadow.corners_in_x87", &SPEC, |b| {
        with_shadow(b[0], SET_2, super::corners_in_x87);
    });
}

#[test]
fn projected_x87_matches() {
    const SPEC: Spec = Spec {
        operands: &[
            Op::RaceBlock {
                reads: &[(0x224088, F32), (0x22409C, F32), (0x2240A0, F32)],
                writes: &[],
            },
            Op::Base {
                lo: 0x1CEF90,
                hi: 0x1CFC90,
                reads: &[
                    (0x1CF058, F32),
                    (0x1CF05C, F32),
                    (0x1CF060, F32),
                    (0x1CF064, F32),
                    (0x1CF068, F32),
                    (0x1CF06C, F32),
                    (0x1CF070, F32),
                    (0x1CF074, F32),
                    (0x1CF078, F32),
                    (0x1CF07C, F32),
                    (0x1CF080, F32),
                    (0x1CF084, F32),
                ],
                writes: &[
                    (0x1CF058, F32),
                    (0x1CF05C, F32),
                    (0x1CF064, F32),
                    (0x1CF068, F32),
                    (0x1CF070, F32),
                    (0x1CF074, F32),
                    (0x1CF07C, F32),
                    (0x1CF080, F32),
                ],
            },
        ],
        sincos: false,
    };
    recorded_writes("effects.draw_shadow.projected_x87", &SPEC, |b| {
        with_shadow(b[1], SET_2, |s| {
            super::projected_x87(
                crate::owners::Owners::the()
                    .scene
                    .scratch
                    .front_end_polygon(),
                s,
            );
        });
    });
}

#[test]
fn light_turned_2_x87_matches() {
    const SPEC: Spec = Spec {
        operands: &[
            Op::Base {
                lo: 0x1CE274,
                hi: 0x1CEF90,
                reads: &[(0x1CEF74, F32), (0x1CEF78, F32), (0x1CEF7C, F32)],
                writes: &[(0x1CE284, F32), (0x1CE288, F32), (0x1CE28C, F32)],
            },
            Op::Base {
                lo: 0x0,
                hi: 0x24,
                reads: &[
                    (0x0, F32),
                    (0x4, F32),
                    (0x8, F32),
                    (0xC, F32),
                    (0x10, F32),
                    (0x14, F32),
                    (0x18, F32),
                    (0x1C, F32),
                    (0x20, F32),
                ],
                writes: &[],
            },
            Op::Base {
                lo: 0x0,
                hi: 0x8,
                reads: &[(0x0, F32), (0x4, F32)],
                writes: &[(0x0, F32), (0x4, F32)],
            },
        ],
        sincos: false,
    };
    recorded_writes("effects.draw_shadow.light_turned_2_x87", &SPEC, |b| {
        with_shadow(b[0], SET_1, |s| {
            with_matrix(b[1], b[2], |m, t| super::light_turned_x87(s, m, t));
        });
    });
}

#[test]
fn second_turned_2_x87_matches() {
    const SPEC: Spec = Spec {
        operands: &[
            Op::Base {
                lo: 0x1CE274,
                hi: 0x1CEF90,
                reads: &[(0x1CE284, F32), (0x1CE288, F32), (0x1CE28C, F32)],
                writes: &[(0x1CE2F0, F32), (0x1CE2F4, F32), (0x1CE2F8, F32)],
            },
            Op::Base {
                lo: 0x0,
                hi: 0x24,
                reads: &[
                    (0x0, F32),
                    (0x4, F32),
                    (0x8, F32),
                    (0xC, F32),
                    (0x10, F32),
                    (0x14, F32),
                    (0x18, F32),
                    (0x1C, F32),
                    (0x20, F32),
                ],
                writes: &[],
            },
            Op::Base {
                lo: 0x0,
                hi: 0x8,
                reads: &[(0x0, F32), (0x4, F32)],
                writes: &[(0x0, F32), (0x4, F32)],
            },
        ],
        sincos: false,
    };
    recorded_writes("effects.draw_shadow.second_turned_2_x87", &SPEC, |b| {
        with_shadow(b[0], SET_1, |s| {
            with_matrix(b[1], b[2], |m, t| super::second_turned_x87(s, m, t));
        });
    });
}

#[test]
fn corners_2_x87_matches() {
    const SPEC: Spec = Spec {
        operands: &[Op::Base {
            lo: 0x1CE274,
            hi: 0x1CEF90,
            reads: &[
                (0x1CE274, F32),
                (0x1CE278, F32),
                (0x1CE27C, F32),
                (0x1CE294, F32),
                (0x1CE2A0, F32),
                (0x1CE2AC, F32),
                (0x1CE2E4, F32),
                (0x1CE2E8, F32),
                (0x1CE2EC, F32),
                (0x1CE2F0, F32),
                (0x1CE2F4, F32),
                (0x1CE2F8, F32),
                (0x1CE30C, F32),
                (0x1CE310, F32),
                (0x1CE33C, F32),
                (0x1CE340, F32),
                (0x1CE344, F32),
                (0x1CE348, F32),
                (0x1CE34C, F32),
                (0x1CE350, F32),
                (0x1CE354, F32),
                (0x1CE358, F32),
                (0x1CE35C, F32),
                (0x1CE360, F32),
                (0x1CE364, F32),
                (0x1CE368, F32),
                (0x1CEF30, F32),
            ],
            writes: &[
                (0x1CE2E4, F32),
                (0x1CE2E8, F32),
                (0x1CE2EC, F32),
                (0x1CE2F0, F32),
                (0x1CE2F4, F32),
                (0x1CE2F8, F32),
                (0x1CE330, F32),
                (0x1CE334, F32),
                (0x1CE338, F32),
                (0x1CE33C, F32),
                (0x1CE340, F32),
                (0x1CE344, F32),
                (0x1CE348, F32),
                (0x1CE34C, F32),
                (0x1CE350, F32),
                (0x1CE354, F32),
                (0x1CE358, F32),
                (0x1CE35C, F32),
                (0x1CE360, F32),
                (0x1CE364, F32),
                (0x1CE368, F32),
            ],
        }],
        sincos: false,
    };
    recorded_writes("effects.draw_shadow.corners_2_x87", &SPEC, |b| {
        with_shadow(b[0], SET_1, super::corners_x87);
    });
}

#[test]
fn corners_in_2_x87_matches() {
    const SPEC: Spec = Spec {
        operands: &[Op::Base {
            lo: 0x1CE274,
            hi: 0x1CEF90,
            reads: &[
                (0x1CE32C, F32),
                (0x1CE330, F32),
                (0x1CE334, F32),
                (0x1CE338, F32),
                (0x1CE33C, F32),
                (0x1CE340, F32),
                (0x1CE344, F32),
                (0x1CE348, F32),
                (0x1CE34C, F32),
                (0x1CE350, F32),
                (0x1CE354, F32),
                (0x1CE358, F32),
                (0x1CE35C, F32),
                (0x1CE360, F32),
                (0x1CE364, F32),
                (0x1CE368, F32),
            ],
            writes: &[
                (0x1CE330, F32),
                (0x1CE334, F32),
                (0x1CE338, F32),
                (0x1CE33C, F32),
                (0x1CE340, F32),
                (0x1CE344, F32),
                (0x1CE348, F32),
                (0x1CE34C, F32),
                (0x1CE350, F32),
                (0x1CE354, F32),
                (0x1CE358, F32),
                (0x1CE35C, F32),
                (0x1CE360, F32),
                (0x1CE364, F32),
                (0x1CE368, F32),
            ],
        }],
        sincos: false,
    };
    recorded_writes("effects.draw_shadow.corners_in_2_x87", &SPEC, |b| {
        with_shadow(b[0], SET_1, super::corners_in_x87);
    });
}

#[test]
fn projected_2_x87_matches() {
    const SPEC: Spec = Spec {
        operands: &[
            Op::RaceBlock {
                reads: &[(0x224088, F32), (0x22409C, F32), (0x2240A0, F32)],
                writes: &[],
            },
            Op::Base {
                lo: 0x1CE274,
                hi: 0x1CEF90,
                reads: &[
                    (0x1CE33C, F32),
                    (0x1CE340, F32),
                    (0x1CE344, F32),
                    (0x1CE348, F32),
                    (0x1CE34C, F32),
                    (0x1CE350, F32),
                    (0x1CE354, F32),
                    (0x1CE358, F32),
                    (0x1CE35C, F32),
                    (0x1CE360, F32),
                    (0x1CE364, F32),
                    (0x1CE368, F32),
                ],
                writes: &[
                    (0x1CE33C, F32),
                    (0x1CE340, F32),
                    (0x1CE348, F32),
                    (0x1CE34C, F32),
                    (0x1CE354, F32),
                    (0x1CE358, F32),
                    (0x1CE360, F32),
                    (0x1CE364, F32),
                ],
            },
        ],
        sincos: false,
    };
    recorded_writes("effects.draw_shadow.projected_2_x87", &SPEC, |b| {
        with_shadow(b[1], SET_1, |s| {
            super::projected_x87(
                crate::owners::Owners::the()
                    .scene
                    .scratch
                    .front_end_polygon(),
                s,
            );
        });
    });
}
