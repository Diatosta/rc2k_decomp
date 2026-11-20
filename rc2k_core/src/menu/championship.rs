//! The two championships' driver: a whole championship from its start or a saved game, section by
//! section and record by record of the record table, with the stages' races, results, services
//! and standings, to its finish. Its steps, on `flow::Game`; `a8` for the A8's differences.

use super::flow::{Call, Game, Var};
use super::leaving::Switch;

/// The records' kinds (their first dword).
pub const STAGE: u32 = 0;
pub const SERVICE: u32 = 1;
pub(crate) const THE_END: u32 = 3;
pub(crate) const SECTION_END: u32 = 4;
pub const WELCOME: u32 = 5;

/// An unknown record kind: fatal error 0x8070DBA0 "Invalid Level Type" (the A8's
/// 0x8070DBA9 "Invalid Level Type(A8)").
pub(crate) fn invalid_level(a8: bool) -> (u32, &'static str) {
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

/// What differs between the two championships, and where a game from a saved section starts.
#[derive(Clone, Copy)]
struct Course {
    a8: bool,
    /// The variable holding the section a saved game starts at (0: from the beginning).
    start_section: Var,
    /// What begins each section's event.
    next_section: Call,
}

/// The whole championship. Ends with the in-game options shown with the values the entry had.
pub fn run<G: Game>(e: &mut G, a8: bool) -> Result<(), G::Error> {
    let course = Course {
        a8,
        start_section: Var::StartSection { a8 },
        next_section: if a8 {
            Call::ResetSingleEvent
        } else {
            Call::BeginEvent
        },
    };
    let mut at = begin(e, course)?;
    loop {
        at = match at {
            At::Section => start_section(e, course),
            At::Record => next_record(e, course)?,
            At::Standings => standings(e, course)?,
            At::RallyResults => rally_results(e, course)?,
            At::Races => races(e, course)?,
            At::Finish => finish(e, course)?,
            At::End => {
                return e.run(Call::ShowInGameOptions);
            }
        };
    }
}

/// The set-up: a new game's drivers and results cleared (a continued game keeps them), the mode,
/// the welcome screen; the first place to go to.
fn begin<G: Game>(e: &mut G, course: Course) -> Result<At, G::Error> {
    let Course {
        a8, start_section, ..
    } = course;
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
    e.set_flag(Switch::LeaveScreen, false);
    e.set_flag(Switch::EventLeft, false);
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
    Ok(if e.yes() {
        At::End
    } else {
        if !a8 {
            e.set(Var::Continuing, 0);
        }
        At::Section
    })
}

/// A section begins: its counters cleared, the section chosen from the saved game's.
fn start_section<G: Game>(e: &mut G, course: Course) -> At {
    e.set_flag(Switch::EscapePressed, false);
    for v in [Var::ServiceTime, Var::ServiceDue, Var::TheEnd] {
        e.set(v, 0);
    }
    e.set(Var::GoesOn, 1);
    e.set(Var::ResetTimes, 0);
    let given = e.get(course.start_section);
    if given != 0 {
        let section = e.get(course.start_section).wrapping_sub(1);
        e.set(Var::Section, section);
    }
    At::Record
}

/// The record at the section's index: a stage goes to its screens, a service is noted, a
/// section's end goes to the standings, an unknown kind is a fatal error.
fn next_record<G: Game>(e: &mut G, course: Course) -> Result<At, G::Error> {
    e.set_flag(Switch::RallyDone, false);
    let section = e.get(Var::Section);
    let index = e.get(Var::Index);
    e.run(Call::FindRecord { section, index })?;
    Ok(match e.get(Var::Record(0)) {
        THE_END => {
            e.set(Var::TheEnd, 1);
            At::Standings
        }
        WELCOME => {
            e.inc(Var::Index);
            e.run(course.next_section)?;
            e.run(Call::WelcomeLoop)?;
            if e.yes() { At::End } else { At::Record }
        }
        SECTION_END => At::Standings,
        SERVICE => {
            e.set(Var::ServiceDue, 1);
            let time = e.get(Var::Record(0x14));
            e.set(Var::ServiceTime, time);
            e.set(Var::CpuSpreadFirst, 1);
            e.set(Var::ServiceNext, 1);
            e.inc(Var::Services);
            e.inc(Var::Index);
            At::Record
        }
        STAGE => stage_screens(e)?,
        _ => {
            let (code, message) = invalid_level(course.a8);
            return Err(e.fatal(code, message));
        }
    })
}

/// A stage's screens before its races; a yes on any of them resets the times.
fn stage_screens<G: Game>(e: &mut G) -> Result<At, G::Error> {
    for screen in [
        Call::RallyStagesLoop,
        Call::NextStageLoop,
        Call::ServicePass,
    ] {
        e.run(screen)?;
        if e.yes() {
            return reset(e);
        }
    }
    Ok(if e.get(Var::GoesOn) == 1 {
        At::Races
    } else {
        At::RallyResults
    })
}

/// The standings after a section: the stage points, the class and (not the A8) the overall
/// standings, then the next section or the finish.
fn standings<G: Game>(e: &mut G, course: Course) -> Result<At, G::Error> {
    let Course {
        a8,
        start_section,
        next_section,
    } = course;
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
        return Ok(At::End);
    }
    if e.get(start_section) != 0 {
        e.set(Var::TheEnd, 1);
        return Ok(At::Finish);
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
        return Ok(At::End);
    }
    if e.get(Var::TheEnd) == 1 {
        return Ok(At::Finish);
    }
    if screens(e, &[Call::SectionFinish])? {
        return Ok(At::End);
    }
    e.run(Call::ChampionshipStandingsReview)?;
    e.set(Var::ResetTimes, 0);
    e.set(Var::GoesOn, 1);
    if e.yes() || e.flag(Switch::AskLeave) || e.flag(Switch::EscapePressed) {
        return Ok(At::End);
    }
    e.inc(Var::Section);
    e.set(Var::Index, 0);
    e.run(next_section)?;
    Ok(At::Section)
}

