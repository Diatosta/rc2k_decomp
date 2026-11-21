use super::{PlaneTuning, QuadWork};
use crate::effects::asm_layout::{with_laid, with_laid_at};
use crate::effects::asm_test::Kind::*;
use crate::effects::asm_test::{Op, Spec, recorded_writes};
use crate::effects::asm_test::{Operand, floats_at, with_settings};
use crate::effects::typed_sync::{overlay_laid, with_overlays};
use std::cell::RefCell;

/// Where the plane tuning sat in the recorded layout.
const PLANE_TUNING_START: usize = 0x934F30;

use crate::effects::windscreen_flakes_store::{WindscreenFlakeTables, WindscreenFlakeTiming};

/// `f` on the plane tuning made from the floats the test buffer `mk` holds at its place.
fn with_plane_tuning<R>(mk: Operand, f: impl FnOnce(&PlaneTuning) -> R) -> R {
    let v = floats_at::<19>(mk, PLANE_TUNING_START);
    f(&PlaneTuning {
        rate_k: v[0],
        shares: [v[1], v[2]],
        places: [[v[3], v[4]], [v[5], v[6]], [v[7], v[8]], [v[9], v[10]]],
        lifts: [v[11], v[12], v[13], v[14]],
        rhw_k: [v[15], v[16], v[17], v[18]],
    })
}

#[test]
fn plane_axes_x87_matches() {
    const SPEC: Spec = Spec {
        operands: &[
            Op::Base {
                lo: 0x137AF0,
                hi: 0x1D003C,
                reads: &[
                    (0x137FDA, F32),
                    (0x137FDE, F32),
                    (0x137FE2, F32),
                    (0x1CFFF0, F32),
                    (0x1CFFF4, F32),
                ],
                writes: &[
                    (0x137AF4, F32),
                    (0x137FAE, F32),
                    (0x137FB6, F32),
                    (0x137FBA, F32),
                    (0x137FBE, F32),
                    (0x137FDA, F32),
                    (0x137FDE, F32),
                    (0x137FE2, F32),
                ],
            },
            Op::Base {
                lo: 0,
                hi: 0x2768,
                reads: &[
                    (0x2720, I32),
                    (0x2724, I32),
                    (0x2728, I32),
                    (0x272C, I32),
                    (0x2730, I32),
                    (0x2734, I32),
                    (0x2740, I32),
                ],
                writes: &[],
            },
        ],
        sincos: false,
    };
    recorded_writes("effects.draw_plane.plane_axes_x87", &SPEC, |b| {
        with_laid::<RefCell<WindscreenFlakeTiming>, _>(b[0], |sti| {
            with_laid::<std::cell::RefCell<WindscreenFlakeTables>, _>(b[0], |sq| {
                with_plane_tuning(b[0], |pt| {
                    with_settings(b[1], |m| {
                        super::plane_axes_x87(m, &mut sti.borrow_mut(), &mut sq.borrow_mut(), pt);
                    });
                });
            });
        });
    });
}

#[test]
fn plane_up_x87_matches() {
    const SPEC: Spec = Spec {
        operands: &[
            Op::Base {
                lo: 0x137B8A,
                hi: 0x1D003C,
                reads: &[
                    (0x137FE6, F32),
                    (0x137FEA, F32),
                    (0x137FEE, F32),
                    (0x1CFFF8, F32),
                ],
                writes: &[
                    (0x137FB2, F32),
                    (0x137FE6, F32),
                    (0x137FEA, F32),
                    (0x137FEE, F32),
                ],
            },
            Op::Base {
                lo: 0,
                hi: 0x2768,
                reads: &[
                    (0x2708, I32),
                    (0x270C, I32),
                    (0x2710, I32),
                    (0x2720, I32),
                    (0x2724, I32),
                    (0x2728, I32),
                ],
                writes: &[],
            },
        ],
        sincos: false,
    };
    recorded_writes("effects.draw_plane.plane_up_x87", &SPEC, |b| {
        with_laid::<std::cell::RefCell<WindscreenFlakeTables>, _>(b[0], |sq| {
            with_plane_tuning(b[0], |pt| {
                with_settings(b[1], |m| super::plane_up_x87(m, &mut sq.borrow_mut(), pt));
            });
        });
    });
}

#[test]
fn plane_place_x87_matches() {
    const SPEC: Spec = Spec {
        operands: &[
            Op::Base {
                lo: 0xB1252,
                hi: 0xC9402,
                reads: &[],
                writes: &[(0xB1262, F32), (0xB1266, F32), (0xB126A, F32)],
            },
            Op::Base {
                lo: 0,
                hi: 0x2768,
                reads: &[(0x2720, I32), (0x2724, I32), (0x2728, I32)],
                writes: &[],
            },
        ],
        sincos: false,
    };
    recorded_writes("effects.draw_plane.plane_place_x87", &SPEC, |b| {
        with_overlays(
            b[0],
            &[overlay_laid(
                &crate::test_support::own::effects().windscreen_drops.hex,
            )],
            || {
                with_settings(b[1], |m| {
                    super::plane_place_x87(
                        &crate::test_support::own::effects().windscreen_drops,
                        m,
                    );
                });
            },
        );
    });
}

