//! The installer's flow: the pages it shows, the work it does between them, and where each
//! page's answer leads.

/// A page the installer shows: a list the player answers.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Page {
    /// The languages to choose from.
    Language,
    /// Install, play (or update), uninstall.
    Main,
    /// The drive to install on (only on systems with drive letters).
    Drive,
    /// The folder to install into.
    Folder,
    /// Not enough room in the folder: go on anyway, or back.
    NoRoom,
    /// What an uninstall removes: everything, or all but the saves and settings.
    Removal,
}

/// A step of the installer: a page shown, or work done with no page.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Step {
    Language,
    Main,
    Drive,
    Folder,
    /// The files copied into the folder.
    CopyFiles,
    NoRoom,
    /// The removal page; `from_main`: come to from the main page (its "back" returns there),
    /// not as the installer's first page.
    Removal {
        from_main: bool,
    },
    /// The installation removed.
    Uninstall,
    /// The installed copy made the one that runs.
    OpenInstalled,
}

impl Step {
    /// The page the step shows, if it shows one.
    pub fn page(self) -> Option<Page> {
        match self {
            Step::Language => Some(Page::Language),
            Step::Main => Some(Page::Main),
            Step::Drive => Some(Page::Drive),
            Step::Folder => Some(Page::Folder),
            Step::NoRoom => Some(Page::NoRoom),
            Step::Removal { .. } => Some(Page::Removal),
            Step::CopyFiles | Step::Uninstall | Step::OpenInstalled => None,
        }
    }
}

/// What the flow's moves depend on besides the answer.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Ways {
    /// The system has drive letters: the drive page is part of the flow.
    pub drives: bool,
}

/// How the copying of the files ended.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Copied {
    /// Everything is installed.
    Installed,
    /// The copy failed or was cancelled, and the player was told.
    Stopped,
    /// The folder cannot be used.
    BadFolder,
    /// The folder has too little room.
    NoRoom,
}

/// An answer its page has no item for.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NoSuchAnswer;

/// The answer every page's "back" (and Escape) gives.
pub const BACK: u32 = 0;

/// Where the installer starts: the removal page for an uninstall, else the languages.
pub fn first_step(uninstalling: bool) -> Step {
    if uninstalling {
        Step::Removal { from_main: false }
    } else {
        Step::Language
    }
}

/// The step after `page` answered `answer`; None ends the installer. `step` must be one that
/// shows a page.
pub fn after_page(step: Step, answer: u32, ways: Ways) -> Result<Option<Step>, NoSuchAnswer> {
    // The page before the folder page, and the page the no-room page goes back to.
    let before_folder = if ways.drives { Step::Drive } else { Step::Main };
    let choose_place = if ways.drives {
        Step::Drive
    } else {
        Step::Folder
    };
    Ok(match (step, answer) {
        (Step::Language, BACK) => None,
        (Step::Language, 1..=9) => Some(Step::Main),
        (Step::Main, BACK) => None,
        (Step::Main, 1) => Some(choose_place),
        (Step::Main, 2) => Some(Step::OpenInstalled),
        (Step::Main, 3) => Some(Step::Removal { from_main: true }),
        (Step::Drive, BACK) => Some(Step::Main),
        (Step::Drive, 1) => Some(Step::Folder),
        (Step::Folder, BACK) => Some(before_folder),
        (Step::Folder, 1) => Some(Step::CopyFiles),
        (Step::NoRoom, BACK) => Some(choose_place),
        (Step::NoRoom, 1) => Some(Step::CopyFiles),
        (Step::Removal { from_main }, BACK) => from_main.then_some(Step::Main),
        (Step::Removal { .. }, 1) => Some(Step::Uninstall),
        _ => return Err(NoSuchAnswer),
    })
}

/// The step after the files' copy; None ends the installer.
pub fn after_copy(copied: Copied) -> Option<Step> {
    match copied {
        Copied::Installed | Copied::Stopped => None,
        Copied::BadFolder => Some(Step::Folder),
        Copied::NoRoom => Some(Step::NoRoom),
    }
}

/// Whether a page answered `answer` starts at its first item when it is shown again: after
/// "back" and after its first way on. Any other answer keeps the highlight where it was.
pub fn resets_highlight(answer: u32) -> bool {
    answer <= 1
}

#[cfg(test)]
mod tests {
    use super::*;

    const WITH_DRIVES: Ways = Ways { drives: true };
    const NO_DRIVES: Ways = Ways { drives: false };

    fn after(step: Step, answer: u32, ways: Ways) -> Option<Step> {
        after_page(step, answer, ways).unwrap()
    }

