//! The two championships' driver (converted 2026-10-21 from `menu::run_championship`
//! 0x4A0E60 and `menu::run_a8_championship` 0x4AA4D7): a whole championship from its
//! start or a saved game, section by section and record by record of `g_recordTable`,
//! with the stages' races, results, services and standings, to its finish. Its steps in
//! the originals' order, on `flow::Game`; `a8` for the A8's differences.

use super::flow::{Call, Game, Var};

/// The records' kinds (their first dword).
pub const STAGE: u32 = 0;
pub const SERVICE: u32 = 1;
pub const THE_END: u32 = 3;
pub const SECTION_END: u32 = 4;
pub const WELCOME: u32 = 5;

/// An unknown record kind: fatal error 0x8070DBA0 "Invalid Level Type" (the A8's
/// 0x8070DBA9 "Invalid Level Type(A8)").
pub fn invalid_level(a8: bool) -> (u32, &'static str) {
    if a8 {
        (0x8070_DBA9, "Invalid Level Type(A8)")
    } else {
        (0x8070_DBA0, "Invalid Level Type")
    }
}

/// Where the driver is.
enum At {
    Section,
    Record,
    Standings,
    RallyResults,
    Races,
    Finish,
    End,
}

/// The calls of `list` in order.
fn calls<G: Game>(e: &mut G, list: &[Call]) -> Result<(), G::Error> {
    for &c in list {
        e.run(c)?;
    }
    Ok(())
}

/// The calls of `list`, then whether a yes was given.
fn screens<G: Game>(e: &mut G, list: &[Call]) -> Result<bool, G::Error> {
    calls(e, list)?;
    Ok(e.yes())
}

/// A yes on a stage's screens: the times reset, on to the rally's results.
fn reset<G: Game>(e: &mut G) -> Result<At, G::Error> {
    e.set(Var::ResetTimes, 1);
    e.run(Call::ResetPlayerTimes)?;
    e.set(Var::Answer, 0);
    Ok(At::RallyResults)
}

/// `reset`, and the event does not go on (a yes after the stage's results).
fn stop<G: Game>(e: &mut G) -> Result<At, G::Error> {
    reset(e)?;
    e.set(Var::GoesOn, 0);
    Ok(At::RallyResults)
}

/// The rally's results after the overall standings (the A8 has only the class pass).
const OVERALL_RALLY: [Call; 3] = [
    Call::ClearStandingsA,
    Call::NumberAllA,
    Call::ShowOverallRallyResults,
];