#[test]
fn plane_facing_x87_matches() {
    const SPEC: Spec = Spec {
        operands: &[Op::Base {
            lo: 0xAF7D0,
            hi: 0x13822A,
            reads: &[
                (0xB1262, F32),
                (0xB1266, F32),
                (0xB126A, F32),
                (0x137FF2, F32),
                (0x137FF6, F32),
                (0x137FFA, F32),
            ],
            writes: &[(0xAF7D8, F32)],
        }],
        sincos: false,
    };
    recorded_writes("effects.draw_plane.plane_facing_x87", &SPEC, |b| {
        with_overlays(
            b[0],
            &[
                overlay_laid(&crate::test_support::own::effects().windscreen_drops.state),
                overlay_laid(&crate::test_support::own::effects().windscreen_drops.hex),
            ],
            || {
                with_laid::<std::cell::RefCell<WindscreenFlakeTables>, _>(b[0], |m| {
                    super::plane_facing_x87(
                        &crate::test_support::own::effects().windscreen_drops,
                        &mut m.borrow_mut(),
                    );
                });
            },
        );
    });
}

#[test]
fn plane_corners_x87_matches() {
    const SPEC: Spec = Spec {
        operands: &[Op::Base {
            lo: 0x137B8A,
            hi: 0x1CFD88,
            reads: &[
                (0x137FAE, F32),
                (0x137FB2, F32),
                (0x137FB6, F32),
                (0x137FBA, F32),
                (0x137FBE, F32),
                (0x137FDA, F32),
                (0x137FDE, F32),
                (0x137FE2, F32),
                (0x137FE6, F32),
                (0x137FEA, F32),
                (0x137FEE, F32),
                (0x13802E, F32),
                (0x138032, F32),
                (0x138036, F32),
                (0x13803A, F32),
                (0x13803E, F32),
                (0x138042, F32),
                (0x1CFD7C, F32),
                (0x1CFD80, F32),
                (0x1CFD84, F32),
            ],
            writes: &[
                (0x1CFD58, F32),
                (0x1CFD5C, F32),
                (0x1CFD60, F32),
                (0x1CFD64, F32),
                (0x1CFD68, F32),
                (0x1CFD6C, F32),
                (0x1CFD70, F32),
                (0x1CFD74, F32),
                (0x1CFD78, F32),
                (0x1CFD7C, F32),
                (0x1CFD80, F32),
                (0x1CFD84, F32),
            ],
        }],
        sincos: false,
    };
    recorded_writes("effects.draw_plane.plane_corners_x87", &SPEC, |b| {
        with_laid::<std::cell::RefCell<WindscreenFlakeTables>, _>(b[0], |sq| {
            with_laid_at::<std::cell::RefCell<QuadWork>, _>(b[0], 0x934BD0, |q| {
                super::plane_corners_x87(&mut sq.borrow_mut(), &mut q.borrow_mut());
            });
        });
    });
}

#[test]
fn plane_projected_x87_matches() {
    const SPEC: Spec = Spec {
        operands: &[
            Op::RaceBlock {
                reads: &[(0x224088, F32), (0x22409C, F32), (0x2240A0, F32)],
                writes: &[],
            },
            Op::Base {
                lo: 0x1CFC90,
                hi: 0x1CFD88,
                reads: &[
                    (0x1CFD58, F32),
                    (0x1CFD5C, F32),
                    (0x1CFD60, F32),
                    (0x1CFD64, F32),
                    (0x1CFD68, F32),
                    (0x1CFD6C, F32),
                    (0x1CFD70, F32),
                    (0x1CFD74, F32),
                    (0x1CFD78, F32),
                    (0x1CFD7C, F32),
                    (0x1CFD80, F32),
                    (0x1CFD84, F32),
                ],
                writes: &[
                    (0x1CFD58, F32),
                    (0x1CFD5C, F32),
                    (0x1CFD64, F32),
                    (0x1CFD68, F32),
                    (0x1CFD70, F32),
                    (0x1CFD74, F32),
                    (0x1CFD7C, F32),
                    (0x1CFD80, F32),
                ],
            },
        ],
        sincos: false,
    };
    recorded_writes("effects.draw_plane.plane_projected_x87", &SPEC, |b| {
        with_laid_at::<std::cell::RefCell<QuadWork>, _>(b[1], 0x934BD0, |q| {
            super::plane_projected_x87(
                crate::owners::Owners::the()
                    .scene
                    .scratch
                    .front_end_polygon(),
                &mut q.borrow_mut(),
            );
        });
    });
}
