//! The stage screens' small set-ups and steps (`menu::view_stage_loop`,
//! `next_stage_loop`, `welcome_loop` and the stage page items; agent E, 2026-10-19). All
//! `pusha` .. `popa` (every register kept). `index` (`dword_755958`) and `section`
//! (`dword_75595C`) pick a record of `g_recordTable` (`records::find_record`, 0xA4 a
//! record); `g_eventMode` (`dword_7556D0`) 2 is the championship.
//! - `menu::count_stage_pages` (0x480032): how many records after the stage's have a first
//!   dword 0, into `dword_755968`.
//! - `menu::play_stage_speech` (0x47F437): in the championship, the stage's speech: with
//!   the record's +4, `dword_A8C188` 1, `dword_A8C184` its entry of `off_75A37E`, eax the
//!   dword that points at, `dword_A8C180` 0, ecx the speech's volume (`dword_755698`),
//!   `sound::start_stream`.
//! - `menu::play_rally_speech` (0x47F40F): the pending key taken (`input::take_key`); in
//!   the championship, eax `off_75A302[section]` (";3files\%speech\rally\rally1.wav" ..),
//!   ecx the speech's volume, `sound::start_stream`.
//! - `menu::load_welcome_picture` (0x4684BD): the welcome screen's background: eax
//!   `off_75987E[section]` (in the championship) or `off_759896[section]`, ebx the title
//!   bar "files\g\graphics\screens\titlebar.p..", edi the entry's address,
//!   `gfx::load_picture_with_overlay` (a set carry: fatal error 0x80712345 without a
//!   message), `gfx::convert_picture`.
//! - `menu::previous_stage_page` (0x46FAEE), `next_stage_page` (0x46FB27): the page
//!   `dword_75596C` one less (0: `dword_755968`) or one more (past `dword_755968`, signed:
//!   0), `index` = `dword_755960` + it, `menu::draw_stage_map` and `play_stage_speech`.
//! - `menu::style_stage_item_a` (0x470F9F): menu item 1 (`menu::item_at`) +0x1C and +0x20
//!   0xFFFFFFA9, +0x50 the font 0x51E9A4. `style_stage_item_b` (0x470FC3): +0x1C 0xFFFFFFA7,
//!   +0x20 `unk_7479AC`'s address, +0x50 0x51E884. What the two looks are is a reading.
//!
//! Refusals (before anything): a section past what `g_recordTable` has in 600 records;
//! more than 256 pages; an `off_75A37E` entry past 0x40 or null; a section past 0x20 for
//! the speeches and pictures; what `item_at` refuses.

use crate::errors::{BadInput, FatalError, Refused};
use crate::framework::{game_addr, game_read, game_write, rva};
use crate::menu::lists_store::MENU_LISTS;
use crate::records::find_record::{find_record_in, has_section};
use rc2k_core::menu::stage_screens;

const TABLE_RVA: usize = rva(0x735576); // g_recordTable
const INDEX_RVA: usize = rva(0x755958); // index
const SECTION_RVA: usize = rva(0x75595C); // section
const MODE_RVA: usize = rva(0x7556D0); // g_eventMode
const PAGES_RVA: usize = rva(0x755968); // dword_755968
const PAGE_RVA: usize = rva(0x75596C); // dword_75596C
const FIRST_RVA: usize = rva(0x755960); // dword_755960
const VOLUME_RVA: usize = rva(0x755698); // dword_755698
const TITLE_BAR_RVA: usize = rva(0x731C54); // "files\g\graphics\screens\titlebar.p.."
const RECORD: u32 = 0xA4;
fn g(a: usize) -> u32 {
    unsafe { game_read(a) }
}
fn p(a: usize, v: u32) {
    unsafe { game_write(a, v) }
}
fn get(a: u32) -> u32 {
    unsafe { (a as *const u32).read_unaligned() }
}

/// The record `index` / `section` pick.
fn stage_record() -> Result<u32, Refused> {
    let table = game_addr(TABLE_RVA) as u32;
    let section = g(SECTION_RVA);
    if !has_section(table, section, 600) {
        return Err(BadInput("a section past the record table").into());
    }
    Ok(find_record_in(table, g(INDEX_RVA) as i32, section))
}

fn championship() -> bool {
    stage_screens::championship(g(MODE_RVA))
}

