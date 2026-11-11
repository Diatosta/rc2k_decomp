//! Front-end pages' "unread" steps that copy settings into the page's work variables.

use crate::audio::owner::audio;
use crate::errors::{BadInput, Refused};
use crate::menu::owner::Menus;
use crate::records::stages_store::{RecordRef, RecordTable};
use crate::system::owner::system;

/// `screens::pick_player_1` .. `_4` (0x46AEC5, 0x46AF21, 0x46AF58, 0x46AF8F): player `n`
/// the current one, its +0x2C into `event.picked_controller` and its 0x2C bytes at
/// +0x50 into the page's copy (`keys.tables.controls`). Refuses (before
/// anything) a player without its record.
pub fn pick_player(menus: &Menus, n: u32) -> Result<(), Refused> {
    // The logic is in `rc2k_core::screens::page_values` (`Player::picked`).
    let (value, copy) = crate::menu::player::player(menus, n)
        .ok_or(BadInput("a player without its record"))?
        .get()
        .picked();
    menus.screens.event.current_player.set(n);
    menus.screens.event.picked_controller.set(value);
    menus.keys.tables.controls.write(&copy);
    Ok(())
}

/// `screens::load_audio_settings` (0x4707E6): the audio options (`g_audioOptions`: quality,
/// channels, Dolby, A3D, EAX, DS3D) into the sound page's work variables (`option_work`).
pub fn load_audio_settings(menus: &Menus) {
    // The logic is in `rc2k_core::screens::page_values::audio_page`.
    let options: rc2k_core::audio::modes::AudioOptions = audio().state.audio_options.get();
    menus
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

/// The stage line both stage pages build in the line buffer.
fn stage_line(id: u32, record: RecordRef) -> Result<(), Refused> {
    let line = stage_line_bytes(id, record)?;
    if line.len() > rc2k_core::screens::page_values::STAGE_LINE_ROOM {
        return Err(BadInput("a stage line past its buffer").into());
    }
    crate::screens::text_buffers_store::TEXT_BUFFERS
        .stage_heading
        .write_at(0, &line, "the stage line")?;
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

/// The stage record of the event's stage table `a`, `b` (its +0x38 the page's value), and the record + `c` x
/// 0xA4 the line is about.
fn stage_record(
    stages: &crate::records::stages_store::StageTables,
    a: u32,
    b: u32,
    c: u32,
) -> Result<(RecordTable, RecordRef), BadInput> {
    let base = stages.group_event_table(a, b)?;
    Ok((base, crate::records::stages_store::row(base, c)?))
}

/// What both stage pages read through: refused before anything.
fn stage_refuses(
    stages: &crate::records::stages_store::StageTables,
    a: u32,
    b: u32,
) -> Result<(), Refused> {
    stages.group_event_table(a, b)?;
    Ok(())
}

/// `screens::stage_options_texts` (0x4692A9): the stage options page's items of the best times menu (items 1 and 2 string 0x19C / 0x19B by
/// `champ_view.switch_f8`, 0x19E / 0x19D by `switch_fc`, item 3 the record's +0x38), the stage
/// line (string 0x481) and the caption string 0x4FC + `state.menus.caption_choice`. Refuses (before anything) no
/// stage records; a record of another track pack is the original's fatal error.
pub fn stage_options_texts(menus: &Menus) -> Result<(), Refused> {
    let (a, b) = (
        menus.play.champ_view.switch_f8.get(),
        menus.tables.captions.shown_group.get(),
    );
    let stages = &system().records.stage_tables;
    stage_refuses(stages, a, b)?;
    // The logic is in `rc2k_core::screens::page_values` (the items and the caption).
    use rc2k_core::screens::page_values::{stage_options_caption, stage_options_items};
    let items = menus.tables.menus.view_best_times.id().items();
    let (first, second) = stage_options_items(a, menus.play.champ_view.switch_fc.get());
    set_item(items, 1, first)?;
    set_item(items, 2, second)?;
    let (base, record) = stage_record(stages, a, b, menus.tables.captions.shown_stage.get())?;
    set_item(items, 3, base.title()?)?;
    stage_line(0x481, record)?;
    let mut caption = Vec::new();
    push_id(
        &mut caption,
        stage_options_caption(menus.screens.state.menus.caption_choice.get()),
    )?;
    caption.push(0);
    crate::screens::text_buffers_store::TEXT_BUFFERS
        .caption
        .write_at(0, &caption, "the caption")?;
    Ok(())
}

/// `screens::stage_choice_texts` (0x469434): the stage choice page's items: item 0 of the
/// courses menu `results.group_titles[stage_group]`, item 1 of the table it
/// points to the record's +0x38 (the original reads the table both ways: `notes/bugs.md`),
/// then the stage line (string 0x482) for the chosen stage number of the event's stage table. Refuses (before anything) no stage records or no
/// item table; another track pack is the original's fatal error.
pub fn stage_choice_texts(menus: &Menus) -> Result<(), Refused> {
    let (a, b) = (
        menus.screens.event.stage_group.get(),
        menus.screens.event.stage_sub.get(),
    );
    let stages = &system().records.stage_tables;
    stage_refuses(stages, a, b)?;
    // Item 1 of the "menu" the head points to (the labels: the slip in `notes/bugs.md`).
    let item_1 = crate::menu::menus_store::labels_slot(
        menus,
        menus.tables.menus.view_courses.id(),
        4 + 0x5C + 0x14,
    )?
    .ok_or(BadInput("no item table"))?;
    let items = menus.tables.menus.view_courses.id().items();
    set_item(
        items,
        0,
        crate::session::read_on_in(
            &menus.play.results,
            &menus.play.results.group_titles,
            a as usize,
            "a group title",
        )?,
    )?;
    let (base, record) = stage_record(stages, a, b, menus.screens.event.stage_number.get())?;
    item_1.write::<u32>(base.title()?, "a menu item")?;
    stage_line(0x482, record)
}
