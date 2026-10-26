//! `net::draw_number_value` (0x416B8E): a number setting's value: for the item being edited
//! the number's text as typed (`byte_95E8F8`); else the number written into the game's text
//! buffer (`strings::format_u32`, or string 0x16E for 0); then on into
//! `net::draw_setting_value`.
//!
//! In:  eax = the number, esi = the item, ebp = y
//! Out: as `net::draw_setting_value` leaves them; no `pusha`
//! Caller: `net::settings_frame` (the most players). Its calls through `part`
//! (`strings::append_nul` inline). Shadow mode runs the original alone (it draws).
//! Self-test: a copy of it and `net::draw_setting_value` after it, whose calls go to
//! stand-ins, against the Rust version: edited, 0, 4.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::net::draw_setting_value::draw_setting_value;
use crate::net::lobby::{
    APPEND_STRING, EDITED, FORMAT_U32, NUMBER_TEXT, Part, TEXT, a, g, lobby_entry,
};
use rc2k_core::net::screens::{self, Number};

/// The Rust version: the registers it leaves.
pub fn draw_number_value(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut r = *entry;
    // Converted: `rc2k_core::net::screens` (`number_value`, `Number`, `NOTHING`).
    let shown = screens::number_value(r.esi == g(EDITED), r.eax);
    if shown == Number::Typed {
        r.eax = a(NUMBER_TEXT);
    } else {
        r.ebx = a(TEXT);
        let kept_ecx = r.ecx;
        r.ecx = 0;
        if shown == Number::Nothing {
            r.eax = screens::NOTHING;
            part(APPEND_STRING, &mut r)?;
        } else {
            part(FORMAT_U32, &mut r)?;
        }
        unsafe { (r.ebx as *mut u8).write(0) };
        r.ebx = r.ebx.wrapping_add(1);
        r.ecx = kept_ecx;
        r.eax = a(TEXT);
    }
    draw_setting_value(&r, part)
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    lobby_entry(r, draw_number_value)
}
