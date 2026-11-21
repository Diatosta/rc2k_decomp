//! The finish routine's time and shares. The clock
//! (`scene::restart_clock`) less the start `start_clock` / `thousand` into `time` (t); the
//! eased time `eased`: t below `limit_a` (or unordered) t; below `limit_b` t less ((t -
//! a) / (b - a))^2 x (b - a) / 2; else a + (b - a) / 2; the share `share`: 0, with
//! `RaceState.results.finished` and t from `limit_c` on 1, below `limit_d` (t - c) / (d - c); and with
//! `RaceState.results.finished` and t from `limit_c` on, `race_done` 1.
//!
//! Gives the clock less the start.

use crate::drive::finish_store::Finish;
use crate::math::x87::loaded;
use crate::math::x87_float::int;
use crate::math::x87_float::ld;
use crate::math::x87_float::stf;
use crate::physics::flow::c0;
use crate::race::owner::Race;

/// `time`: the clock less the start (`since`) over `thousand`.
fn time_from(finish: &mut Finish, since: u32, thousand: f32) {
    finish.time = stf(int(since as i32) / ld(thousand));
}

/// The eased time between the two limits: t less ((t - a) / (b - a))^2 x ((b - a) x half).
fn eased_middle_from(finish: &mut Finish, half: f32) {
    let q = (ld(finish.time) - ld(finish.limit_a)) / (ld(finish.limit_b) - ld(finish.limit_a));
    let square = q * q;
    let span = (ld(finish.limit_b) - ld(finish.limit_a)) * ld(half);
    let product = square * span;
    finish.eased = stf(-product + ld(finish.time));
}

/// The eased time past the second limit: a + (b - a) x half.
fn eased_end_from(finish: &mut Finish, half: f32) {
    let span = (ld(finish.limit_b) - ld(finish.limit_a)) * ld(half);
    finish.eased = stf(span + ld(finish.limit_a));
}

/// The share between the last two limits: (t - c) / (d - c).
fn share_from(finish: &mut Finish) {
    let q = (ld(finish.time) - ld(finish.limit_c)) / (ld(finish.limit_d) - ld(finish.limit_c));
    finish.share = stf(q);
}

/// The core from the clock's reading `now` (already in the scene clock).
fn timing_from(race: &Race, now: u32, scene: &crate::scene::owner::Scene) {
    let (thousand, half) = {
        let numbers = race.course.numbers;
        (numbers.thousand.get(), numbers.half.get())
    };
    let finished = race.shared.state.results.finished.get() != 0;
    let late = {
        let mut finish = race.replay.finish_mut();
        let since = now.wrapping_sub(finish.start_clock);
        scene.draw_work_mut().scratch = since;
        time_from(&mut finish, since, thousand);
        if c0(finish.time, finish.limit_a) {
            finish.eased = loaded(finish.time);
        } else if c0(finish.time, finish.limit_b) {
            eased_middle_from(&mut finish, half);
        } else {
            eased_end_from(&mut finish, half);
        }
        finish.share = 0.0;
        let late = !c0(finish.time, finish.limit_c);
        if finished && late {
            finish.share = 1.0;
            if c0(finish.time, finish.limit_d) {
                share_from(&mut finish);
            }
        }
        late
    };
    if late && finished {
        race.effects_settings().race_done.set(1);
    }
}

/// (the exact one, for `drive::finish_run`).
pub fn finish_timing(
    race: &Race,
    scene: &crate::scene::owner::Scene,
    picture_clock: &crate::clock::tick_count::PictureClock,
) {
    let now = crate::scene::restart_clock::restart_clock(
        &race.shared,
        crate::clock::tick_count::tick_count(picture_clock),
    );
    timing_from(race, now, scene);
}

#[cfg(test)]
mod reference_vectors {
    use super::*;
    use crate::race::owner_replay::Replay;

    /// `time` = the clock less the start (`scratch`) over `thousand`.
    fn time(race: &Race, scene: &crate::scene::owner::Scene) {
        let since = scene.draw_work().scratch;
        let thousand = crate::test_support::race_places::number_constant(".thousand").get();
        time_from(&mut race.replay.finish_mut(), since, thousand);
    }

    /// The eased time between the two limits: t less ((t - a) / (b - a))^2 x ((b - a) x half).
    fn eased_middle(race: &Race) {
        let half = crate::test_support::race_places::number_constant(".half").get();
        eased_middle_from(&mut race.replay.finish_mut(), half);
    }

    /// The eased time past the second limit: a + (b - a) x half.
    fn eased_end(race: &Race) {
        let half = crate::test_support::race_places::number_constant(".half").get();
        eased_end_from(&mut race.replay.finish_mut(), half);
    }

    /// The share between the last two limits: (t - c) / (d - c).
    fn share(replay: &Replay) {
        share_from(&mut replay.finish_mut());
    }

    use crate::test_support::own::race;
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
                plain: |_| time(race(), crate::test_support::own::scene()),
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
