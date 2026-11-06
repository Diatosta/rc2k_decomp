//! Front-end pages' "unread" steps that copy settings into the page's work variables (stream
//! A, part F1, 2026-10-18).

use crate::errors::{BadInput, Refused};
use crate::records::stages_store::{RecordRef, RecordTable};

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
    crate::menu::owner::menus()
        .screens
        .event
        .current_player
        .set(n);
    crate::menu::owner::menus()
        .screens
        .event
        .dword_7558c4
        .set(value);
    crate::menu::owner::menus()
        .keys
        .tables
        .controls
        .write(&copy);
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
    crate::menu::owner::menus()
        .screens
        .option_work
        .set_audio_page(rc2k_core::screens::page_values::audio_page(&options));
}

/// `strings::append_string_by_id` into `out`.
fn push_id(out: &mut Vec<u8>, id: u32) -> Result<(), BadInput> {
    crate::strings::append_string_by_id::push_string(out, id)
}

/// The stage line string `id`, " ", the record's +0x40 as a fraction of 6
/// (`strings::append_fraction`), ": ", the name of its track-pack entry
/// (`records::trakpak_entry`'s first dword, a string id), and its NUL.
pub fn stage_line_bytes(id: u32, record: RecordRef) -> Result<Vec<u8>, Refused> {
    crate::screens::draw_stage_heading::heading_text(record, id)
}

/// The stage line both stage pages build in `byte_A8BD54`.
fn stage_line(id: u32, record: RecordRef) -> Result<(), Refused> {
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
fn stage_record(a: u32, b: u32, c: u32) -> Result<(RecordTable, RecordRef), BadInput> {
    let base = crate::records::stages_store::STAGE_TABLES.group_event_table(a, b)?;
    Ok((base, crate::records::stages_store::row(base, c)?))
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
    let (a, b) = (
        crate::menu::owner::menus().play.champ_view.switch_f8.get(),
        crate::menu::owner::menus()
            .tables
            .captions
            .shown_group
            .get(),
    );
    stage_refuses(a, b)?;
    // Converted: `rc2k_core::screens::page_values` (the items and the caption).
    use rc2k_core::screens::page_values::{stage_options_caption, stage_options_items};
    let items = &crate::menu::owner::menus()
        .tables
        .menus
        .view_best_times
        .items;
    let (first, second) = stage_options_items(
        a,
        crate::menu::owner::menus().play.champ_view.switch_fc.get(),
    );
    set_item(items, 1, first)?;
    set_item(items, 2, second)?;
    let (base, record) = stage_record(
        a,
        b,
        crate::menu::owner::menus()
            .tables
            .captions
            .shown_stage
            .get(),
    )?;
    set_item(items, 3, base.title()?)?;
    stage_line(0x481, record)?;
    let mut caption = Vec::new();
    push_id(
        &mut caption,
        stage_options_caption(
            crate::menu::owner::menus()
                .screens
                .state
                .menus
                .caption_choice
                .get(),
        ),
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
    let (a, b) = (
        crate::menu::owner::menus().screens.event.stage_group.get(),
        crate::menu::owner::menus().screens.event.stage_sub.get(),
    );
    stage_refuses(a, b)?;
    // Item 1 of the "menu" the head points to (the labels: the slip in `notes/bugs.md`).
    let item_1 = crate::menu::menus_store::labels_slot(
        &crate::menu::owner::menus().tables.menus.view_courses.head,
        4 + 0x5C + 0x14,
    )?
    .ok_or(BadInput("no item table"))?;
    let items = &crate::menu::owner::menus().tables.menus.view_courses.items;
    set_item(
        items,
        0,
        crate::session::read_on_in(
            &crate::menu::owner::menus().play.results,
            &crate::menu::owner::menus().play.results.group_titles,
            a as usize,
            "a group title",
        )?,
    )?;
    let (base, record) = stage_record(
        a,
        b,
        crate::menu::owner::menus().screens.event.stage_number.get(),
    )?;
    item_1.write::<u32>(base.title()?, "a menu item")?;
    stage_line(0x482, record)
}
