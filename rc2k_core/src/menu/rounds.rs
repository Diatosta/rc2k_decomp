//! The single rally's and the time trial's events (from
//! `menu::single_rally` 0x482382, `menu::single_rally_split` 0x4824C1, `_two` 0x4826A2,
//! `_three` 0x4828B0, `_four` 0x482B7A, `menu::time_trial_one` 0x48AE39, `_split`
//! 0x48AFD0, `_two` 0x48B23D, `_three` 0x48B54B, `_four` 0x48B90F): the players' car
//! choices, then round after round the stage, the races and the pages after them, until a
//! yes or a leaving ends it. Each event's steps in its original's order, on `flow::Game`.

use super::flow::{Call, CarList, Game, Var};
use super::leaving::Switch;
use crate::Invalid;

/// Players 2 and 3's times (+0x88) in the time trial (3 597 928 and 3 602 000 ms).
pub const TIME_2: u32 = 0x36F268;
pub const TIME_3: u32 = 0x36F650;

/// A race's result: none, and left (-2).
pub const NO_RESULT: u32 = u32::MAX;
pub const LEFT: u32 = 0xFFFF_FFFE;

/// The single rally's car list: the first with `dword_7554D8` 1, else the second; the
/// cheats' with cheat bit 1.
fn rally_car_list<G: Game>(e: &mut G) {
    let first = e.get(Var::CarListKind) == 1;
    let cheat = e.get(Var::Cheats) & 2 != 0;
    e.hand_cars(if cheat {
        CarList::Cheat
    } else if first {
        CarList::First
    } else {
        CarList::Second
    });
}

/// The time trial's car list: the second, through `menu::cheat_car_list`.
fn time_trial_car_list<G: Game>(e: &mut G) -> Result<(), G::Error> {
    e.hand_cars(CarList::Second);
    e.run(Call::CheatCarList)
}

/// The car choices of players 0 .. n: false when one was left (`dword_758806` 1).
/// `list_again[k]`: the time trial's list made again before player k's car is taken.
fn choose_cars<G: Game>(
    e: &mut G,
    n: u32,
    time_trial: bool,
    list_again: &[u32],
) -> Result<bool, G::Error> {
    for k in 0..n {
        e.set(Var::Current, k);
        if time_trial {
            time_trial_car_list(e)?;
        } else {
            rally_car_list(e);
        }
        e.run(Call::SelectCarLoop)?;
        if e.flag(Switch::LeftByKey) {
            return Ok(false);
        }
        if list_again.contains(&k) {
            time_trial_car_list(e)?;
        }
        e.run(Call::TakeShownCar)?;
    }
    Ok(true)
}

/// The four players' repairs.
fn repair_all<G: Game>(e: &mut G) -> Result<(), G::Error> {
    for k in 0..4 {
        e.set(Var::Current, k);
        e.run(Call::RepairCar)?;
    }
    Ok(())
}

/// Players 2 and 3's times.
fn times_2_3<G: Game>(e: &mut G) {
    e.set(Var::PlayerTime(2), TIME_2);
    e.set(Var::PlayerTime(3), TIME_3);
}

/// The races of players 0 .. n one after another (the single rally repairs each car
/// first): a race left resets the players' times and sets the front end up again (None);
/// else the results and-ed together, kept as the original keeps them between the races.
fn race_series<G: Game>(
    e: &mut G,
    n: u32,
    kind: u32,
    repair: bool,
) -> Result<Option<u32>, G::Error> {
    let mut all = 0;
    for k in 0..n {
        e.set(Var::Current, k);
        if repair {
            e.run(Call::RepairCar)?;
        }
        e.run(Call::Race {
            next: k + 1 < n,
            kind,
        })?;
        if e.get(Var::RaceResult) == LEFT {
            e.run(Call::ResetPlayerTimes)?;
            e.run(Call::ReloadFrontEnd)?;
            return Ok(None);
        }
        all = if k == 0 {
            e.get(Var::RaceResult)
        } else {
            all & e.get(Var::RaceResult)
        };
    }
    Ok(Some(all))
}

