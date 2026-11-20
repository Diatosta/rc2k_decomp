//! The installer's flow run: each step shows its page (`install::run_page`) or does its work, and
//! the answer says which step follows (`rc2k_core::install::flow`). It ends when a step leads
//! nowhere or the game is quitting.

use crate::errors::{BadInput, Refused};
use crate::install::installer::logged;
use rc2k_core::install::flow::{self, Step, Ways};
use rc2k_core::install::model::List;

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
                let answer = i::run_page::run_page(install, o, page)?;
                if flow::resets_highlight(answer) {
                    let list = List::of(page, install.run.drive_count.get());
                    install.highlights.set(list, 0);
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
    use rc2k_core::install::flow::Page;

    /// Every answer an item of a page can leave it with (an item that links on holds its
    /// answer; Escape and the folder page's keys give 0 and 1; the removal page makes its 2 a
    /// 1) is one the flow has a way for, with and without drive letters.
    #[test]
    fn every_answer_a_page_can_give_has_a_way() {
        use rc2k_core::install::model::{Facts, Kind};
        let facts = Facts {
            same_version: true,
            installed: true,
            update_entry: None,
            languages: 3,
            drives: 5,
            path_problem: 0,
            folder_most: 0x7C,
            folder_width: 0x2E9,
        };
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
            let def =
                rc2k_core::install::model::build(List::of(page, facts.drives), &facts).unwrap();
            let mut answers = vec![0, 1];
            for item in &def.items {
                if let Kind::Link {
                    answer,
                    action: None,
                } = item.kind
                    && answer & 0x80 == 0
                {
                    answers.push(u32::from(answer));
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
