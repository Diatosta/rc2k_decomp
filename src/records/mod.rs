use crate::framework::GameHook;

pub mod default_stage_records;
pub mod find_record;
pub mod record;
pub mod trakpak_entry;

pub static HOOKS: &[&dyn GameHook] = &[
    &find_record::HOOK,
    &trakpak_entry::HOOK,
    &default_stage_records::HOOK,
];