/// The split screen's one race: both cars' results and-ed.
fn split_race<G: Game>(e: &mut G, kind: u32) -> Result<u32, G::Error> {
    e.run(Call::Race { next: false, kind })?;
    let both = e.get(Var::RaceResult) & e.get(Var::RaceResult2);
    Ok(both)
}

/// Players 0 .. n's figures kept for the stage.
fn keep_bests<G: Game>(e: &mut G, n: u32) -> Result<(), G::Error> {
    for k in 0..n {
        e.set(Var::Current, k);
        e.run(Call::KeepStageBest)?;
    }
    Ok(())
}

/// Players `from` .. n: whether the time qualifies, and if it does, the time entered.
fn qualify<G: Game>(e: &mut G, from: u32, n: u32) -> Result<(), G::Error> {
    for k in from..n {
        e.set(Var::Current, k);
        e.run(Call::TimeTrialQualifies)?;
        if e.get(Var::Qualifies) == 1 {
            e.run(Call::EnterTimeTrialBest)?;
        }
    }
    Ok(())
}

/// The best times page when one qualified: false after a yes on it.
fn best_times<G: Game>(e: &mut G) -> Result<bool, G::Error> {
    if e.flag(Switch::Entered) {
        e.run(Call::TimeTrialBestTimesLoop)?;
        if e.yes() {
            return Ok(false);
        }
    }
    Ok(true)
}

/// The review, the in-game options hidden for `hide` players or shown: true when it ends
/// the event (a yes when `ask` says so, `dword_755569`, `dword_75555D`).
fn review<G: Game>(e: &mut G, hide: Option<u32>, ask: bool) -> Result<bool, G::Error> {
    match hide {
        Some(players) => e.run(Call::HideInGameOptions { players })?,
        None => e.run(Call::ShowInGameOptions)?,
    }
    e.run(Call::SingleRallyReview)?;
    Ok((ask && e.yes()) || e.flag(Switch::AskLeave) || e.flag(Switch::EventLeft))
}

/// The end: done, the leave flags 0, the in-game options shown.
fn end<G: Game>(e: &mut G) -> Result<(), G::Error> {
    e.set_flag(Switch::EventOver, true);
    e.set_flag(Switch::EventLeft, false);
    e.set_flag(Switch::AskLeave, false);
    e.set_flag(Switch::LeftWithCarry, false);
    e.run(Call::ShowInGameOptions)
}

/// The stage choice: true when a yes or the leaving ends the event.
fn stage_chosen<G: Game>(e: &mut G, screen: Call) -> Result<bool, G::Error> {
    e.run(screen)?;
    Ok(!e.yes() && !e.flag(Switch::EventLeft))
}

/// `menu::single_rally`: one player (event mode 0x18).
pub fn single_rally<G: Game>(e: &mut G) -> Result<(), G::Error> {
    e.run(Call::BeginEvent)?;
    e.run(Call::ClearHudBits)?;
    e.run(Call::TakePlayer1Units)?;
    e.set(Var::PlayerCount, 1);
    e.set(Var::EventMode, 0x18);
    e.set_flag(Switch::EventLeft, false);
    e.run(Call::ClearEventState)?;
    e.set(Var::Current, 0);
    rally_car_list(e);
    e.run(Call::SelectCarLoop)?;
    if e.flag(Switch::LeftByKey) {
        return Ok(());
    }
    e.run(Call::TakeShownCar)?;
    loop {
        if !stage_chosen(e, Call::SelectStageLoop)? {
            break;
        }
        let current = e.get(Var::Current);
        if current >= 4 {
            return Err(Invalid("a current player past the four").into());
        }
        e.set(Var::PlayerEntry(current), 1);
        e.run(Call::RepairCar)?;
        e.run(Call::Race {
            next: false,
            kind: 0,
        })?;
        if e.get(Var::RaceResult) != NO_RESULT {
            e.set(Var::Current, 0);
            e.run(Call::StageResultLoop)?;
            if e.yes() {
                break;
            }
        }
        e.run(Call::HideInGameOptions { players: 1 })?;
        e.run(Call::SingleRallyReview)?;
        if e.flag(Switch::EscapePressed) || e.flag(Switch::AskLeave) || e.flag(Switch::EventLeft) {
            break;
        }
    }
    end(e)
}

