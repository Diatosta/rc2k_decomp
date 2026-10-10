//! Functions that lay out parts of a screen: fixed positions, fonts and string ids.

use crate::framework::{GameHook, game_ptr, rva};
use crate::text::font::Font;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};

pub mod draw_brake_balance_marker;
pub mod draw_empty_savegame;
pub mod draw_inspection_status;
pub mod draw_load_setup_list;
pub mod draw_menu_labels;
pub mod draw_repair_times;
pub mod draw_ride_height_installed_marker;
pub mod draw_ride_height_marker;
pub mod draw_save_setup_list;
pub mod draw_savegame_details;
pub mod draw_savegame_preview;
pub mod draw_selected_stage_menu;
pub mod draw_service_area;
pub mod draw_service_stage_info;
pub mod draw_service_time;
pub mod draw_setup_slot_names;
pub mod draw_stage_heading;
pub mod draw_stage_info;
pub mod draw_stage_records;
pub mod draw_steering_sensitivity_marker;
pub mod draw_suspension_stiffness_installed_marker;
pub mod draw_suspension_stiffness_marker;
pub mod draw_time_trial_stage_info;
pub mod harness;
pub mod savegame_preview;
pub mod setup_markers;
pub mod stage_panel;

pub static HOOKS: &[&dyn GameHook] = &[
    &draw_stage_heading::HOOK,
    &draw_stage_records::HOOK,
    &draw_stage_info::HOOK,
    &draw_service_stage_info::HOOK,
    &draw_time_trial_stage_info::HOOK,
    &draw_selected_stage_menu::HOOK,
    &draw_service_time::HOOK,
    &draw_repair_times::HOOK,
    &draw_inspection_status::HOOK,
    &draw_steering_sensitivity_marker::HOOK,
    &draw_brake_balance_marker::HOOK,
    &draw_ride_height_marker::HOOK,
    &draw_suspension_stiffness_marker::HOOK,
    &draw_ride_height_installed_marker::HOOK,
    &draw_suspension_stiffness_installed_marker::HOOK,
    &draw_service_area::HOOK,
    &draw_setup_slot_names::HOOK,
    &draw_save_setup_list::HOOK,
    &draw_load_setup_list::HOOK,
    &draw_empty_savegame::HOOK,
    &draw_savegame_details::HOOK,
    &draw_savegame_preview::HOOK,
    &draw_menu_labels::HOOK,
];

/// Console switch `blank on [prefix]` / `blank off`, kept for every hook in this module: a
/// blanked hook returns at once, running neither the original nor the Rust version, so
/// whatever it draws disappears from the screen. That is how to confirm, in the game, what a
/// screen function draws. Without a prefix every hook of this module is blanked; with one,
/// only the hooks whose name starts with it, or the one hook with exactly that name. It is harmless for these functions because they
/// preserve every register; a new hook in this module must check `blanked(name)` right after
/// `count_call()`.
static BLANK: AtomicBool = AtomicBool::new(false);
static BLANK_PREFIX: Mutex<String> = Mutex::new(String::new());

pub fn set_blank(on: bool, prefix: &str) {
    if let Ok(mut p) = BLANK_PREFIX.lock() {
        *p = prefix.to_string();
    }
    BLANK.store(on, Ordering::Relaxed);
}

/// True if the hook called `name` is blanked at the moment. A prefix that is itself the full
/// name of a hook selects only that hook (`screens::draw_time_trial_stage_info` is also how
/// `screens::draw_selected_stage_menu` starts).
pub fn blanked(name: &str) -> bool {
    BLANK.load(Ordering::Relaxed)
        && BLANK_PREFIX
            .lock()
            .is_ok_and(|prefix| crate::registry::selects(prefix.as_str(), name))
}

/// byte_A8BB54: the text buffer these functions build their lines in.
pub const LINE_BUFFER_RVA: usize = rva(0xA8BB54);

pub fn line_buffer() -> *mut u8 {
    game_ptr(LINE_BUFFER_RVA)
}

/// A font object in game memory, by its address in IDA.
pub fn font_at(addr: usize) -> *mut Font {
    game_ptr(rva(addr))
}

/// Copies `bytes` (a text with its NUL) to the line buffer.
pub unsafe fn set_line(bytes: &[u8]) {
    unsafe { std::ptr::copy_nonoverlapping(bytes.as_ptr(), line_buffer(), bytes.len()) };
}

/// True if the font object at `addr` has a glyph table, so that text can be measured with it.
pub fn font_is_loaded(addr: usize) -> bool {
    unsafe { font_at(addr).read_unaligned() }.glyph_table != 0
}
