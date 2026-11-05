//! The itinerary pages (`sub_472C8C` and its sisters; stream A, part F1, 2026-10-18): a
//! section of the record table (`g_recordTable`, records 0xA4 bytes; +0 the tag: 0 a stage,
//! 1 a service, 3 / 4 the end; +0x10 its time of day as hhmm) listed row by row from y 0xE8
//! (`seconds`), the rows counted in `dword_7559B0`, the stages in `dword_7559B4`, the
//! services in `dword_755A0C`.

use crate::errors::{FatalError, Refused};
use crate::menu::champ_store::CHAMP_VIEW;
use crate::menu::event_store::EVENT_STATE;
use crate::menu::lists_store::MENU_LISTS;
use crate::records::stages_store::RecordRef;
use crate::screens::best_scores::{record_at, section_there};
use crate::screens::game_screen::GameScreen;
use crate::text::draw_text::Canvas;

const INVALID_STAGE: FatalError = FatalError {
    code: 0x8070_D91A,
    message: "Invalid Stage ID",
};

/// `screens::draw_itinerary` (0x472C8C): the section's heading (event mode 0x13
/// `dword_75A28A[section]`, else its first record's +0x38 string) at (0x2EB, 0x77), the column
/// heads (strings 0x482, 0x47E, 0x480, 0x47F at y 0xD1), then from record 1: a service its
/// name (string 0x478 + its number, kept in `dword_755A08`) and time; a stage (the row
/// `index` in the highlight fonts) its track-pack entry's name, its length (hundredths of a
/// mile, `strings::append_distance_2dp` and the unit's string 0x56F / 0x570; none for 0), its
/// number, and its time; tags 3 and 4 end the list, any other is fatal error 0x8070D91A
/// "Invalid Stage ID". Every register kept. Refuses (before anything) a section past the
/// table.
pub fn draw_itinerary(canvas: &mut dyn Canvas) -> Result<(), Refused> {
    // Converted: `rc2k_core::screens::itinerary`.
    use rc2k_core::screens::itinerary::Row;
    {
        if !section_there()? {
            return Err(crate::errors::BadInput("a section past the record table").into());
        }
        let first = record_at(0)?;
        let heading = if CHAMP_VIEW.event_mode.get() == 0x13 {
            crate::session::read_on_in(
                &MENU_LISTS,
                &MENU_LISTS.section_titles,
                EVENT_STATE.section.get() as usize,
                "a section title",
            )?
        } else {
            first.record()?.name()
        };
        let base = record_at(1)?;
        let record = move |n: u32| base.offset(n.wrapping_sub(1));
        let row = |n: u32| -> Result<Row, Refused> {
            let r = record(n)?;
            match r.tag() {
                1 => Ok(Row::Service {
                    hhmm: r.record()?.hhmm(),
                }),
                3 | 4 => Ok(Row::End),
                0 => {
                    let entry = crate::records::trakpak_entry::record_entry(r)?;
                    Ok(Row::Stage {
                        name: entry.read::<u32>("a track pack entry")?,
                        length: entry.offset(0xC).read::<u32>("a track pack entry")?,
                        hhmm: r.record()?.hhmm(),
                    })
                }
                _ => Err(INVALID_STAGE.into()),
            }
        };
        let fault = crate::menu::event_flow::Fault::default();
        let ends_at = |n: u32| fault.ok(record(n).map(RecordRef::tag)) == u32::MAX;
        let drawn = rc2k_core::screens::itinerary::itinerary(
            &mut GameScreen::<
                rc2k_core::screens::itinerary::Font,
                rc2k_core::screens::itinerary::Work,
            >::new(canvas),
            heading,
            &row,
            &ends_at,
            EVENT_STATE.index.get(),
            crate::strings::units::units(),
        );
        fault.check()?;
        drawn
    }
}