/// The start of the single rally's events for two to four players.
fn rally_start<G: Game>(e: &mut G) -> Result<(), G::Error> {
    e.run(Call::BeginEvent)?;
    e.run(Call::ClearHudBits)?;
    e.run(Call::TakePlayer1Units)
}

/// `menu::single_rally_split`: two players on a split screen (event mode 5).
pub fn single_rally_split<G: Game>(e: &mut G) -> Result<(), G::Error> {
    rally_start(e)?;
    e.set(Var::PlayerCount, 2);
    e.set(Var::EventMode, 5);
    e.set_flag(Switch::EventLeft, false);
    e.run(Call::ClearEventState)?;
    if !choose_cars(e, 2, false, &[])? {
        return Ok(());
    }
    loop {
        e.set(Var::Current, 0);
        e.run(Call::ReadyPlayers)?;
        repair_all(e)?;
        if !stage_chosen(e, Call::SelectStageLoop)? {
            break;
        }
        e.set(Var::Current, 0);
        e.set(Var::PlayerEntry(0), 1);
        // Both cars repaired. (The original passed 0 and 1, which `repair_car` does not
        // read: player 0's car was repaired twice, player 1's kept the last stage's damage.
        // Fixed bug; the current player is 0 again for the race, as the original's.)
        for player in 0..2 {
            e.set(Var::Current, player);
            e.run(Call::RepairCar)?;
        }
        e.set(Var::Current, 0);
        if split_race(e, 0)? != NO_RESULT {
            e.run(Call::SingleRallyTwoResults)?;
            if e.yes() {
                break;
            }
        }
        if review(e, Some(2), false)? {
            break;
        }
    }
    end(e)
}

/// `menu::single_rally_two`: two players in turn (event mode 8).
pub fn single_rally_two<G: Game>(e: &mut G) -> Result<(), G::Error> {
    rally_start(e)?;
    e.set(Var::PlayerCount, 2);
    e.set(Var::EventMode, 8);
    e.set_flag(Switch::EventLeft, false);
    e.run(Call::ClearEventState)?;
    if !choose_cars(e, 2, false, &[])? {
        return Ok(());
    }
    loop {
        e.set(Var::Current, 0);
        e.run(Call::ReadyPlayers)?;
        repair_all(e)?;
        if !stage_chosen(e, Call::SelectStageLoop)? {
            break;
        }
        if let Some(all) = race_series(e, 2, 0, true)?
            && all != NO_RESULT
        {
            e.run(Call::SingleRallyTwoResults)?;
            if e.yes() {
                break;
            }
        }
        if review(e, Some(2), true)? {
            break;
        }
    }
    end(e)
}

/// The three and four players' rounds after the car choices (the four players' start
/// without the current player 0 and the fourth's time); `dword_758714` counts them.
fn rally_rounds<G: Game>(e: &mut G, n: u32) -> Result<(), G::Error> {
    loop {
        if n == 3 {
            e.set(Var::Current, 0);
        }
        e.run(Call::ReadyPlayers)?;
        repair_all(e)?;
        if n == 3 {
            e.set(Var::PlayerTime(3), TIME_3);
        }
        if !stage_chosen(e, Call::SelectStageLoop)? {
            return Ok(());
        }
        if let Some(all) = race_series(e, n, 0, true)?
            && all != NO_RESULT
        {
            e.run(Call::SingleRallyThreeFourResults)?;
            if e.yes() {
                return Ok(());
            }
            if e.get(Var::Round) != 0 {
                e.run(Call::SingleRallyOverallResults)?;
                if e.yes() {
                    return Ok(());
                }
            }
        }
        if review(e, Some(n), true)? {
            return Ok(());
        }
        e.inc(Var::Round);
    }
}

