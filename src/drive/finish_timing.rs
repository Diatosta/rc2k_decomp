//! `drive::finish_timing` (0x45948A): the finish routine's time and shares. The clock
//! (`scene::restart_clock`) less the start `start_clock` / `thousand` into `time` (t); the
//! eased time `eased`: t below `limit_a` (or unordered) t; below `limit_b` t less ((t -
//! a) / (b - a))^2 x (b - a) / 2; else a + (b - a) / 2; the share `share`: 0, with
//! `dword_75B108` and t from `limit_c` on 1, below `limit_d` (t - c) / (d - c); and with
//! `dword_75B108` and t from `limit_c` on, `race_done` 1.
//!
//! Returns: the clock less the start
//!
//! Caller: `drive::finish_run`.

use crate::math::x87::loaded;
use crate::math::x87_float::int;
use crate::math::x87_float::ld;
use crate::math::x87_float::st;
use crate::physics::flow::c0;
use crate::race::effects_settings_store::EFFECTS_SETTINGS;
use crate::race::owner::Race;
use crate::race::owner_replay::Replay;
use crate::scene::owner::scene;

/// `time` = the clock less the start (`scratch`) over `thousand`.
fn time(race: &Race) {
    race.replay
        .finish
        .time
        .set_bits(st(int(scene().drawing.scene.scratch.get() as i32)
            / ld(race.course.numbers.thousand.bits())));
}

/// The eased time between the two limits: t less ((t - a) / (b - a))^2 x ((b - a) x half).
fn eased_middle(race: &Race) {
    let q = (ld(race.replay.finish.time.bits()) - ld(race.replay.finish.limit_a.bits()))
        / (ld(race.replay.finish.limit_b.bits()) - ld(race.replay.finish.limit_a.bits()));
    let square = q * q;
    let span = (ld(race.replay.finish.limit_b.bits()) - ld(race.replay.finish.limit_a.bits()))
        * ld(race.course.numbers.half.bits());
    let product = square * span;
    race.replay
        .finish
        .eased
        .set_bits(st(-product + ld(race.replay.finish.time.bits())));
}

/// The eased time past the second limit: a + (b - a) x half.
fn eased_end(race: &Race) {
    let span = (ld(race.replay.finish.limit_b.bits()) - ld(race.replay.finish.limit_a.bits()))
        * ld(race.course.numbers.half.bits());
    race.replay
        .finish
        .eased
        .set_bits(st(span + ld(race.replay.finish.limit_a.bits())));
}

/// The share between the last two limits: (t - c) / (d - c).
fn share(replay: &Replay) {
    let q = (ld(replay.finish.time.bits()) - ld(replay.finish.limit_c.bits()))
        / (ld(replay.finish.limit_d.bits()) - ld(replay.finish.limit_c.bits()));
    replay.finish.share.set_bits(st(q));
}

/// The core from the clock's reading `now` (already in `g_sceneClock`).
pub fn timing_from(race: &Race, now: u32, scene: &crate::scene::owner::Scene) {
    let replay = &race.replay;
    let since = now.wrapping_sub(replay.finish.start_clock.get());
    scene.drawing.scene.scratch.set(since);
    time(race);
    if c0(replay.finish.time.bits(), replay.finish.limit_a.bits()) {
        replay
            .finish
            .eased
            .set_bits(loaded(&replay.finish.time).to_bits());
    } else if c0(replay.finish.time.bits(), replay.finish.limit_b.bits()) {
        eased_middle(race);
    } else {
        eased_end(race);
    }
    replay.finish.share.set_bits(0);
    let late = !c0(replay.finish.time.bits(), replay.finish.limit_c.bits());
    if race.shared.state.results.finished.get() != 0 && late {
        replay.finish.share.set_bits(0x3F80_0000);
        if c0(replay.finish.time.bits(), replay.finish.limit_d.bits()) {
            share(&race.replay);
        }
    }
    if late && race.shared.state.results.finished.get() != 0 {
        EFFECTS_SETTINGS.race_done.set(1);
    }
}