fn small_section() -> Result<u32, Refused> {
    Ok(stage_screens::small_section(g(SECTION_RVA))?)
}

pub fn count_stage_pages() -> Result<(), Refused> {
    let record = stage_record()?;
    // Converted: `rc2k_core::menu::event::stage_pages`.
    let n = rc2k_core::menu::event::stage_pages(|k| get(record + RECORD * k))
        .ok_or(BadInput("more than 256 stage pages"))?;
    p(PAGES_RVA, n);
    Ok(())
}

/// `menu::play_stage_speech`.
pub unsafe fn play_stage_speech() -> Result<(), Refused> {
    let record = stage_record()?;
    if championship() {
        // Converted: `rc2k_core::menu::stage_screens::stage_speech`.
        let at = stage_screens::stage_speech(get(record + 4), |n| {
            get(MENU_LISTS.stage_speeches[0].addr() as u32 + 4 * n)
        })?;
        crate::sound::store::CAR_STREAM.on.set(1);
        crate::sound::store::CAR_STREAM.stream.set(at);
        crate::sound::store::CAR_STREAM.place.set(0);
        crate::sound::start_stream::start_stream(get(at), g(VOLUME_RVA));
    }
    Ok(())
}

/// `menu::play_rally_speech`: a key taken (and dropped), then the rally's speech.
pub unsafe fn play_rally_speech() -> Result<(), Refused> {
    // Converted: `rc2k_core::menu::stage_screens::rally_speech`.
    let section = stage_screens::rally_speech(g(MODE_RVA), || g(SECTION_RVA))?.unwrap_or(0);
    crate::input::take_key::take_key();
    if championship() {
        crate::sound::start_stream::start_stream(
            get(MENU_LISTS.rally_speeches[0].addr() as u32 + 4 * section),
            g(VOLUME_RVA),
        );
    }
    Ok(())
}

/// `menu::load_welcome_picture`: the section's welcome picture under the title bar,
/// converted; not loaded: the fatal error 0x80712345.
pub unsafe fn load_welcome_picture() -> Result<(), Refused> {
    let section = small_section()?;
    // Converted: `rc2k_core::menu::stage_screens::welcome`.
    let table = match stage_screens::welcome(g(MODE_RVA)) {
        stage_screens::Welcome::Championship => MENU_LISTS.welcome_championship[0].addr(),
        stage_screens::Welcome::Other => MENU_LISTS.welcome_other[0].addr(),
    };
    let path = get(table as u32 + 4 * section);
    let title_bar = game_addr(TITLE_BAR_RVA) as *mut u8;
    if unsafe {
        crate::gfx::load_picture_with_overlay::load_picture_with_overlay(path as *mut u8, title_bar)
    }? {
        return Err(FatalError {
            code: 0x8071_2345,
            message: "",
        }
        .into());
    }
    unsafe { crate::gfx::convert_picture::convert_picture_now() }?;
    Ok(())
}

/// `menu::previous_stage_page` (`up` false) / `next_stage_page`: the page stepped, the
/// stage's map drawn and its speech played.
pub unsafe fn step_stage_page(up: bool) -> Result<(), Refused> {
    let (page, pages) = (g(PAGE_RVA), g(PAGES_RVA));
    // Converted: `rc2k_core::menu::steps::stage_page`.
    let page = rc2k_core::menu::steps::stage_page(page, pages, up);
    p(PAGE_RVA, page);
    p(INDEX_RVA, g(FIRST_RVA).wrapping_add(page));
    unsafe {
        crate::menu::stage_choice::draw_stage_map()?;
        play_stage_speech()
    }
}

/// The stage item's two looks: (+0x1C, +0x20, +0x50).
pub fn style_stage_item(b: bool) -> Result<(), Refused> {
    let item = crate::menu::item_at::item_at(1)?;
    let (a, c, font) = if b {
        (0xFFFF_FFA7, game_addr(rva(0x7479AC)) as u32, 0x51E884)
    } else {
        (0xFFFF_FFA9, 0xFFFF_FFA9, 0x51E9A4)
    };
    unsafe {
        ((item + 0x1C) as *mut u32).write_unaligned(a);
        ((item + 0x20) as *mut u32).write_unaligned(c);
        ((item + 0x50) as *mut u32).write_unaligned(game_addr(rva(font)) as u32);
    }
    Ok(())
}
