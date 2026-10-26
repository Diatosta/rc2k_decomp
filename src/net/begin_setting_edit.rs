//! `net::begin_setting_edit` (0x416C26): Enter on a setting of the settings screen starts
//! its editing (`net::begin_edit`, width 0x30C, `palette`): item 1 the player's name
//! (`g_machineName`), item 3 the host's address (`unk_52ABE4`), item 2 the most players as
//! a text (`strings::format_u32` of `dword_52ABD0` into `byte_95E8F8`); any other nothing.
//!
//! In:  nothing (the item is `g_listChoice`)
//! Out: eax = the item, ebx = the text, ecx = 0x30C, edx = `palette` when an edit starts;
//!      for other items ebx = `dword_52ABD0`; no `pusha`
//! Caller: `net::settings_loop`. Its calls through `part` (`strings::append_nul` inline).
//! Shadow mode runs the original alone. Self-test: a copy whose calls go to stand-ins,
//! against the Rust version, on items 0 .. 4.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::net::lobby::{
    CHOICE, FORMAT_U32, HOST_ADDRESS, MACHINE_NAME, MOST_PLAYERS, NUMBER_TEXT, PALETTE, Part, a, g,
    lobby_entry,
};
use rc2k_core::net::editing::{self, Setting};

const BEGIN_EDIT: u32 = 0x416E1A; // net::begin_edit

/// The Rust version: the registers it leaves.
pub fn begin_setting_edit(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut r = *entry;
    // Converted: `rc2k_core::net::editing` (`Setting`, `WIDTH`).
    r.eax = g(CHOICE);
    let setting = Setting::of(r.eax);
    r.ebx = match setting {
        Setting::Name => a(MACHINE_NAME),
        Setting::HostAddress => a(HOST_ADDRESS),
        _ => g(MOST_PLAYERS),
    };
    match setting {
        Setting::Name | Setting::HostAddress => {}
        Setting::MostPlayers => {
            let item = r.eax;
            r.eax = r.ebx;
            r.ebx = a(NUMBER_TEXT);
            r.ecx = 0;
            part(FORMAT_U32, &mut r)?;
            unsafe { (r.ebx as *mut u8).write(0) };
            r.ebx = a(NUMBER_TEXT);
            r.eax = item;
        }
        Setting::None => return Ok(r),
    }
    r.ecx = editing::WIDTH;
    r.edx = a(PALETTE);
    part(BEGIN_EDIT, &mut r)?;
    Ok(r)
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    lobby_entry(r, begin_setting_edit)
}
