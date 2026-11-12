//! `screens::draw_savegame_details` (0x46D907): the load-game screen's preview of the saved
//! game that was read into the buffer at 0x51EBB4.
//!
//! After the headings and row numbers it copies the saved game's seven dwords into its own
//! variables (the shown saved game's figures) and points the shown saved cars at the saved cars.
//! Then:
//! - a title line at (143, 353): string 0x1D9 (0x1DA if the saved mode is 0x13), " - ", the
//!   title of the saved section (from a table in mode 0x13, else the section's first record
//!   at +0x38), a space, and for the saved stage's record "<record +0x40>/6" unless that
//!   value is 0;
//! - one row per saved car: its name (car +0x124), four numbers (car +0x14C, +0x150, then
//!   +0x144 and +0x148, each of those two replaced by string 0x57A if the car's dword at
//!   +0xB0, respectively +0xCC, is 0), a flag sprite (0x1C + car +4) and a car sprite
//!   (0x44 + car +8, with 0x18 and 0x19 swapped). A negative number is drawn as "-".
//!
//! Rows 2 to 4 only show a name and sprites if the first saved dword (the number of players,
//! by this use; a reading) reaches them; otherwise string 0x578 and two 0x57A stand there,
//! but the four numbers of that saved car are drawn all the same. The sprites are blitted
//! with whatever ebx holds as the palette (the end of the text drawn before them);
//! reproduced.
//! Called by `draw_savegame_preview`.
//!
//! Screen: seen in the owner's screenshot of 2026-10-10, the load-game screen ("CARREGAR
//! CAMPEONATO"), under the list of ten slots: the title "Campeonato Mobil One - Rali Vauxhall
//! do País de Gales 3/6" in yellow, the headings "jogador", "rali" and "campeonato" (player,
//! rally, championship), and row 1 with a flag, "Diatosta", a car and "1  1  -  -". So the
//! two numbers under "rali" are car +0x14C and +0x150 and the two under "campeonato" are
//! +0x144 and +0x148; string 0x57A is "-" and string 0x578 is "n/d". The save had one
//! player, and rows 2 to 4 showed "-", "n/d", "-" and four "-". What each of the two numbers
//! of a pair is (position and points, say) has not been worked out.

use crate::errors::Refused;
use crate::menu::owner::Tables;
use crate::records::find_record::find_record_in;
use crate::saves::gather_savegame::CAR_LEN;
use crate::saves::owner::Saves;
use crate::screens::stage_panel::section_title;
use crate::system::owner::System;
use crate::text::draw_text::Canvas;

// ---- game data ----

const PAYLOAD_OFFSET: usize = 0x30; // 0x51EBE4

/// Returns the carry flag of the last text drawn.
pub fn draw_savegame_details(
    o: crate::owners::Owners<'static>,
    tables: &Tables,
    saves: &Saves,
    system: &System,
    canvas: &mut dyn Canvas,
) -> Result<bool, Refused> {
    // The logic is in `rc2k_core::screens::savegame_preview::details`; the saved game's dwords copied
    // into the preview's variables first (the original after the frame, which reads none).
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
                section_title(&tables.lists, section)?
            } else {
                find_record_in(table, 0, section)?.record()?.name()
            },
            stage: find_record_in(table, index, section)?.record()?.number(),
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
