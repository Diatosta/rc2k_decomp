//! Front-end pages' "unread" steps that copy settings into the page's work variables (stream
//! A, part F1, 2026-10-18).

use crate::errors::{BadInput, Refused};
use crate::menu::captions_store::CAPTIONS;
use crate::menu::champ_store::CHAMP_VIEW;
use crate::menu::event_store::EVENT_STATE;
use crate::menu::keys_store::KEY_TABLES;
use crate::menu::menus_store::MENUS;
use crate::menu::option_work_store::OPTION_WORK;
use crate::menu::results_store::RESULT_FIGURES;
use crate::menu::state_store::MENU_STATE;

/// `screens::pick_player_1` .. `_4` (0x46AEC5, 0x46AF21, 0x46AF58, 0x46AF8F): player `n`
/// the current one (`dword_75598C`), its +0x2C into `dword_7558C4` and its 0x2C bytes at
/// +0x50 into the page's copy (`unk_7555B1`). Every register kept. Refuses (before
/// anything) a player without its record.
pub fn pick_player(n: u32) -> Result<(), Refused> {
    // Converted: `rc2k_core::screens::page_values` (`Player::picked`).
    let (value, copy) = crate::menu::player::player(n)
        .ok_or(BadInput("a player without its record"))?
        .get()
        .picked();
    EVENT_STATE.current_player.set(n);
    EVENT_STATE.dword_7558c4.set(value);
    KEY_TABLES.controls.write(&copy);
    Ok(())
}

/// `screens::load_audio_settings` (0x4707E6): the audio options (`g_audioOptions`: quality,
/// channels, Dolby, A3D, EAX, DS3D) into the sound page's work variables (`dword_75566C`,
/// `dword_755680`, `dword_75567C`, `dword_755678`, `dword_755674`, `dword_755670`). Every
/// register kept.
pub fn load_audio_settings() {
    // Converted: `rc2k_core::screens::page_values::audio_page`.
    let options: rc2k_core::audio::modes::AudioOptions =
        crate::audio::store::AUDIO.audio_options.get();
    OPTION_WORK.set_audio_page(rc2k_core::screens::page_values::audio_page(&options));
}

/// `strings::append_string_by_id` into `out`.
fn push_id(out: &mut Vec<u8>, id: u32) -> Result<(), BadInput> {
    crate::strings::append_string_by_id::push_string(out, id)
}

/// The stage line string `id`, " ", the record's +0x40 as a fraction of 6
/// (`strings::append_fraction`), ": ", the name of its track-pack entry
/// (`records::trakpak_entry`'s first dword, a string id), and its NUL.
pub fn stage_line_bytes(id: u32, record: u32) -> Result<Vec<u8>, Refused> {
    crate::screens::draw_stage_heading::heading_text(record, id)
}

/// The stage line both stage pages build in `byte_A8BD54`.
fn stage_line(id: u32, record: u32) -> Result<(), Refused> {
    let line = stage_line_bytes(id, record)?;
    if line.len() > rc2k_core::screens::page_values::STAGE_LINE_ROOM {
        return Err(BadInput("a stage line past its buffer").into());
    }
    crate::pointers::GamePtr::to(&crate::screens::text_buffers_store::TEXT_BUFFERS.stage_heading)
        .write_bytes(&line, "the stage line")?;
    Ok(())
}

/// Menu item `n`'s value (+0x14).
fn set_item(items: &[crate::menu::menus_store::MenuItem], n: u32, v: u32) -> Result<(), BadInput> {
    items
        .get(n as usize)
        .ok_or(BadInput("an item past its menu's end"))?
        .string
        .set(v);
    Ok(())
}

/// The stage record `off_735512[a][b]` (its +0x38 the page's value), and the record + `c` x
/// 0xA4 the line is about.
fn stage_record(a: u32, b: u32, c: u32) -> Result<(u32, u32), BadInput> {
    let base = crate::records::stages_store::STAGE_TABLES.group_event_table(a, b)?;
    Ok((base, base.wrapping_add(c.wrapping_mul(0xA4))))
}

/// What both stage pages read through: refused before anything.
fn stage_refuses(a: u32, b: u32) -> Result<(), Refused> {
    crate::records::stages_store::STAGE_TABLES.group_event_table(a, b)?;
    Ok(())
}

/// `screens::stage_options_texts` (0x4692A9): the stage options page's items in the table at
/// `off_74DCE0` (its address: items 1 and 2 string 0x19C / 0x19B by `dword_7556F8`, 0x19E /
/// 0x19D by `dword_7556FC`, item 3 the record's +0x38), the stage line (string 0x481) and the
/// caption string 0x4FC + `dword_75AB30`. Every register kept. Refuses (before anything) no
/// stage records; a record of another track pack is the original's fatal error.
pub fn stage_options_texts() -> Result<(), Refused> {
    let (a, b) = (CHAMP_VIEW.switch_f8.get(), CAPTIONS.shown_group.get());
    stage_refuses(a, b)?;
    // Converted: `rc2k_core::screens::page_values` (the items and the caption).
    use rc2k_core::screens::page_values::{stage_options_caption, stage_options_items};
    let items = &MENUS.view_best_times.items;
    let (first, second) = stage_options_items(a, CHAMP_VIEW.switch_fc.get());
    set_item(items, 1, first)?;
    set_item(items, 2, second)?;
    let (base, record) = stage_record(a, b, CAPTIONS.shown_stage.get())?;
    set_item(
        items,
        3,
        crate::pointers::GamePtr::new(base + 0x38).read::<u32>("a stage record")?,
    )?;
    stage_line(0x481, record)?;
    let mut caption = Vec::new();
    push_id(
        &mut caption,
        stage_options_caption(MENU_STATE.menus.caption_choice.get()),
    )?;
    caption.push(0);
    crate::pointers::GamePtr::to(&crate::screens::text_buffers_store::TEXT_BUFFERS.caption)
        .write_bytes(&caption, "the caption")?;
    Ok(())
}

/// `screens::stage_choice_texts` (0x469434): the stage choice page's items: item 0 of the
/// table at `off_74DB68` (its address) `dword_758718[dword_7558C8]`, item 1 of the table it
/// points to the record's +0x38 (the original reads the table both ways: `notes/bugs.md`),
/// then the stage line (string 0x482) for record `dword_7558CC` of `off_735512[dword_7558C8]
/// [dword_7558D0]`. Every register kept. Refuses (before anything) no stage records or no
/// item table; another track pack is the original's fatal error.
pub fn stage_choice_texts() -> Result<(), Refused> {
    let (a, b) = (EVENT_STATE.stage_group.get(), EVENT_STATE.stage_sub.get());
    stage_refuses(a, b)?;
    let pointed = crate::menu::menus_store::labels_place_of(&MENUS.view_courses.head)?;
    if pointed == 0 {
        return Err(BadInput("no item table").into());
    }
    let items = &MENUS.view_courses.items;
    set_item(
        items,
        0,
        crate::session::read_on_in(
            &RESULT_FIGURES,
            &RESULT_FIGURES.group_titles,
            a as usize,
            "a group title",
        )?,
    )?;
    let (base, record) = stage_record(a, b, EVENT_STATE.stage_number.get())?;
    // Item 1 of the "menu" the head points to (the labels: the slip in `notes/bugs.md`).
    crate::pointers::GamePtr::new(pointed + 4 + 0x5C + 0x14).write::<u32>(
        crate::pointers::GamePtr::new(base + 0x38).read::<u32>("a stage record")?,
        "a menu item",
    )?;
    stage_line(0x482, record)
}
