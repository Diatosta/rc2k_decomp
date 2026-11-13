//! A rally stage's information page (`screens::draw_rally_stage_info` and
//! `screens::draw_stage_best_and_fastest`): the stage record and its track-pack entry: start and
//! finish times, the time allowed, the length, the surface's shares, the weather, the
//! description, the stage's number, and the best and fastest.

use crate::errors::{BadInput, Refused};
use crate::menu::owner::Play;
use crate::records::stages_store::{Holder as RecordHolder, RecordRef};
use crate::screens::game_screen::{Held, HeldText};
use crate::text::draw_text::Canvas;

/// The page's stage record, on the system's tables.
pub fn stage_record_in(
    system: &crate::system::owner::System,
    play: &Play,
) -> Result<RecordRef, BadInput> {
    let base = system
        .records()
        .stage_tables
        .level_table(play.arcade.level.get())?;
    crate::records::stages_store::row(
        &system.records().stage_tables,
        base,
        play.arcade.stage.get(),
    )
}

/// The "best" and "fastest" lines (`rc2k_core::screens::stage_panel::records`): the best of
/// the given times and the stage record's fastest (nationality, name, car, speed). The fastest's
/// flag sprite gets `Arcade.level` as its palette (unused). The speed is formatted before
/// anything is drawn (its refusals first).
pub fn draw_stage_best_and_fastest(
    o: crate::owners::Owners<'static>,
    play: &Play,
    system: &crate::system::owner::System,
    times: &rc2k_core::saves::entries::BestTime,
    canvas: &mut dyn Canvas,
) -> Result<(), Refused> {
    use rc2k_core::screens::stage_panel::{CarRow, Holder};
    {
        let record = stage_record_in(system, play)?;
        let units = crate::strings::units::units();
        crate::strings::append_speed::append_speed(
            crate::strings::append_string_by_id::StringOwners::of(o),
            record
                .record(&system.records().stage_tables)?
                .holder(RecordHolder::Fastest)
                .speed,
            true,
            units,
        )?;
        let name = times
            .name_text()
            .and_then(|t| Held::new(&t))
            .ok_or(BadInput("a best time's name without its end"))?;
        let best = Holder {
            nation: times.head.nation,
            name: HeldText::Held(name),
            car: times.head.car,
            figure: times.head.key,
        };
        let held = record
            .record(&system.records().stage_tables)?
            .holder(RecordHolder::Fastest);
        let top = Holder {
            nation: held.nation,
            name: HeldText::Held(
                record.holder_text(&system.records().stage_tables, RecordHolder::Fastest)?,
            ),
            car: held.car,
            figure: held.speed,
        };
        rc2k_core::screens::stage_panel::records(
            &mut crate::screens::stage_panel::PanelScreen::new(canvas, o),
            best,
            top,
            CarRow::Second,
            crate::strings::append_time::show_hours(),
            units,
        )?;
        Ok(())
    }
}

/// The rally times heading (`best_scores::draw_times_heading`), the stage panel's body
/// (`stage_panel::draw_stage_body`: the map's markers with `Arcade.level` as their palette,
/// times, length, surface, weather, description and number; an entry kept in
/// `RaceState.settings.rivals`'s second dword), then `draw_stage_best_and_fastest` on the stage's
/// times. Refuses no stage record or times table.
pub fn draw_rally_stage_info(
    o: crate::owners::Owners<'static>,
    system: &crate::system::owner::System,
    gfx: &crate::gfx::owner::Gfx,
    canvas: &mut dyn Canvas,
) -> Result<(), Refused> {
    let menus = o.menus;
    let play = &menus.play;
    use crate::screens::best_scores::{RALLY_STAGE_TIMES, draw_times_heading};
    let record = stage_record_in(system, play)?;
    let course_times = &crate::saves::best_times::COURSE_TIMES;
    if !course_times.is_taken() {
        return Err(BadInput("no stage record or times table").into());
    }
    draw_times_heading(o, system, &RALLY_STAGE_TIMES, canvas)?;
    // the two headings in `FontObjects.objects[14].first_char` here (the other stage pages:
    // `FontObjects.objects[17].first_char`)
    crate::screens::stage_panel::draw_stage_body_in(
        o,
        gfx,
        record,
        true,
        crate::gfx::typefaces::fonts::TIME,
        canvas,
    )?;
    let times = crate::screens::stage_panel::best_of(
        course_times,
        record.record(&system.records().stage_tables)?.event_slot(),
    )?;
    draw_stage_best_and_fastest(o, play, system, &times, canvas)
}
