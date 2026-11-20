//! The installer's flow run: each step shows its page (`install::run_page`) or does its work, and
//! the answer says which step follows (`rc2k_core::install::flow`). It ends when a step leads
//! nowhere or the game is quitting.

use crate::errors::{BadInput, Refused};
use crate::install::installer::logged;
use crate::install::page_links::ListId;
use crate::memory::list_handle::ListMenu;
use rc2k_core::install::flow::{self, Page, Step, Ways};

/// The list `page` is shown as. The drive page has three, by the number of drives found.
fn list_of(install: &'static crate::install::owner::Install, page: Page) -> ListMenu {
    let list = match page {
        Page::Language => ListId::Language,
        Page::Main => ListId::Main,
        Page::Drive => crate::install::list_drives::drive_list(install.run.drive_count.get()),
        Page::Folder => ListId::Folder,
        Page::NoRoom => ListId::Room,
        Page::Removal => ListId::Removal,
    };
    ListMenu::installer(install, list)
}

/// Whether the language page has nothing to ask: this version is installed already, or there
/// is at most one language.
fn language_is_settled(install: &crate::install::owner::Install) -> bool {
    rc2k_core::install::pages::language_page_skipped(
        install.run.same_version.get() == 1,
        install.names.language_count.get(),
    )
}

/// The steps from `first` on.
pub(super) fn run_pages(
    install: &'static crate::install::owner::Install,
    o: crate::owners::Owners<'static>,
    first: Step,
) -> Result<(), Refused> {
    use crate::install as i;
    use crate::install::folders::{Folders, Native};
    let ways = Ways {
        drives: <Native as Folders>::DRIVES,
    };
    let mut step = first;
    loop {
        let next = match (step, step.page()) {
            (Step::Language, _) if language_is_settled(install) => Some(Step::Main),
            (_, Some(page)) => {
                let list = list_of(install, page);
                let answer = i::run_page::run_page(install, o, page, list)?;
                if flow::resets_highlight(answer) {
                    list.set_highlight(0);
                }
                flow::after_page(step, answer, ways)
                    .map_err(|_| BadInput("an answer the installer's page has no way for"))?
            }
            (Step::CopyFiles, None) => flow::after_copy(logged(
                "install",
                i::install_files::install_files(install, o),
            )?),
            (Step::Uninstall, None) => {
                logged("uninstall", i::uninstall::uninstall(install, o))?;
                None
            }
            (Step::OpenInstalled, None) => {
                i::open_installed::open_installed(install)?;
                None
            }
            (Step::Language | Step::Main | Step::Drive | Step::Folder | Step::NoRoom, None)
            | (Step::Removal { .. }, None) => {
                return Err(BadInput("an installer page with no list").into());
            }
        };
        let Some(next) = next else {
            return Ok(());
        };
        step = next;
        if o.system.session.flags.quitting.is_set() {
            return Ok(());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every answer an item of a page can leave it with (an item that links on holds its
    /// answer; Escape and the folder page's keys give 0 and 1; the removal page makes its 2 a
    /// 1) is one the flow has a way for, with and without drive letters.
    #[test]
    fn every_answer_a_page_can_give_has_a_way() {
        let _lock = crate::test_support::test_lock();
        let install = crate::test_support::own::install();
        let steps = [
            Step::Language,
            Step::Main,
            Step::Drive,
            Step::Folder,
            Step::NoRoom,
            Step::Removal { from_main: true },
            Step::Removal { from_main: false },
        ];
        for step in steps {
            let page = step.page().unwrap();
            let lists = match page {
                Page::Drive => vec![ListId::Drives8, ListId::Drives16, ListId::DrivesMore],
                _ => vec![],
            };
            let mut menus: Vec<ListMenu> = lists
                .into_iter()
                .map(|list| ListMenu::installer(install, list))
                .collect();
            if menus.is_empty() {
                menus.push(list_of(install, page));
            }
            for list in menus {
                let mut answers = vec![0, 1];
                for n in 0..u32::from(list.count()) {
                    let item = list.item(n);
                    if item.kind() == 1 && item.target() == 0 && item.count() & 0x80 == 0 {
                        answers.push(u32::from(item.count()));
                    }
                }
                if page == Page::Removal {
                    answers.retain(|&answer| answer != 2);
                }
                for drives in [true, false] {
                    if page == Page::Drive && !drives {
                        continue;
                    }
                    let ways = Ways { drives };
                    for &answer in &answers {
                        assert!(
                            flow::after_page(step, answer, ways).is_ok(),
                            "{step:?} answer {answer} {ways:?}"
                        );
                    }
                }
            }
        }
    }

    /// The drive page's list is the one that fits the drives found.
    #[test]
    fn the_drive_page_is_the_list_that_fits_the_drives() {
        use crate::install::list_drives::drive_list;
        assert_eq!(drive_list(0), ListId::Drives8);
        assert_eq!(drive_list(8), ListId::Drives8);
        assert_eq!(drive_list(9), ListId::Drives16);
        assert_eq!(drive_list(16), ListId::Drives16);
        assert_eq!(drive_list(17), ListId::DrivesMore);
    }
}