/// The rally's results: the section's records walked, the class pass and (not the A8) the
/// overall results.
fn rally_results<G: Game>(e: &mut G, course: Course) -> Result<At, G::Error> {
    e.set_flag(Switch::RallyDone, true);
    e.set_flag(Switch::EventLeft, false);
    e.set_flag(Switch::LeaveScreen, false);
    e.set_flag(Switch::LeftByKey, false);
    e.set(Var::GoesOn, 0);
    if screens(
        e,
        &[
            Call::WalkSectionRecords,
            Call::StandingsAFromAll,
            Call::ClassRallyResultsPass,
        ],
    )? {
        return Ok(At::End);
    }
    if !course.a8 && screens(e, &OVERALL_RALLY)? {
        return Ok(At::End);
    }
    Ok(At::Standings)
}

/// The stage's races and the screens after them (results, rally results when it was the last
/// stage, best times, inspection and the review), then on to the next record.
fn races<G: Game>(e: &mut G, course: Course) -> Result<At, G::Error> {
    let a8 = course.a8;
    if !a8 && e.get(Var::ResetTimes) == 1 {
        return Ok(At::RallyResults);
    }
    e.run(Call::RunEventRaces)?;
    if e.get(Var::GoesOn) == 0 {
        return Ok(At::RallyResults);
    }
    if screens(
        e,
        &[
            Call::AddPlayerTimes,
            Call::StandingsAFromAll,
            Call::ClassStageResultsPass,
        ],
    )? {
        return reset(e);
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
        return reset(e);
    }
    if e.get(Var::StagesDone) as i32 >= 1 {
        let next = e.next_record_kind()?;
        if next == THE_END || next == SECTION_END {
            e.set_flag(Switch::RallyDone, true);
        }
        if screens(e, &[Call::StandingsAFromAll, Call::ClassRallyResultsPass])? {
            return stop(e);
        }
        if !a8 && screens(e, &OVERALL_RALLY)? {
            return stop(e);
        }
    }
    e.run(Call::KeepRecordBest)?;
    e.run(Call::RecordEntriesPass)?;
    if e.flag(Switch::Entered) && screens(e, &[Call::BestTimesLoop])? {
        return stop(e);
    }
    let reviewed = match e.next_record_kind()? {
        THE_END => false,
        SECTION_END => return Ok(At::Standings),
        SERVICE => {
            e.set(Var::CpuSpreadFirst, 1);
            e.set(Var::ServiceNext, 1);
            true
        }
        _ => {
            e.set(Var::ServiceNext, 0);
            if screens(e, &[Call::InspectionPass])? {
                return stop(e);
            }
            if e.get(Var::GoesOn) == 0 {
                return Ok(At::RallyResults);
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
        if e.yes() || e.flag(Switch::EscapePressed) || e.flag(Switch::AskLeave) {
            return Ok(At::End);
        }
    }
    e.inc(Var::Index);
    e.inc(Var::StagesDone);
    if !a8 {
        e.set(Var::Services, 0);
    }
    Ok(At::Record)
}

/// The finish: the championship's final screens and score tables (not for a game started from a
/// saved section), then the end review.
fn finish<G: Game>(e: &mut G, course: Course) -> Result<At, G::Error> {
    if e.get(course.start_section) == 0 {
        if screens(e, &[Call::ChampionshipFinish])? {
            return Ok(At::End);
        }
        if screens(e, &[Call::ClearScoreMarks, Call::BestClassScores])? {
            return Ok(At::End);
        }
        if !course.a8 {
            e.run(Call::BestOverallScores)?;
        }
    }
    e.run(Call::ChampionshipEndReview)?;
    Ok(At::End)
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
