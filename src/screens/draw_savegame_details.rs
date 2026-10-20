//! `screens::draw_savegame_details` (0x46D907): the load-game screen's preview of the saved
//! game that was read into the buffer at 0x51EBB4.
//!
//! In:  nothing
//! Out: every register preserved; carry as the last text drawn leaves it
//! After the headings and row numbers it copies the saved game's seven dwords into its own
//! variables (`dword_755150` .. `dword_755168`) and points `dword_75516C` at the saved cars.
//! Then:
//!   - a title line at (143, 353): string 0x1D9 (0x1DA if the saved mode is 0x13), " - ", the
//!     title of the saved section (from a table in mode 0x13, else the section's first record
//!     at +0x38), a space, and for the saved stage's record "<record +0x40>/6" unless that
//!     value is 0;
//!   - one row per saved car: its name (car +0x124), four numbers (car +0x14C, +0x150, then
//!     +0x144 and +0x148, each of those two replaced by string 0x57A if the car's dword at
//!     +0xB0, respectively +0xCC, is 0), a flag sprite (0x1C + car +4) and a car sprite
//!     (0x44 + car +8, with 0x18 and 0x19 swapped). A negative number is drawn as "-".
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
use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_read, game_write, hook_asm, rva};
use crate::records::find_record::{find_record_in, has_section};
use crate::saves::gather_savegame::CAR_LEN;
use crate::saves::write_savegame_slot::FILE_RVA;
use crate::screens::draw_empty_savegame::{preview_unavailable, state as text_state};
use crate::screens::draw_stage_info::RECORD_TABLE_RVA;
use crate::screens::harness::{Left, check_draw, dispatch_draw, push_dwords};
use crate::screens::line_buffer;
use crate::screens::stage_panel::{dword, table_dword};
use crate::service::harness::{Region, at, capture, restore};
use crate::strings::string_table::is_table_id;
use crate::text::draw_text::Canvas;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "screens::draw_savegame_details",
    rva(0x46D907),
    thunk,
    &ORIGINAL,
)
.self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- game data ----

/// Where the seven dwords of the previewed saved game are kept, in the order they are saved:
/// `dword_755150` .. `dword_755168`.
const PREVIEW_RVAS: [usize; 7] = [
    rva(0x755150), // the number of players, by its use here
    rva(0x755154),
    rva(0x755158), // the stage's index in its section
    rva(0x75515C), // the section
    rva(0x755160),
    rva(0x755164), // the mode: 0x13 takes the title from the table
    rva(0x755168),
];
const SAVED_CARS_PTR_RVA: usize = rva(0x75516C); // dword_75516C
const PAYLOAD_OFFSET: usize = 0x30; // 0x51EBE4
const SECTION_TITLES_RVA: usize = rva(0x75A28A); // dword_75A28A[section]: title string ids

// ---- Rust version ----

/// Returns the carry flag of the last text drawn.
pub unsafe fn draw_savegame_details(canvas: &mut dyn Canvas) -> Result<bool, Refused> {
    // Converted: `rc2k_core::screens::savegame_preview::details`; the saved game's dwords copied
    // into the preview's variables first (the original after the frame, which reads none).
    use rc2k_core::screens::savegame_preview::{Car, Title, details};
    unsafe {
        let payload = (game_addr(FILE_RVA) + PAYLOAD_OFFSET) as u32;
        for (n, at) in PREVIEW_RVAS.into_iter().enumerate() {
            game_write(at, dword(payload + 4 * n as u32));
        }
        let cars = payload + 4 * PREVIEW_RVAS.len() as u32;
        game_write(SAVED_CARS_PTR_RVA, cars);
        let players: i32 = game_read(PREVIEW_RVAS[0]);
        let index: i32 = game_read(PREVIEW_RVAS[2]);
        let section: u32 = game_read(PREVIEW_RVAS[3]);
        let mode_13 = game_read::<u32>(PREVIEW_RVAS[5]) == 0x13;
        let table = game_addr(RECORD_TABLE_RVA) as u32;
        let title = Title {
            mode_13,
            section: if mode_13 {
                table_dword(SECTION_TITLES_RVA, section)
            } else {
                dword(find_record_in(table, 0, section) + 0x38)
            },
            stage: dword(find_record_in(table, index, section) + 0x40),
        };
        let car = |row: usize| {
            let car = cars + (row * CAR_LEN) as u32;
            Car {
                name: car + 0x124,
                nation: dword(car + 4),
                car: dword(car + 8),
                numbers: [0x14C, 0x150, 0x144, 0x148].map(|o| dword(car + o)),
                shown: [dword(car + 0xB0) != 0, dword(car + 0xCC) != 0],
            }
        };
        details(
            &mut crate::screens::game_screen::PreviewScreen::new(canvas),
            title,
            players,
            &[car(0), car(1), car(2), car(3)],
        )
    }
}

// ---- hook ----

fn state() -> Vec<u8> {
    let mut s = text_state();
    push_dwords(&mut s, &PREVIEW_RVAS);
    push_dwords(&mut s, &[SAVED_CARS_PTR_RVA]);
    s
}

fn rust(_: &Regs, canvas: &mut dyn Canvas) -> Result<Left, Refused> {
    Ok(Left::carry(unsafe { draw_savegame_details(canvas) }?))
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_draw(&HOOK, r, call_original, &rust, &state);
}

// ---- self-test: the saved game that is in the buffer, if one is ----

/// Why what the buffer holds cannot be drawn safely, or None. The original scans the record
/// table for the saved section and follows what it finds, without checks.
pub unsafe fn details_unusable() -> Option<&'static str> {
    unsafe {
        let payload = (game_addr(FILE_RVA) + PAYLOAD_OFFSET) as u32;
        if game_read::<u32>(FILE_RVA) != crate::saves::write_savegame_slot::MAGIC {
            return Some("no saved game is in the buffer");
        }
        let (index, section, mode) = (dword(payload + 8), dword(payload + 12), dword(payload + 20));
        let table = game_addr(RECORD_TABLE_RVA) as u32;
        if index >= 64 || section >= 64 || !has_section(table, section, 600) {
            return Some("the saved section or stage is not in the record table");
        }
        let title = if mode == 0x13 {
            table_dword(SECTION_TITLES_RVA, section)
        } else {
            dword(find_record_in(table, 0, section) + 0x38)
        };
        if !is_table_id(title) || !is_table_id(0x1D9) || !is_table_id(0x1DA) {
            return Some("the saved section has no title string");
        }
        None
    }
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    if let Some(why) = preview_unavailable().or_else(|| unsafe { details_unusable() }) {
        return t.skip(why);
    }
    let mut regions: Vec<Region> = PREVIEW_RVAS.map(at).to_vec();
    regions.push(at(SAVED_CARS_PTR_RVA));
    regions.push((
        crate::strings::text_from_ids::text() as u32,
        crate::strings::text_from_ids::TEXT_LEN,
    ));
    regions.push((line_buffer() as u32, 0x80));
    // The number of players decides what rows 2 to 4 show: every value is tried.
    let players_at = (game_addr(FILE_RVA) + PAYLOAD_OFFSET) as u32;
    regions.push((players_at, 4));
    let saved = capture(&regions);
    for players in [1u32, 2, 3, 4] {
        check_draw(
            &mut t,
            &format!("players={players}"),
            call_original,
            Regs::SENTINEL,
            &|| {
                restore(&regions, &saved);
                unsafe { (players_at as *mut u32).write_unaligned(players) };
            },
            &rust,
            &state,
        );
    }
    restore(&regions, &saved);
    t.finish()
}
