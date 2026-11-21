//! The installer's variables the made texts live in (`strings::string_table::MadeTextBuffers`):
//! the drive rows, the languages found and the credits.

use crate::install::owner::Install;
use crate::memory::place::Run;
use crate::strings::string_table::{MadeText, MadeTextBuffers};

impl MadeTextBuffers for Install {
    fn chain(&self, text: MadeText) -> Option<Vec<&dyn Run>> {
        match text {
            MadeText::DriveRow(_) => Some(vec![&self.drive_rows]),
            MadeText::Language(_) => Some(vec![&self.names.languages]),
            MadeText::Credit(_) => Some(vec![&self.vars.credits]),
            MadeText::Notice(_) | MadeText::StageHeading(_) | MadeText::PageNumber(_) => None,
        }
    }
}
