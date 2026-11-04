//! A rally stage's information page (`sub_489AC8` -> `sub_4744A9`; stream A, part F1,
//! 2026-10-18): the stage record (`off_73555A[dword_7587EC]` + 0xA4 x `dword_7587F0`) and
//! its track-pack entry: start and finish times, the time allowed, the length, the surface's
//! shares, the weather, the description, the stage's number, and the best and fastest.

use crate::errors::{BadInput, Refused};
use crate::menu::arcade_store::ARCADE_GAME;
use crate::screens::game_screen::{Held, HeldText};
use crate::text::draw_text::Canvas;

use crate::menu::event_flow::dword as get;

/// The page's stage record (none: 0).
pub fn stage_record() -> Result<u32, BadInput> {
    let base = crate::records::stages_store::STAGE_TABLES.level_table(ARCADE_GAME.level.get())?;
    crate::records::stages_store::row(base, ARCADE_GAME.stage.get())
}

/// `screens::draw_stage_best_and_fastest` (0x4744A9): the "best" and "fastest" lines
/// (`rc2k_core::screens::stage_panel::records`, converted 2026-10-20): the best of the times at edi
/// and the stage record's fastest (+0x74 nationality, +0x54 name, +0x78 car, +0x30 speed).
/// The fastest's flag sprite gets `dword_7587EC` as its palette (ebx as the original loads
/// it). The speed is formatted before anything is drawn (its refusals first). Every
/// register kept.
pub fn draw_stage_best_and_fastest(
    times: &rc2k_core::saves::entries::BestTime,
    canvas: &mut dyn Canvas,
) -> Result<(), Refused> {
    use rc2k_core::screens::stage_panel::{CarRow, Holder};
    {
        let record = stage_record()?;
        let units = crate::strings::units::units();
        crate::strings::append_speed::append_speed(get(record + 0x30)?, true, units)?;
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
        let top = Holder {
            nation: get(record + 0x74)?,
            name: HeldText::Place(
                crate::records::stages_store::STAGE_TABLES.record_text(record, 0x54),
            ),
            car: get(record + 0x78)?,
            figure: get(record + 0x30)?,
        };
        rc2k_core::screens::stage_panel::records(
            &mut crate::screens::stage_panel::PanelScreen::new(canvas),
            best,
            top,
            CarRow::Second,
            crate::strings::append_time::show_hours(),
            units,
        )?;
        Ok(())
    }
}

/// `screens::draw_rally_stage_info` (0x489AC8): the heading
/// (`screens::draw_rally_times_heading`'s), the stage panel's body
/// (`stage_panel::draw_stage_body`: the map's markers with `dword_7587EC` as their palette,
/// times, length, surface, weather, description and number; entry +0x34 kept in
/// `dword_75ACD0`'s second dword), then `draw_stage_best_and_fastest` on its times
/// (`g_courseBestTimes`, the stage the record's +0x3C). Every register kept. Refuses (before anything) no
/// stage record or times table.
pub fn draw_rally_stage_info(canvas: &mut dyn Canvas) -> Result<(), Refused> {
    use crate::screens::best_scores::{RALLY_STAGE_TIMES, draw_times_heading};
    let record = stage_record()?;
    let course_times = &crate::saves::best_times::COURSE_TIMES;
    if !course_times.is_taken() {
        return Err(BadInput("no stage record or times table").into());
    }
    draw_times_heading(&RALLY_STAGE_TIMES, canvas)?;
    // the two headings in `stru_51E564` here (the other stage pages: `stru_51E5C4`)
    crate::screens::stage_panel::draw_stage_body_in(
        record,
        true,
        crate::gfx::typefaces::fonts::TIME,
        canvas,
    )?;
    let times = crate::screens::stage_panel::best_of(course_times, get(record + 0x3C)?)?;
    draw_stage_best_and_fastest(&times, canvas)
}
