//! A rally stage's information page (`sub_489AC8` -> `sub_4744A9`; stream A, part F1,
//! 2026-10-18): the stage record (`off_73555A[dword_7587EC]` + 0xA4 x `dword_7587F0`) and
//! its track-pack entry: start and finish times, the time allowed, the length, the surface's
//! shares, the weather, the description, the stage's number, and the best and fastest.

use crate::errors::{BadInput, Refused};
use crate::menu::arcade_store::ARCADE_GAME;
use crate::menu::champ_store::CHAMP_VIEW;
use crate::text::draw_text::Canvas;

unsafe fn get(a: u32) -> u32 {
    unsafe { (a as *const u32).read_unaligned() }
}

/// The page's stage record (none: 0).
pub fn stage_record() -> u32 {
    let base = crate::records::stages_store::STAGE_TABLES.level_table(ARCADE_GAME.level.get());
    if base == 0 {
        return 0;
    }
    base.wrapping_add(ARCADE_GAME.stage.get().wrapping_mul(0xA4))
}

/// `screens::draw_stage_best_and_fastest` (0x4744A9): the "best" and "fastest" lines
/// (`rc2k_core::screens::stage_panel::records`, converted 2026-10-20): the best of the times at edi
/// and the stage record's fastest (+0x74 nationality, +0x54 name, +0x78 car, +0x30 speed).
/// The fastest's flag sprite gets `dword_7587EC` as its palette (ebx as the original loads
/// it). The speed is formatted before anything is drawn (its refusals first). Every
/// register kept.
pub unsafe fn draw_stage_best_and_fastest(
    times: u32,
    canvas: &mut dyn Canvas,
) -> Result<(), Refused> {
    use rc2k_core::screens::stage_panel::{CarRow, Holder};
    unsafe {
        let record = stage_record();
        if record == 0 {
            return Err(BadInput("no stage record").into());
        }
        let units = crate::strings::units::units();
        crate::strings::append_speed::append_speed(get(record + 0x30), true, units)?;
        let best = Holder {
            nation: get(times + 8),
            name: times + 0x20,
            car: get(times + 0x10),
            figure: get(times + 0x14),
        };
        let top = Holder {
            nation: get(record + 0x74),
            name: record + 0x54,
            car: get(record + 0x78),
            figure: get(record + 0x30),
        };
        rc2k_core::screens::stage_panel::records(
            &mut crate::screens::stage_panel::PanelScreen::new(canvas),
            best,
            top,
            ARCADE_GAME.level.get(),
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
/// (`dword_7556E0` + 0x618 x record +0x3C). Every register kept. Refuses (before anything) no
/// stage record or times table.
pub unsafe fn draw_rally_stage_info(canvas: &mut dyn Canvas) -> Result<(), Refused> {
    use crate::screens::best_scores::{RALLY_STAGE_TIMES, draw_times_heading};
    unsafe {
        let record = stage_record();
        if record == 0 || CHAMP_VIEW.course_best_times.get() == 0 {
            return Err(BadInput("no stage record or times table").into());
        }
        draw_times_heading(&RALLY_STAGE_TIMES, canvas)?;
        // the two headings in `stru_51E564` here (the other stage pages: `stru_51E5C4`)
        crate::screens::stage_panel::draw_stage_body_in(
            record,
            ARCADE_GAME.level.get(),
            true,
            crate::gfx::typefaces::fonts::TIME,
            canvas,
        )?;
        let times = CHAMP_VIEW
            .course_best_times
            .get()
            .wrapping_add(get(record + 0x3C).wrapping_mul(0x618));
        draw_stage_best_and_fastest(times, canvas)
    }
}
