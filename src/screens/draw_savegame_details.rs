//! `screens::draw_savegame_details` (0x46D907): the load-game screen's preview of the saved
//! game that was read into the buffer at 0x51EBB4.
//!
//! In:  nothing
//! Out: every register preserved; carry as the last text drawn leaves it
//! After the headings and row numbers it copies the saved game's seven dwords into its own
//! variables (`dword_755150` .. `dword_755168`) and points `dword_75516C` at the saved cars.
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
use crate::menu::lists_store::MENU_LISTS;
use crate::records::find_record::find_record_in;
use crate::saves::gather_savegame::CAR_LEN;
use crate::saves::preview_store::SAVEGAME_PREVIEW;
use crate::saves::write_savegame_slot::SAVE_GAMES;
use crate::screens::stage_panel::{dword, table_dword_at};
use crate::text::draw_text::Canvas;

// ---- game data ----

const PAYLOAD_OFFSET: usize = 0x30; // 0x51EBE4

// ---- Rust version ----

/// Returns the carry flag of the last text drawn.
pub fn draw_savegame_details(canvas: &mut dyn Canvas) -> Result<bool, Refused> {
    // Converted: `rc2k_core::screens::savegame_preview::details`; the saved game's dwords copied
    // into the preview's variables first (the original after the frame, which reads none).
    use rc2k_core::screens::savegame_preview::{Car, Title, details};
    {
        let payload = SAVE_GAMES.file.addr() + PAYLOAD_OFFSET as u32;
        for (n, cell) in SAVEGAME_PREVIEW.saved.iter().enumerate() {
            cell.set(dword(payload + 4 * n as u32)?);
        }
        let cars = payload + 4 * SAVEGAME_PREVIEW.saved.len() as u32;
        SAVEGAME_PREVIEW.saved_cars.set(cars);
        let players = SAVEGAME_PREVIEW.saved[0].get() as i32;
        let index = SAVEGAME_PREVIEW.saved[2].get() as i32;
        let section = SAVEGAME_PREVIEW.saved[3].get();
        let mode_13 = SAVEGAME_PREVIEW.saved[5].get() == 0x13;
        let table = crate::records::stages_store::STAGE_TABLES.championship();
        let title = Title {
            mode_13,
            section: if mode_13 {
                table_dword_at(&MENU_LISTS, &MENU_LISTS.section_titles, section)?
            } else {
                dword(find_record_in(table, 0, section)? + 0x38)?
            },
            stage: dword(find_record_in(table, index, section)? + 0x40)?,
        };
        let f = crate::menu::event_flow::Fault::default();
        let car = |row: usize| {
            let car = cars + (row * CAR_LEN) as u32;
            Car {
                // The saved car's copy of its player's name (+0x124), read as far as its NUL
                // inside the read file.
                name: crate::screens::game_screen::HeldText::Place(
                    crate::pointers::GamePtr::to(&SAVE_GAMES.file)
                        .offset(car.wrapping_sub(SAVE_GAMES.file.addr()) + 0x124),
                ),
                nation: f.get(car + 4),
                car: f.get(car + 8),
                numbers: [0x14C, 0x150, 0x144, 0x148].map(|o| f.get(car + o)),
                shown: [f.get(car + 0xB0) != 0, f.get(car + 0xCC) != 0],
            }
        };
        let cars = [car(0), car(1), car(2), car(3)];
        f.check()?;
        details(
            &mut crate::screens::game_screen::PreviewScreen::new(canvas),
            title,
            players,
            &cars,
        )
    }
}
