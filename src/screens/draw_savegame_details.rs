//! The load-game screen's preview of the saved game that was read into the buffer: headings and row
//! numbers, a title line (the mode's string, the saved section's title and, for the saved stage's
//! record, `<record>/6` unless 0), and one row per saved car: name, four numbers, a flag sprite and
//! a car sprite. A negative number is drawn as "-".
//!
//! Rows 2 to 4 only show a name and sprites if the first saved dword (the number of players, by
//! this use; a reading) reaches them; otherwise string 0x578 and two 0x57A stand there, but the
//! four numbers of that saved car are drawn all the same (kept).
//!
//! Seen in a screenshot of the load-game screen: the title in yellow, the headings (player, rally,
//! championship), and row 1 with a flag, a name, a car and "1  1  -  -"; so the two numbers under
//! "rally" and the two under "championship" are two pairs of car fields. String 0x57A is "-" and
//! string 0x578 is "n/d". What each number of a pair is (position and points, say) has not been
//! worked out.

use crate::errors::Refused;
use crate::records::find_record::find_record_in;
use crate::saves::gather_savegame::CAR_LEN;
use crate::saves::owner::Saves;
use crate::screens::stage_panel::section_title;
use crate::system::owner::System;
use crate::text::draw_text::Canvas;

// ---- game data ----

const PAYLOAD_OFFSET: usize = 0x30;

/// Returns whether the last text drawn was too wide for its box.
pub fn draw_savegame_details(
    o: crate::owners::Owners<'static>,
    saves: &Saves,
    system: &System,
    canvas: &mut dyn Canvas,
) -> Result<bool, Refused> {
    // The logic is in `rc2k_core::screens::savegame_preview::details`; the saved game's dwords
    // are copied into the preview's variables first.
    use rc2k_core::screens::savegame_preview::{Car, Title, details};
    {
        // The read file's dwords and cars, by offset in it.
        let file = &saves.save_games.file;
        let saved_dword = |at: usize| -> Result<u32, crate::errors::BadInput> {
            let b = file.read_at(at, 4, "a saved game's head")?;
            Ok(u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
        };
        for (n, cell) in saves.savegame_preview.saved.iter().enumerate() {
            cell.set(saved_dword(PAYLOAD_OFFSET + 4 * n)?);
        }
        let cars_at = PAYLOAD_OFFSET + 4 * saves.savegame_preview.saved.len();
        let players = saves.savegame_preview.saved[0].get() as i32;
        let index = saves.savegame_preview.saved[2].get() as i32;
        let section = saves.savegame_preview.saved[3].get();
        let a8_mode = saves.savegame_preview.saved[5].get() == 0x13;
        let table = system.records().stage_tables.championship();
        let title = Title {
            a8_mode,
            section: if a8_mode {
                section_title(&o.menus.lists, section)?
            } else {
                find_record_in(&system.records().stage_tables, table, 0, section)?
                    .record(&system.records().stage_tables)?
                    .name()
            },
            stage: find_record_in(&system.records().stage_tables, table, index, section)?
                .record(&system.records().stage_tables)?
                .number(),
        };
        let car = |row: usize| -> Result<Car<_>, crate::errors::BadInput> {
            let at = cars_at + row * CAR_LEN;
            let bytes = file.read_at(at, CAR_LEN, "a saved car")?;
            let p = rc2k_core::menu::player::Player::from_le_bytes(
                bytes.try_into().expect("CAR_LEN bytes"),
            );
            Ok(Car {
                // The saved car's copy of its player's name (+0x124), read as far as its NUL
                // inside the read file.
                name: crate::screens::game_screen::HeldText::SavedCarName(at + 0x124),
                nation: p.nationality,
                car: p.car_number,
                numbers: [
                    p.number_standings_a,
                    p.number_all_a,
                    p.number_standings_b,
                    p.number_all_b,
                ],
                shown: [p.stage_points[6] != 0, p.overall_points[6] != 0],
            })
        };
        let cars = [car(0)?, car(1)?, car(2)?, car(3)?];
        details(
            &mut crate::screens::game_screen::PreviewScreen::new(canvas, o),
            title,
            players,
            &cars,
        )
    }
}
