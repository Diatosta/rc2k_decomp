//! `drive::finish_timing` (0x45948A): the finish routine's time and shares. The clock
//! (`scene::restart_clock`) less the start `start_clock` / `thousand` into `time` (t); the
//! eased time `eased`: t below `limit_a` (or unordered) t; below `limit_b` t less ((t -
//! a) / (b - a))^2 x (b - a) / 2; else a + (b - a) / 2; the share `share`: 0, with
//! `RaceState.results.finished` and t from `limit_c` on 1, below `limit_d` (t - c) / (d - c); and with
//! `RaceState.results.finished` and t from `limit_c` on, `race_done` 1.
//!
//! Returns: the clock less the start
//!
//! Caller: `drive::finish_run`.

use crate::math::x87::loaded;
use crate::math::x87_float::int;
use crate::math::x87_float::ld;
use crate::math::x87_float::stf;
use crate::physics::flow::c0;
use crate::race::owner::Race;
use crate::race::owner_replay::Replay;

/// `time` = the clock less the start (`scratch`) over `thousand`.
fn time(race: &Race, scene: &crate::scene::owner::Scene) {
    race.replay.finish.time.set(stf(
        int(scene.drawing.scene.scratch.get() as i32) / ld(race.course.numbers.thousand.get())
    ));
}

/// The eased time between the two limits: t less ((t - a) / (b - a))^2 x ((b - a) x half).
fn eased_middle(race: &Race) {
    let q = (ld(race.replay.finish.time.get()) - ld(race.replay.finish.limit_a.get()))
        / (ld(race.replay.finish.limit_b.get()) - ld(race.replay.finish.limit_a.get()));
    let square = q * q;
    let span = (ld(race.replay.finish.limit_b.get()) - ld(race.replay.finish.limit_a.get()))
        * ld(race.course.numbers.half.get());
    let product = square * span;
    race.replay
        .finish
        .eased
        .set(stf(-product + ld(race.replay.finish.time.get())));
}

/// The eased time past the second limit: a + (b - a) x half.
fn eased_end(race: &Race) {
    let span = (ld(race.replay.finish.limit_b.get()) - ld(race.replay.finish.limit_a.get()))
        * ld(race.course.numbers.half.get());
    race.replay
        .finish
        .eased
        .set(stf(span + ld(race.replay.finish.limit_a.get())));
}

/// The share between the last two limits: (t - c) / (d - c).
fn share(replay: &Replay) {
    let q = (ld(replay.finish.time.get()) - ld(replay.finish.limit_c.get()))
        / (ld(replay.finish.limit_d.get()) - ld(replay.finish.limit_c.get()));
    replay.finish.share.set(stf(q));
}

/// The core from the clock's reading `now` (already in `g_sceneClock`).
pub fn timing_from(race: &Race, now: u32, scene: &crate::scene::owner::Scene) {
    let replay = &race.replay;
    let since = now.wrapping_sub(replay.finish.start_clock.get());
    scene.drawing.scene.scratch.set(since);
    time(race, scene);
    if c0(replay.finish.time.get(), replay.finish.limit_a.get()) {
        replay.finish.eased.set(loaded(&replay.finish.time));
    } else if c0(replay.finish.time.get(), replay.finish.limit_b.get()) {
        eased_middle(race);
    } else {
        eased_end(race);
    }
    replay.finish.share.set(0.0);
    let late = !c0(replay.finish.time.get(), replay.finish.limit_c.get());
    if race.shared.state.results.finished.get() != 0 && late {
        replay.finish.share.set(1.0);
        if c0(replay.finish.time.get(), replay.finish.limit_d.get()) {
            share(&race.replay);
        }
    }
    if late && race.shared.state.results.finished.get() != 0 {
        race.effects_settings().race_done.set(1);
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

#[cfg(test)]
mod reference_vectors {
    use super::*;
    use crate::race::owner::race;
    use crate::test_support::vectors::block_vectors::{Block, In, Operand, Out, sparse};

    fn blocks() -> Vec<Block> {
        vec![
            Block {
                name: "finish_timing time",
                operands: vec![Operand::RaceBlock {
                    reads: &[(0x22400C, In::I), (0x1C5D18, In::F)],
                    writes: &[(0x24BB64, Out::F)],
                }],
                ulps: 0,
                plain: |_| time(race(), crate::scene::owner::scene()),
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
                plain: |_| eased_middle(race()),
            },
            Block {
                name: "finish_timing end",
                operands: vec![Operand::RaceBlock {
                    reads: &[(0x24BB48, In::F), (0x24BB44, In::F), (0x1C5E40, In::F)],
                    writes: &[(0x24BB68, Out::F)],
                }],
                ulps: 0,
                plain: |_| eased_end(race()),
            },
            Block {
                name: "finish_timing share",
                operands: vec![Operand::RaceBlock {
                    reads: &[(0x24BB64, In::F), (0x24BB70, In::F), (0x24BB74, In::F)],
                    writes: &[(0x24BB6C, Out::F)],
                }],
                ulps: 0,
                plain: |_| share(&race().replay),
            },
        ]
    }

    #[test]
    fn finish_timing_matches_the_assembly() {
        sparse("drive.finish_timing", &blocks(), 3000, 202);
    }
}
