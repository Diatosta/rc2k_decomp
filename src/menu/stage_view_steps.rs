//! The stage screens' small set-ups and steps (`menu::view_stage_loop`, `next_stage_loop`,
//! `welcome_loop` and the stage page items). `event.stage_in_section` and `event.section` pick a
//! record
//! of the record table (`records::find_record`, 0xA4 a record); event mode 2 is the
//! championship.
//! - `count_stage_pages`: how many records after the stage's have a first dword 0, into
//!   `event.stage.pages`.
//! - `previous_stage_page` / `next_stage_page`: `event.stage.page` one less or more, wrapping
//!   over 0 .. `event.stage.pages`; `index` = `event.stage.first_on_page` + the page, then the map
//!   is drawn and the stage's speech played.
//! - `play_stage_speech`, `play_rally_speech` (which first takes the pending key): in the
//!   championship, the stage's or the section's speech through `sound::start_stream` at the
//!   options' speech level.
//! - `load_welcome_picture`: the section's picture (from the championship's list or the other)
//!   with the title bar; a failed load is fatal error 0x80712345.
//! - `style_stage_item_a` / `_b`: the look of menu item 1 (`menu::item_at`); what the two looks
//!   are is a reading.
//!
//! Refusals (before anything): a section past what the record table has in 600 records; more
//! than 256 pages; a speech list entry past 0x40 or null; a section past 0x20 for the speeches
//! and pictures; what `item_at` refuses.

use crate::errors::Fault;
use crate::errors::{BadInput, FatalError, Refused};
use crate::menu::champ_store::ChampView;
use crate::menu::owner::Menus;
use crate::menu::owner::Screens;
use crate::owners::Owners;
use crate::records::find_record::{find_record_in, has_section};
use crate::records::stages_store::RecordRef;
use crate::sound::store::CarStream;
use crate::system::owner::System;
use rc2k_core::menu::stage_screens;

/// The record `index` / `section` pick.
fn stage_record(screens: &Screens, system: &System) -> Result<RecordRef, Refused> {
    let table = system.records().stage_tables.championship();
    let section = screens.event.section.get();
    if !has_section(&system.records().stage_tables, table, section, 600)? {
        return Err(BadInput("a section past the record table").into());
    }
    Ok(find_record_in(
        &system.records().stage_tables,
        table,
        screens.event.stage_in_section.get() as i32,
        section,
    )?)
}

fn championship(champ: &ChampView) -> bool {
    stage_screens::championship(champ.event_mode.get())
}

fn small_section(menus: &Menus) -> Result<u32, Refused> {
    Ok(stage_screens::small_section(
        menus.screens.event.section.get(),
    )?)
}

pub fn count_stage_pages(screens: &Screens, system: &System) -> Result<(), Refused> {
    let stages = &system.records().stage_tables;
    let record = stage_record(screens, system)?;
    let fault = Fault::default();
    let n = rc2k_core::menu::event::stage_pages(|k| {
        fault.ok(record.offset(stages, k).map(|r| r.tag(stages)))
    });
    fault.check()?;
    let n = n.ok_or(BadInput("more than 256 stage pages"))?;
    screens.event.stage.pages.set(n);
    Ok(())
}

/// `menu::play_stage_speech`.
pub fn play_stage_speech(o: crate::owners::Owners<'static>) -> Result<(), Refused> {
    let (menus, sound, audio, system) = (o.menus, o.sound, o.audio, o.system);
    let record = stage_record(&menus.screens, system)?;
    if championship(&menus.play.champ_view) {
        let fault = std::cell::Cell::new(None);
        let at = stage_screens::stage_speech(
            record.record(&system.records().stage_tables)?.track(),
            |n| {
                menus.lists.stage_speech(n as usize).unwrap_or_else(|e| {
                    fault.set(Some(e));
                    0
                })
            },
        );
        if let Some(e) = fault.take() {
            return Err(e.into());
        }
        let at = at?;
        *sound.race.car_stream.get().borrow_mut() = CarStream {
            on: 1,
            stream: at,
            place: 0,
        };
        crate::sound::start_stream::start_stream(
            o,
            crate::menu::texts_store::speech_entry(&menus.texts, &audio.sample_names, at, 0)?,
            menus.options.snapshot().menu_voices_volume,
        );
    }
    Ok(())
}

/// `menu::play_rally_speech`: a key taken (and dropped), then the rally's speech.
pub fn play_rally_speech(o: crate::owners::Owners<'static>) -> Result<(), Refused> {
    let (menus, input) = (o.menus, o.input);
    let section = stage_screens::rally_speech(menus.play.champ_view.event_mode.get(), || {
        menus.screens.event.section.get()
    })?
    .unwrap_or(0);
    crate::input::take_key::take_key(input);
    if championship(&menus.play.champ_view) {
        crate::sound::start_stream::start_stream(
            o,
            menus.lists.rally_speech(section as usize)?,
            menus.options.snapshot().menu_voices_volume,
        );
    }
    Ok(())
}

/// `menu::load_welcome_picture`: the section's welcome picture under the title bar;
/// not loaded: the fatal error 0x80712345.
pub fn load_welcome_picture(o: Owners<'static>) -> Result<(), Refused> {
    let menus = o.menus;
    let section = small_section(menus)?;
    let championship = matches!(
        stage_screens::welcome(menus.play.champ_view.event_mode.get()),
        stage_screens::Welcome::Championship
    );
    let path = crate::menu::texts_store::text_at(
        &menus.texts,
        menus
            .lists
            .welcome_picture(championship, section as usize)?,
    )?;
    if crate::gfx::load_picture_with_overlay::load_picture_with_overlay_in(
        &path,
        &menus.texts.titlebar_png,
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
    crate::gfx::convert_picture::convert_picture_now_in(o.gfx)?;
    Ok(())
}

/// `menu::previous_stage_page` (`up` false) / `next_stage_page`: the page stepped, the
/// stage's map drawn and its speech played.
pub(super) fn step_stage_page(o: Owners<'static>, up: bool) -> Result<(), Refused> {
    let menus = o.menus;
    let screens = &menus.screens;
    let (page, pages) = (
        screens.event.stage.page.get(),
        screens.event.stage.pages.get(),
    );
    let page = rc2k_core::menu::steps::stage_page(page, pages, up);
    screens.event.stage.page.set(page);
    screens
        .event
        .stage_in_section
        .set(screens.event.stage.first_on_page.get().wrapping_add(page));
    crate::menu::stage_choice::draw_stage_map(o)?;
    play_stage_speech(o)
}

/// The stage item's two looks: what is behind its text, its sprite and its highlight font.
pub(super) fn style_stage_item(menus: &Menus, b: bool) -> Result<(), Refused> {
    use crate::menu::item_sprites_store::ItemSprite;
    let item = crate::menu::item_at::item_at(menus, 1)?;
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
