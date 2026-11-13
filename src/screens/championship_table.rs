//! The championship's standings table: ten rows from `champ_view.first_row` of class
//! `event.class` in the championship history (entries with the player flag, car, flag, class,
//! name, six rounds' points (below 0: none), total and position).

use crate::errors::{FatalError, Refused};
use crate::screens::game_screen::{HeldText, held_name};
use crate::text::draw_text::Canvas;
use rc2k_core::saves::rankings::HistoryEntry;

const NOT_FOUND: FatalError = FatalError {
    code: 0x8070_A7FA,
    message: "Position in GB_ChampHST Not Found (see Log)",
};

/// The first history entry of class `class` and position `position` (the first entry is looked
/// at before the end test); none before an entry of -1 is fatal error 0x8070A7FA. `walked`: the
/// history's entries as far as its end entry.
pub fn find_history_entry(
    walked: &[HistoryEntry],
    class: u32,
    position: u32,
) -> Result<usize, Refused> {
    // The logic is in `rc2k_core::screens::championship_table::find_history`.
    use rc2k_core::screens::championship_table::find_history;
    let found = find_history(
        |n| walked.get(n as usize).map_or(u32::MAX, |e| e.first),
        |n| (walked[n as usize].group, walked[n as usize].position),
        class,
        position,
    );
    found.map(|n| n as usize).ok_or(NOT_FOUND.into())
}

/// The heads, the six rounds' flags (sprites 0x90 to 0x95 in event mode 0x13, else 0x8A to
/// 0x8F), then the ten rows (`LoopFlags.champ_row` from `ChampView.first_row` to
/// `ChampView.row_end` = it + 10): position, name, the rounds' points ("-" alone for below 0),
/// the total, the car's and flag's sprites; the player's own row in the highlight fonts; each
/// text from the last one's y, rows 0x19 apart. The sprites take the last text's leftover as
/// their palette (unused). Refuses no history table; a position not in it is a fatal error.
pub fn draw_championship_table(
    o: crate::owners::Owners<'static>,
    canvas: &mut dyn Canvas,
) -> Result<(), Refused> {
    let menus = o.menus;
    let walked = crate::menu::standings::walked_entries_b()?;
    // The logic is in `rc2k_core::screens::championship_table`; the ten entries are looked up
    // first (a position not in the history is a fatal error, raised before the rows rather than
    // after those before it).
    let (class, first) = (
        menus.screens.event.class.get(),
        menus.play.champ_view.first_row.get(),
    );
    let mut rows = Vec::with_capacity(10);
    for n in 0..10u32 {
        let e = &walked[find_history_entry(&walked, class, first.wrapping_add(n))?];
        rows.push(rc2k_core::screens::championship_table::Row {
            mine: (e.player as i32) > 0,
            nation: e.nation,
            car: e.car,
            name: held_name(e.name_text())?,
            points: std::array::from_fn(|r| e.stage[r] as i32),
            total: e.stage[6],
            position: e.position,
        });
    }
    let rows: [_; 10] = rows.try_into().unwrap_or_else(|_| unreachable!());
    rc2k_core::screens::championship_table::championship_table(
        &mut crate::screens::game_screen::GameScreen::<
            rc2k_core::screens::championship_table::Font,
            rc2k_core::screens::championship_table::Work,
            HeldText,
        >::new(canvas, o),
        first,
        menus.play.champ_view.event_mode.get() == 0x13,
        &rows,
    )
}