/// The whole championship. Ends with the in-game options shown from the entry's registers.
pub fn run<G: Game>(e: &mut G, a8: bool) -> Result<(), G::Error> {
    let start_section = Var::StartSection { a8 };
    let next_section = if a8 {
        Call::ResetSingleEvent
    } else {
        Call::BeginEvent
    };
    if e.get(Var::Continuing) == 1 {
        e.set(Var::StagesDone, 2);
    }
    e.set(Var::ResetTimes, 0);
    if e.get(Var::Continuing) != 1 {
        calls(
            e,
            &[
                Call::BeginEvent,
                Call::ShuffleDrivers,
                Call::ClearResultSlots,
                Call::ResetDrivers,
                Call::ResetDriverCondition,
            ],
        )?;
        if a8 {
            e.run(Call::ClearHudBits)?;
            e.set(Var::Services, 0);
        } else {
            e.set(Var::Services, 0);
            e.run(Call::ClearHudBits)?;
        }
    }
    e.run(Call::TakePlayer1Units)?;
    e.run(Call::HideAllInGameOptions)?;
    e.set(Var::Answer, 0);
    e.set(Var::LeaveScreen, 0);
    e.set(Var::EventLeft, 0);
    e.set(Var::Mode, if a8 { 0x13 } else { 2 });
    e.set(Var::ModeAB40, if a8 { 1 } else { 2 });
    if e.get(start_section) == 0 {
        let welcome = if e.get(Var::Mode) == 2 {
            Call::ChampionshipWelcome
        } else {
            Call::A8Welcome
        };
        e.run(welcome)?;
    }
    e.run(Call::StopStream)?;
    let mut at = if e.yes() {
        At::End
    } else {
        if !a8 {
            e.set(Var::Continuing, 0);
        }
        At::Section
    };
    loop {
        at = match at {
            At::Section => {
                for v in [
                    Var::EscapePressed,
                    Var::ServiceTime,
                    Var::ServiceDue,
                    Var::TheEnd,
                ] {
                    e.set(v, 0);
                }
                e.set(Var::GoesOn, 1);
                e.set(Var::ResetTimes, 0);
                let given = e.get(start_section);
                if given != 0 {
                    let section = e.get(start_section).wrapping_sub(1);
                    e.set(Var::Section, section);
                }
                At::Record
            }
            At::Record => {
                e.set(Var::RallyDone, 0);
                let section = e.get(Var::Section);
                let index = e.get(Var::Index);
                e.run(Call::FindRecord { section, index })?;
                match e.get(Var::Record(0)) {
                    THE_END => {
                        e.set(Var::TheEnd, 1);
                        At::Standings
                    }
                    WELCOME => {
                        let figure = e.get(Var::Record(0x38));
                        e.set(Var::WelcomeFigure, figure);
                        e.inc(Var::Index);
                        e.set(Var::Welcomed, 1);
                        e.run(next_section)?;
                        e.run(Call::WelcomeLoop)?;
                        if e.yes() { At::End } else { At::Record }
                    }
                    SECTION_END => At::Standings,
                    SERVICE => {
                        e.set(Var::ServiceDue, 1);
                        let time = e.get(Var::Record(0x14));
                        e.set(Var::ServiceTime, time);
                        let figure = e.get(Var::Record(0xC));
                        e.set(Var::ServiceRecordC, figure);
                        e.set(Var::CpuSpreadFirst, 1);
                        e.set(Var::ServiceNext, 1);
                        e.inc(Var::Services);
                        e.inc(Var::Index);
                        At::Record
                    }
                    STAGE => 'stage: {
                        for screen in [
                            Call::RallyStagesLoop,
                            Call::NextStageLoop,
                            Call::ServicePass,
                        ] {
                            e.run(screen)?;
                            if e.yes() {
                                break 'stage reset(e)?;
                            }
                        }
                        if e.get(Var::GoesOn) == 1 {
                            At::Races
                        } else {
                            At::RallyResults
                        }
                    }
                    _ => {
                        let (code, message) = invalid_level(a8);
                        return Err(e.fatal(code, message));
                    }
                }
            }
            At::Standings => 'standings: {
                if screens(
                    e,
                    &[
                        Call::StandingsBFromAll,
                        Call::SortBByFigure,
                        Call::AwardStagePoints,
                        Call::SortBByFigure,
                        Call::SortBByPoints,
                        Call::ClassChampionshipPass,
                    ],
                )? {
                    break 'standings At::End;
                }
                if e.get(start_section) != 0 {
                    e.set(Var::TheEnd, 1);
                    break 'standings At::Finish;
                }
                if !a8
                    && screens(
                        e,
                        &[
                            Call::SortBByFigure,
                            Call::AwardOverallPoints,
                            Call::ClearStandingsB,
                            Call::SortBByFigure,
                            Call::SortBByScore,
                            Call::NumberAllB,
                            Call::ShowChampionshipStandings,
                        ],
                    )?
                {
                    break 'standings At::End;
                }
                if e.get(Var::TheEnd) == 1 {
                    break 'standings At::Finish;
                }
                if screens(e, &[Call::SectionFinish])? {
                    break 'standings At::End;
                }
                e.run(Call::ChampionshipStandingsReview)?;
                e.set(Var::ResetTimes, 0);
                e.set(Var::GoesOn, 1);
                if e.yes() || e.get(Var::AskLeave) == 1 || e.get(Var::EscapePressed) == 1 {
                    break 'standings At::End;
                }
                e.inc(Var::Section);
                e.set(Var::Index, 0);
                e.run(next_section)?;
                At::Section
            }
            At::RallyResults => 'results: {
                e.set(Var::RallyDone, 1);
                for v in [
                    Var::EventLeft,
                    Var::LeaveScreen,
                    Var::LeftByKey,
                    Var::GoesOn,
                ] {
                    e.set(v, 0);
                }
                if screens(
                    e,
                    &[
                        Call::WalkSectionRecords,
                        Call::StandingsAFromAll,
                        Call::ClassRallyResultsPass,
                    ],
                )? {
                    break 'results At::End;
                }
                if !a8 && screens(e, &OVERALL_RALLY)? {
                    break 'results At::End;
                }
                At::Standings
            }
            At::Races => 'races: {
                if !a8 && e.get(Var::ResetTimes) == 1 {
                    break 'races At::RallyResults;
                }
                e.run(Call::RunEventRaces)?;
                if e.get(Var::GoesOn) == 0 {
                    break 'races At::RallyResults;
                }
                if screens(
                    e,
                    &[
                        Call::AddPlayerTimes,
                        Call::StandingsAFromAll,
                        Call::ClassStageResultsPass,
                    ],
                )? {
                    break 'races reset(e)?;
                }
                if !a8
                    && screens(
                        e,
                        &[
                            Call::ClearStandingsA,
                            Call::NumberAllA,
                            Call::ShowOverallStageResults,
                        ],
                    )?
                {
                    break 'races reset(e)?;
                }
                if e.get(Var::StagesDone) as i32 >= 1 {
                    let next = e.next_record_kind()?;
                    if next == THE_END || next == SECTION_END {
                        e.set(Var::RallyDone, 1);
                    }
                    if screens(e, &[Call::StandingsAFromAll, Call::ClassRallyResultsPass])? {
                        break 'races stop(e)?;
                    }
                    if !a8 && screens(e, &OVERALL_RALLY)? {
                        break 'races stop(e)?;
                    }
                }
                e.run(Call::KeepRecordBest)?;
                e.run(Call::RecordEntriesPass)?;
                if e.get(Var::Entered) == 1 && screens(e, &[Call::BestTimesLoop])? {
                    break 'races stop(e)?;
                }
                let reviewed = match e.next_record_kind()? {
                    THE_END => false,
                    SECTION_END => break 'races At::Standings,
                    SERVICE => {
                        e.set(Var::CpuSpreadFirst, 1);
                        e.set(Var::ServiceNext, 1);
                        true
                    }
                    _ => {
                        e.set(Var::ServiceNext, 0);
                        if screens(e, &[Call::InspectionPass])? {
                            break 'races stop(e)?;
                        }
                        if e.get(Var::GoesOn) == 0 {
                            break 'races At::RallyResults;
                        }
                        true
                    }
                };
                if reviewed {
                    let review = if e.get(Var::ReviewKind) != 0 || e.get(Var::ServiceNext) == 1 {
                        Call::ChampionshipServiceReview
                    } else {
                        Call::ChampionshipRacesReview
                    };
                    e.run(review)?;
                    if e.yes() || e.get(Var::EscapePressed) == 1 || e.get(Var::AskLeave) == 1 {
                        break 'races At::End;
                    }
                }
                e.inc(Var::Index);
                e.inc(Var::StagesDone);
                if !a8 {
                    e.set(Var::Services, 0);
                }
                At::Record
            }
            At::Finish => 'finish: {
                if e.get(start_section) == 0 {
                    if screens(e, &[Call::ChampionshipFinish])? {
                        break 'finish At::End;
                    }
                    if screens(e, &[Call::ClearScoreMarks, Call::BestClassScores])? {
                        break 'finish At::End;
                    }
                    if !a8 {
                        e.run(Call::BestOverallScores)?;
                    }
                }
                e.run(Call::ChampionshipEndReview)?;
                At::End
            }
            At::End => {
                return e.run(Call::ShowInGameOptions);
            }
        };
    }
}

