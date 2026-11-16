//! The stage screens' choices: the speeches, the welcome picture and the items that pick a page.

use crate::Invalid;

/// The event mode in the championship.
pub const CHAMPIONSHIP: u32 = 2;

/// The event type of the time trial.
pub const TIME_TRIAL: u32 = 3;

/// The stage speeches' table has this many entries.
pub const STAGE_SPEECHES: u32 = 0x40;

/// The highest section the rally speeches' and welcome pictures' tables are read at; a higher
/// one is refused.
pub const LAST_SECTION: u32 = 0x20;

/// The speeches play in the championship only.
pub fn championship(mode: u32) -> bool {
    mode == CHAMPIONSHIP
}

/// `menu::play_stage_speech`: the speech entry for the record's speech number (`entry`
/// reads the table). Refused: a number past the table, a null entry.
pub fn stage_speech(speech: u32, entry: impl FnOnce(u32) -> u32) -> Result<u32, Invalid> {
    if speech >= STAGE_SPEECHES {
        return Err(Invalid("a stage speech past 0x40"));
    }
    match entry(speech) {
        0 => Err(Invalid("a null stage speech")),
        at => Ok(at),
    }
}

/// The section a table by section is read at. Refused: past `LAST_SECTION`.
pub fn small_section(section: u32) -> Result<u32, Invalid> {
    if section > LAST_SECTION {
        return Err(Invalid("a section past 0x20 (a table read unchecked)"));
    }
    Ok(section)
}

/// `menu::play_rally_speech`: in the championship the section whose speech plays (held
/// to the table), elsewhere none (and the section not looked at).
pub fn rally_speech(mode: u32, section: impl FnOnce() -> u32) -> Result<Option<u32>, Invalid> {
    if championship(mode) {
        small_section(section()).map(Some)
    } else {
        Ok(None)
    }
}

/// The welcome pictures' two tables.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Welcome {
    /// `MenuLists.welcome_championship[0]`, the championship's.
    Championship,
    /// `MenuLists.welcome_other[0]`, the others' (the A8 championship's).
    Other,
}

/// `menu::load_welcome_picture`: the table by the event mode.
pub fn welcome(mode: u32) -> Welcome {
    if championship(mode) {
        Welcome::Championship
    } else {
        Welcome::Other
    }
}

/// `menu::view_stage_details`: the time trial's stage details page (true) or the stage view, by
/// event type.
pub fn time_trial_details(event_type: u32) -> bool {
    event_type == TIME_TRIAL
}

/// `menu::new_simulation_if_unlocked`: a new simulation starts with `RaceState.settings.a8_game` 1.
pub fn simulation_unlocked(flag: u32) -> bool {
    flag == 1
}

/// Refused a new simulation: the menu's sound 8, then this long a sleep (ms).
pub const REFUSED_SOUND: u32 = 8;
pub const REFUSED_SLEEP: u32 = 0xFA;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn choices() {
        assert_eq!(stage_speech(3, |n| 0x100 + n), Ok(0x103));
        assert!(stage_speech(0x40, |_| unreachable!()).is_err());
        assert!(stage_speech(1, |_| 0).is_err());
        assert_eq!(rally_speech(2, || 0x20), Ok(Some(0x20)));
        assert!(rally_speech(2, || 0x21).is_err());
        assert_eq!(rally_speech(0x13, || unreachable!()), Ok(None));
        assert_eq!(welcome(2), Welcome::Championship);
        assert_eq!(welcome(0x13), Welcome::Other);
        assert!(time_trial_details(3) && !time_trial_details(4));
        assert!(simulation_unlocked(1) && !simulation_unlocked(2));
    }
}
