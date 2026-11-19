//! The installer's variables the made texts live in (`strings::string_table::MadeTextBuffers`):
//! the drive rows, the languages found and the credits.

use crate::install::owner::Install;
use crate::memory::place::Run;
use crate::strings::string_table::{MadeText, MadeTextBuffers};

impl MadeTextBuffers for Install {
    fn chain(&self, text: MadeText) -> Option<Vec<&dyn Run>> {
        match text {
            MadeText::DriveRow(_) => Some(vec![
                &self.drive_rows.drive_rows,
                &self.drive_rows._gap_52a49f,
            ]),
            MadeText::Language(_) => {
                let names = &self.names;
                Some(vec![
                    &names.languages,
                    &names._gap_51c2ee,
                    &names.lang_cfg,
                    &names._gap_51c378,
                    &names.room_by_mode,
                    &names._tail,
                ])
            }
            MadeText::Credit(_) => {
                let vars = &self.vars;
                Some(vec![
                    &vars.credits_a,
                    &vars.clio_car_name,
                    &vars.credits_b,
                    &vars._tail,
                ])
            }
            MadeText::Notice(_) | MadeText::StageHeading(_) | MadeText::PageNumber(_) => None,
        }
    }
}