/// `menu::single_rally_three`: three players in turn (event mode 8).
pub fn single_rally_three<G: Game>(e: &mut G) -> Result<(), G::Error> {
    rally_start(e)?;
    e.run(Call::ClearEventState)?;
    e.set(Var::Round, 0);
    e.set(Var::PlayerCount, 3);
    e.set(Var::EventMode, 8);
    e.set_flag(Switch::EventLeft, false);
    if !choose_cars(e, 3, false, &[])? {
        return Ok(());
    }
    rally_rounds(e, 3)?;
    end(e)
}

/// `menu::single_rally_four`: four players in turn (event mode 8; no round or leave
/// flag cleared).
pub fn single_rally_four<G: Game>(e: &mut G) -> Result<(), G::Error> {
    rally_start(e)?;
    e.run(Call::ClearEventState)?;
    e.set(Var::PlayerCount, 4);
    e.set(Var::EventMode, 8);
    if !choose_cars(e, 4, false, &[])? {
        return Ok(());
    }
    rally_rounds(e, 4)?;
    end(e)
}

/// `menu::time_trial_one`: one player (event mode 0x14).
pub fn time_trial_one<G: Game>(e: &mut G) -> Result<(), G::Error> {
    e.run(Call::ClearHudBits)?;
    e.run(Call::TakePlayer1Units)?;
    e.set(Var::PlayerCount, 1);
    e.set(Var::EventMode, 0x14);
    e.set(Var::EventType, 3);
    e.set_flag(Switch::EventLeft, false);
    e.set(Var::Mode, 3);
    e.run(Call::BeginEvent)?;
    if !choose_cars(e, 1, true, &[])? {
        return Ok(());
    }
    loop {
        if !stage_chosen(e, Call::TimeTrialSelectStage)? {
            break;
        }
        e.run(Call::BeginEvent)?;
        e.set(Var::ServiceDue, 1);
        e.run(Call::TimeTrialService)?;
        if e.yes() {
            break;
        }
        e.run(Call::Race {
            next: false,
            kind: 3,
        })?;
        if e.get(Var::RaceResult) != NO_RESULT {
            e.set(Var::Current, 0);
            e.run(Call::StageResultLoop)?;
            if e.yes() {
                break;
            }
            e.run(Call::KeepStageBest)?;
            e.set(Var::Qualifies, 0);
            e.run(Call::TimeTrialQualifies)?;
            if e.get(Var::Qualifies) == 1 {
                e.run(Call::ClearTimeTrialMarks)?;
                e.run(Call::EnterTimeTrialBest)?;
                e.run(Call::TimeTrialBestTimesLoop)?;
                if e.yes() {
                    break;
                }
            }
        }
        if review(e, Some(1), true)? {
            break;
        }
    }
    end(e)
}

/// The time trial's start for two to four players: `g_eventMode` 3, the event begun,
/// the players' +0x284 cleared, player 1's units.
fn time_trial_start<G: Game>(e: &mut G) -> Result<(), G::Error> {
    e.set(Var::Mode, 3);
    e.run(Call::BeginEvent)?;
    e.run(Call::ClearHudBits)?;
    e.run(Call::TakePlayer1Units)
}

/// The variables the two players' time trials clear (`dword_7587B8`, `_BC`, `_A8`, `_AC`).
const CLEARED_TWO: [u32; 4] = [4, 5, 0, 1];
/// The three players' (`dword_7587B8` .. `_C4`, then `_A8` .. `_B4`).
const CLEARED_THREE: [u32; 8] = [4, 5, 6, 7, 0, 1, 2, 3];

/// The two players' time trials' start (event mode `mode`).
fn time_trial_two_start<G: Game>(e: &mut G, mode: u32) -> Result<bool, G::Error> {
    time_trial_start(e)?;
    e.set(Var::PlayerCount, 2);
    e.set(Var::EventMode, mode);
    e.set(Var::EventType, 3);
    e.set_flag(Switch::EventLeft, false);
    for k in CLEARED_TWO {
        e.set(Var::Cleared(k), 0);
    }
    choose_cars(e, 2, true, &[])
}

