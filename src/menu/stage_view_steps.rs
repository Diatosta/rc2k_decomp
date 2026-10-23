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
use crate::framework::{Regs, game_addr, game_read, game_write, rva};
use crate::menu::parts::Part;
use crate::records::find_record::{find_record_in, has_section};
use crate::screens::harness::Left;
use rc2k_core::menu::stage_screens;

const TABLE_RVA: usize = rva(0x735576); // g_recordTable
const INDEX_RVA: usize = rva(0x755958); // index
const SECTION_RVA: usize = rva(0x75595C); // section
const MODE_RVA: usize = rva(0x7556D0); // g_eventMode
const PAGES_RVA: usize = rva(0x755968); // dword_755968
const PAGE_RVA: usize = rva(0x75596C); // dword_75596C
const FIRST_RVA: usize = rva(0x755960); // dword_755960
const VOLUME_RVA: usize = rva(0x755698); // dword_755698
const SPEECH_ON_RVA: usize = rva(0xA8C188); // dword_A8C188
const SPEECH_AT_RVA: usize = rva(0xA8C184); // dword_A8C184
const SPEECH_CLEAR_RVA: usize = rva(0xA8C180); // dword_A8C180
const STAGE_SPEECHES_RVA: usize = rva(0x75A37E); // off_75A37E
const RALLY_SPEECHES_RVA: usize = rva(0x75A302); // off_75A302
const WELCOME_RVA: usize = rva(0x75987E); // off_75987E
const WELCOME_A8_RVA: usize = rva(0x759896); // off_759896
const TITLE_BAR_RVA: usize = rva(0x731C54); // "files\g\graphics\screens\titlebar.p.."
const RECORD: u32 = 0xA4;
const LOAD_PICTURE: usize = 0x4118D1;
const CONVERT_PICTURE: usize = 0x406126;
const DRAW_STAGE_MAP: usize = 0x4683CF;
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

/// `menu::play_stage_speech`: the registers it leaves.
pub unsafe fn play_stage_speech(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let record = stage_record()?;
    let mut r = *entry;
    r.edi = record;
    r.ebx = g(SECTION_RVA);
    r.eax = get(record + 4);
    if championship() {
        // Converted: `rc2k_core::menu::stage_screens::stage_speech`.
        let at = stage_screens::stage_speech(r.eax, |n| g(STAGE_SPEECHES_RVA + 4 * n as usize))?;
        p(SPEECH_ON_RVA, 1);
        p(SPEECH_AT_RVA, at);
        r.eax = get(at);
        p(SPEECH_CLEAR_RVA, 0);
        r.ecx = g(VOLUME_RVA);
        part(0, &mut r)?;
    }
    Ok(*entry)
}

/// `menu::play_rally_speech`: the registers it leaves.
pub unsafe fn play_rally_speech(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    // Converted: `rc2k_core::menu::stage_screens::rally_speech`.
    let section = stage_screens::rally_speech(g(MODE_RVA), || g(SECTION_RVA))?.unwrap_or(0);
    let mut r = *entry;
    part(1, &mut r)?;
    if championship() {
        r.eax = g(RALLY_SPEECHES_RVA + 4 * section as usize);
        r.ecx = g(VOLUME_RVA);
        part(0, &mut r)?;
    }
    Ok(*entry)
}

/// `menu::load_welcome_picture`: the registers it leaves.
pub unsafe fn load_welcome_picture(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let section = small_section()?;
    let mut r = *entry;
    r.ebx = section;
    // Converted: `rc2k_core::menu::stage_screens::welcome`.
    let table = match stage_screens::welcome(g(MODE_RVA)) {
        stage_screens::Welcome::Championship => WELCOME_RVA,
        stage_screens::Welcome::Other => WELCOME_A8_RVA,
    };
    r.edi = game_addr(table) as u32 + 4 * section;
    r.eax = get(r.edi);
    r.ebx = game_addr(TITLE_BAR_RVA) as u32;
    part(0, &mut r)?;
    if r.carry() {
        return Err(FatalError {
            code: 0x8071_2345,
            message: "",
        }
        .into());
    }
    part(1, &mut r)?;
    Ok(*entry)
}

/// `menu::previous_stage_page` (`up` false) / `next_stage_page`: the registers it leaves.
pub unsafe fn step_stage_page(entry: &Regs, up: bool, part: Part) -> Result<Regs, Refused> {
    let (page, pages) = (g(PAGE_RVA), g(PAGES_RVA));
    // Converted: `rc2k_core::menu::steps::stage_page`.
    let page = rc2k_core::menu::steps::stage_page(page, pages, up);
    p(PAGE_RVA, page);
    let index = g(FIRST_RVA).wrapping_add(page);
    p(INDEX_RVA, index);
    // eax at the calls: the index.
    let mut r = Regs {
        eax: index,
        ..*entry
    };
    part(0, &mut r)?;
    part(1, &mut r)?;
    Ok(*entry)
}

/// The stage item's two looks: (+0x1C, +0x20, +0x50).
pub fn style_stage_item(b: bool) -> Result<Left, Refused> {
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
    Ok(Left::nothing())
}

// ---- in the game ----

/// The calls by place, as each function's list has them: 0 `start_stream`, 1 `take_key`
/// (the speeches); 0 the picture's loading, 1 its conversion; 0 `draw_stage_map`, 1
/// `play_stage_speech` (the pages).
pub fn speech_part(place: u32, r: &mut Regs) -> Result<(), Refused> {
    match place {
        0 => crate::sound::senders::send_in_game(&crate::sound::start_stream::SENDER, r),
        _ => r.eax = r.eax & !0xFF | crate::input::take_key::take_key() as u32,
    }
    Ok(())
}

pub fn picture_part(place: u32, r: &mut Regs) -> Result<(), Refused> {
    let at = if place == 0 {
        LOAD_PICTURE
    } else {
        CONVERT_PICTURE
    };
    crate::calls::call(rva(at), r);
    Ok(())
}

pub fn page_part(place: u32, r: &mut Regs) -> Result<(), Refused> {
    if place == 0 {
        crate::calls::call(rva(DRAW_STAGE_MAP), r);
    } else {
        *r = unsafe { play_stage_speech(r, &mut speech_part) }?;
    }
    Ok(())
}

/// `dispatch_page` without its hook (typed tables).
pub fn page_entry(r: &mut Regs, up: bool) -> Result<(), Refused> {
    crate::menu::parts::parted_entry(
        r,
        &|r, part| unsafe { step_stage_page(r, up, part) },
        page_part,
    )
}

/// `dispatch_plain` without its hook (typed tables).
pub fn plain_entry(r: &mut Regs, rust: crate::service::harness::Rust) -> Result<(), Refused> {
    crate::service::harness::state_entry(r, rust)
}

// ---- self-tests ----
