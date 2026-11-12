//! The stage screens' small set-ups and steps (`menu::view_stage_loop`, `next_stage_loop`,
//! `welcome_loop` and the stage page items). `event.index` and `event.section` pick a record
//! of the record table (`records::find_record`, 0xA4 a record); the event mode 2 is the
//! championship.
//! - `menu::count_stage_pages`: how many records after the stage's have a first dword 0, into
//!   `event.stage_pages`.
//! - `menu::play_stage_speech`: in the championship, the stage's speech: the record's +4
//!   picks an entry of the speech name list, which names the sound file; the speech's volume
//!   is the options' speech level; `sound::start_stream`.
//! - `menu::play_rally_speech`: the pending key taken (`input::take_key`); in the
//!   championship, the section's rally speech (";3files\%speech\rally\rally1.wav" ..) at the
//!   speech's volume, `sound::start_stream`.
//! - `menu::load_welcome_picture`: the welcome screen's background: the section's picture
//!   (in the championship the championship list, else the other) with the title bar
//!   "files\g\graphics\screens\titlebar.p..", `gfx::load_picture_with_overlay` (a set carry:
//!   fatal error 0x80712345 without a message), `gfx::convert_picture`.
//! - `menu::previous_stage_page`, `next_stage_page`: `event.stage_page` one less (0:
//!   `event.stage_pages`) or one more (past `stage_pages`, signed: 0), `index` =
//!   `event.first_page_stage` + it, `menu::draw_stage_map` and `play_stage_speech`.
//! - `menu::style_stage_item_a`: menu item 1 (`menu::item_at`) +0x1C and +0x20 0xFFFFFFA9,
//!   +0x50 the font 0x51E9A4. `style_stage_item_b`: +0x1C 0xFFFFFFA7, +0x20 the address of
//!   a sprite list, +0x50 0x51E884. What the two looks are is a reading.
//!
//! Refusals (before anything): a section past what `g_recordTable` has in 600 records;
//! more than 256 pages; a speech list entry past 0x40 or null; a section past 0x20 for
//! the speeches and pictures; what `item_at` refuses.

use crate::errors::{BadInput, FatalError, Refused};
use crate::menu::event_flow::Fault;
use crate::menu::owner::Menus;
use crate::menu::owner::Screens;
use crate::owners::Owners;
use crate::records::find_record::{find_record_in, has_section};
use crate::records::stages_store::RecordRef;
use crate::system::owner::system;
use rc2k_core::menu::stage_screens;

/// The record `index` / `section` pick.
fn stage_record(screens: &Screens) -> Result<RecordRef, Refused> {
    let table = system().records.stage_tables.championship();
    let section = screens.event.section.get();
    if !has_section(table, section, 600)? {
        return Err(BadInput("a section past the record table").into());
    }
    Ok(find_record_in(
        table,
        screens.event.index.get() as i32,
        section,
    )?)
}

fn championship(menus: &Menus) -> bool {
    stage_screens::championship(menus.play.champ_view.event_mode.get())
}

fn small_section(menus: &Menus) -> Result<u32, Refused> {
    Ok(stage_screens::small_section(
        menus.screens.event.section.get(),
    )?)
}

pub fn count_stage_pages(screens: &Screens) -> Result<(), Refused> {
    let record = stage_record(screens)?;
    // The logic is in `rc2k_core::menu::event::stage_pages`.
    let fault = Fault::default();
    let n = rc2k_core::menu::event::stage_pages(|k| fault.ok(record.offset(k).map(RecordRef::tag)));
    fault.check()?;
    let n = n.ok_or(BadInput("more than 256 stage pages"))?;
    screens.event.stage_pages.set(n);
    Ok(())
}

/// `menu::play_stage_speech`.
pub fn play_stage_speech(menus: &Menus, sound: &crate::sound::owner::Sound) -> Result<(), Refused> {
    let record = stage_record(&menus.screens)?;
    if championship(menus) {
        // The logic is in `rc2k_core::menu::stage_screens::stage_speech`.
        let fault = std::cell::Cell::new(None);
        let at = stage_screens::stage_speech(record.record()?.track(), |n| {
            crate::session::read_on_in(
                &menus.tables.lists,
                &menus.tables.lists.stage_speeches,
                n as usize,
                "a stage speech",
            )
            .unwrap_or_else(|e| {
                fault.set(Some(e));
                0
            })
        });
        if let Some(e) = fault.take() {
            return Err(e.into());
        }
        let at = at?;
        sound.race.car_stream.get().on.set(1);
        sound.race.car_stream.get().stream.set(at);
        sound.race.car_stream.get().place.set(0);
        crate::sound::start_stream::start_stream(
            crate::menu::texts_store::speech_entry(at, 0)?,
            menus.screens.option_work.speech_level.get(),
        );
    }
    Ok(())
}