/// `menu::time_trial_split`: two players on a split screen (event mode 4).
pub fn time_trial_split<G: Game>(e: &mut G) -> Result<(), G::Error> {
    if !time_trial_two_start(e, 4)? {
        return Ok(());
    }
    loop {
        e.set(Var::Current, 0);
        e.run(Call::ReadyPlayers)?;
        for k in 0..2 {
            e.set(Var::Current, k);
            e.run(Call::RepairCar)?;
        }
        times_2_3(e);
        if !stage_chosen(e, Call::TimeTrialSelectStage)? {
            break;
        }
        e.set(Var::ServiceDue, 1);
        e.run(Call::TimeTrialService)?;
        if split_race(e, 3)? != NO_RESULT {
            e.run(Call::TimeTrialResultsLoop)?;
            if e.yes() {
                break;
            }
            keep_bests(e, 2)?;
            e.set_flag(Switch::Entered, false);
            e.run(Call::ClearTimeTrialMarks)?;
            qualify(e, 0, 2)?;
        }
        if !best_times(e)? {
            break;
        }
        if review(e, Some(2), false)? {
            break;
        }
    }
    end(e)
}

/// `menu::time_trial_two`: two players in turn (event mode 0x15).
pub fn time_trial_two<G: Game>(e: &mut G) -> Result<(), G::Error> {
    if !time_trial_two_start(e, 0x15)? {
        return Ok(());
    }
    loop {
        e.set(Var::Current, 0);
        e.run(Call::ReadyPlayers)?;
        repair_all(e)?;
        for (k, time) in [(2, TIME_2), (3, TIME_3)] {
            e.set(Var::Current, k);
            let current = e.get(Var::Current);
            e.set(Var::PlayerTime(current), time);
        }
        if !stage_chosen(e, Call::TimeTrialSelectStage)? {
            break;
        }
        e.set(Var::ServiceDue, 1);
        e.run(Call::TimeTrialService)?;
        if let Some(all) = race_series(e, 2, 3, false)? {
            // All results -1: the original's jump goes to player 1's qualifying.
            let first = if all != NO_RESULT {
                e.run(Call::TimeTrialResultsLoop)?;
                if e.yes() {
                    break;
                }
                keep_bests(e, 2)?;
                e.set_flag(Switch::Entered, false);
                e.run(Call::ClearTimeTrialMarks)?;
                0
            } else {
                1
            };
            qualify(e, first, 2)?;
            if !best_times(e)? {
                break;
            }
        }
        if review(e, Some(2), true)? {
            break;
        }
    }
    end(e)
}

/// The three and four players' time trials' rounds after the car choices: `four` for
/// `menu::time_trial_four`'s differences (no current player 0 before `ready_players`, the
/// service's yes looked at, no stage handed, the review after the in-game options shown).
fn time_trial_rounds<G: Game>(e: &mut G, n: u32, four: bool) -> Result<(), G::Error> {
    loop {
        if !four {
            e.set(Var::Current, 0);
        }
        e.run(Call::ReadyPlayers)?;
        repair_all(e)?;
        times_2_3(e);
        if !stage_chosen(e, Call::TimeTrialSelectStage)? {
            return Ok(());
        }
        e.set(Var::ServiceDue, 1);
        e.run(Call::TimeTrialService)?;
        if four && e.yes() {
            return Ok(());
        }
        if let Some(all) = race_series(e, n, 3, false)?
            && all != NO_RESULT
        {
            e.run(Call::TimeTrialResultsLoop)?;
            if e.yes() {
                return Ok(());
            }
            keep_bests(e, n)?;
            e.set_flag(Switch::Entered, false);
            e.run(Call::ClearTimeTrialMarks)?;
            qualify(e, 0, n)?;
            if !best_times(e)? {
                return Ok(());
            }
        }
        if review(e, if four { None } else { Some(n) }, true)? {
            return Ok(());
        }
    }
}

