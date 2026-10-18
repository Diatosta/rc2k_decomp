use crate::framework::GameHook;

pub mod default_stage_records;
pub mod find_record;
pub mod find_stage_record;
pub mod record;
pub mod seal_time;
pub mod trakpak_entry;

pub static HOOKS: &[&dyn GameHook] = &[
    &find_record::HOOK,
    &find_stage_record::HOOK,
    &seal_time::HOOK,
    &trakpak_entry::HOOK,
    &default_stage_records::HOOK,
];
