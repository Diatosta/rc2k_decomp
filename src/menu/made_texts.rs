//! The menus' variables the made texts live in (`strings::string_table::MadeTextBuffers`): the
//! build notice and the screens' text buffers.

use crate::memory::place::Run;
use crate::menu::owner::Menus;
use crate::strings::string_table::{MadeText, MadeTextBuffers};

impl MadeTextBuffers for Menus {
    fn chain(&self, text: MadeText) -> Option<Vec<&dyn Run>> {
        match text {
            MadeText::Notice(_) => {
                let notice = &self.panels.notice;
                Some(vec![&notice.notice_first, &notice.notice_second])
            }
            MadeText::StageHeading(_) | MadeText::PageNumber(_) => {
                let text = &self.text_buffers;
                Some(vec![
                    &text.stage_heading,
                    &text.caption,
                    &text.title,
                    &text.number_text,
                    &text.stage_result_value,
                    &text.page_numbers,
                ])
            }
            MadeText::DriveRow(_) | MadeText::Language(_) | MadeText::Credit(_) => None,
        }
    }
}