    #[test]
    fn an_install_goes_from_the_languages_to_the_copy() {
        assert_eq!(first_step(false), Step::Language);
        for language in 1..=9 {
            assert_eq!(
                after(Step::Language, language, WITH_DRIVES),
                Some(Step::Main)
            );
        }
        assert_eq!(after(Step::Main, 1, WITH_DRIVES), Some(Step::Drive));
        assert_eq!(after(Step::Drive, 1, WITH_DRIVES), Some(Step::Folder));
        assert_eq!(after(Step::Folder, 1, WITH_DRIVES), Some(Step::CopyFiles));
        assert_eq!(after_copy(Copied::Installed), None);
        assert_eq!(after_copy(Copied::Stopped), None);
    }

    #[test]
    fn back_leads_to_the_page_before() {
        assert_eq!(after(Step::Language, BACK, WITH_DRIVES), None);
        assert_eq!(after(Step::Main, BACK, WITH_DRIVES), None);
        assert_eq!(after(Step::Drive, BACK, WITH_DRIVES), Some(Step::Main));
        assert_eq!(after(Step::Folder, BACK, WITH_DRIVES), Some(Step::Drive));
        assert_eq!(after(Step::NoRoom, BACK, WITH_DRIVES), Some(Step::Drive));
    }

    #[test]
    fn a_folder_that_will_not_do_is_asked_for_again() {
        assert_eq!(after_copy(Copied::BadFolder), Some(Step::Folder));
        assert_eq!(after_copy(Copied::NoRoom), Some(Step::NoRoom));
        assert_eq!(after(Step::NoRoom, 1, WITH_DRIVES), Some(Step::CopyFiles));
    }

    #[test]
    fn without_drive_letters_the_drive_page_is_no_part_of_the_flow() {
        assert_eq!(after(Step::Main, 1, NO_DRIVES), Some(Step::Folder));
        assert_eq!(after(Step::Folder, BACK, NO_DRIVES), Some(Step::Main));
        assert_eq!(after(Step::Folder, 1, NO_DRIVES), Some(Step::CopyFiles));
        assert_eq!(after(Step::NoRoom, BACK, NO_DRIVES), Some(Step::Folder));
        // No answer of any page leads to the drive page.
        let steps = [
            Step::Language,
            Step::Main,
            Step::Folder,
            Step::NoRoom,
            Step::Removal { from_main: true },
            Step::Removal { from_main: false },
        ];
        for step in steps {
            for answer in 0..10 {
                assert_ne!(after_page(step, answer, NO_DRIVES), Ok(Some(Step::Drive)));
            }
        }
    }

    #[test]
    fn the_main_page_leads_to_each_of_its_items() {
        assert_eq!(after(Step::Main, 2, WITH_DRIVES), Some(Step::OpenInstalled));
        assert_eq!(
            after(Step::Main, 3, WITH_DRIVES),
            Some(Step::Removal { from_main: true })
        );
    }

    #[test]
    fn an_uninstall_asks_what_to_remove_first() {
        let at_start = first_step(true);
        assert_eq!(at_start, Step::Removal { from_main: false });
        assert_eq!(after(at_start, BACK, WITH_DRIVES), None);
        assert_eq!(after(at_start, 1, WITH_DRIVES), Some(Step::Uninstall));
        let from_main = Step::Removal { from_main: true };
        assert_eq!(after(from_main, BACK, WITH_DRIVES), Some(Step::Main));
        assert_eq!(after(from_main, 1, WITH_DRIVES), Some(Step::Uninstall));
    }

    #[test]
    fn an_answer_a_page_has_no_item_for_is_refused() {
        assert_eq!(
            after_page(Step::Language, 10, WITH_DRIVES),
            Err(NoSuchAnswer)
        );
        assert_eq!(after_page(Step::Main, 4, WITH_DRIVES), Err(NoSuchAnswer));
        assert_eq!(after_page(Step::Drive, 2, WITH_DRIVES), Err(NoSuchAnswer));
        assert_eq!(after_page(Step::Folder, 2, WITH_DRIVES), Err(NoSuchAnswer));
        assert_eq!(after_page(Step::NoRoom, 3, WITH_DRIVES), Err(NoSuchAnswer));
        assert_eq!(
            after_page(Step::Removal { from_main: true }, 2, WITH_DRIVES),
            Err(NoSuchAnswer)
        );
        assert_eq!(
            after_page(Step::CopyFiles, 1, WITH_DRIVES),
            Err(NoSuchAnswer)
        );
    }

    #[test]
    fn only_work_steps_show_no_page() {
        assert_eq!(
            Step::Removal { from_main: true }.page(),
            Some(Page::Removal)
        );
        assert_eq!(Step::NoRoom.page(), Some(Page::NoRoom));
        for step in [Step::CopyFiles, Step::Uninstall, Step::OpenInstalled] {
            assert_eq!(step.page(), None);
        }
        assert!(resets_highlight(0) && resets_highlight(1) && !resets_highlight(2));
    }
}
