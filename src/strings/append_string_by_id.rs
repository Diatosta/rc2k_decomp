//! Appends the text for a string id. Returns the new cursor and the string pointer (pointer-table
//! branches) or the id itself (string-table branch).

use crate::errors::BadInput;
use crate::strings::expand_entry::expand_entry;
use crate::strings::string_table::{driver_name_slot, driver_name_text, made_text};
use rc2k_core::strings::table::Source;

/// What is returned for a text kept in a variable: the text's address, which as the next string id
/// (`text_from_ids`) is past every table entry. Any id past the table does the same, nothing being
/// appended for it.
pub const AFTER_VARIABLE_TEXT: u32 = 0x8000_0000;

/// The owners the text of a string id can come from: the made texts live in the screens' text
/// buffers, the menus' notice, and the installer's rows, names and variables; the driver names in
/// the strings.
#[derive(Clone, Copy)]
pub struct StringOwners<'a> {
    pub strings: &'a crate::strings::owner::Strings,
    pub menus: &'a crate::menu::owner::Menus,
    pub install: &'a crate::install::owner::Install,
    pub system: &'a crate::system::owner::System,
}

impl<'a> StringOwners<'a> {
    pub fn of(o: crate::owners::Owners<'a>) -> Self {
        Self {
            strings: o.strings,
            menus: o.menus,
            install: o.install,
            system: o.system,
        }
    }
}

/// The text of an own id (`rc2k_core::strings::own`) in the language the game loaded, English
/// where its file lacks it; none for an id no page uses.
pub fn own_text(strings: &crate::strings::owner::Strings, id: u32) -> Vec<u8> {
    let file = strings
        .language
        .own
        .read()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    rc2k_core::strings::own::text(id, *file)
        .unwrap_or_default()
        .to_vec()
}

/// `emit` receives each run of bytes to append. Returns the value described at
/// `AFTER_VARIABLE_TEXT`.
pub fn append_string_by_id(
    o: StringOwners<'_>,
    id: u32,
    emit: &mut impl FnMut(&[u8]),
) -> Result<u32, BadInput> {
    let fault = crate::menu::event_flow::Fault::default();
    let source = Source::of(id, || fault.ok(driver_name_slot(o.strings, id)) != 0);
    fault.check()?;
    let direct = match source {
        Source::Made => Some(made_text(o.menus, o.install, id)?),
        Source::Pointer => Some(driver_name_text(
            o.strings,
            driver_name_slot(o.strings, id)?,
        )?),
        Source::Own => Some(own_text(o.strings, id)),
        Source::Table => None,
    };
    match direct {
        Some(text) => {
            emit(&text);
            Ok(AFTER_VARIABLE_TEXT)
        }
        None => {
            expand_entry(o.strings, id, emit);
            Ok(id)
        }
    }
}

/// Appends the text of string `id` to `out`.
pub fn push_string(o: StringOwners<'_>, out: &mut Vec<u8>, id: u32) -> Result<(), BadInput> {
    append_string_by_id(o, id, &mut |bytes| out.extend_from_slice(bytes))?;
    Ok(())
}
