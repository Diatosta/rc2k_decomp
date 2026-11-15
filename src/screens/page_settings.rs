//! Front-end pages' "unread" steps that copy settings into the page's work variables.

use crate::errors::{BadInput, Refused};
use crate::menu::owner::Menus;
use crate::owners::Owners;
use crate::records::stages_store::{RecordRef, RecordTable};
use crate::screens::text_buffers_store::TextBuffers;
use crate::screens::text_buffers_store::text_buffers;

/// Player `n` (1 to 4) becomes the current one
/// and its 0x2C bytes of bindings into the page's copy (`keys.tables.controls`). Refuses a player
/// without its record.
pub fn pick_player(menus: &Menus, n: u32) -> Result<(), Refused> {
    // The logic is in `rc2k_core::screens::page_values` (`Player::picked`).
    let (_, copy) = crate::menu::player::player(menus, n)
        .ok_or(BadInput("a player without its record"))?
        .get()
        .picked();
    menus.screens.event.current_player.set(n);
    menus.keys.tables.controls.write(&copy);
    Ok(())
}

/// The audio options (quality, channels, Dolby, A3D, EAX, DS3D) into the sound page's work
/// variables (the options).
pub fn load_audio_settings(menus: &Menus, audio: &crate::audio::owner::Audio) {
    // The logic is in `rc2k_core::screens::page_values::audio_page`.
    let options: rc2k_core::audio::modes::AudioOptions = audio.state.audio_options.get();
    menus
        .options
        .set_audio_page(rc2k_core::screens::page_values::audio_page(&options));
}

/// `strings::append_string_by_id` into `out`.
fn push_id(
    so: crate::strings::append_string_by_id::StringOwners<'_>,
    out: &mut Vec<u8>,
    id: u32,
) -> Result<(), BadInput> {
    crate::strings::append_string_by_id::push_string(so, out, id)
}

/// The stage line string `id`, " ", the record's +0x40 as a fraction of 6
/// (`strings::append_fraction`), ": ", the name of its track-pack entry
/// (`records::trakpak_entry`'s first dword, a string id), and its NUL.
pub fn stage_line_bytes(
    so: crate::strings::append_string_by_id::StringOwners<'_>,
    id: u32,
    record: RecordRef,
) -> Result<Vec<u8>, Refused> {
    crate::screens::draw_stage_heading::heading_text(so, record, id)
}

/// The stage line both stage pages build in the line buffer.
fn stage_line(
    so: crate::strings::append_string_by_id::StringOwners<'_>,
    text: &TextBuffers,
    id: u32,
    record: RecordRef,
) -> Result<(), Refused> {
    let line = stage_line_bytes(so, id, record)?;
    if line.len() > rc2k_core::screens::page_values::STAGE_LINE_ROOM {
        return Err(BadInput("a stage line past its buffer").into());
    }
    text.stage_heading.write_at(0, &line, "the stage line")?;
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

/// The stage record of the event's stage table `a`, `b` (its +0x38 the page's value), and the
/// record + `c` x 0xA4 the line is about.
fn stage_record(
    stages: &crate::records::stages_store::StageTables,
    a: u32,
    b: u32,
    c: u32,
) -> Result<(RecordTable, RecordRef), BadInput> {
    let base = stages.group_event_table(a, b)?;
    Ok((base, crate::records::stages_store::row(stages, base, c)?))
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

/// The stage options page's items of the best times menu (items 1 and 2 string 0x19C / 0x19B by
/// `champ_view.switches.records_set`, 0x19E / 0x19D by `switches.records_alt`, item 3 the
/// record's name), the stage line (string 0x481) and the caption string 0x4FC +
/// `state.menus.caption_choice`. Refuses no stage records; a record of another track pack is a
/// fatal error.
pub fn stage_options_texts(o: Owners<'static>) -> Result<(), Refused> {
    let menus = o.menus;
    let (a, b) = (
        menus.play.champ_view.switches.records_set.get(),
        menus.play.champ_view.shown_stage.group.get(),
    );
    let text = text_buffers();
    let stages = &o.system.records().stage_tables;
    stage_refuses(stages, a, b)?;
    // The logic is in `rc2k_core::screens::page_values` (the items and the caption).
    use rc2k_core::screens::page_values::{stage_options_caption, stage_options_items};
    let items = menus
        .tables
        .menus
        .view_best_times
        .id()
        .items(&menus.tables.menus);
    let (first, second) = stage_options_items(a, menus.play.champ_view.switches.records_alt.get());
    set_item(items, 1, first)?;
    set_item(items, 2, second)?;
    let (base, record) =
        stage_record(stages, a, b, menus.play.champ_view.shown_stage.number.get())?;
    set_item(items, 3, base.title(&o.system.records().stage_tables)?)?;
    stage_line(
        crate::strings::append_string_by_id::StringOwners::of(o),
        text,
        0x481,
        record,
    )?;
    let mut caption = Vec::new();
    push_id(
        crate::strings::append_string_by_id::StringOwners::of(o),
        &mut caption,
        stage_options_caption(menus.screens.state.menus.caption_choice.get()),
    )?;
    caption.push(0);
    text.caption.write_at(0, &caption, "the caption")?;
    Ok(())
}

/// The stage choice page's items: item 0 of the courses menu `results.group_titles[stage.group]`,
/// item 1 of the table it points to the record's name (the table is read both ways, kept as is),
/// then the stage line (string 0x482) for the chosen stage number of the event's stage table.
/// Refuses no stage records or no item table; another track pack is a fatal error.
pub fn stage_choice_texts(o: Owners<'static>) -> Result<(), Refused> {
    let menus = o.menus;
    let (a, b) = (
        menus.screens.event.stage.group.get(),
        menus.screens.event.stage.sub.get(),
    );
    let text = text_buffers();
    let stages = &o.system.records().stage_tables;
    stage_refuses(stages, a, b)?;
    // Item 1 of the "menu" the head points to (the labels; kept as is).
    let item_1 = crate::menu::menus_store::labels_slot(
        menus.tables.menus.view_courses.id(),
        4 + 0x5C + 0x14,
    )?
    .ok_or(BadInput("no item table"))?;
    let items = menus
        .tables
        .menus
        .view_courses
        .id()
        .items(&menus.tables.menus);
    set_item(items, 0, crate::screens::stage_panel::group_title(a)?)?;
    let (base, record) = stage_record(stages, a, b, menus.screens.event.stage.number.get())?;
    menus
        .tables
        .menus
        .labels
        .set_dword(item_1, base.title(&o.system.records().stage_tables)?)?;
    stage_line(
        crate::strings::append_string_by_id::StringOwners::of(o),
        text,
        0x482,
        record,
    )
}
