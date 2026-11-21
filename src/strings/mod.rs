//! The game's texts. Every text shown has a number; the language file gives the words for each
//! number in the chosen language. The language is loaded by `language`; this module looks texts
//! up, and builds the texts put together from pieces: times, speeds in the chosen units, numbers,
//! percentages. It also holds the keyboard tables that belong to a language.

pub(crate) mod append_percent;
pub(crate) mod append_speed;
pub(crate) mod append_string_by_id;
pub(crate) mod append_time;
pub(crate) mod copy_printable;
mod expand_entry;
pub(crate) mod format_seal_code;
pub(crate) mod format_u32;
pub(crate) mod hex_append;
pub(crate) mod lexicon_memory;
pub(crate) mod locale;
pub(crate) mod owner;
pub(crate) mod rot13_into_text;
pub(crate) mod same_text;
pub(crate) mod string_table;
pub(crate) mod text_buffer;
pub(crate) mod text_from_ids;
pub(crate) mod text_length;
pub(crate) mod units;
