//! The itinerary pages (`screens::draw_itinerary` and its sisters): a section of the record
//! table (records of 0xA4 bytes; a tag: 0 a stage, 1 a service, 3 / 4 the end; the time of day as
//! hhmm) listed row by row, with the rows, stages and services counted in the draw work.

use crate::errors::{FatalError, Refused};
use crate::screens::best_scores::{record_at, section_there};
use crate::screens::game_screen::GameScreen;
use crate::text::draw_text::Canvas;

const INVALID_STAGE: FatalError = FatalError {
    code: 0x8070_D91A,
    message: "Invalid Stage ID",
};

/// The section's heading (event mode 0x13: `MenuLists.section_titles[0][section]`, else its
/// first record's string), the column heads, then from record 1: a service its name and time;
/// a stage (the row `index` in the highlight fonts) its track-pack entry's name, its length
/// (hundredths of a mile and the unit's string; none for 0), its number and its time; tags 3 and
/// 4 end the list, any other is fatal error 0x8070D91A "Invalid Stage ID". Refuses a section
/// past the table.
pub fn draw_itinerary(
    o: crate::owners::Owners<'static>,
    system: &crate::system::owner::System,
    canvas: &mut dyn Canvas,
) -> Result<(), Refused> {
    let menus = o.menus;
    // The logic is in `rc2k_core::screens::itinerary`.
    use rc2k_core::screens::itinerary::Row;
    {
        if !section_there(menus, system)? {
            return Err(crate::errors::BadInput("a section past the record table").into());
        }
        let stages = &system.records().stage_tables;
        let first = record_at(menus, system, 0)?;
        let heading = if menus.play.champ_view.event_mode.get() == 0x13 {
            crate::screens::stage_panel::section_title(
                &menus.tables.lists,
                menus.screens.event.section.get(),
            )?
        } else {
            first.record(stages)?.name()
        };
        let base = record_at(menus, system, 1)?;
        let record = move |n: u32| base.offset(stages, n.wrapping_sub(1));
        let row = |n: u32| -> Result<Row, Refused> {
            let r = record(n)?;
            match r.tag(stages) {
                1 => Ok(Row::Service {
                    hhmm: r.record(stages)?.hhmm(),
                }),
                3 | 4 => Ok(Row::End),
                0 => {
                    let entry = crate::records::trakpak_entry::track_of(stages, r)?;
                    Ok(Row::Stage {
                        name: entry.name(stages)?,
                        length: entry.length(stages)?,
                        hhmm: r.record(stages)?.hhmm(),
                    })
                }
                _ => Err(INVALID_STAGE.into()),
            }
        };
        let fault = crate::menu::event_flow::Fault::default();
        let ends_at = |n: u32| fault.ok(record(n).map(|r| r.tag(stages))) == u32::MAX;
        let drawn = rc2k_core::screens::itinerary::itinerary(
            &mut GameScreen::<
                rc2k_core::screens::itinerary::Font,
                rc2k_core::screens::itinerary::Work,
            >::new(canvas, o),
            heading,
            &row,
            &ends_at,
            menus.screens.event.index.get(),
            crate::strings::units::units(),
        );
        fault.check()?;
        drawn
    }
}