/// (the exact one, for `drive::finish_run`).
pub fn finish_timing(race: &Race, scene: &crate::scene::owner::Scene) {
    let now = crate::scene::restart_clock::restart_clock(
        &race.shared,
        crate::system::tick_count::tick_count(),
    );
    timing_from(race, now, scene);
}

#[cfg(all(test, target_arch = "x86"))]
mod asm_reference {
    use super::*;
    use crate::race::owner::race;
    use crate::scene::asm_test::{Block, In, Operand, Out, reference, sparse};

    // The original's instructions, one function per run.
    reference!(ref_time(rc)
        "fild dword ptr [{rc} + 0x22400C]",
        "fdiv dword ptr [{rc} + 0x1C5D18]",
        "fstp dword ptr [{rc} + 0x24BB64]");
    reference!(ref_middle(rc)
        "fld dword ptr [{rc} + 0x24BB64]",
        "fsub dword ptr [{rc} + 0x24BB44]",
        "fld dword ptr [{rc} + 0x24BB48]",
        "fsub dword ptr [{rc} + 0x24BB44]",
        ".byte 0xDE, 0xF9", // fdivp st(1), st
        ".byte 0xD8, 0xC8", // fmul st, st
        "fld dword ptr [{rc} + 0x24BB48]",
        "fsub dword ptr [{rc} + 0x24BB44]",
        "fmul dword ptr [{rc} + 0x1C5E40]",
        ".byte 0xDE, 0xC9", // fmulp st(1), st
        ".byte 0xD9, 0xE0", // fchs
        "fadd dword ptr [{rc} + 0x24BB64]",
        "fstp dword ptr [{rc} + 0x24BB68]");
    reference!(ref_end(rc)
        "fld dword ptr [{rc} + 0x24BB48]",
        "fsub dword ptr [{rc} + 0x24BB44]",
        "fmul dword ptr [{rc} + 0x1C5E40]",
        "fadd dword ptr [{rc} + 0x24BB44]",
        "fstp dword ptr [{rc} + 0x24BB68]");
    reference!(ref_share(rc)
        "fld dword ptr [{rc} + 0x24BB64]",
        "fsub dword ptr [{rc} + 0x24BB70]",
        "fld dword ptr [{rc} + 0x24BB74]",
        "fsub dword ptr [{rc} + 0x24BB70]",
        ".byte 0xDE, 0xF9", // fdivp st(1), st
        "fstp dword ptr [{rc} + 0x24BB6C]");

    fn blocks() -> Vec<Block> {
        vec![
            Block {
                name: "finish_timing time",
                operands: vec![Operand::RaceBlock {
                    reads: &[(0x22400C, In::I), (0x1C5D18, In::F)],
                    writes: &[(0x24BB64, Out::F)],
                }],
                ulps: 0,
                reference: |a| unsafe { ref_time(a[0]) },
                plain: |_| time(race()),
            },
            Block {
                name: "finish_timing middle",
                operands: vec![Operand::RaceBlock {
                    reads: &[
                        (0x24BB64, In::F),
                        (0x24BB44, In::F),
                        (0x24BB48, In::F),
                        (0x1C5E40, In::F),
                    ],
                    writes: &[(0x24BB68, Out::F)],
                }],
                ulps: 0,
                reference: |a| unsafe { ref_middle(a[0]) },
                plain: |_| eased_middle(race()),
            },
            Block {
                name: "finish_timing end",
                operands: vec![Operand::RaceBlock {
                    reads: &[(0x24BB48, In::F), (0x24BB44, In::F), (0x1C5E40, In::F)],
                    writes: &[(0x24BB68, Out::F)],
                }],
                ulps: 0,
                reference: |a| unsafe { ref_end(a[0]) },
                plain: |_| eased_end(race()),
            },
            Block {
                name: "finish_timing share",
                operands: vec![Operand::RaceBlock {
                    reads: &[(0x24BB64, In::F), (0x24BB70, In::F), (0x24BB74, In::F)],
                    writes: &[(0x24BB6C, Out::F)],
                }],
                ulps: 0,
                reference: |a| unsafe { ref_share(a[0]) },
                plain: |_| share(&race().replay),
            },
        ]
    }

    #[test]
    fn finish_timing_matches_the_assembly() {
        sparse(&blocks(), 3000, 202);
    }
}