/// `menu::play_rally_speech`: a key taken (and dropped), then the rally's speech.
pub fn play_rally_speech(menus: &Menus) -> Result<(), Refused> {
    // The logic is in `rc2k_core::menu::stage_screens::rally_speech`.
    let section = stage_screens::rally_speech(menus.play.champ_view.event_mode.get(), || {
        menus.screens.event.section.get()
    })?
    .unwrap_or(0);
    crate::input::take_key::take_key();
    if championship(menus) {
        crate::sound::start_stream::start_stream(
            crate::session::read_on_in(
                &menus.tables.lists,
                &menus.tables.lists.rally_speeches,
                section as usize,
                "a rally speech",
            )?,
            menus.screens.option_work.speech_level.get(),
        );
    }
    Ok(())
}

/// `menu::load_welcome_picture`: the section's welcome picture under the title bar,
/// converted; not loaded: the fatal error 0x80712345.
pub fn load_welcome_picture(o: Owners<'_>) -> Result<(), Refused> {
    let menus = o.menus;
    let section = small_section(menus)?;
    // The logic is in `rc2k_core::menu::stage_screens::welcome`.
    let table: &[crate::session::PlainDword] =
        match stage_screens::welcome(menus.play.champ_view.event_mode.get()) {
            stage_screens::Welcome::Championship => &menus.tables.lists.welcome_championship,
            stage_screens::Welcome::Other => &menus.tables.lists.welcome_other,
        };
    let path = crate::menu::texts_store::text_at(crate::session::read_on_in(
        &menus.tables.lists,
        table,
        section as usize,
        "a welcome picture",
    )?)?;
    if crate::gfx::load_picture_with_overlay::load_picture_with_overlay_in(
        &path,
        &menus.tables.texts.titlebar_png,
        o.gfx,
        o.system,
        o.strings,
    )? {
        return Err(FatalError {
            code: 0x8071_2345,
            message: "",
        }
        .into());
    }
    crate::gfx::convert_picture::convert_picture_now_in(o.gfx, o.system)?;
    Ok(())
}

/// `menu::previous_stage_page` (`up` false) / `next_stage_page`: the page stepped, the
/// stage's map drawn and its speech played.
pub fn step_stage_page(o: Owners<'_>, up: bool) -> Result<(), Refused> {
    let (menus, sound) = (o.menus, o.sound);
    let screens = &menus.screens;
    let (page, pages) = (
        screens.event.stage_page.get(),
        screens.event.stage_pages.get(),
    );
    // The logic is in `rc2k_core::menu::steps::stage_page`.
    let page = rc2k_core::menu::steps::stage_page(page, pages, up);
    screens.event.stage_page.set(page);
    screens
        .event
        .index
        .set(screens.event.first_page_stage.get().wrapping_add(page));
    crate::menu::stage_choice::draw_stage_map(o)?;
    play_stage_speech(menus, sound)
}

/// The stage item's two looks: (+0x1C, +0x20, +0x50).
pub fn style_stage_item(screens: &Screens, b: bool) -> Result<(), Refused> {
    use crate::menu::item_sprites_store::ItemSprite;
    let item = crate::menu::item_at::item_at(screens, 1)?;
    let (a, c, font) = if b {
        (
            0xFFFF_FFA7,
            ItemSprite::List(1),
            crate::gfx::typefaces::fonts::STAGE_ITEM_B,
        )
    } else {
        (
            0xFFFF_FFA9,
            ItemSprite::Sprite(0xFFFF_FFA9),
            crate::gfx::typefaces::fonts::STAGE_TITLE,
        )
    };
    item.behind.set(a);
    item.sprites.set(c)?;
    item.highlight_font.set(font);
    Ok(())
}