/// `menu::time_trial_three`: three players in turn (event mode 0x16).
pub fn time_trial_three<G: Game>(e: &mut G) -> Result<(), G::Error> {
    time_trial_start(e)?;
    e.set(Var::Round, 0);
    e.set(Var::PlayerCount, 3);
    e.set(Var::EventMode, 0x16);
    e.set(Var::EventType, 3);
    e.set_flag(Switch::EventLeft, false);
    for k in CLEARED_THREE {
        e.set(Var::Cleared(k), 0);
    }
    if !choose_cars(e, 3, true, &[1, 2])? {
        return Ok(());
    }
    time_trial_rounds(e, 3, false)?;
    end(e)
}

/// `menu::time_trial_four`: four players in turn (event mode 0x17; nothing else cleared;
/// player 0's list made again before its car is taken).
pub fn time_trial_four<G: Game>(e: &mut G) -> Result<(), G::Error> {
    time_trial_start(e)?;
    e.set(Var::PlayerCount, 4);
    e.set(Var::EventMode, 0x17);
    e.set(Var::EventType, 3);
    e.set_flag(Switch::EventLeft, false);
    if !choose_cars(e, 4, true, &[0])? {
        return Ok(());
    }
    time_trial_rounds(e, 4, true)?;
    end(e)
}

#[cfg(test)]
mod tests {
    use super::super::flow::Var;
    use super::super::flow::log::Log;
    use super::*;

    #[test]
    fn split_screen_repairs_both_cars() {
        // One stage, then the stage choice's yes.
        let mut log = Log::default().on("SelectStageLoop", vec![vec![], vec![(Var::Answer, 1)]]);
        single_rally_split(&mut log).unwrap();
        let race = log
            .steps
            .iter()
            .position(|s| s.starts_with("Race "))
            .unwrap();
        assert_eq!(
            log.steps[race - 6..race],
            [
                "PlayerEntry(0)=1",
                "Current=0",
                "RepairCar",
                "Current=1",
                "RepairCar",
                "Current=0"
            ]
        );
    }

    #[test]
    fn car_choice_left() {
        let mut log = Log::default().on_switches(
            "SelectCarLoop",
            vec![vec![], vec![(Switch::LeftByKey, true)]],
        );
        time_trial_three(&mut log).unwrap();
        assert_eq!(log.count("SelectCarLoop"), 2);
        assert_eq!(log.count("CheatCarList"), 2);
        assert!(!log.has("Done=1"));
    }

    #[test]
    fn rally_rounds_and_end() {
        // A race left (-2) in the first round, then the stage choice's yes.
        let mut log = Log::default()
            .on(
                "Race",
                vec![vec![(Var::RaceResult, 5)], vec![(Var::RaceResult, LEFT)]],
            )
            .on("SelectStageLoop", vec![vec![], vec![(Var::Answer, 1)]]);
        single_rally_three(&mut log).unwrap();
        assert_eq!(log.count("Race { next: true, kind: 0 }"), 2);
        assert!(log.has("ReloadFrontEnd"));
        assert!(!log.has("SingleRallyThreeFourResults"));
        assert!(log.has("Round=1"));
        assert_eq!(log.steps.last().unwrap(), "ShowInGameOptions");
    }

    #[test]
    fn time_trial_qualifying() {
        let mut log = Log::default()
            .with(Var::PlayerCount, 2)
            .on("Race", vec![vec![(Var::RaceResult, NO_RESULT)]])
            .on("TimeTrialQualifies", vec![vec![(Var::Qualifies, 1)]])
            .on_switches(
                "TimeTrialSelectStage",
                vec![vec![], vec![(Switch::EventLeft, true)]],
            );
        time_trial_two(&mut log).unwrap();
        // All -1: only player 1's qualifying, without the player count.
        assert!(log.has("TimeTrialQualifies"));
        assert!(!log.has("TimeTrialResultsLoop"));
        assert!(log.has("EnterTimeTrialBest"));
        assert!(log.has("PlayerTime(3)=3602000"));
    }

    #[test]
    fn single_rally_refuses() {
        // The stage choice leaves a current player past the four.
        let mut log = Log::default()
            .with(Var::Cheats, 2)
            .on("SelectStageLoop", vec![vec![(Var::Current, 4)]]);
        assert!(single_rally(&mut log).is_err());
        assert!(log.has("CarList(Cheat)"));
    }
}