#[cfg(test)]
mod tests {
    use super::super::flow::Var;
    use super::super::flow::log::Log;
    use super::super::flow::log::Said;
    use super::*;

    #[test]
    fn welcome_yes_ends() {
        let mut log = Log::default().on("ChampionshipWelcome", vec![vec![(Var::Answer, 1)]]);
        run(&mut log, false).unwrap();
        assert_eq!(log.steps.last().unwrap(), "ShowInGameOptions");
        assert!(!log.has("Continuing=0"));
    }

    #[test]
    fn stage_then_end() {
        // Record 0 a stage, the next an end (the log's records all read 0 but for the
        // kind written by FindRecord's script).
        let mut log = Log::default()
            .with(Var::StagesDone, 1)
            .on(
                "FindRecord",
                vec![
                    vec![(Var::Record(0), STAGE)],
                    vec![(Var::Record(0), THE_END)],
                ],
            )
            .reply(
                "NextRecordKind",
                vec![Said::Kind(THE_END), Said::Kind(THE_END)],
            );
        run(&mut log, false).unwrap();
        assert!(log.has("RunEventRaces"));
        assert!(log.has("RallyDone=1"));
        assert!(log.has("ChampionshipFinish"));
        assert!(log.has("BestOverallScores"));
        assert!(log.has("StagesDone=2"));
        assert!(log.has("Services=0"));

        let mut log = Log::default()
            .with(Var::StartSection { a8: true }, 3)
            .on("FindRecord", vec![vec![(Var::Record(0), 7)]]);
        assert_eq!(
            run(&mut log, true),
            Err(crate::Invalid("Invalid Level Type(A8)"))
        );
        assert!(log.has("Section=2"));
        assert!(!log.has("A8Welcome"));
    }
}
